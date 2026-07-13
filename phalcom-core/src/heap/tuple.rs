//! A native, heap-backed fixed-arity immutable product.
//!
//! Realizes [ADR-0032 §1](../../../docs/adr/0032-collections-representation-and-literals.md)
//! (native heap-arm representation) and
//! [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)
//! (the raw-primitive floor amendment): `Tuple` is a dedicated
//! [`crate::heap::Object::Tuple`] heap variant, mirroring
//! [`crate::heap::ListObject`] — **not** an [`crate::heap::InstanceObject`].
//! Unlike `List`, immutability is a **representation guarantee**: the backing
//! `Box<[Value]>` is a fixed-length slice, and [`TupleObject`] exposes no
//! mutation accessor at all (`docs/spec/v0.2/core/tuple-and-range.md` §1) — a
//! later diff cannot accidentally reintroduce mutation the way a missing
//! selector could.

use crate::value::Value;

/// A native, fixed-length immutable slice of [`Value`]s.
///
/// The three VM-blessed floor primitives
/// ([ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md),
/// `phalcom-core/src/primitive/tuple.rs`) operate directly on this buffer;
/// the surfaced `at(_)`/`size`/`each(_)`/`==`/`hash` protocol is defined in
/// `.ph` over those primitives (`tuple-and-range.md` §1).
#[derive(Debug, Clone, PartialEq)]
pub struct TupleObject {
    /// The tuple's elements, in order — fixed at construction, never resized
    /// or written in place.
    elements: Box<[Value]>,
}

impl TupleObject {
    /// Builds a tuple from an owned, fixed-length element buffer.
    pub fn new(elements: Box<[Value]>) -> Self {
        Self { elements }
    }

    /// Returns the element count (the tuple's fixed arity).
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns `true` if the tuple is the empty tuple `()`.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the element at `index`, or `None` if `index` is out of range.
    ///
    /// The caller (the `at_` primitive) surfaces an out-of-range read as
    /// the kernel `None` singleton, never a panic — mirrors
    /// [`crate::heap::ListObject::get`].
    pub fn get(&self, index: usize) -> Option<Value> {
        self.elements.get(index).copied()
    }

    /// Borrows every element, in order.
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }
}
