use std::fmt::{self};

#[derive(PartialEq)]
pub enum ParserError {
    UnexpectedCharacter(char),
    UnexpectedToken(String),
    UnexpectedEndOfInput,
    ExpectedToken(String),
    ExpectedAfter(String, String),
    ExpectedAfterCustom(String, String, String),
    InvalidAssignment(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

impl fmt::Debug for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

pub fn get_print_error(error: &ParserError) -> String {
    match error {
        ParserError::UnexpectedCharacter(c) => format!("(P001): Unexpected character `{}`", c),
        ParserError::UnexpectedToken(token) => format!("(P002): Unexpected token `{}`", token),
        ParserError::UnexpectedEndOfInput => "(P003): Unexpected end of input".to_string(),
        ParserError::ExpectedToken(token) => format!("(P004): Expected token `{}`", token),
        ParserError::ExpectedAfter(expected, found) => {
            format!("(P005): Expected `{}` after `{}`", expected, found)
        }
        ParserError::ExpectedAfterCustom(expected, found, message) => {
            format!(
                "(P005): Expected `{}` after `{}` {}",
                expected, found, message
            )
        }
        ParserError::InvalidAssignment(message) => {
            format!("(P006): Invalid assignment {}", message)
        }
    }
}
