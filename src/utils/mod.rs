pub trait StringUtils {
    fn is_quoted(&self) -> bool;
    fn quote(&mut self) -> &Self;
    fn unquote(&mut self) -> &Self;
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn is_quoted(&self) -> bool {
        let c = Some('"' as u8);
        let mut b = self.as_str().bytes();
        b.len() > 0 && b.nth(0) == c && b.nth(b.len() - 1) == c
    }

    fn quote(&mut self) -> &String {
        if !self.is_quoted() {
            let c = '"';
            let mut buf: String = String::new();
            buf.push(c);
            buf.push_str(self);
            buf.push(c);
            *self = buf;
        }
        self
    }

    fn unquote(&mut self) -> &String {
        if self.is_quoted() {
            *self = self.substring(1, self.len() - 2)
        }
        self
    }

    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len).collect()
    }
}
