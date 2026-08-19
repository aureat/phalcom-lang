//! A native, heap-backed fixed-arity immutable product.
//!
//! Realizes [ADR-0032 §1](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)
//! (native heap-arm representation) and
//! [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! (the raw-primitive floor amendment): `Tuple` is a dedicated
//! [`crate::heap::Object::Tuple`] heap variant, mirroring
//! [`crate::heap::ListObject`] — **not** an [`crate::heap::InstanceObject`].
//! Unlike `List`, immutability is a **representation guarantee**: the backing
//! `Box<[Value]>` is a fixed-length slice, and [`TupleObject`] exposes no
//! mutation accessor at all (`docs/spec/v0.2/core/tuple-and-range.md` §1) — a
//! later diff cannot accidentally reintroduce mutation the way a missing
//! selector could.

use crate::interner::Symbol;
use crate::value::Value;

/// A native, fixed-length immutable slice of [`Value`]s.
///
/// The three VM-blessed floor primitives
/// ([ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md),
/// `phalcom-core/src/primitive/tuple.rs`) operate directly on this buffer;
/// the surfaced `at(_)`/`size`/`each(_)`/`==`/`hash` protocol is defined in
/// `.ph` over those primitives (`tuple-and-range.md` §1).
#[derive(Debug, Clone, PartialEq)]
pub struct TupleObject {
    /// The tuple's elements, in order — fixed at construction, never resized
    /// or written in place.
    values: Box<[Value]>,
    labels: Box<[Symbol]>,
}

impl TupleObject {
    /// Builds a tuple from an owned, fixed-length element buffer.
    pub(crate) fn new(values: Box<[Value]>, labels: Box<[Symbol]>) -> Self {
        assert!(labels.len() <= values.len(), "tuple labels must be a values suffix");
        assert!(
            labels.iter().enumerate().all(|(i, label)| !labels[..i].contains(label)),
            "tuple labels must be unique"
        );
        Self { values, labels }
    }

    /// Builds a positional tuple from a vector of values.
    pub fn positional(values: Vec<Value>) -> Self {
        Self::new(values.into_boxed_slice(), Box::new([]))
    }

    /// Returns the element count (the tuple's fixed arity).
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn positional_len(&self) -> usize {
        self.values.len() - self.labels.len()
    }
    pub fn labeled_len(&self) -> usize {
        self.labels.len()
    }
    pub fn values(&self) -> &[Value] {
        &self.values
    }
    pub fn positionals(&self) -> &[Value] {
        &self.values[..self.positional_len()]
    }
    pub fn labeled_values(&self) -> &[Value] {
        &self.values[self.positional_len()..]
    }
    pub fn labels(&self) -> &[Symbol] {
        &self.labels
    }

    /// Returns the element at `index`, or `None` if `index` is out of range.
    ///
    /// The caller (the `at_` primitive) surfaces an out-of-range read as
    /// immediate `None`, never a panic — mirrors
    /// [`crate::heap::ListObject::get`].
    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.get(index).copied()
    }

    pub fn get_label(&self, label: Symbol) -> Option<Value> {
        self.labels.iter().position(|candidate| *candidate == label).map(|i| self.labeled_values()[i])
    }

    pub fn labeled_entries(&self) -> impl Iterator<Item = (Symbol, Value)> + '_ {
        self.labels.iter().copied().zip(self.labeled_values().iter().copied())
    }
}
