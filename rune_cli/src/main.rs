use clap::Parser;

use crate::cli::{Cli, print_warning};

mod cli;
mod errors;

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
}
