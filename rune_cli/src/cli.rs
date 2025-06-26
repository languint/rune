use std::{env, fs, path::Path};

use clap::{Parser, Subcommand, command};
use owo_colors::OwoColorize;

use crate::errors::CliError;

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommand {
    Build,
}

#[derive(Parser, Debug)]
#[command(author = "longuint", about = "Rune CLI", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub quiet: bool,
}

#[inline]
pub fn print_value(label: &str, value: &str, depth: usize) {
    println!("{}{}: `{}`", " ".repeat(depth), label.bold(), value);
}

#[inline]
pub fn print_section(label: &str, depth: usize) {
    println!("{}{}", " ".repeat(depth), label.bold().green());
}

#[inline]
pub fn print_error(error: &str, depth: usize) {
    println!(
        "{}{}{} {}",
        " ".repeat(depth),
        "Error".bold().red(),
        ":".bold(),
        error.red()
    );
}

#[inline]
pub fn print_warning(warning: &str, depth: usize) {
    println!(
        "{}{}{} {}",
        " ".repeat(depth),
        "warning".bold().yellow(),
        ":".bold(),
        warning
    );
}

pub fn get_current_directory() -> Result<std::path::PathBuf, CliError> {
    let result = env::current_dir();

    if result.is_err() {
        return Err(CliError::InternalError(format!(
            "Failed to get current directory: {}",
            result.unwrap_err()
        )));
    }

    Ok(result.unwrap())
}

pub fn make_folder(current_dir: &Path, name: &str) -> Result<(), CliError> {
    fs::create_dir_all(current_dir.join(name))
        .map_err(|e| CliError::IOError(format!("Failed to create folder: {}", e)))
}

pub fn folder_exists(current_dir: &Path, name: &str) -> Result<(), CliError> {
    let path = current_dir.join(name);

    match path.exists() {
        true => Ok(()),
        false => Err(CliError::IOError(format!(
            "Folder {} does not exist!",
            name
        ))),
    }
}

pub fn read_file(file_path: &Path) -> Result<String, CliError> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| CliError::IOError(format!("Failed to read file: {}", e)))?;

    Ok(content)
}
