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
    /// Declaration component ordinal to exact specialization logical slot.
    /// `None` means the declaration has a fixed, identity-ordered layout.
    pub component_slots: Option<Box<[u32]>>,
}

/// Complete specialization supplied to exact descriptor interning. Identity is
/// assigned by the registry rather than by the caller.
pub struct RuntimeDataSpecialization {
    pub semantic_owner: DeclarationId,
    pub behavior_class: ClassId,
    pub exact_type: RuntimeTypeRef,
    pub layout: ProductLayoutId,
    pub component_slots: Option<Box<[u32]>>,
}

/// VM-level registry managing interned data descriptors.
#[derive(Debug, Clone, Default)]
pub struct RuntimeDataRegistry {
    descriptors: Vec<RuntimeDataDescriptor>,
    descriptor_by_key: HashMap<(DeclarationId, Option<RuntimeTypeRef>), RuntimeDataDescriptorId>,
    descriptor_by_class: HashMap<ClassId, RuntimeDataDescriptorId>,
    behavior_class_by_declaration: HashMap<DeclarationId, ClassId>,
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
        self.register_with_component_slots(semantic_owner, behavior_class, exact_type, layout, None)
    }

    pub fn register_with_component_slots(
        &mut self,
        semantic_owner: DeclarationId,
        behavior_class: ClassId,
        exact_type: Option<RuntimeTypeRef>,
        layout: ProductLayoutId,
        component_slots: Option<Box<[u32]>>,
    ) -> RuntimeDataDescriptorId {
        let behavior_class = self.register_behavior_class(semantic_owner.clone(), behavior_class);
        let key = (semantic_owner.clone(), exact_type);
        if let Some(&existing) = self.descriptor_by_key.get(&key) {
            if component_slots.is_some()
                && let Some(descriptor) = self.descriptors.get_mut(existing.0 as usize)
            {
                descriptor.component_slots = component_slots;
            }
            return existing;
        }

        let runtime_id = RuntimeDataDescriptorId(self.descriptors.len() as u32);
        let descriptor = RuntimeDataDescriptor {
            semantic_owner,
            runtime_id,
            behavior_class,
            exact_type,
            layout,
            component_slots,
        };

        self.descriptors.push(descriptor);
        self.descriptor_by_key.insert(key, runtime_id);
        self.descriptor_by_class.entry(behavior_class).or_insert(runtime_id);
        runtime_id
    }

    /// Interns an exact specialization across metadata/overlay representations.
    /// The first descriptor remains the representative, so already-issued data
    /// values retain their identity. Raw handles cache aliases only; runtime
    /// typing's complete recursive comparison decides equivalence.
    pub fn intern_exact_with_component_slots(
        &mut self,
        descriptor: RuntimeDataSpecialization,
        context: &crate::typing::TypingContextData,
        typing: &crate::typing::RuntimeTypingRegistry,
    ) -> Result<RuntimeDataDescriptorId, &'static str> {
        let exact_type = descriptor.exact_type;
        let key = (descriptor.semantic_owner.clone(), Some(exact_type));
        let existing = self.descriptor_by_key.get(&key).copied().or_else(|| {
            self.descriptors
                .iter()
                .find(|candidate| {
                    candidate.semantic_owner == descriptor.semantic_owner
                        && candidate
                            .exact_type
                            .is_some_and(|ty| crate::typing::identity::equivalent(context, typing, ty, exact_type))
                })
                .map(|candidate| candidate.runtime_id)
        });
        if let Some(id) = existing {
            let canonical = &self.descriptors[id.0 as usize];
            // Direct closed lowering may use compact lanes while a generic
            // body uses Value lanes. Construction consumes the representative's
            // layout, as with the raw-handle interner, so physical layout is
            // deliberately not part of exact type identity.
            if canonical.component_slots != descriptor.component_slots {
                return Err("equivalent exact data types have inconsistent component mappings");
            }
            self.descriptor_by_key.insert(key, id);
            return Ok(id);
        }
        Ok(self.register_with_component_slots(
            descriptor.semantic_owner,
            descriptor.behavior_class,
            Some(exact_type),
            descriptor.layout,
            descriptor.component_slots,
        ))
    }

    /// Registers one declaration's shared behavior class independently of any
    /// fixed or specialized product layout.
    pub fn register_behavior_class(&mut self, owner: DeclarationId, class: ClassId) -> ClassId {
        *self.behavior_class_by_declaration.entry(owner).or_insert(class)
    }

    pub fn behavior_class_for_declaration(&self, owner: &DeclarationId) -> Option<ClassId> {
        self.behavior_class_by_declaration.get(owner).copied()
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

    /// Enumerates the canonical behavior descriptors, including their
    /// semantic owners, for runtime publication of source-owned data values.
    pub fn descriptors(&self) -> impl Iterator<Item = &RuntimeDataDescriptor> {
        self.descriptors.iter()
    }

    /// Enumerates all behavior classes rooted by this registry for GC root marking.
    pub fn all_behavior_classes(&self) -> impl Iterator<Item = ClassId> + '_ {
        self.behavior_class_by_declaration.values().copied()
    }
}
