pub mod element;
pub mod enums;
pub mod keyval;
pub mod parser;
pub mod utils;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
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
}
