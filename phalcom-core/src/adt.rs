//! Runtime ADT Identities, Descriptors, and Registry (Part 4).
//!
//! Separates static semantic identities (`VariantId`) from VM-local runtime
//! identities (`RuntimeVariantId`) and physical layout tags (`CaseDiscriminant`).
//! Invariant I-RT-1: `VariantId != RuntimeVariantId != CaseDiscriminant`.

use crate::heap::ClassId;
use crate::value::Value;
use phalcom_modules::DeclarationId;
use phalcom_semantic::identity::VariantId;
use std::collections::HashMap;

/// VM-local runtime identity of an enum root.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeEnumId(pub u32);

impl RuntimeEnumId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// VM-local runtime identity of an ADT variant.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeVariantId(pub u32);

impl RuntimeVariantId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Physical dense discriminant assigned per enum in declaration order (0, 1, 2, ...).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CaseDiscriminant(pub u32);

impl CaseDiscriminant {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Runtime shape of a variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeVariantShape {
    Singleton,
    Constructor,
}

/// Runtime descriptor for an enum root.
#[derive(Clone, Debug)]
pub struct RuntimeEnumDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeEnumId,
    pub root_class: ClassId,
    pub variants: Vec<RuntimeVariantId>,
}

/// Runtime descriptor for an ADT variant.
#[derive(Clone, Debug)]
pub struct RuntimeVariantDescriptor {
    pub semantic_id: VariantId,
    pub runtime_id: RuntimeVariantId,
    pub enum_id: RuntimeEnumId,
    pub discriminant: CaseDiscriminant,
    pub shape: RuntimeVariantShape,
    pub payload_arity: u16,
    pub behavior_class: ClassId,
    pub singleton: Option<Value>,
}

/// The VM-level registry managing all runtime enum and variant descriptors.
#[derive(Clone, Debug, Default)]
pub struct RuntimeAdtRegistry {
    enums: Vec<RuntimeEnumDescriptor>,
    variants: Vec<RuntimeVariantDescriptor>,
    enum_by_declaration: HashMap<DeclarationId, RuntimeEnumId>,
    variant_by_semantic_id: HashMap<VariantId, RuntimeVariantId>,
    variant_by_behavior_class: HashMap<ClassId, RuntimeVariantId>,
    enum_by_root_class: HashMap<ClassId, RuntimeEnumId>,
}

impl RuntimeAdtRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new enum root.
    pub fn register_enum(&mut self, semantic_owner: DeclarationId, root_class: ClassId) -> RuntimeEnumId {
        if let Some(&existing) = self.enum_by_declaration.get(&semantic_owner) {
            return existing;
        }

        let runtime_id = RuntimeEnumId(self.enums.len() as u32);
        let desc = RuntimeEnumDescriptor {
            semantic_owner: semantic_owner.clone(),
            runtime_id,
            root_class,
            variants: Vec::new(),
        };

        self.enums.push(desc);
        self.enum_by_declaration.insert(semantic_owner, runtime_id);
        self.enum_by_root_class.insert(root_class, runtime_id);
        runtime_id
    }

    /// Registers a variant for an existing enum.
    #[allow(clippy::too_many_arguments)]
    pub fn register_variant(
        &mut self,
        semantic_id: VariantId,
        enum_id: RuntimeEnumId,
        discriminant: CaseDiscriminant,
        shape: RuntimeVariantShape,
        payload_arity: u16,
        behavior_class: ClassId,
        singleton: Option<Value>,
    ) -> RuntimeVariantId {
        if let Some(&existing) = self.variant_by_semantic_id.get(&semantic_id) {
            return existing;
        }

        let runtime_id = RuntimeVariantId(self.variants.len() as u32);
        let desc = RuntimeVariantDescriptor {
            semantic_id: semantic_id.clone(),
            runtime_id,
            enum_id,
            discriminant,
            shape,
            payload_arity,
            behavior_class,
            singleton,
        };

        self.variants.push(desc);
        self.variant_by_semantic_id.insert(semantic_id, runtime_id);
        self.variant_by_behavior_class.insert(behavior_class, runtime_id);

        if let Some(enum_desc) = self.enums.get_mut(enum_id.0 as usize) {
            enum_desc.variants.push(runtime_id);
        }

        runtime_id
    }

    #[inline]
    pub fn enum_descriptor(&self, id: RuntimeEnumId) -> Option<&RuntimeEnumDescriptor> {
        self.enums.get(id.0 as usize)
    }

    #[inline]
    pub fn variant_descriptor(&self, id: RuntimeVariantId) -> Option<&RuntimeVariantDescriptor> {
        self.variants.get(id.0 as usize)
    }

    #[inline]
    pub fn variant_descriptor_mut(&mut self, id: RuntimeVariantId) -> Option<&mut RuntimeVariantDescriptor> {
        self.variants.get_mut(id.0 as usize)
    }

    #[inline]
    pub fn enum_by_declaration(&self, decl: &DeclarationId) -> Option<RuntimeEnumId> {
        self.enum_by_declaration.get(decl).copied()
    }

    #[inline]
    pub fn variant_by_semantic(&self, id: &VariantId) -> Option<RuntimeVariantId> {
        self.variant_by_semantic_id.get(id).copied()
    }

    #[inline]
    pub fn variant_by_class(&self, class: ClassId) -> Option<RuntimeVariantId> {
        self.variant_by_behavior_class.get(&class).copied()
    }

    #[inline]
    pub fn enum_by_root_class(&self, class: ClassId) -> Option<RuntimeEnumId> {
        self.enum_by_root_class.get(&class).copied()
    }

    #[inline]
    pub fn is_enum_root(&self, class: ClassId) -> bool {
        self.enum_by_root_class.contains_key(&class)
    }

    /// Enumerates all ClassId handles owned by the registry to be kept alive as GC roots.
    pub fn enumerate_class_roots(&self, mut push: impl FnMut(ClassId)) {
        for enum_desc in &self.enums {
            push(enum_desc.root_class);
        }
        for variant_desc in &self.variants {
            push(variant_desc.behavior_class);
        }
    }
}
