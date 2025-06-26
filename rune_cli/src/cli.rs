use std::{env, fmt::format, fs, path::PathBuf, process};

use clap::{Parser, Subcommand, command};
use owo_colors::OwoColorize;

use crate::errors::CliError;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Build,
}

#[derive(Parser, Debug)]
#[command(author = "longuint", about = "Rune CLI", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
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
        "{}{} {}",
        " ".repeat(depth),
        "error".bold().red(),
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

pub fn make_folder(current_dir: &PathBuf, name: &str) -> Result<(), CliError> {
    fs::create_dir_all(current_dir.join(name))
        .map_err(|e| CliError::IOError(format!("Failed to create folder: {}", e)))
}
