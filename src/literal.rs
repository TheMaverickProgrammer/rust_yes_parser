use crate::enums::Glyphs;

#[derive(Eq, Hash, Clone)]
pub struct Literal {
    pub begin: u8,
    pub end: u8,
}

impl Literal {
    pub fn new(begin: u8, end: u8) -> Result<Literal, &'static str> {
        if Glyphs::is_reserved(begin) {
            return Err("Literal::begin cannot contain a reserved character.");
        }

        if Glyphs::is_reserved(end) {
            return Err("Literal::end cannot contain a reserved character.");
        }

        Ok(Literal { begin, end })
    }

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
