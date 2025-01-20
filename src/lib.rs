use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use element::Element;
use element_parser::ElementParser;
use enums::{Elements, ErrorCodes, Glyphs};
use literal::Literal;

pub mod element;
pub mod element_parser;
pub mod enums;
pub mod keyval;
pub mod literal;
pub mod utils;

pub struct ParsedElement {
    line_number: usize,
    data: Elements,
}

impl ParsedElement {
    pub fn new(line_number: usize, data: Elements) -> ParsedElement {
        ParsedElement { line_number, data }
    }
}

#[allow(dead_code)]
pub struct LineError {
    line_number: usize,
    message: String,
    code: ErrorCodes,
}

impl LineError {
    pub fn from_error_code(line_number: usize, code: ErrorCodes) -> LineError {
        LineError {
            line_number,
            message: code.values().to_owned(),
            code,
        }
    }

    pub fn custom(line_number: usize, message: String) -> LineError {
        LineError {
            line_number,
            message,
            code: ErrorCodes::Runtime,
        }
    }
}
pub struct YesDocParser {
    pub total_lines: usize,
    building_line: Option<String>,
    attrs: Vec<Element>,
    pub parsed_elements: Vec<ParsedElement>,
    pub errors: Vec<LineError>,
}

impl YesDocParser {
    pub fn from_file(file: &File, literals: Option<Vec<Literal>>) -> YesDocParser {
        let reader = BufReader::new(file);

        let mut parser = YesDocParser {
            total_lines: 0,
            building_line: None,
            attrs: Vec::new(),
            parsed_elements: Vec::new(),
            errors: Vec::new(),
        };

        let mut list = match literals {
            Some(ref custom) => custom.clone(),
            None => Vec::new(),
        };

        list.push(Literal::build_quotes());

        for line in reader.lines() {
            parser.process(&mut line.unwrap(), &literals);
        }

        parser.organize();

        parser
    }

    pub fn from_string(body: &str, literals: Option<Vec<Literal>>) -> YesDocParser {
        let mut parser = YesDocParser {
            total_lines: 0,
            building_line: None,
            attrs: Vec::new(),
            parsed_elements: Vec::new(),
            errors: Vec::new(),
        };

        let mut list = match literals {
            Some(ref custom) => custom.clone(),
            None => Vec::new(),
        };

        list.push(Literal::build_quotes());

        for line in body.split("\n") {
            parser.process(&mut String::from(line), &literals);
        }

        parser.organize();

        parser
    }

    fn organize(&mut self) {
        // Hoist globals to the top of the list in order they were entered.
        self.parsed_elements
            .sort_by(|a, b| match (&a.data, &b.data) {
                (Elements::Global(_), Elements::Global(_)) => a.line_number.cmp(&b.line_number),
                (Elements::Global(_), _) => std::cmp::Ordering::Less,
                (_, Elements::Global(_)) => std::cmp::Ordering::Greater,
                _ => a.line_number.cmp(&b.line_number),
            });
    }

    fn process(&mut self, line: &mut String, literals: &Option<Vec<Literal>>) {
        self.total_lines += 1;

        let backslash = Glyphs::Backslash.value() as char;
        if line.ends_with(backslash) {
            *line = line.replace(backslash, "");

            if let Some(ref mut str) = self.building_line {
                *str += &line;
            } else {
                self.building_line = Some(line.clone());
            }

            return;
        } else if let Some(ref mut str) = self.building_line {
            *line += str;
            self.building_line = None;
        }

        let mut element_parser = ElementParser::read(self.total_lines, line, &literals);

        if !element_parser.is_ok() {
            self.errors.push(LineError::from_error_code(
                element_parser.line_number,
                element_parser.error.unwrap(),
            ));
        }

        let consumed = match element_parser.element {
            Some(Elements::Attribute(ref data)) => {
                self.attrs.push(Elements::copy(data));
                true
            }
            Some(Elements::Standard {
                ref mut attrs,
                data: _,
            }) => {
                for a in &self.attrs {
                    attrs.push(Elements::copy(a));
                }

                self.attrs.clear();
                false
            }
            _ => false,
        };

        if consumed {
            return;
        }

        self.parsed_elements.push(ParsedElement::new(
            self.total_lines,
            element_parser
                .element
                .expect("Expected element_parser.is_ok() to signal valid elements."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{enums::Elements, literal::Literal, utils::StringUtils, YesDocParser};

    #[test]
    fn is_quoted() {
        let hwq = "\"Hello, world!\"";
        let str: String = hwq.to_owned();
        assert_eq!(str.is_quoted(), true);
    }

    #[test]
    fn quote_string() {
        let hw = "Hello, world!";
        let hwq = "\"Hello, world!\"";
        let mut str: String = hw.to_owned();
        assert_eq!(str.quote(), hwq);
    }

    #[test]
    fn unquote_string() {
        let hw = "Hello, world!";
        let mut str: String = hw.to_owned();
        str.quote();
        assert_eq!(str.unquote(), hw);
    }

    #[test]
    fn substring() {
        let hw = "Hello, world!";
        let str: String = hw.to_owned();
        assert_eq!(str.substring(7, 5), "world");
    }

    #[test]
    fn trim() {
        let hw = "Hello, world!";
        let padded_hw = "   Hello, world!    ";
        let mut str = hw.to_owned();
        assert_eq!(str.trim(), hw);
        assert_eq!(padded_hw.trim(), hw);
    }

    #[test]
    fn parse_macro_content() {
        let content = "!macro teardown_textbox(tb) = \"call common.textbox_teardown tb=\"tb";
        let doc = YesDocParser::from_string(content, Some(vec![Literal::build_quotes()]));
        assert_eq!(doc.parsed_elements.len(), 1);

        let first = doc.parsed_elements.first();
        assert_eq!(first.is_some(), true);

        let element = match &first.unwrap().data {
            Elements::Global(data) => data,
            _ => panic!("Should not happen!"),
        };

        assert_eq!(element.args.first().is_some(), true);
        let arg = element.args.first().unwrap();

        assert_eq!(arg.key.as_ref().unwrap(), "teardown_textbox(tb)");
        assert_eq!(arg.val, "\"call common.textbox_teardown tb=\"tb");
    }
}
