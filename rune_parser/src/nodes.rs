#[derive(Debug, Clone, PartialEq)]
pub enum Nodes {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    // Reference(Box<Nodes>),
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
    // #[inline]
    // pub fn new_reference(value: Nodes) -> Self {
    //     Nodes::Reference(Box::new(value))
    // }
}
