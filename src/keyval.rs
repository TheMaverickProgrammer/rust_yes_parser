pub struct KeyVal {
    pub key: Option<String>,
    pub val: String,
    keyContainsSpace: bool,
    valueContainsSpace: bool,
}

impl KeyVal {
    pub fn new(key: Option<String>, val: String) -> KeyVal {
        KeyVal {
            key,
            val,
            keyContainsSpace: true,
            valueContainsSpace: true,
        }
    }

    pub fn is_nameless(&self) -> bool {
        self.key == None
    }
}
