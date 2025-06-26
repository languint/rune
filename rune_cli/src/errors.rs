use std::fmt::{self};

#[derive(PartialEq)]
pub enum CliError {
    InternalError(String),
    FolderCreationError(String),
    InvalidConfig(String),
    IOError(String),
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

impl ToString for CliError {
    fn to_string(&self) -> String {
        get_print_error(self)
    }
}

pub fn get_print_error(error: &CliError) -> String {
    match error {
        CliError::InternalError(msg) => format!("(C000): Internal error: {}", msg),
        CliError::FolderCreationError(msg) => format!("(C001): Folder creation error: {}", msg),
        CliError::InvalidConfig(msg) => format!("(C002): Invalid configuration: {}", msg),
        CliError::IOError(msg) => format!("(C003): IO error: {}", msg),
    }
}
