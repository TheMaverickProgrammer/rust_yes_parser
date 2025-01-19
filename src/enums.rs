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

pub enum Elements {
    Standard { attrs: Vec<Element>, data: Element },
    Attribute(Element),
    Global(Element),
    Comment(Element),
}

impl Elements {
    pub fn new_standard(label: String) -> Elements {
        Elements::Standard {
            attrs: Vec::new(),
            data: Element::new(label),
        }
    }

    pub fn new_attribute(label: String) -> Elements {
        Elements::Attribute(Element::new(label))
    }

    pub fn new_global(label: String) -> Elements {
        Elements::Global(Element::new(label))
    }

    pub fn new_comment(message: String) -> Elements {
        Elements::Comment(Element::new(message))
    }

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

    pub fn upsert_keyval(&mut self, keyval: KeyVal) {
        match self {
            Elements::Standard { data, .. } => data.upsert(keyval),
            Elements::Attribute(data) => data.upsert(keyval),
            Elements::Global(data) => data.upsert(keyval),
            Elements::Comment(data) => data.upsert(keyval),
        }
    }
}

impl fmt::Display for Elements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (glyph, element) = match self {
            Elements::Standard { data, .. } => (Glyphs::None, data),
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
