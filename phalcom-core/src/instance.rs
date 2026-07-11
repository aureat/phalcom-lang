//! User-defined object instances.
//!
//! An [`InstanceObject`] is a heap [`Object`](crate::heap::Object): a [`ClassId`]
//! plus a per-instance field table. Its class link is a handle
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md)); its fields hold
//! `Copy` [`Value`]s.

use crate::heap::{ClassId, Heap};
use crate::interner::Symbol;
use crate::value::Value;
use indexmap::IndexMap;

/// An instance of a user-defined class: its class handle and its fields.
#[derive(Debug, Clone)]
pub struct InstanceObject {
    /// Handle to the class this object is an instance of.
    pub class: ClassId,
    /// Instance fields, keyed by name [`Symbol`].
    pub fields: IndexMap<Symbol, Value>,
}

impl InstanceObject {
    /// Creates an instance of `class` with no fields set.
    pub fn new(class: ClassId) -> Self {
        Self {
            class,
            fields: IndexMap::new(),
        }
    }

    /// Renders this instance's debug form, `"<ClassName instance>"`, resolving
    /// the class name through `heap`.
    pub fn to_debug(&self, heap: &Heap) -> String {
        format!("<{} instance>", heap.class(self.class).name)
    }
}
