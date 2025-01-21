use std::{cmp::min, collections::HashMap, usize};

use crate::{
    enums::{Delimiters, Elements, ErrorCodes, Glyphs},
    keyval::KeyVal,
    literal::Literal,
    utils::StringUtils,
};

/// [ElementTypes] is a structure used to assist [ElementParser::read].
#[derive(PartialEq)]
enum ElementTypes {
    Standard,
    Attribute,
    Global,
}

/// [TokenWalkInfo] is a structure used to assist [ElementParser::evaluateKeyVals].
struct TokenWalkInfo {
    /// This is the [String]] to be evaluated into a valid [KeyVal] pair.
    pub data: String,

    /// If non-zero, this is the [TokenWalkInfo::data] index of the [Glyphs::Equal] symbol.
    pub pivot: Option<usize>,
}

impl TokenWalkInfo {
    pub fn has_pivot(&self) -> bool {
        if let Some(_) = self.pivot {
            return true;
        }

        false
    }

    fn calc_pivot(a: Option<usize>, b: usize) -> Option<usize> {
        if let Some(x) = a {
            if x < b {
                return None;
            } else {
                return Some(x - b);
            }
        }

        return None;
    }
}

pub struct ElementParser {
    delimiter: Delimiters,
    pub element: Option<Elements>,
    pub error: Option<ErrorCodes>,
    pub line_number: usize,
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

    pub fn read(line_number: usize, line: &str, literals: &Option<Vec<Literal>>) -> ElementParser {
        // Step 1: Trim whitespace and start at the first valid character
        let slice = line.trim().as_bytes();
        let len = slice.len();

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
            let c = slice[pos];

            // Find first non-space character.
            if c == Glyphs::Space.value() {
                pos += 1;
                continue;
            }

            // We are on our first non-reserved character.
            if !Glyphs::is_reserved(c) {
                break;
            }

            // Step 2: if the first valid character is reserved prefix
            // then tag the element and continue searching for the name start pos
            match Glyphs::from(c) {
                Glyphs::At => {
                    if element_type != ElementTypes::Standard {
                        p.set_error(ErrorCodes::BadTokenPosAttribute);
                        return p;
                    }

                    element_type = ElementTypes::Attribute;
                    pos += 1;
                    continue;
                }
                Glyphs::Bang => {
                    if element_type != ElementTypes::Standard {
                        p.set_error(ErrorCodes::BadTokenPosBang);
                        return p;
                    }

                    element_type = ElementTypes::Global;
                    pos += 1;
                    continue;
                }
                Glyphs::Hash => {
                    if element_type == ElementTypes::Standard {
                        if let Ok(str) = String::from_utf8(slice.to_owned()) {
                            p.element = Some(Elements::new_comment(str.substring(pos + 1, len)));
                            return p;
                        }
                    }
                }
                _ => break,
            };
        }

        // Step 3: find end of element name (first space or EOL)
        let end = match slice.iter().position(|&b| b == Glyphs::Space.value()) {
            None => len,
            Some(idx) => min(len, idx),
        };

        let name: String;
        if let Ok(str) = String::from_utf8(slice.to_owned()) {
            name = str.substring(pos, end - pos).unquote().clone();
        } else {
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
        p.parse_tokens(slice, end, &literals);
        p
    }

    fn parse_tokens(&mut self, slice: &[u8], mut start: usize, literals: &Option<Vec<Literal>>) {
        let len = slice.len();

        // Find first non-space character
        while start < len {
            if slice[start] == Glyphs::Space.value() {
                start += 1;
                continue;
            }

            // Current character is non-space
            break;
        }

        if start >= len {
            return;
        }

        // Collect and then evaluate all KeyVal args
        let walk_info = self.collect_tokens(slice, start, &literals);
        self.evaluate_keyvals(walk_info);
    }

    fn collect_tokens(
        &mut self,
        slice: &[u8],
        start: usize,
        literals: &Option<Vec<Literal>>,
    ) -> Vec<TokenWalkInfo> {
        let mut ud_literals = HashMap::<&Literal, Option<usize>>::new();

        // Populate our table with the provided literals, if any.
        // Initially, they're mapped value will be None.
        match literals {
            Some(ref list) => {
                for literal in list {
                    ud_literals.insert(literal, None);
                }
            }
            None => (),
        }

        let len = slice.len();
        let mut curr = start;
        let mut tokens = Vec::new();

        // Step 1: Learn appropriate delimiter by iterating over tokens
        // in search for the first comma. [literals] cause the [current]
        // index to jump to the matching [Literal.end] character and resumes
        // iterating normally.
        //
        // If EOL is reached, comma is chosen to be the delimiter so that
        // tokens with one [KeyVal] argument can have spaces around it,
        // since it is the case when it is obvious there are no other
        // arguments to parse.
        let mut space: Option<usize> = None;
        let mut equal: Option<usize> = None;
        let mut equal_count: usize = 0;
        let mut spaces_bf_eq: usize = 0;
        let mut spaces_af_eq: usize = 0;
        let mut tokens_bf_eq: usize = 0;
        let mut tokens_af_eq: usize = 0;
        let mut token_walking = false;
        let mut active_literal: Option<&Literal> = None;

        while curr < len {
            let c = slice[curr];
            let is_comma = Glyphs::Comma.value() == c;
            let is_space = Glyphs::Space.value() == c;
            let is_equal = Glyphs::Equal.value() == c;

            // This variable denotes whether or not the current character
            // is associated with the `active_literal` begin or end values.
            // This can be false while `active_literal` is `Some(x)` value
            // which would represent the case that we are walking a literal
            // string span which has not yet terminated.
            let mut is_literal = false;

            if let Some(ref literal) = active_literal {
                if literal.end == c {
                    is_literal = true;
                }
            } else {
                if !is_space && !is_equal {
                    // The leading equals char determines how the rest of the document
                    // will be parsed when no comma delimiter is set.
                    if !token_walking {
                        if equal == None {
                            tokens_bf_eq += 1;
                        } else {
                            tokens_af_eq += 1;
                        }
                    }

                    token_walking = true;

                    // Clear the spaces metrics.
                    if equal == None {
                        spaces_bf_eq = 0;
                    } else {
                        spaces_af_eq = 0;
                    }
                } else if is_space {
                    if token_walking {
                        // Count spaces before and after equals character.
                        if equal == None {
                            spaces_bf_eq += 1;
                        } else {
                            spaces_af_eq += 1;
                        }
                    }
                    token_walking = false;

                    if space == None {
                        space = Some(curr);
                    }
                } else if is_equal {
                    token_walking = false;

                    if equal == None {
                        equal = Some(curr);
                    }

                    equal_count += 1;
                }

                let mut continue_loop = false;

                for literal in ud_literals.keys() {
                    if literal.begin == c {
                        is_literal = true;
                        active_literal = Some(literal);
                        ud_literals.insert(*literal, Some(curr));

                        curr += 1;
                        continue_loop = true;
                        break;
                    }
                }

                if continue_loop {
                    continue;
                }
            }

            // Ensure literals are terminated before evaluating delimiters.
            if is_literal {
                // If [is_literal] is true, then [active_literal] should
                // never be [Option::None].
                assert!(
                    active_literal != None,
                    "Expected active_literal to be Some() while parsing a literal character!"
                );

                if let Some(ref key) = active_literal {
                    let value = ud_literals
                        .get_mut(key)
                        .expect("Expected key for active_literal to be valid.");

                    // Effectively, these next two conditional branches toggle
                    // whether or not we are reading a literal span.
                    if *value == None {
                        value.replace(curr);
                    } else {
                        value.take();
                        active_literal = None;
                    }
                }

                curr += 1;
                continue;
            }

            // Look ahead for terminating literal
            if let Some(ref key) = active_literal {
                let offset: Option<usize> = slice.iter().skip(curr).position(|&b| b == key.end);
                if let Some(pos) = offset {
                    curr += pos;
                    continue;
                } else {
                    // This loop will never resolve the delimiter because
                    // there is a missing terminating literal.
                    break;
                }
            }

            if is_comma {
                self.set_delimiter(Delimiters::Comma);
                break;
            }

            curr += 1;
        }

        // Edge case: one KeyVal pair can have spaces around them
        // while being parsed correctly per the spec.
        let one_token_exists = equal_count == 1
            && tokens_bf_eq == 1
            && tokens_af_eq <= 1
            && spaces_bf_eq.abs_diff(spaces_af_eq) <= 1
            && space != None;

        // EOL with no comma delimiter found.
        if self.delimiter == Delimiters::Unset {
            if one_token_exists {
                // Edge case #2: no delimiter was found
                // and only **one** key provided, which means
                // the KeyVal pair is likely to be surrounded by
                // whitespace and should be permitted. The Comma
                // delimiter allows for surrounding whitespace.
                self.set_delimiter(Delimiters::Comma);
            } else {
                // No space token found so there is no other delimiter.
                // Spaces will be used.
                self.set_delimiter(Delimiters::Space);
            }
        }

        // Step 2: Use learned delimiter to collect the tokens
        curr = start;
        equal = None;
        active_literal = None;
        let mut last_token_idx = start;

        while curr < len {
            let c = slice[curr];
            let is_equal = Glyphs::Equal.value() == c;
            let is_delim = self.delimiter.value() == c;

            let mut is_literal = false;
            if let Some(ref literal) = active_literal {
                // Test if this is the matching end literal.
                if literal.end == c {
                    is_literal = true
                }
            } else {
                // An equal glyph was found outside a string literal.
                // Track it to help with token parsing later.
                if is_equal {
                    equal = Some(curr);
                    curr += 1;
                    continue;
                }

                // No active literal span indicates this delimiter is valid.
                if is_delim {
                    if let Ok(ref str) = String::from_utf8(slice.to_vec()) {
                        tokens.push(TokenWalkInfo {
                            data: str.substring(last_token_idx, curr - last_token_idx),
                            pivot: TokenWalkInfo::calc_pivot(equal, last_token_idx),
                        });
                    }

                    curr += 1;
                    last_token_idx = curr;
                    continue;
                }

                // Test all literals to determine if we begin a string span
                for literal in ud_literals.keys() {
                    if literal.begin == c {
                        is_literal = true;
                        active_literal = Some(literal);
                        break;
                    }
                }
            }

            // Ensure literals are terminated before evaluating delimiters.
            if is_literal {
                assert!(
                    active_literal != None,
                    "Expected active_literal to be Some() while parsing a literal character!"
                );

                if let Some(ref key) = active_literal {
                    let value = ud_literals
                        .get_mut(key)
                        .expect("Expected key for active_literal to be valid.");

                    if *value == None {
                        value.replace(curr);
                    } else {
                        value.take();
                        active_literal = None;
                    }
                }

                curr += 1;
                continue;
            }

            // Look ahead for terminating literal
            if let Some(ref key) = active_literal {
                let offset: Option<usize> = slice.iter().skip(curr).position(|&b| b == key.end);
                if let Some(pos) = offset {
                    curr += pos;
                    continue;
                } else {
                    // This loop will never resolve the delimiter because
                    // there is a missing terminating literal.
                    break;
                }
            }

            // Advance and repeat the loop
            curr += 1;
        }

        // There was a pending token remaining that was not terminated.
        if last_token_idx < len {
            if let Ok(ref str) = String::from_utf8(slice.to_vec()) {
                tokens.push(TokenWalkInfo {
                    data: str.substring(last_token_idx, len - last_token_idx),
                    pivot: TokenWalkInfo::calc_pivot(equal, last_token_idx),
                });
            }
        }

        tokens
    }

    fn evaluate_keyvals(&mut self, tokens: Vec<TokenWalkInfo>) {
        for token in tokens {
            // Edge case: token is just the equal chararacter.
            // Treat this as no key and no value.
            if let Some(&c) = token.data.as_bytes().first() {
                if c == Glyphs::Equal.value() {
                    continue;
                }
            }

            let len = token.data.len();
            // Named key values are seperated by equal (=) char.
            if token.has_pivot() {
                let keyval = KeyVal::new(
                    Some(
                        token
                            .data
                            .substring(0, token.pivot.unwrap())
                            .trim()
                            .unquote()
                            .clone(),
                    ),
                    token
                        .data
                        .substring(token.pivot.unwrap() + 1, len - token.pivot.unwrap())
                        .trim()
                        .unquote()
                        .clone(),
                );

                self.element.as_mut().unwrap().upsert_keyval(keyval);
                continue;
            }

            // Upsert the nameless key value
            let keyval = KeyVal::new(None, token.data.clone().trim().unquote().to_string());
            self.element.as_mut().unwrap().upsert_keyval(keyval);
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
