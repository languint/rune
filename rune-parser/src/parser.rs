use crate::nodes::Nodes;
use crate::tokens::Token;
use anyhow::{Result, anyhow};
use logos::Logos;

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
        value: Box<Expr>,
    },
    Block(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Not,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(input: String) -> Result<Self> {
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
                        let string_content = slice[1..slice.len() - 1].to_string();
                        tokens.push(Token::String(string_content));
                    } else if slice == "true" || slice == "false" {
                        tokens.push(Token::Boolean(slice == "true"));
                    } else if slice.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        tokens.push(Token::Identifier(slice.to_string()));
                    } else {
                        return Err(anyhow!("Unexpected character: {}", slice));
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
    pub fn parse(&mut self) -> Result<Vec<Expr>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Expr> {
        let expr = self.expression()?;

        // Consume `;`
        self.match_token(&Token::Semicolon);

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        // Check for `let`
        if self.match_token(&Token::KeywordLet) {
            if let Some(Token::Identifier(name)) = self.peek().cloned() {
                self.advance(); // consume identifier

                if !self.match_token(&Token::Equals) {
                    return Err(anyhow!("Expected '=' after identifier in let declaration"));
                }

                let value = self.assignment()?;
                return Ok(Expr::LetDeclaration {
                    identifier: name,
                    value: Box::new(value),
                });
            } else {
                return Err(anyhow!("Expected identifier after 'let'"));
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
            return Err(anyhow!("Invalid assignment target"));
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
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
                        return Err(anyhow!("Expected ')' after expression"));
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
                        return Err(anyhow!("Expected '}}' after block"));
                    }

                    Ok(Expr::Block(statements))
                }
                _ => Err(anyhow!("Unexpected token: {:?}", token)),
            }
        } else {
            Err(anyhow!("Unexpected end of input"))
        }
    }
}

impl Parser {
    fn term(&mut self) -> Result<Expr> {
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

    fn factor(&mut self) -> Result<Expr> {
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

    fn unary(&mut self) -> Result<Expr> {
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
    fn or(&mut self) -> Result<Expr> {
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

    fn and(&mut self) -> Result<Expr> {
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

    fn equality(&mut self) -> Result<Expr> {
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

    fn comparison(&mut self) -> Result<Expr> {
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
                identifier: "x".to_string(),
                value: Box::new(Expr::Literal(Nodes::new_integer(10))),
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
                identifier: "x".to_string(),
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
                identifier: "x".to_string(),
                value: Box::new(Expr::Literal(Nodes::Integer(10))),
            }
        );

        assert_eq!(
            statements[1],
            Expr::LetDeclaration {
                identifier: "y".to_string(),
                value: Box::new(Expr::Literal(Nodes::new_integer(20))),
            }
        );

        assert_eq!(
            statements[2],
            Expr::Binary {
                left: Box::new(Expr::Literal(Nodes::new_identifier("x".to_string()))),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Literal(Nodes::new_identifier("y".to_string()))),
            }
        );
    }

    #[test]
    fn test_block_with_braces() {
        let mut parser = Parser::new(String::from("{ let x = 10; x + 5 }")).unwrap();
        let statements = parser.parse().unwrap();
        assert_eq!(statements.len(), 1);

        if let Expr::Block(block_statements) = &statements[0] {
            assert_eq!(block_statements.len(), 2);
        } else {
            panic!("Expected block expression");
        }
    }
}
