use crate::enums::Glyphs;

/// Common [String] utils that are used to simplify parsing.
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
    /// Returns true if the [String] is surrounded by quotes "".
    /// If the [String] has surrounding whitespace, this will return false.
    /// Call [Self::trim] to be sure [self] has no surrounding whitespace.
    fn is_quoted(&self) -> bool {
        let c = Some(Glyphs::Quote.value());
        let mut b = self.as_str().bytes();
        b.len() > 0 && b.nth(0) == c && b.nth(b.len() - 1) == c
    }

    /// If the [String] is not already surrended by quotes "", then
    /// this will add quote characters to the front and back of [self].
    /// If [self] is already surrounded by quotes, this is a no-op.
    /// See [Self::is_quoted].
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

    /// If the [String] is surrended by quotes "", then this will remove
    /// the quote characters at the front and back of [self].
    /// If [self] is already unquoted, this is a no-op.
    /// See [Self::is_quoted].
    fn unquote(&mut self) -> &mut String {
        if self.is_quoted() {
            *self = self.substring(1, self.len() - 2)
        }
        self
    }

    /// Returns a copy of [self] with a subset of the contents
    /// starting from [start] to [start+len].
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len).collect()
    }

    /// While [self] has leading whitespace, those space characters are
    /// consumed and [self] is modified in-place.
    ///
    /// If the first character of [self] is not a whitespace token, then
    /// this is a no-op.
    fn ltrim(&mut self) -> &mut Self {
        let b = self.as_str().bytes().enumerate();

        let mut substr = None;
        for (i, c) in b {
            if c != Glyphs::Space.value() as u8 {
                substr = Some(self.substring(i, self.len() - i));
                break;
            }
        }

        if let Some(s) = substr {
            *self = s
        }

        self
    }

    /// While [self] has trailing whitespace, those space characters are
    /// consumed and [self] is modified in-place.
    ///
    /// If the last character of [self] is not a whitespace token, then
    /// this is a no-op.
    fn rtrim(&mut self) -> &mut Self {
        let b = self.as_str().bytes().enumerate().rev();

        let mut substr = None;
        for (i, c) in b {
            if c != Glyphs::Space.value() as u8 {
                substr = Some(self.substring(0, i + 1));
                break;
            }
        }

        if let Some(s) = substr {
            *self = s
        }

        self
    }

    /// Clears all whitespace surrounding [self], if any.
    /// See [Self::ltrim] and [Self::rtrim].
    fn trim(&mut self) -> &mut Self {
        self.ltrim();
        self.rtrim();

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::StringUtils;

    #[test]
    fn is_quoted() {
        let hwq = "\"Hello, world!\"";
        let str: String = hwq.to_owned();
        assert_eq!(str.is_quoted(), true);
    }

    #[test]
    fn quote_string() {
        let hw = "Hello, world!";
        let hwq = "\"Hello, world!\"";
        let mut str: String = hw.to_owned();
        assert_eq!(str.quote(), hwq);
    }

    #[test]
    fn unquote_string() {
        let hw = "Hello, world!";
        let mut str: String = hw.to_owned();
        str.quote();
        assert_eq!(str.unquote(), hw);
    }

    #[test]
    fn substring() {
        let hw = "Hello, world!";
        let str: String = hw.to_owned();
        assert_eq!(str.substring(7, 5), "world");
    }

    #[test]
    fn trim() {
        let hw = "Hello, world!";
        let mut padded_hw = "   Hello, world!    ".to_owned();
        let mut str = hw.to_owned();
        assert_eq!(str.trim(), hw);
        assert_eq!(padded_hw.trim(), hw);
    }
}
