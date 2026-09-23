use core::fmt;
use std::fmt::{Debug, Display};

impl std::error::Error for ParsingError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsingError {
    NoRepopulationExpression,
    InvalidExpression(String)
}

impl From<ParsingError> for std::io::Error {
    fn from(e: ParsingError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsingError::NoRepopulationExpression => {
                Display::fmt("repopulation_expression must be provided with recursive_config.refill_type is custom_expression", f)
            },
            ParsingError::InvalidExpression(s) => {
                write!(f, "could not parse expression: {s}")
            }
        }
    }
}