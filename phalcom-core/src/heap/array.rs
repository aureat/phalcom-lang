//! A native, heap-backed fixed-extent sequence of Phalcom [`Value`]s.

use crate::value::Value;

/// Error returned when attempting to mutate an Array out of bounds or when frozen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayMutationError {
    Frozen,
    OutOfBounds,
}

/// A native fixed-extent array of values.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}

impl ArrayObject {
    /// Builds an array from an existing boxed slice.
    pub fn new(elements: Box<[Value]>) -> Self {
        Self {
            elements,
            frozen: false,
        }
    }

    /// Builds an array from an owned `Vec<Value>`.
    pub fn from_vec(elements: Vec<Value>) -> Self {
        Self {
            elements: elements.into_boxed_slice(),
            frozen: false,
        }
    }

    /// Borrows the array's elements.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    /// Returns the element count.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the array has no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the element at `index`, or `None` if out of range.
    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }

    /// Overwrites the element at `index`.
    pub fn set(&mut self, index: usize, value: Value) -> Result<(), ArrayMutationError> {
        if self.frozen {
            return Err(ArrayMutationError::Frozen);
        }
        if let Some(slot) = self.elements.get_mut(index) {
            *slot = value;
            Ok(())
        } else {
            Err(ArrayMutationError::OutOfBounds)
        }
    }

    /// Returns true if the array is frozen (read-only const materialization).
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Marks the array as frozen.
    #[allow(dead_code)]
    pub(crate) fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }
}

/// Compiler/VM-private growable builder used during dynamic spread Array construction.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralBuilderObject {
    elements: Vec<Value>,
}

impl ArrayLiteralBuilderObject {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.elements.push(value);
    }

    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    pub fn finish(self) -> ArrayObject {
        ArrayObject::from_vec(self.elements)
    }

    /// Transfers initialized values exactly once at private finalization.
    pub(crate) fn take_elements(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.elements)
    }
}

impl Default for ArrayLiteralBuilderObject {
    fn default() -> Self {
        Self::new()
    }
}
