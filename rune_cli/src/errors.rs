use std::fmt::{self};

#[derive(PartialEq)]
pub enum CliError {
    InternalError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

pub fn get_print_error(error: &CliError) -> String {
    match error {
        CliError::InternalError(msg) => format!("(C000): Internal error: {}", msg),
    }
}
