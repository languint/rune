use std::fmt;

use crate::parser::{
    nodes::Nodes,
    ops::{BinaryOp, UnaryOp},
    types::Types,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Nodes),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Assignment {
        identifier: String,
        value: Box<Expr>,
    },
    LetDeclaration {
        identifier: String,
        var_type: Option<Types>,
        value: Box<Expr>,
    },
    IfElse {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Block(Vec<Expr>),
    Print(Box<Expr>),
    MethodCall {
        target: Box<Expr>,
        method_name: String,
        arguments: Vec<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(node) => write!(f, "{:?}", node),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {:?} {})", left, operator, right),
            Expr::Unary { operator, operand } => {
                write!(f, "{:?}{}", operator, operand)
            }
            Expr::Assignment { identifier, value } => {
                write!(f, "{} = {}", identifier, value)
            }
            Expr::LetDeclaration {
                identifier,
                value,
                var_type,
            } => {
                write!(f, "let {}: {:?} = {}", identifier, var_type, value)
            }
            Expr::IfElse {
                condition,
                then_branch,
                else_branch,
            } => write!(
                f,
                "if {} {{ {} }} else {{ {} }}",
                condition,
                then_branch,
                else_branch
                    .as_ref()
                    .map_or("".to_string(), |e| e.to_string())
            ),
            Expr::Block(exprs) => write!(
                f,
                "{{ {} }}",
                exprs
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("; ")
            ),
            Expr::Print(expr) => write!(f, "print {}", expr),
            Expr::MethodCall {
                target,
                method_name,
                arguments,
            } => write!(
                f,
                "{}.{}({})",
                target,
                method_name,
                arguments
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
