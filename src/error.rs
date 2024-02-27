use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
struct LinkError;

#[derive(Debug, PartialEq)]
struct LoadError {
    msg: String,
}

impl LoadError {
    fn from(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    pub fn from(msg: &str) -> Self {
        Self {
            msg: String::from(msg),
        }
    }

    pub fn from_parseint(_err: ParseIntError) -> Self {
        Self {
            msg: String::from("Could not parse int"),
        }
    }
}
