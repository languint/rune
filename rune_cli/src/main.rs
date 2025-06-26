use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{self, Command},
    time::Instant,
};

use clap::Parser;
use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use owo_colors::OwoColorize;
use rune_parser::parser;

use crate::{
    cli::{Cli, CliCommand, print_error, print_section, print_value, print_warning, read_file},
    config::find_target_files,
    errors::CliError,
};

mod cli;
mod config;
mod errors;

const DEFAULT_EXTENSION: &str = "rn";

#[derive(Debug, PartialEq)]
enum LogLevel {
    Verbose,
    Quiet,
    Default,
}

fn main() {
    let cli = Cli::parse();

    let log_level = match (cli.quiet, cli.verbose) {
        (true, true) => {
            print_warning("quiet and verbose flags passed, using verbose", 0);
            LogLevel::Verbose
        }
        (true, false) => LogLevel::Quiet,
        (false, true) => LogLevel::Verbose,
        (false, false) => LogLevel::Default,
    };

    let current_dir = cli::get_current_directory();

    if current_dir.is_err() {
        let err = current_dir.unwrap_err();
        print_error(err.to_string().as_str(), 0);
        process::exit(1);
    }

    let current_dir = current_dir.unwrap();

    match cli.command {
        CliCommand::Build => build(&current_dir, log_level),
    }
}

fn build(current_dir: &PathBuf, log_level: LogLevel) {
    println!("{} `build`", "Running".green().bold());

    let config = config::get_config(&current_dir);

    if config.is_err() {
        let err = config.unwrap_err();
        print_error(err.to_string().as_str(), 0);
        process::exit(1);
    }

    let config = config.unwrap();

    if log_level == LogLevel::Verbose {
        print_section("Config", 4);
        print_value("Title", config.title.as_str(), 5);
        print_value("Version", config.version.as_str(), 5);
    }

    let source_dir = config.build.source_dir.or(Some("src".into())).unwrap();
    let target_dir = config.build.target_dir.or(Some("target".into())).unwrap();

    if let Err(err) = cli::folder_exists(current_dir, source_dir.as_str()) {
        print_error(err.to_string().as_str(), 0);
        process::exit(1);
    }

    let source_dir = &current_dir.join(source_dir);
    let target_dir = &current_dir.join(target_dir);

    let targets = find_target_files(source_dir, DEFAULT_EXTENSION);

    if targets.is_empty() {
        print_warning("No target files found.", 0);
        process::exit(1);
    }

    println!("{} {} target(s).", "Found".bold().green(), targets.len());

    let start = Instant::now();
    for target_file in targets {
        let source = read_file(&source_dir.join(&target_file));

        if source.is_err() {
            print_error(source.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let source = source.unwrap();

        let context = Context::create();
        let mut codegen = rune_core::codegen::CodeGen::new(&context, source.as_str());

        let parser = parser::Parser::new(source);

        if parser.is_err() {
            print_error(parser.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let mut parser = parser.unwrap();

        let statements = parser.parse();

        if statements.is_err() {
            print_error(statements.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let statements = statements.unwrap();

        let result = codegen.compile_statements(&statements);

        if result.is_err() {
            print_error(result.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        Target::initialize_x86(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple);

        if target.is_err() {
            print_error(target.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let target = target.unwrap();
        let target_machine = target.create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        );

        if target_machine.is_none() {
            print_error("Failed to create target machine", 0);
            process::exit(1);
        }

        let target_machine = target_machine.unwrap();

        let mem_buffer = target_machine.write_to_memory_buffer(&codegen.module, FileType::Object);

        if mem_buffer.is_err() {
            print_error(mem_buffer.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let mem_buffer = mem_buffer.unwrap();

        let file_name = target_file.file_stem();

        if file_name.is_none() {
            print_error("Failed to get file name", 0);
            process::exit(1);
        }

        let file_name = file_name.unwrap().to_str();

        if file_name.is_none() {
            print_error("Could not convert file name to string", 0);
            process::exit(1);
        }

        let file_name = file_name.unwrap();

        let obj_path = target_dir.join(format!("{}.o", file_name));
        let obj_file = File::create(&obj_path)
            .map_err(|e| CliError::IOError(format!("Failed to create object file `{}`", e)));

        if obj_file.is_err() {
            print_error(obj_file.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let mut obj_file = obj_file.unwrap();
        let result = obj_file.write_all(mem_buffer.as_slice());
        if result.is_err() {
            print_error(result.err().unwrap().to_string().as_str(), 0);
            process::exit(1);
        }

        let bin_path = target_dir.join(file_name);

        // Use a C compiler (like gcc or clang) to link the object file into an executable
        let output = Command::new("cc") // common alias for the system's C compiler
            .arg(&obj_path)
            .arg("-o")
            .arg(&bin_path)
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    print_error(
                        &format!("Linker failed with status {}:\n{}", output.status, stderr),
                        0,
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_error(
                    &format!(
                        "Failed to execute linker: {}. Is 'cc' (or 'gcc'/'clang') in your PATH?",
                        e
                    ),
                    0,
                );
                process::exit(1);
            }
        }

        println!("{} `{}`.", "Compiled".bold().yellow(), file_name.bold(),);
    }
    let end = Instant::now();
    let duration = end - start;

    if log_level == LogLevel::Verbose {
        print_value(
            "Compile Duration",
            format!("{}ms", duration.as_millis()).as_str(),
            0,
        );
    }
}
