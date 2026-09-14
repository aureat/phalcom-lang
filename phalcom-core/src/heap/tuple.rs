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

use crate::product::{ProductStorage, RuntimeAnonymousProductDescriptorId};

/// A native, fixed-length immutable Tuple product.
///
/// The three VM-blessed floor primitives
/// ([ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md),
/// `phalcom-core/src/primitive/tuple.rs`) operate directly on this buffer;
/// the surfaced `at(_)`/`size`/`each(_)`/`==`/`hash` protocol is defined in
/// `.ph` over those primitives (`tuple-and-range.md` §1).
#[derive(Debug, Clone, PartialEq)]
pub struct TupleObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}

impl TupleObject {
    pub(crate) fn new(descriptor: RuntimeAnonymousProductDescriptorId, storage: ProductStorage) -> Self {
        Self { descriptor, storage }
    }

    pub fn descriptor(&self) -> RuntimeAnonymousProductDescriptorId {
        self.descriptor
    }

    pub fn storage(&self) -> &ProductStorage {
        &self.storage
    }
}
