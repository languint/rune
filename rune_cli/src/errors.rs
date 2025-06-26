use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum CliError {
    InternalError(String),
    InvalidConfig(String),
    IOError(String),
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

pub fn get_print_error(error: &CliError) -> String {
    match error {
        CliError::InternalError(msg) => format!("(C000): Internal error: {}", msg),
        CliError::InvalidConfig(msg) => format!("(C001): Invalid configuration: {}", msg),
        CliError::IOError(msg) => format!("(C002): IO error: {}", msg),
    }
}
