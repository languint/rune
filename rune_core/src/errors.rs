use std::fmt::{self};

#[derive(PartialEq)]
pub enum CodeGenError {
    UndefinedVariable(String),
    TypeMismatch(String, String),
    TypeMismatchCustom(String),
    InvalidOperation(String),
    NoFunction,
    StringError(String),
    OperatorNotSupported(String, String),
    InternalError(String),
    StoreError(String),
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

impl fmt::Debug for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_print_error(self))
    }
}

pub fn get_print_error(error: &CodeGenError) -> String {
    match error {
        CodeGenError::InternalError(msg) => format!("(C000): Internal error: {}", msg),
        CodeGenError::UndefinedVariable(v) => format!("(C001): Undefined variable `{}`", v),
        CodeGenError::TypeMismatch(expected, actual) => format!(
            "(C002): Type mismatch, expected `{}` but got `{}`",
            expected, actual
        ),
        CodeGenError::TypeMismatchCustom(msg) => format!("(C002): Type mismatch: {}", msg),
        CodeGenError::InvalidOperation(op) => format!("(C003): Invalid operation `{}`", op),
        CodeGenError::NoFunction => "(C004): No function found".into(),
        CodeGenError::StringError(msg) => format!("(C005): String error: {}", msg),
        CodeGenError::OperatorNotSupported(op1, op2) => {
            format!("(C006): Operator `{}` not supported for `{}`", op1, op2)
        }
        CodeGenError::StoreError(var) => format!("(C007): Store error for variable `{}`", var),
    }
}
