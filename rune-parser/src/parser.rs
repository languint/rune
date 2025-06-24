use anyhow::Error;

pub struct Parser {
    input: String,
}

impl Parser {
    pub fn new(input: String) -> Self {
        Parser { input }
    }
}

impl Parser {
    pub fn parse(&self) -> Result<(), Error> {
        todo!()
    }
}
