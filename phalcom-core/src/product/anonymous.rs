//! Runtime descriptor and registry for materialized anonymous Tuple and Record products.
//!
//! Realizes LANG005.C1.P3 Task 5: Joins shape metadata, physical storage layout,
//! and optional exact structural type into a single VM-interned descriptor.

use super::shape::{AnonymousProductKind, ProductShapeId};
use crate::product::ProductLayoutId;
use crate::typing::handle::RuntimeTypeRef;
use std::collections::HashMap;

/// Identifies an interned anonymous product descriptor in [`RuntimeAnonymousProductDescriptorRegistry`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeAnonymousProductDescriptorId(pub u32);

/// Runtime descriptor tying shape + layout + optional exact retained type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAnonymousProductDescriptor {
    pub runtime_id: RuntimeAnonymousProductDescriptorId,
    pub kind: AnonymousProductKind,
    pub shape: ProductShapeId,
    pub layout: ProductLayoutId,
    /// Present only when semantic metadata establishes a retained exact
    /// structural type; never fabricated from runtime payload values.
    pub exact_type: Option<RuntimeTypeRef>,
}

/// VM-level registry managing interned anonymous product descriptors.
#[derive(Clone, Debug, Default)]
pub struct RuntimeAnonymousProductDescriptorRegistry {
    descriptors: Vec<RuntimeAnonymousProductDescriptor>,
    descriptor_by_key: HashMap<(AnonymousProductKind, ProductShapeId, ProductLayoutId, Option<RuntimeTypeRef>), RuntimeAnonymousProductDescriptorId>,
}

impl RuntimeAnonymousProductDescriptorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns an anonymous product descriptor.
    pub fn register(
        &mut self,
        kind: AnonymousProductKind,
        shape: ProductShapeId,
        layout: ProductLayoutId,
        exact_type: Option<RuntimeTypeRef>,
    ) -> RuntimeAnonymousProductDescriptorId {
        let key = (kind, shape, layout, exact_type);
        if let Some(&existing) = self.descriptor_by_key.get(&key) {
            return existing;
        }

        let runtime_id = RuntimeAnonymousProductDescriptorId(self.descriptors.len() as u32);
        let descriptor = RuntimeAnonymousProductDescriptor {
            runtime_id,
            kind,
            shape,
            layout,
            exact_type,
        };

        self.descriptors.push(descriptor);
        self.descriptor_by_key.insert(key, runtime_id);
        runtime_id
    }

    pub fn get(&self, id: RuntimeAnonymousProductDescriptorId) -> Option<&RuntimeAnonymousProductDescriptor> {
        self.descriptors.get(id.0 as usize)
    }
}
