#[derive(Debug, Clone, PartialEq)]
pub enum Nodes {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
}

impl Nodes {
    #[inline]
    pub fn new_integer(value: i64) -> Self {
        Nodes::Integer(value)
    }
    #[inline]
    pub fn new_float(value: f64) -> Self {
        Nodes::Float(value)
    }
    #[inline]
    pub fn new_string(value: String) -> Self {
        Nodes::String(value)
    }
    #[inline]
    pub fn new_boolean(value: bool) -> Self {
        Nodes::Boolean(value)
    }
    #[inline]
    pub fn new_identifier(value: String) -> Self {
        Nodes::Identifier(value)
    }
}
