//! Bounded, owned storage for initialized Phalcom values.

use crate::value::Value;

/// Failure from mutating a bounded [`BufferObject`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferMutationError {
    /// The logical capacity has already been reached.
    CapacityExceeded,
    /// The requested index is outside the initialized prefix.
    OutOfBounds,
}

/// Owned mutable storage whose logical capacity never changes.
///
/// `elements` contains exactly the initialized prefix. Its allocator capacity
/// is an implementation detail and is never used as the Telos capacity.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferObject {
    logical_capacity: usize,
    elements: Vec<Value>,
}

impl BufferObject {
    /// Creates an empty buffer after reserving storage for its logical capacity.
    ///
    /// # Errors
    ///
    /// Returns the host reservation error without publishing a partial buffer.
    pub fn try_new(logical_capacity: usize) -> Result<Self, std::collections::TryReserveError> {
        let mut elements = Vec::new();
        elements.try_reserve_exact(logical_capacity)?;
        Ok(Self { logical_capacity, elements })
    }

    /// Returns the count of initialized values.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns whether no values have been initialized.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the Telos-visible logical capacity.
    pub fn capacity(&self) -> usize {
        self.logical_capacity
    }

    /// Returns whether the initialized prefix has reached logical capacity.
    pub fn is_full(&self) -> bool {
        self.elements.len() >= self.logical_capacity
    }

    /// Reads one initialized value.
    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }

    /// Replaces one initialized value without changing size or capacity.
    pub fn set(&mut self, index: usize, value: Value) -> Result<(), BufferMutationError> {
        let slot = self.elements.get_mut(index).ok_or(BufferMutationError::OutOfBounds)?;
        *slot = value;
        Ok(())
    }

    /// Appends a value if logical capacity remains.
    ///
    /// A full buffer is left unchanged. Construction reserves enough backing
    /// storage for every successful append, so this cannot grow beyond the
    /// already reserved allocation.
    pub fn push(&mut self, value: Value) -> Result<(), BufferMutationError> {
        if self.is_full() {
            return Err(BufferMutationError::CapacityExceeded);
        }
        self.elements.push(value);
        Ok(())
    }

    /// Borrows exactly the initialized prefix.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }
}

#[cfg(test)]
mod tests {
    use super::{BufferMutationError, BufferObject};
    use crate::value::Value;

    #[test]
    fn zero_capacity_buffer_is_empty_and_cannot_grow() {
        let mut buffer = BufferObject::try_new(0).expect("zero-capacity reservation");
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert!(buffer.is_empty());
        assert!(buffer.is_full());
        assert_eq!(buffer.push(Value::int(1)), Err(BufferMutationError::CapacityExceeded));
        assert!(buffer.elements().is_empty());
    }

    #[test]
    fn push_stops_at_logical_capacity_and_preserves_the_initialized_prefix() {
        let mut buffer = BufferObject::try_new(2).expect("two-value reservation");
        buffer.push(Value::int(10)).expect("first append");
        buffer.push(Value::int(20)).expect("second append");
        assert!(buffer.is_full());
        assert_eq!(buffer.elements(), &[Value::int(10), Value::int(20)]);

        for _ in 0..2 {
            assert_eq!(buffer.push(Value::int(30)), Err(BufferMutationError::CapacityExceeded));
            assert_eq!(buffer.elements(), &[Value::int(10), Value::int(20)]);
        }
    }

    #[test]
    fn successful_pushes_reuse_the_initial_reservation() {
        let mut buffer = BufferObject::try_new(64).expect("bounded reservation");
        let allocation = buffer.elements.as_ptr();
        let reserved = buffer.elements.capacity();
        assert!(reserved >= buffer.capacity());
        for index in 0..64 {
            buffer.push(Value::int(index)).expect("in-capacity append");
            assert_eq!(buffer.elements.as_ptr(), allocation, "push retains backing allocation");
            assert_eq!(buffer.elements.capacity(), reserved, "push does not reserve again");
        }
    }

    #[test]
    fn set_replaces_only_initialized_values() {
        let mut buffer = BufferObject::try_new(3).expect("three-value reservation");
        buffer.push(Value::int(1)).expect("first append");
        buffer.push(Value::int(2)).expect("second append");

        buffer.set(1, Value::int(8)).expect("replace initialized value");
        assert_eq!(buffer.elements(), &[Value::int(1), Value::int(8)]);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(buffer.set(2, Value::int(9)), Err(BufferMutationError::OutOfBounds));
        assert_eq!(buffer.elements(), &[Value::int(1), Value::int(8)]);
    }

    #[test]
    fn unrepresentable_reservation_fails_without_a_buffer() {
        assert!(BufferObject::try_new(usize::MAX).is_err());
    }
}
