//! Immutable dense storage for statically dimensioned Vector values.

use crate::value::Value;

/// Owned, immutable Vector storage. Semantic identity remains in the type layer.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorObject {
    elements: Box<[Value]>,
}

impl VectorObject {
    /// Takes ownership of the final dense payload.
    pub(crate) fn new(elements: Box<[Value]>) -> Self {
        Self { elements }
    }

    /// Returns the immutable storage view.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns whether storage is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Reads an existing element without exposing mutation.
    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }
}
