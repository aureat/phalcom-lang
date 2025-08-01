use phalcom_common::{phref_new, PhRef};

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
        let mut hash = 5381u32;
        for byte in value.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
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

pub fn phstring_new(value: String) -> PhString {
    phref_new(StringObject::from_string(value))
}
