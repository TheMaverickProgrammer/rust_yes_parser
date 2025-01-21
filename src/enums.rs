use std::fmt;

use crate::{element::Element, keyval::KeyVal};

#[derive(PartialEq)]
pub enum Delimiters {
    Unset,
    Comma,
    Space,
}

impl Delimiters {
    pub fn value(&self) -> u8 {
        match *self {
            Delimiters::Unset => 0,
            Delimiters::Comma => ',' as u8,
            Delimiters::Space => ' ' as u8,
        }
    }
}

/// [Elements] represent the four possible element types in the
/// YES spec.
/// - [Elements::Standard] are elements whose purpose is user-defined.
/// - [Elements::Attribute] are elements which will be associated automatically
///     with the next valid [Elements::Standard] element. Attributes stack.
/// - [Elements::Global] are elements which will be hoisted to the top of the
///     parsed document result and should impact the document globally.
/// - [Elements::Comment] is documentation put in place by a tool or person.
///
/// Typically the parser is responsible for assembling these variants.
/// See the implemenation for ways to construct a new variant.
pub enum Elements {
    Standard {
        attrs: Vec<Element>,
        element: Element,
    },
    Attribute(Element),
    Global(Element),
    Comment(Element),
}

impl Elements {
    /// Constructs a new [Elements::Standard] with [label] to be identified
    /// with later. The initial [Elements::Standard::attrs] vector is empty.
    pub fn new_standard(label: String) -> Elements {
        Elements::Standard {
            attrs: Vec::new(),
            element: Element::new(label),
        }
    }

    /// Constructs a new [Elements::Attribute] with [label] to be identified
    /// with later.
    pub fn new_attribute(label: String) -> Elements {
        Elements::Attribute(Element::new(label))
    }

    /// Constructs a new [Elements::Global] with [label] to be identified
    /// with later.
    pub fn new_global(label: String) -> Elements {
        Elements::Global(Element::new(label))
    }

    /// Constructs a new [Elements::Comment] with a [message].
    pub fn new_comment(message: String) -> Elements {
        Elements::Comment(Element::new(message))
    }

    /// Returns a copy of the data structure [Element].
    pub fn copy(other: &Element) -> Element {
        let mut args = Vec::new();
        for kv in &other.args {
            args.push(KeyVal::copy(&kv));
        }
        Element {
            text: other.text.clone(),
            args,
        }
    }

    /// Simplifies a call to the data structure [Element] by pattern matching.
    /// See [Element::upsert].
    pub fn upsert_keyval(&mut self, keyval: KeyVal) {
        match self {
            Elements::Standard { element: data, .. } => data.upsert(keyval),
            Elements::Attribute(data) => data.upsert(keyval),
            Elements::Global(data) => data.upsert(keyval),
            Elements::Comment(data) => data.upsert(keyval),
        }
    }
}

impl fmt::Display for Elements {
    /// Prints the element with its associated prefix character, if any, and
    /// all keyvals, if any.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (glyph, element) = match self {
            Elements::Standard { element: data, .. } => (Glyphs::None, data),
            Elements::Attribute(data) => (Glyphs::At, data),
            Elements::Global(data) => (Glyphs::Bang, data),
            Elements::Comment(data) => (Glyphs::Hash, data),
        };

        let char_glyph = glyph.value() as char;
        if element.args.is_empty() {
            write!(f, "{}{}", char_glyph, element.text)
        } else {
            let args_len = element.args.len();
            let mut args = String::new();
            for i in 0..args_len {
                args += &element.args[i].to_string();
                if i < args_len - 1 {
                    args += ", ";
                }
            }

            write!(f, "{}{} {}", char_glyph, element.text, args)
        }
    }
}

#[derive(PartialEq)]
pub enum Glyphs {
    None,
    Equal,
    At,
    Bang,
    Hash,
    Space,
    Comma,
    Quote,
    Backslash,
}

impl Glyphs {
    pub fn value(&self) -> u8 {
        match *self {
            Glyphs::At => '@' as u8,
            Glyphs::Bang => '!' as u8,
            Glyphs::Comma => ',' as u8,
            Glyphs::Equal => '=' as u8,
            Glyphs::Hash => '#' as u8,
            Glyphs::None => 0,
            Glyphs::Quote => '"' as u8,
            Glyphs::Space => ' ' as u8,
            Glyphs::Backslash => '\\' as u8,
        }
    }

    pub fn from(char: u8) -> Glyphs {
        match char {
            val if val == '@' as u8 => Glyphs::At,
            val if val == '!' as u8 => Glyphs::Bang,
            val if val == ',' as u8 => Glyphs::Comma,
            val if val == '=' as u8 => Glyphs::Equal,
            val if val == '#' as u8 => Glyphs::Hash,
            val if val == '"' as u8 => Glyphs::Quote,
            val if val == ' ' as u8 => Glyphs::Space,
            val if val == '\\' as u8 => Glyphs::Backslash,
            _ => Glyphs::None,
        }
    }

    /// If the input [char] is one of the spec-reserved characters,
    /// returns true.
    pub fn is_reserved(char: u8) -> bool {
        match Glyphs::from(char) {
            Glyphs::At => true,
            Glyphs::Bang => true,
            Glyphs::Comma => true,
            Glyphs::Equal => true,
            Glyphs::Hash => true,
            Glyphs::Quote => true,
            _ => false,
        }
    }
}

/// A collection of spec-defined error codes to help inform the end-user
/// why a failure to parse occurred.
///
/// For custom file formats using the spec, a custom error message is desired.
/// For this case, use [ErrorCodes::Runtime].
#[derive(PartialEq)]
pub enum ErrorCodes {
    BadTokenPosAttribute,
    BadTokenPosBang,
    EolNoData,
    EolMissingElement,
    EolMissingAttribute,
    EolMissingGlobal,
    UnterminatedQuote,
    Runtime,
}

impl ErrorCodes {
    /// Return the [str] message associated with this code.
    pub fn values(&self) -> &str {
        match *self {
            ErrorCodes::BadTokenPosAttribute => "Element using attribute prefix out-of-place.",
            ErrorCodes::BadTokenPosBang => "Element using global prefix out-of-place.",
            ErrorCodes::EolNoData => "Nothing to parse (EOL).",
            ErrorCodes::EolMissingElement => "Missing element name (EOL).",
            ErrorCodes::EolMissingAttribute => "Missing attribute name (EOL).",
            ErrorCodes::EolMissingGlobal => "Missing global identifier (EOL).",
            ErrorCodes::UnterminatedQuote => "Missing end quote in expression.",
            ErrorCodes::Runtime => "Unexpected runtime error.",
        }
    }
}
