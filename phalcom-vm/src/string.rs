use phalcom_common::PhRef;

#[derive(Debug, Clone, PartialEq)]
pub struct StringObject {
    value: String,
    hash: u32,
}

impl StringObject {
    pub fn from_string(value: String) -> Self {
        let hash = Self::calculate_hash(&value);
        Self { value, hash }
    }

    pub fn from_str(value: &str) -> Self {
        Self::from_string(value.to_owned())
    }

    pub fn calculate_hash(value: &str) -> u32 {
        let mut hash = 5381;
        for byte in value.bytes() {
            hash = ((hash << 5) + hash) + byte as u32; // hash * 33 + byte
        }
        hash
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }
}

pub type PhString = PhRef<StringObject>;
