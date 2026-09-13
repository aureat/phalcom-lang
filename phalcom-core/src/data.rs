//! Runtime descriptor and registry for materialized data declarations.

use crate::heap::ClassId;
use crate::product::ProductLayoutId;
use crate::typing::handle::RuntimeTypeRef;
use phalcom_semantic::identity::DeclarationId;
use std::collections::HashMap;

/// Identifies an interned concrete data descriptor in [`RuntimeDataRegistry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RuntimeDataDescriptorId(pub u32);

/// Runtime metadata for one concrete specialization of a `data` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDataDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeDataDescriptorId,
    pub behavior_class: ClassId,
    pub exact_type: Option<RuntimeTypeRef>,
    pub layout: ProductLayoutId,
}

/// VM-level registry managing interned data descriptors.
#[derive(Debug, Clone, Default)]
pub struct RuntimeDataRegistry {
    descriptors: Vec<RuntimeDataDescriptor>,
    descriptor_by_key: HashMap<(DeclarationId, Option<RuntimeTypeRef>), RuntimeDataDescriptorId>,
    descriptor_by_class: HashMap<ClassId, RuntimeDataDescriptorId>,
}

impl RuntimeDataRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers/interns a concrete data descriptor for a given semantic owner and exact type.
    pub fn register(
        &mut self,
        semantic_owner: DeclarationId,
        behavior_class: ClassId,
        exact_type: Option<RuntimeTypeRef>,
        layout: ProductLayoutId,
    ) -> RuntimeDataDescriptorId {
        let key = (semantic_owner.clone(), exact_type);
        if let Some(&existing) = self.descriptor_by_key.get(&key) {
            return existing;
        }

        let runtime_id = RuntimeDataDescriptorId(self.descriptors.len() as u32);
        let descriptor = RuntimeDataDescriptor {
            semantic_owner,
            runtime_id,
            behavior_class,
            exact_type,
            layout,
        };

        self.descriptors.push(descriptor);
        self.descriptor_by_key.insert(key, runtime_id);
        self.descriptor_by_class.entry(behavior_class).or_insert(runtime_id);
        runtime_id
    }

    /// Attaches canonical nominal metadata without changing already-issued value identity.
    pub fn bind_nominal_type(&mut self, id: RuntimeDataDescriptorId, exact_type: RuntimeTypeRef) {
        let descriptor = &mut self.descriptors[id.0 as usize];
        descriptor.exact_type = Some(exact_type);
        self.descriptor_by_key.insert((descriptor.semantic_owner.clone(), Some(exact_type)), id);
    }

    /// Looks up a descriptor by its ID.
    pub fn descriptor(&self, id: RuntimeDataDescriptorId) -> Option<&RuntimeDataDescriptor> {
        self.descriptors.get(id.0 as usize)
    }

    /// Looks up a descriptor ID by its behavior class.
    pub fn descriptor_by_behavior_class(&self, class: ClassId) -> Option<RuntimeDataDescriptorId> {
        self.descriptor_by_class.get(&class).copied()
    }

    /// Looks up a descriptor ID by semantic owner declaration.
    pub fn descriptor_by_declaration(&self, owner: &DeclarationId) -> Option<RuntimeDataDescriptorId> {
        self.descriptors.iter().find(|d| &d.semantic_owner == owner).map(|d| d.runtime_id)
    }

    /// Enumerates all behavior classes rooted by this registry for GC root marking.
    pub fn all_behavior_classes(&self) -> impl Iterator<Item = ClassId> + '_ {
        self.descriptors.iter().map(|d| d.behavior_class)
    }
}
