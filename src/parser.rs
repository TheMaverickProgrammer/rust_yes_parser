use std::cmp::min;

use crate::{
    enums::{Delimiters, Elements, ErrorCodes, Glyphs},
    keyval::KeyVal,
    utils::StringUtils,
};

#[derive(PartialEq)]
enum ElementTypes {
    Standard,
    Attribute,
    Global,
}

pub struct ElementParser {
    delimiter: Delimiters,
    element: Option<Elements>,
    error: Option<ErrorCodes>,
    line_number: usize,
}

impl ElementParser {
    pub fn is_ok(&self) -> bool {
        match self.error {
            None => true,
            _ => false,
        }
    }

    fn set_error(&mut self, error: ErrorCodes) {
        self.error = Some(error);
    }

    fn set_delimiter(&mut self, delim: Delimiters) {
        if self.delimiter != Delimiters::Unset {
            return;
        }

        self.delimiter = delim;
    }

    pub fn read(line_number: usize, line: &str) -> ElementParser {
        // Step 1: Trim whitespace and start at the first valid character
        let line_slice = line.trim().as_bytes();
        let len = line_slice.len();

        let mut p = ElementParser {
            delimiter: Delimiters::Unset,
            element: None,
            error: None,
            line_number,
        };

        if len == 0 {
            p.set_error(ErrorCodes::EolNoData);
            return p;
        }

        let mut element_type = ElementTypes::Standard;

        let mut pos = 0;
        while pos < len {
            let token = line_slice[pos];
            if token == Glyphs::Space.value() {
                pos = pos + 1;
                continue;
            }

            // We are on our first non-reserved token.
            if !Glyphs::is_reserved(token) {
                break;
            }

            // Step 2: if the first valid character is reserved prefix
            // then tag the element and continue searching for the name start pos
            match Glyphs::from(token) {
                Glyphs::At => {
                    if element_type != ElementTypes::Standard {
                        p.set_error(ErrorCodes::BadTokenPosAttribute);
                        return p;
                    }

                    element_type = ElementTypes::Attribute;
                    pos = pos + 1;
                    continue;
                }
                Glyphs::Bang => {
                    if element_type != ElementTypes::Standard {
                        p.set_error(ErrorCodes::BadTokenPosBang);
                        return p;
                    }

                    element_type = ElementTypes::Global;
                    pos = pos + 1;
                    continue;
                }

                Glyphs::Hash => {
                    if element_type == ElementTypes::Standard {
                        if let Ok(str) = String::from_utf8(line_slice.to_owned()) {
                            p.element = Some(Elements::new_comment(str.substring(pos, len)));
                            return p;
                        }
                    }
                }
                _ => break,
            };
        }

        // Step 3: find end of element name (first space or EOL)
        let end = match line_slice.iter().position(|&b| b == Glyphs::Space.value()) {
            None => len,
            Some(idx) => min(len, idx),
        };

        let name: String;

        if let Ok(str) = String::from_utf8(line_slice.to_owned()) {
            name = str.substring(pos, end).unquote().clone();
        } else {
            p.set_error(ErrorCodes::Runtime);
            return p;
        }

        if name.is_empty() {
            p.set_error(match p.element {
                Some(ref el) => match el {
                    Elements::Attribute(_) => ErrorCodes::EolMissingAttribute,
                    Elements::Global(_) => ErrorCodes::EolMissingGlobal,
                    _ => ErrorCodes::EolMissingElement,
                },
                _ => ErrorCodes::EolMissingElement,
            });

            return p;
        }

        // Comment element case handled already above
        p.element = Some(match element_type {
            ElementTypes::Attribute => Elements::new_attribute(name),
            ElementTypes::Global => Elements::new_global(name),
            _ => Elements::new_standard(name),
        });

        // Step 4: parse tokens, if any and return results
        p.parse_tokens(line_slice, end);
        p
    }

    fn parse_tokens(&mut self, slice: &[u8], start: usize) {
        let mut end = start;

        // Evaluate all tokens on line
        while end < slice.len() {
            end = self.parse_token_step(slice, end + 1);

            // Abort early if there is a problem
            if !self.is_ok() {
                return;
            }
        }
    }

    fn parse_token_step(&mut self, slice: &[u8], mut start: usize) -> usize {
        let len = slice.len();

        // Find first non-space character
        while start < len {
            if slice[start] == Glyphs::Space.value() {
                start = start + 1;
                continue;
            }

            // Current character is non-space
            break;
        }

        if start >= len {
            return len;
        }

        let end = self.evaluate_next_delimiter(slice, start);
        self.evaluate_token(slice, start, end);
        end
    }

    fn evaluate_next_delimiter(&mut self, slice: &[u8], start: usize) -> usize {
        todo!();
    }

    fn evaluate_token(&mut self, slice: &[u8], start: usize, end: usize) {
        let mut expr: String;

        // Trim spaces around the token for keyval assignments expressions.
        // e.g. `key=val`
        if let Ok(str) = String::from_utf8(slice.to_owned()) {
            expr = str.substring(start, end).trim().clone();
        } else {
            self.set_error(ErrorCodes::Runtime);
            return;
        }

        // Edge case: expression is empty or the equal glyph.
        // Treat this as no key and no value.
        if match expr.bytes().nth(0) {
            Some(c) => c == Glyphs::Equal.value(),
            None => true,
        } {
            return;
        }

        let equal = slice.iter().position(|&b| b == Glyphs::Equal.value());

        // Named key values are seperated by equal (=) char
        if let Some(pos) = equal {
            let keyval = KeyVal::new(
                Some(expr.substring(0, pos).trim().unquote().clone()),
                expr.substring(pos + 1, expr.len()).trim().unquote().clone(),
            );

            self.element.as_mut().unwrap().upsert_keyval(keyval);
            return;
        }

        // Upsert the nameless key value
        let keyval = KeyVal::new(None, expr.unquote().to_string());
        self.element.as_mut().unwrap().upsert_keyval(keyval);
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
