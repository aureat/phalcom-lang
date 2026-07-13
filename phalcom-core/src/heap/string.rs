//! Immutable strings.
//!
//! A [`StringObject`] is stored in the [`Heap`](crate::heap::Heap) as an
//! [`Object::Str`](crate::heap::Object::Str) and referenced from a
//! [`Value::Obj`](crate::value::Value::Obj) handle
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)). Its content is
//! immutable and it caches a djb2 content hash for fast comparison.

/// An immutable string value with a cached content hash.
#[derive(Debug, Clone, PartialEq)]
pub struct StringObject {
    /// The string's UTF-8 contents.
    value: String,
    /// Cached djb2 hash of [`Self::value`].
    hash: u32,
}

impl StringObject {
    /// Builds a string object from an owned [`String`], computing its hash.
    pub fn from_string(value: String) -> Self {
        let hash = Self::calculate_hash(&value);
        Self { value, hash }
    }

    /// Builds a string object from a string slice.
    ///
    /// This is an infallible inherent constructor, not the fallible
    /// [`std::str::FromStr::from_str`] trait method, hence the lint allowance.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        Self::from_string(value.to_owned())
    }

    /// Computes the djb2 hash of `value`.
    pub fn calculate_hash(value: &str) -> u32 {
        let mut hash = 5381u32;
        for byte in value.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }

    /// Returns the string's contents as a slice.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns a copy of the string's contents.
    pub fn value(&self) -> String {
        self.value.clone()
    }

    /// Returns the cached content hash.
    pub fn hash(&self) -> u32 {
        self.hash
    }
}
