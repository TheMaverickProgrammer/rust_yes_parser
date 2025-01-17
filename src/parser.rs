use crate::{element::Element, enums::Delimiters, enums::ErrorCodes};

pub struct Parser {
    delimiter: Delimiters,
    element: Option<Element>,
    error: Option<ErrorCodes>,
    lineNumber: usize,
}

impl Parser {
    pub fn read(lineNumber: usize, line: &str) -> Parser {
        Parser {
            delimiter: Delimiters::Unset,
            element: None,
            error: None,
            lineNumber,
        }
    }
}

pub enum ParseResult {
    Success {
        line: usize,
    },
    Fail {
        line: usize,
        message: String,
        code: ErrorCodes,
    },
}
