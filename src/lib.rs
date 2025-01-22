//! Your Extensible Script Parser
//!
//! Provides `YesDocParser` to parse YES documents from file or from strings.
//! The entry-points are:
//! - `YesDocParser::from_file(&File, Option<Vec<Literal>>) -> YesDocParser`
//! - `YesDocParser::from_string(&str, Option<Vec<Literal>>) -> YesDocParser`
//!
//! Both take an optional list of `Literal` structs which denote custom
//! `begin` and `end` tokens. Both entry-points will append the result from
//! `List::build_quotes()` regardless if any custom literals are also provided.
//!
//! Literals instruct the parser which span of characters, called a token,
//! will be considered when finding the next key-value pair. This implies,
//! by default, that quoted strings can be parsed correctly so that they can
//! be key or a value even if they contain reserved symbols.
use std::{
    cmp::Ordering,
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

pub enum ParseResult {
    Ok {
        line_number: usize,
        data: Elements,
    },
    Err {
        line_number: usize,
        message: String,
        code: ErrorCodes,
    },
}

impl ParseResult {
    /// Constructs and returns [ParserResult::Err] with a line number
    /// and spec-associated [ErrorCodes] serialized as a string into
    /// the field [ParserResult::Err::message].
    pub fn error(line_number: usize, code: ErrorCodes) -> ParseResult {
        ParseResult::Err {
            line_number,
            message: code.values().to_owned(),
            code,
        }
    }

    /// Constructs and returns [ParserResult::Err] with a line number
    /// and a custom message. [ParserResult::Err::code] will be
    /// set to [ErrorCodes::Runtime]. This construction should be used
    /// for specialized error messages when using YES format for custom
    /// purposes.
    pub fn custom_error(line_number: usize, message: String) -> ParseResult {
        ParseResult::Err {
            line_number,
            message,
            code: ErrorCodes::Runtime,
        }
    }
}

/// The entry-point for parsing YES documents and scriplets.
/// It is responsible for tracking the total number of lines fed,
/// the line being built (in the event of multi-lines),
/// the attributes for the next standard element, and collecting
/// the results of the [ElementParser::read] routine.
pub struct YesDocParser {
    total_lines: usize,
    building_line: Option<String>,
    attrs: Vec<Element>,
    results: Vec<ParseResult>,
}

impl YesDocParser {
    /// Returns a list of [ParserResult] values read from an input [file].
    pub fn from_file(file: &File, literals: Option<Vec<Literal>>) -> Vec<ParseResult> {
        let reader = BufReader::new(file);

        let mut parser = YesDocParser {
            total_lines: 0,
            building_line: None,
            attrs: Vec::new(),
            results: Vec::new(),
        };

        let mut literals = match literals {
            Some(ref custom) => custom.clone(),
            None => Vec::new(),
        };

        literals.insert(0, Literal::build_quotes());

        let literals = Some(literals);

        for line in reader.lines() {
            parser.process(&mut line.unwrap(), &literals);
        }

        parser.organize();

        parser.results
    }

    /// Returns a list of [ParserResult] values read from [body].
    pub fn from_string(body: &str, literals: Option<Vec<Literal>>) -> Vec<ParseResult> {
        let mut parser = YesDocParser {
            total_lines: 0,
            building_line: None,
            attrs: Vec::new(),
            results: Vec::new(),
        };

        let mut literals = match literals {
            Some(ref custom) => custom.clone(),
            None => Vec::new(),
        };

        literals.insert(0, Literal::build_quotes());

        let literals = Some(literals);

        for line in body.split("\n") {
            parser.process(&mut String::from(line), &literals);
        }

        parser.organize();

        parser.results
    }

    /// Hoist globals to the top of the list in order they were entered.
    /// This makes it easier to use the results when all [Elements::Global]
    /// elements are at the front of the result set and can be applied before
    /// other elements are read by the end-user.
    fn organize(&mut self) {
        self.results.sort_by(|a, b| {
            let (a, a_is_global) = match a {
                ParseResult::Ok { line_number, data } => (
                    line_number,
                    match data {
                        Elements::Global(_) => true,
                        _ => false,
                    },
                ),
                ParseResult::Err { line_number, .. } => (line_number, false),
            };

            let (b, b_is_global) = match b {
                ParseResult::Ok { line_number, data } => (
                    line_number,
                    match data {
                        Elements::Global(_) => true,
                        _ => false,
                    },
                ),
                ParseResult::Err { line_number, .. } => (line_number, false),
            };

            match (a_is_global, b_is_global) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.cmp(b),
            }
        });
    }

    /// Builds a new string, [Self::building_line], from the input [line].
    /// This accounts for the [Glyphs::Backslash] character in the spec.
    fn process(&mut self, line: &mut String, literals: &Option<Vec<Literal>>) {
        self.total_lines += 1;

        let backslash = Glyphs::Backslash.value() as char;
        if line.ends_with(backslash) {
            *line = line.replace(backslash, "");

            if let Some(ref mut str) = self.building_line {
                *str += line;
            } else {
                self.building_line = Some(line.clone());
            }

            return;
        } else if let Some(ref mut str) = self.building_line {
            *line = str.clone() + line;
        }

        self.building_line = None;

        let mut element_parser = ElementParser::read(self.total_lines, line, &literals);

        if !element_parser.is_ok() {
            self.results.push(ParseResult::error(
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
                element: _,
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

        self.results.push(ParseResult::Ok {
            line_number: self.total_lines,
            data: element_parser
                .element
                .expect("Expected element_parser.is_ok() to signal valid elements."),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{enums::Elements, literal::Literal, ParseResult, YesDocParser};

    #[test]
    fn parse_macro_content() {
        let content = "!macro teardown_textbox(tb) = \"call common.textbox_teardown tb=\"tb";
        let results: Vec<ParseResult> =
            YesDocParser::from_string(content, Some(vec![Literal::build_quotes()]));
        assert_eq!(results.len(), 1);

        let first = results.first();
        assert_eq!(first.is_some(), true);

        let element = match &first.unwrap() {
            ParseResult::Ok {
                line_number: _,
                data: Elements::Global(element),
            } => element,
            _ => panic!("Global expected!"),
        };
        assert_eq!(element.text, "macro");

        assert_eq!(element.args.first().is_some(), true);
        let arg = element.args.first().unwrap();

        assert_eq!(arg.key.as_ref().unwrap(), "teardown_textbox(tb)");
        assert_eq!(arg.val, "\"call common.textbox_teardown tb=\"tb");
    }

    #[test]
    fn parse_multiline() {
        let content = "var msg: str=\"apple, bananas, coconut, diamond, eggplant\\\n\
            , fig, grape, horse, igloo, joke, kangaroo\\\n\
            , lemon, notebook, mango\"\n\
            var list2: [int]=[1\\\n\
            , 2, 3, 4, 5, 6, 7]";

        let results = YesDocParser::from_string(
            content,
            Some(vec![Literal {
                begin: '[' as u8,
                end: ']' as u8,
            }]),
        );
        assert_eq!(results.len(), 2);

        let first = results.first();
        assert_eq!(first.is_some(), true);

        let element = match &first.unwrap() {
            ParseResult::Ok {
                line_number: _,
                data: Elements::Standard { attrs: _, element },
            } => element,
            _ => panic!("Standard element expected!"),
        };

        assert_eq!(element.text, "var");
        assert_eq!(element.args.len(), 2);

        assert_eq!(element.args.first().is_some(), true);
        let arg = element.args.first().unwrap();
        assert_eq!(arg.val, "msg:");

        let arg2 = element.args.iter().nth(1).unwrap();
        assert_eq!(arg2.key.is_some(), true);
        assert_eq!(arg2.key.as_ref().unwrap(), "str");
        assert_eq!(arg2.val.len(), 108);

        let second = results.iter().nth(1);
        assert_eq!(second.is_some(), true);

        let element = match &second.unwrap() {
            ParseResult::Ok {
                line_number: _,
                data: Elements::Standard { attrs: _, element },
            } => element,
            _ => panic!("Standard element expected!"),
        };

        assert_eq!(element.text, "var");
        assert_eq!(element.args.len(), 2);

        assert_eq!(element.args.first().is_some(), true);
        let arg = element.args.first().unwrap();
        assert_eq!(arg.val, "list2:");

        let arg2 = element.args.iter().nth(1).unwrap();
        assert_eq!(arg2.key.is_some(), true);
        assert_eq!(arg2.key.as_ref().unwrap(), "[int]");
        assert_eq!(arg2.val.len(), 21);
    }

    #[test]
    fn delimiter_test1() {
        let content = "x a=b -c";
        let results = YesDocParser::from_string(content, None);
        assert_eq!(results.len(), 1);

        let first = results.first();
        assert_eq!(first.is_some(), true);

        let element = match &first.unwrap() {
            ParseResult::Ok {
                line_number: _,
                data: Elements::Standard { attrs: _, element },
            } => element,
            _ => panic!("Standard element expected!"),
        };

        assert_eq!(element.text, "x");
        assert_eq!(element.args.len(), 2);

        let arg1 = element.args.iter().nth(0).unwrap();
        assert_eq!(arg1.key.is_some(), true);
        assert_eq!(arg1.key.as_ref().unwrap(), "a");
        assert_eq!(arg1.val, "b");

        let arg2 = element.args.iter().nth(1).unwrap();
        assert_eq!(arg2.key.is_none(), true);
        assert_eq!(arg2.val, "-c");
    }

    #[test]
    fn comma_delimiter_test() {
        let content = "frame duration = 1.0s , width = 10, height=20";
        let results = YesDocParser::from_string(content, None);
        assert_eq!(results.len(), 1);

        for result in &results {
            match result {
                ParseResult::Ok { line_number, data } => println!("#{}: {}", line_number, data),
                ParseResult::Err {
                    line_number,
                    message,
                    ..
                } => println!("There was an error on line #{}: {}!", line_number, message),
            }
        }

        let data = if let Some(ref result) = results.first() {
            match result {
                ParseResult::Ok {
                    line_number: _,
                    data: Elements::Standard { attrs: _, element },
                } => element,
                _ => panic!("Standard element expected!"),
            }
        } else {
            panic!("Expected this iterator to have Some() for parsed_elements!");
        };

        assert_eq!(data.text, "frame");

        let args = &data.args;
        assert_eq!(args.len(), 3);

        for arg in args {
            println!("{}", arg);
        }
    }
}
