use std::str::FromStr;

use crate::keyval::KeyVal;

pub struct Element {
    text: String,
    args: Vec<KeyVal>,
}

impl Element {
    pub fn new(text: String) -> Element {
        Element {
            text: text,
            args: Vec::new(),
        }
    }

    pub fn upsert(&mut self, kv: KeyVal) {
        // We cannot update nameless keyvals, so we insert as-is.
        if kv.is_nameless() {
            self.args.push(kv);
            return;
        }

        if let Ok(prev) = self.args.binary_search_by(|kv| match &kv.key {
            None => std::cmp::Ordering::Less,
            Some(k) => k.cmp(kv.key.as_ref().unwrap()),
        }) {
            self.args[prev].val = kv.val;
            return;
        }

        // Else, there was no match.
        // Insert new keyval.
        self.args.push(kv);
    }

    pub fn has_key(&self, key: String) -> bool {
        if let Ok(_) = self.args.binary_search_by(|kv| match &kv.key {
            None => std::cmp::Ordering::Less,
            Some(k) => k.cmp(&key),
        }) {
            return true;
        }

        false
    }

    pub fn has_keys(&self, keys: Vec<String>) -> bool {
        for iter in keys.into_iter() {
            if self.has_key(iter) {
                return true;
            }
        }

        return false;
    }

    pub fn get_key_value<T>(&self, key: String) -> Option<T>
    where
        T: FromStr,
    {
        if let Ok(idx) = self.args.binary_search_by(|kv| match &kv.key {
            None => std::cmp::Ordering::Less,
            Some(k) => k.cmp(&key),
        }) {
            return match self.args[idx].val.parse::<T>() {
                Ok(t) => Some(t),
                Err(_) => None,
            };
        }

        None
    }

    pub fn get_key_value_or<T>(&self, key: String, or: T) -> T
    where
        T: FromStr,
    {
        if let Ok(idx) = self.args.binary_search_by(|kv| match &kv.key {
            None => std::cmp::Ordering::Less,
            Some(k) => k.cmp(&key),
        }) {
            return match self.args[idx].val.parse::<T>() {
                Ok(t) => t,
                Err(_) => or,
            };
        }

        or
    }
}
