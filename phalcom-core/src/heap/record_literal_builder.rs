//! Private, GC-traced assembly state for dynamic Record literals.

use crate::value::Value;
use phalcom_semantic::field_key::FieldKey;

/// Compiler-owned, deliberately unobservable Record construction accumulator.
#[derive(Default)]
pub struct RecordLiteralBuilderObject {
    entries: Vec<(FieldKey, Value)>,
}

impl RecordLiteralBuilderObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, key: FieldKey, value: Value) {
        self.entries.push((key, value));
    }

    pub fn contains_key(&self, key: &FieldKey) -> bool {
        self.entries.iter().any(|(candidate, _)| candidate == key)
    }

    pub fn entries(&self) -> &[(FieldKey, Value)] {
        &self.entries
    }

    pub fn take_entries(&mut self) -> Vec<(FieldKey, Value)> {
        std::mem::take(&mut self.entries)
    }
}
