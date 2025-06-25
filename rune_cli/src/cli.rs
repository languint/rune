use clap::{Parser, command};
use owo_colors::OwoColorize;

use crate::errors::CliError;

#[derive(Parser, Debug)]
#[command(author = "longuint", about = "Rune CLI", version = "0.1.0")]
pub struct Cli {
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub quiet: bool,
}

#[inline]
pub fn print_value(label: &str, value: &str, depth: usize) {
    println!("{}{}: `{}`", " ".repeat(depth), label.bold().green(), value);
}

#[inline]
pub fn print_section(label: &str, depth: usize) {
    println!("{}{}", " ".repeat(depth), label.bold().green());
}

#[inline]
pub fn print_error(error: &CliError, depth: usize) {
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
