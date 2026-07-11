//! A native, heap-backed array of Phalcom [`Value`]s.
//!
//! Realizes [ADR-0020](../../../docs/adr/0020-kernel-list-native-array-protocol.md):
//! `List` is a dedicated [`crate::heap::Object::List`] heap variant — mirroring
//! [`crate::string::StringObject`] — **not** an
//! [`crate::instance::InstanceObject`] built on U7's field-slot layout (U-LIST
//! has no technical dependency on U7). There is no `Rc`/`RefCell`: mutation
//! goes through `&mut Heap` like every other heap object
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)), so there is no
//! borrow-panic surface.

use crate::value::Value;

/// A native array-backed list.
///
/// The six VM-blessed floor primitives
/// ([ADR-0019](../../../docs/adr/0019-freeze-vm-blessed-primitive-floor.md),
/// `phalcom-core/src/primitive/list.rs`) operate directly on this buffer;
/// the surfaced `at(_:)`/`size`/`add(_:)`/`each(_:)` protocol is defined in
/// `.ph` over those primitives (ADR-0020).
#[derive(Debug, Clone, PartialEq)]
pub struct ListObject {
    /// The list's elements, in order.
    elements: Vec<Value>,
}

impl ListObject {
    /// Builds a list object from an owned element buffer.
    pub fn new(elements: Vec<Value>) -> Self {
        Self { elements }
    }

    /// Borrows the list's elements.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }

    /// Returns the element count.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the list has no elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the element at `index`, or `None` if `index` is out of range.
    ///
    /// The caller (the `rawAt` primitive) surfaces an out-of-range read as
    /// the kernel `None` singleton, never a panic — this method just reports
    /// range membership.
    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }

    /// Overwrites the element at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of range. Callers (the `rawSet` primitive)
    /// must bounds-check first and surface a catchable [`crate::error::RuntimeError`]
    /// instead of reaching this panic.
    pub fn set(&mut self, index: usize, value: Value) {
        self.elements[index] = value;
    }

    /// Appends `value` to the end of the list.
    ///
    /// Growth is `Vec`'s ordinary amortized doubling — this is the "grow"
    /// floor primitive from ADR-0020 §3, folded into push rather than
    /// exposed as a separate primitive (see the U-LIST return contract).
    pub fn push(&mut self, value: Value) {
        self.elements.push(value);
    }
}
