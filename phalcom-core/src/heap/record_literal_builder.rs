//! Private, GC-traced assembly state for dynamic Record literals.

use crate::interner::Symbol;
use crate::value::Value;

/// Compiler-owned, deliberately unobservable Record construction accumulator.
#[derive(Default)]
pub struct RecordLiteralBuilderObject {
    entries: Vec<(Symbol, Value)>,
}

impl RecordLiteralBuilderObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, label: Symbol, value: Value) {
        self.entries.push((label, value));
    }

    pub fn entries(&self) -> &[(Symbol, Value)] {
        &self.entries
    }

    pub fn take_entries(&mut self) -> Vec<(Symbol, Value)> {
        std::mem::take(&mut self.entries)
    }
}
