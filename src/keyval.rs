use crate::{enums::Glyphs, utils::StringUtils};

pub struct KeyVal {
    pub key: Option<String>,
    pub val: String,
    key_contains_space: bool,
    value_contains_space: bool,
}

impl KeyVal {
    pub fn new(key: Option<String>, val: String) -> KeyVal {
        KeyVal {
            key_contains_space: match key {
                None => false,
                Some(ref k) => k.find(|x| x == Glyphs::Space.value() as char).is_some(),
            },
            value_contains_space: val.find(|x| x == Glyphs::Space.value() as char).is_some(),
            key,
            val,
        }
    }

    pub fn copy(other: &KeyVal) -> KeyVal {
        KeyVal::new(other.key.clone(), other.val.clone())
    }

    pub fn is_nameless(&self) -> bool {
        self.key == None
    }
}

impl ToString for KeyVal {
    fn to_string(&self) -> String {
        let v = match self.value_contains_space {
            true => {
                let mut clone = self.val.clone();
                clone.quote();
                clone
            }
            false => self.val.clone(),
        };

        if self.is_nameless() {
            return format!("{}", v);
        }

        let k = match self.key_contains_space {
            true => {
                let mut clone = self.key.clone().unwrap();
                clone.quote();
                clone
            }
            false => self.key.clone().unwrap(),
        };

        format!("{}={}", k, v)
    }
}