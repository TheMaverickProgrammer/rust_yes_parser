use crate::enums::Glyphs;

#[derive(Eq, Hash)]
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
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.begin == other.begin && self.end == other.end
    }
}
