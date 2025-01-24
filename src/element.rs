use std::str::FromStr;

use crate::keyval::KeyVal;

/// The data structure [Element] used by all variants internally.
pub struct Element {
    pub text: String,
    pub args: Vec<KeyVal>,
}

impl Element {
    /// Constructs a new [Element] with [Element::text].
    /// [Element::args] will be an empty vector.
    pub fn new(text: String) -> Element {
        Element {
            text,
            args: Vec::new(),
        }
    }

    /// Find an entry in [Element::args] with a matching [KeyVal::key] and
    /// update its [KeyVal::val] field. If no such matching [KeyVal] is found
    /// or if the keyval [kv] is nameless, then simply inserts [kv] into the
    /// back of [Element::args].
    ///
    /// See [KeyVal::is_nameless].
    pub fn upsert(&mut self, kv: KeyVal) {
        // We cannot update nameless keyvals, so we insert as-is.
        if kv.is_nameless() {
            self.args.push(kv);
            return;
        }

        if let Some(prev) = self.args.iter().position(|arg| match &arg.key {
            None => false,
            Some(k) => k == kv.key.as_ref().unwrap(),
        }) {
            self.args[prev].val = kv.val;
            return;
        }

        // Else, there was no match.
        // Insert new keyval.
        self.args.push(kv);
    }

    /// Returns true if there is a [KeyVal] in [Element::args] which has
    /// an identical [KeyVal::key] field as the input [key].
    /// Nameless keyvals will never match and always return false.
    pub fn has_key(&self, key: &String) -> bool {
        if let Some(_) = self.args.iter().position(|arg| match &arg.key {
            None => false,
            Some(k) => k == key,
        }) {
            return true;
        }

        false
    }

    /// Tests every key [String] in [keys] with [Element::has_key].
    /// If and only if they are all found, then this returns true.
    /// If any key fails, then this returns false.
    pub fn has_keys(&self, keys: &Vec<String>) -> bool {
        for iter in keys.into_iter() {
            if !self.has_key(iter) {
                return false;
            }
        }

        return true;
    }

    /// Finds the matching [KeyVal] whose [KeyVal::key] field is [key] and
    /// returns the [KeyVal::val] value coerced into type [T] as [Some].
    ///
    /// If no such key is found, or the value could not be coerced into [T],
    /// then [None] is returned.
    pub fn get_key_value<T>(&self, key: &String) -> Option<T>
    where
        T: FromStr,
    {
        if let Some(idx) = self.args.iter().position(|kv| match &kv.key {
            None => false,
            Some(k) => k == key,
        }) {
            return match self.args[idx].val.parse::<T>() {
                Ok(t) => Some(t),
                Err(_) => None,
            };
        }

        None
    }

    /// A variation of [Element::get_key_value] which accepts an explicit [or]
    /// input value of type [T]. If the former method would return [None], then
    /// this method returns [or].
    pub fn get_key_value_or<T>(&self, key: &String, or: T) -> T
    where
        T: FromStr,
    {
        if let Some(idx) = self.args.iter().position(|kv| match &kv.key {
            None => false,
            Some(k) => k == key,
        }) {
            return match self.args[idx].val.parse::<T>() {
                Ok(t) => t,
                Err(_) => or,
            };
        }

        or
    }
}
