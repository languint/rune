pub mod expr;
pub mod nodes;
pub mod ops;
pub mod tokens;
pub mod types;

use crate::errors::ParserError;
use crate::parser::expr::Expr;
use crate::parser::nodes::Nodes;
use crate::parser::ops::{BinaryOp, UnaryOp};
use crate::parser::tokens::Token;
use crate::parser::types::Types;
use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(input: String) -> Result<Self, ParserError> {
        let mut lexer = Token::lexer(&input);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next() {
            match token {
                Ok(t) => tokens.push(t),
                Err(_) => {
                    let slice = lexer.slice();
                    if let Ok(num) = slice.parse::<i64>() {
                        tokens.push(Token::Integer(num));
                    } else if let Ok(num) = slice.parse::<f64>() {
                        tokens.push(Token::Float(num));
                    } else if slice.starts_with('"') && slice.ends_with('"') {
                        let string_content = slice[1..slice.len() - 1].into();
                        tokens.push(Token::String(string_content));
                    } else if slice == "true" || slice == "false" {
                        tokens.push(Token::Boolean(slice == "true"));
                    } else if slice.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        tokens.push(Token::Identifier(slice.into()));
                    } else {
                        return Err(ParserError::UnexpectedCharacter(
                            slice.chars().next().unwrap(),
                        ));
                    }
                }
            }
        }

        Ok(Parser { tokens, current: 0 })
    }
}

impl Parser {
    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token) = self.peek() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut statements = Vec::new();

        loop {
            if self.is_at_end() {
                break;
            }
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Expr, ParserError> {
        let expr = self.expression()?;

        // Consume `;`
        self.match_token(&Token::Semicolon);

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        if let Some(Token::KeywordIf) = self.peek() {
            return self.if_else();
        }
        if let Some(Token::KeywordPrint) = self.peek() {
            return self.print();
        }
        self.assignment()
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Integer(value) => {
                    self.advance();
                    Ok(Expr::Literal(Nodes::Integer(value)))
                }
                Token::Float(value) => {
                    self.advance();
                    Ok(Expr::Literal(Nodes::Float(value)))
                }
                Token::String(value) => {
                    self.advance();
                    Ok(Expr::Literal(Nodes::String(value)))
                }
                Token::Boolean(value) => {
                    self.advance();
                    Ok(Expr::Literal(Nodes::Boolean(value)))
                }
                Token::Identifier(name) => {
                    self.advance();
                    Ok(Expr::Literal(Nodes::Identifier(name)))
                }
                Token::LeftParen => {
                    self.advance(); // consume `(`
                    let expr = self.expression()?;
                    if !self.match_token(&Token::RightParen) {
                        return Err(ParserError::ExpectedAfter(")".into(), "expression".into()));
                    }
                    Ok(expr)
                }
                Token::LeftBrace => {
                    self.advance(); // consume `{`
                    let mut statements = Vec::new();

                    while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
                        statements.push(self.statement()?);
                    }

                    if self.previous() != Some(&Token::RightBrace) {
                        return Err(ParserError::ExpectedAfter("}".into(), "block".into()));
                    }

                    Ok(Expr::Block(statements))
                }

                _ => Err(ParserError::UnexpectedToken(format!("{:?}", token))),
            }
        } else {
            Err(ParserError::UnexpectedEndOfInput)
        }
    }
}

impl Parser {
    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_term_op() {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_factor_op() {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if let Some(op) = self.match_unary_op() {
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                operand: Box::new(expr),
            });
        }

        self.primary()
    }
}

impl Parser {
    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.and()?;

        while self.match_token(&Token::Or) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;

        while self.match_token(&Token::And) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_equality_op() {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_comparison_op() {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
}

impl Parser {
    fn match_equality_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::NotEquals) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(&Token::EqualsEquals) {
            Some(BinaryOp::Equal)
        } else {
            None
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::GreaterThan) {
            Some(BinaryOp::Greater)
        } else if self.match_token(&Token::GreaterThanEquals) {
            Some(BinaryOp::GreaterEqual)
        } else if self.match_token(&Token::LessThan) {
            Some(BinaryOp::Less)
        } else if self.match_token(&Token::LessThanEquals) {
            Some(BinaryOp::LessEqual)
        } else {
            None
        }
    }

    fn match_term_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::Minus) {
            Some(BinaryOp::Subtract)
        } else if self.match_token(&Token::Plus) {
            Some(BinaryOp::Add)
        } else {
            None
        }
    }

    fn match_factor_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&Token::Slash) {
            Some(BinaryOp::Divide)
        } else if self.match_token(&Token::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(&Token::Percent) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        if self.match_token(&Token::Minus) {
            Some(UnaryOp::Minus)
        } else if self.match_token(&Token::Bang) {
            Some(UnaryOp::Not)
        } else {
            None
        }
    }
}

impl Parser {
    fn parse_type(&mut self) -> Result<Types, ParserError> {
        if let Some(token) = self.peek().cloned() {
            match token {
                Token::Identifier(type_name) => {
                    self.advance();
                    match type_name.as_str() {
                        "i32" => Ok(Types::I32),
                        "i64" => Ok(Types::I64),
                        "bool" => Ok(Types::Bool),
                        "f32" => Ok(Types::F32),
                        "f64" => Ok(Types::F64),
                        "String" => Ok(Types::String),
                        _ => Err(ParserError::UnexpectedToken(format!(
                            "unknown type: {}",
                            type_name
                        ))),
                    }
                }
                Token::TypeI32 => {
                    self.advance();
                    Ok(Types::I32)
                }
                Token::TypeI64 => {
                    self.advance();
                    Ok(Types::I64)
                }
                Token::TypeBool => {
                    self.advance();
                    Ok(Types::Bool)
                }
                Token::TypeF32 => {
                    self.advance();
                    Ok(Types::F32)
                }
                Token::TypeF64 => {
                    self.advance();
                    Ok(Types::F64)
                }
                Token::TypeString => {
                    self.advance();
                    Ok(Types::String)
                }
                _ => {
                    dbg!(self.peek().cloned());
                    Err(ParserError::ExpectedToken("type".into()))
                }
            }
        } else {
            dbg!(self.peek().cloned());
            Err(ParserError::ExpectedToken("type".into()))
        }
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        // Check for `let`
        if self.match_token(&Token::KeywordLet) {
            if let Some(Token::Identifier(name)) = self.peek().cloned() {
                self.advance(); // consume identifier

                // Check for optional type annotation
                let var_type = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                if !self.match_token(&Token::Equals) {
                    return Err(ParserError::ExpectedAfterCustom(
                        "=".into(),
                        "".into(),
                        "identifier".into(),
                    ));
                }

                let value = self.assignment()?;
                return Ok(Expr::LetDeclaration {
                    identifier: name,
                    var_type,
                    value: Box::new(value),
                });
            } else {
                return Err(ParserError::ExpectedAfter(
                    "identifier".into(),
                    "let".into(),
                ));
            }
        }

        let expr = self.or()?;

        if self.match_token(&Token::Equals) {
            if let Expr::Literal(Nodes::Identifier(name)) = expr {
                let value = self.assignment()?;
                return Ok(Expr::Assignment {
                    identifier: name,
                    value: Box::new(value),
                });
            }
            return Err(ParserError::InvalidAssignment(
                "target must be an identifier".into(),
            ));
        }

        Ok(expr)
    }
}

impl Parser {
    fn if_else(&mut self) -> Result<Expr, ParserError> {
        if !self.match_token(&Token::KeywordIf) {
            return Err(ParserError::ExpectedToken("if".into()));
        }

        let condition_expr = self.expression()?;
        let condition = Box::new(condition_expr);

        if !self.match_token(&Token::LeftBrace) {
            return Err(ParserError::ExpectedAfter(
                "{".into(),
                "if condition".into(),
            ));
        }

        let mut then_statements = Vec::new();
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            then_statements.push(self.statement()?);
        }

        if self.previous() != Some(&Token::RightBrace) {
            return Err(ParserError::ExpectedAfter("}".into(), "if-block".into()));
        }

        let then_branch = Expr::Block(then_statements);

        let else_branch = if self.match_token(&Token::KeywordElse) {
            if !self.match_token(&Token::LeftBrace) {
                return Err(ParserError::ExpectedAfter("{".into(), "else".into()));
            }

            let mut else_statements = Vec::new();
            while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
                else_statements.push(self.statement()?);
            }

            if self.previous() != Some(&Token::RightBrace) {
                return Err(ParserError::ExpectedAfter("}".into(), "else-block".into()));
            }

            Some(Box::new(Expr::Block(else_statements)))
        } else {
            None
        };

        Ok(Expr::IfElse {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }
}

impl Parser {
    fn print(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&Token::KeywordPrint) {
            if let Some(Token::LeftParen) = self.peek().cloned() {
                self.advance(); // consume `(`

                let expr = self.or()?;

                if let Some(Token::RightParen) = self.peek().cloned() {
                    self.advance(); // consume `)`
                } else {
                    return Err(ParserError::ExpectedAfterCustom(
                        ")".into(),
                        "print".into(),
                        "expression".into(),
                    ));
                }

                Ok(Expr::Print(Box::new(expr)))
            } else {
                Err(ParserError::ExpectedAfter("(".into(), "print".into()))
            }
        } else {
            Err(ParserError::ExpectedAfter(
                "print".into(),
                "statement".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_declaration() {
        let mut parser = Parser::new(String::from("let x = 10")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0],
            Expr::LetDeclaration {
                identifier: "x".into(),
                var_type: None,
                value: Box::new(Expr::Literal(Nodes::new_integer(10))),
            }
        );
    }

    #[test]
    fn test_let_declaration_with_type() {
        let mut parser = Parser::new(String::from("let x: i32 = 10")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0],
            Expr::LetDeclaration {
                identifier: "x".into(),
                var_type: Some(Types::I32),
                value: Box::new(Expr::Literal(Nodes::Integer(10))),
            }
        );
    }

    #[test]
    fn test_assignment() {
        let mut parser = Parser::new(String::from("x = 10")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0],
            Expr::Assignment {
                identifier: "x".into(),
                value: Box::new(Expr::Literal(Nodes::new_integer(10))),
            }
        );
    }

    #[test]
    fn test_multiple_statements_with_semicolons() {
        let mut parser =
            Parser::new(String::from("let x = 10; let y = 20; x + y")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 3);

        assert_eq!(
            statements[0],
            Expr::LetDeclaration {
                identifier: "x".into(),
                var_type: None,
                value: Box::new(Expr::Literal(Nodes::Integer(10))),
            }
        );

        assert_eq!(
            statements[1],
            Expr::LetDeclaration {
                identifier: "y".into(),
                var_type: None,
                value: Box::new(Expr::Literal(Nodes::new_integer(20))),
            }
        );

        assert_eq!(
            statements[2],
            Expr::Binary {
                left: Box::new(Expr::Literal(Nodes::new_identifier("x".into()))),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Literal(Nodes::new_identifier("y".into()))),
            }
        );
    }

    #[test]
    fn test_block_with_braces() {
        let mut parser =
            Parser::new(String::from("{ let x = 10; x + 5 }")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);

        if let Expr::Block(block_statements) = &statements[0] {
            assert_eq!(block_statements.len(), 2);
        } else {
            panic!("Expected block expression");
        }
    }

    #[test]
    fn if_block() {
        let mut parser =
            Parser::new(String::from("if cond1 == cond2 {}")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);

        if let Expr::IfElse {
            condition,
            then_branch,
            else_branch,
        } = &statements[0]
        {
            assert_eq!(
                **condition,
                Expr::Binary {
                    left: Box::new(Expr::Literal(Nodes::new_identifier("cond1".into()))),
                    operator: BinaryOp::Equal,
                    right: Box::new(Expr::Literal(Nodes::new_identifier("cond2".into()))),
                }
            );
            if let Expr::Block(block_statements) = then_branch.as_ref() {
                assert_eq!(block_statements.len(), 0);
            } else {
                panic!("Expected block expression");
            }
            assert!(else_branch.is_none());
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn else_block() {
        let mut parser =
            Parser::new(String::from("if cond1 == cond2 {} else {}")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);

        if let Expr::IfElse {
            condition,
            then_branch,
            else_branch,
        } = &statements[0]
        {
            assert_eq!(
                **condition,
                Expr::Binary {
                    left: Box::new(Expr::Literal(Nodes::new_identifier("cond1".into()))),
                    operator: BinaryOp::Equal,
                    right: Box::new(Expr::Literal(Nodes::new_identifier("cond2".into()))),
                }
            );
            if let Expr::Block(block_statements) = then_branch.as_ref() {
                assert_eq!(block_statements.len(), 0);
            } else {
                panic!("Expected block expression for then branch");
            }
            assert!(else_branch.is_some());
            if let Some(else_expr) = else_branch {
                if let Expr::Block(block_statements) = else_expr.as_ref() {
                    assert_eq!(block_statements.len(), 0);
                } else {
                    panic!("Expected block expression for else branch");
                }
            }
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn if_bang_cond() {
        let mut parser =
            Parser::new(String::from("if !cond1 {} else {}")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);

        if let Expr::IfElse {
            condition,
            then_branch,
            else_branch,
        } = &statements[0]
        {
            assert_eq!(
                **condition,
                Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(Expr::Literal(Nodes::new_identifier("cond1".into()))),
                }
            );
            if let Expr::Block(block_statements) = then_branch.as_ref() {
                assert_eq!(block_statements.len(), 0);
            } else {
                panic!("Expected block expression for then branch");
            }
            assert!(else_branch.is_some());
            if let Some(else_expr) = else_branch {
                if let Expr::Block(block_statements) = else_expr.as_ref() {
                    assert_eq!(block_statements.len(), 0);
                } else {
                    panic!("Expected block expression for else branch");
                }
            }
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn invalid_char_should_panic() {
        let result = Parser::new(String::from("@"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParserError::UnexpectedCharacter('@'));
    }

    #[test]
    fn type_annotation() {
        let mut parser = Parser::new(String::from("let x: i32 = 42;")).expect("Expected Parser");
        let statements = parser.parse().expect("Expected statements");
        assert_eq!(statements.len(), 1);

        if let Expr::LetDeclaration {
            identifier,
            value,
            var_type,
        } = &statements[0]
        {
            assert_eq!(identifier, "x");
            assert_eq!(var_type, &Some(Types::I32));
            assert_eq!(value, &Box::new(Expr::Literal(Nodes::Integer(42))));
        } else {
            panic!("Expected let expression");
        }
    }
}
