use crate::enums::Glyphs;

pub trait StringUtils {
    fn is_quoted(&self) -> bool;
    fn quote(&mut self) -> &mut Self;
    fn unquote(&mut self) -> &mut Self;
    fn ltrim(&mut self) -> &mut Self;
    fn rtrim(&mut self) -> &mut Self;
    fn trim(&mut self) -> &mut Self;
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn is_quoted(&self) -> bool {
        let c = Some(Glyphs::Quote.value());
        let mut b = self.as_str().bytes();
        b.len() > 0 && b.nth(0) == c && b.nth(b.len() - 1) == c
    }

    fn quote(&mut self) -> &mut String {
        if !self.is_quoted() {
            let c = Glyphs::Quote.value() as char;
            let mut buf: String = String::new();
            buf.push(c);
            buf.push_str(self);
            buf.push(c);
            *self = buf;
        }
        self
    }

    fn unquote(&mut self) -> &mut String {
        if self.is_quoted() {
            *self = self.substring(1, self.len() - 2)
        }
        self
    }

    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len).collect()
    }

    fn ltrim(&mut self) -> &mut Self {
        let b = self.as_str().bytes();
        let _: Vec<u8> = b
            .take_while(|c| *c == Glyphs::Space.value() as u8)
            .collect();
        self
    }

    fn rtrim(&mut self) -> &mut Self {
        let b = self.as_str().bytes().rev();
        let _: Vec<u8> = b
            .take_while(|c| *c == Glyphs::Space.value() as u8)
            .collect();
        self
    }

    fn trim(&mut self) -> &mut Self {
        self.ltrim();
        self.rtrim();

        self
    }
}
