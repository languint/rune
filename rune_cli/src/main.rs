use std::{path::PathBuf, process};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::cli::{Cli, Command, print_error, print_section, print_value, print_warning};

mod cli;
mod config;
mod errors;

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
        Command::Build => build(&current_dir, log_level),
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
}
