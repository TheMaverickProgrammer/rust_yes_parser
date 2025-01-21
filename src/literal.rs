use crate::enums::Glyphs;

#[derive(Eq, Hash, Clone)]
pub struct Literal {
    pub begin: u8,
    pub end: u8,
}

impl Literal {
    /// Construcs a new [Literal]. Convert [char]s to [u8]s.
    /// If [begin] or [end] are reserved for the YES spec, then
    /// [Glyphs::is_reserved] will cause an [Err] to return.
    pub fn new(begin: u8, end: u8) -> Result<Literal, &'static str> {
        if Glyphs::is_reserved(begin) {
            return Err("Literal::begin cannot contain a reserved character.");
        }

        if Glyphs::is_reserved(end) {
            return Err("Literal::end cannot contain a reserved character.");
        }

        Ok(Literal { begin, end })
    }

    /// Constructs a [Literal] set which represent quoted strings.
    /// e.g. the span of characters between "".
    /// Both [Literal::begin] and [Literal::end] will be set to the value
    /// of [Glyphs::Quote]. This Literal is always passed into the parser.
    pub fn build_quotes() -> Literal {
        Literal {
            begin: Glyphs::Quote.value(),
            end: Glyphs::Quote.value(),
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.begin == other.begin && self.end == other.end
    }
}
