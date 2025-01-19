use std::fmt;

use crate::utils::StringUtils;

pub struct KeyVal {
    pub key: Option<String>,
    pub val: String,
    key_contains_space: bool,
    value_contains_space: bool,
}

impl KeyVal {
    pub fn new(key: Option<String>, val: String) -> KeyVal {
        KeyVal {
            key,
            val,
            key_contains_space: true,
            value_contains_space: true,
        }
    }

    pub fn is_nameless(&self) -> bool {
        self.key == None
    }
}

impl fmt::Display for KeyVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self.value_contains_space {
            true => {
                let mut clone = self.val.clone();
                clone.quote();
                clone
            }
            false => self.val.clone(),
        };

        if self.is_nameless() {
            return write!(f, "{}", v);
        }

        let k = match self.key_contains_space {
            true => {
                let mut clone = self.key.clone().unwrap();
                clone.quote();
                clone
            }
            false => self.key.clone().unwrap(),
        };

        write!(f, "{}={}", k, v)
    }
}
