use crate::element::Element;

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
    Standard(Element),
    Attribute(Element),
    Global(Element),
    Comment(Element),
}

impl Elements {
    pub fn standard() -> Elements {
        todo!()
    }

    pub fn attribute() -> Elements {
        todo!()
    }

    pub fn global() -> Elements {
        todo!()
    }

    pub fn comment() -> Elements {
        todo!()
    }
}

pub enum Glyphs {
    None,
    Equal,
    At,
    Bang,
    Hash,
    Space,
    Comma,
    Quote,
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
        }
    }
}
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
