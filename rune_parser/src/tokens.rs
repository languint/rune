use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    // Arithmetic operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    // Assignment and equality
    #[token("=")]
    Equals,
    #[token("==")]
    EqualsEquals,
    #[token("!=")]
    NotEquals,

    // Comparison operators
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
    #[token(">=")]
    GreaterThanEquals,
    #[token("<=")]
    LessThanEquals,

    // Logical operators
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(";")]
    Semicolon,

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\[nrt"\\])*""#, |lex| {
        let slice = lex.slice();
        // Remove quotes and handle escape sequences
        let content = &slice[1..slice.len()-1];
        Some(content.replace("\\n", "\n")
                   .replace("\\r", "\r")
                   .replace("\\t", "\t")
                   .replace("\\\"", "\"")
                   .replace("\\\\", "\\"))
    })]
    String(String),

    #[regex(r"true|false", |lex| match lex.slice() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })]
    Boolean(bool),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))]
    Identifier(String),

    #[token("let")]
    KeywordLet,
    #[token("if")]
    KeywordIf,
    #[token("else")]
    KeywordElse,
    #[token("while")]
    KeywordWhile,
    #[token("for")]
    KeywordFor,
    #[token("->")]
    Arrow,
    #[token("=>")]
    BigArrow,
}
