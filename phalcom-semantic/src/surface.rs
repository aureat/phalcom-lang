//! Semantic declaration and module surface definitions.

use crate::dispatch::CallableSignature;
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use crate::types::evidence::TypeKnowledge;
use phalcom_common::selector::Selector;
use std::collections::HashMap;

/// Published member surface for a specific dispatch side (instance or class).
#[derive(Clone, Debug, Default)]
pub struct MemberSurface {
    pub fields: HashMap<String, TypeKnowledge>,
    pub field_ids: HashMap<FieldId, TypeKnowledge>,
    pub callables: HashMap<CallableId, TypeKnowledge>,
    pub callable_signatures: HashMap<Selector, CallableSignature>,
}

impl MemberSurface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_field(&mut self, decl_id: Option<&DeclarationId>, side: DispatchSide, name: impl Into<String>, ty: TypeKnowledge) {
        let name_str = name.into();
        if let Some(id) = decl_id {
            let field_id = FieldId::new(id.clone(), name_str.clone(), side);
            self.field_ids.insert(field_id, ty.clone());
        }
        self.fields.insert(name_str, ty);
    }

    pub fn get_field(&self, name: &str) -> Option<&TypeKnowledge> {
        self.fields.get(name)
    }

    pub fn add_callable(&mut self, decl_id: Option<&DeclarationId>, side: DispatchSide, signature: CallableSignature) {
        if let Some(id) = decl_id {
            let callable_id = CallableId::new(id.clone(), signature.selector.clone(), side);
            self.callables.insert(callable_id, signature.return_type.clone());
        }
        self.callable_signatures.insert(signature.selector.clone(), signature);
    }

    pub fn get_callable(&self, selector: &Selector) -> Option<&CallableSignature> {
        self.callable_signatures.get(selector)
    }
}

/// Published semantic surface for a module or class declaration containing both instance and class surfaces.
#[derive(Clone, Debug, Default)]
pub struct DeclarationSurface {
    pub id: Option<DeclarationId>,
    pub instance: MemberSurface,
    pub class: MemberSurface,
}

impl DeclarationSurface {
    pub fn new(id: Option<DeclarationId>) -> Self {
        Self {
            id,
            instance: MemberSurface::new(),
            class: MemberSurface::new(),
        }
    }

    pub fn surface(&self, side: DispatchSide) -> &MemberSurface {
        match side {
            DispatchSide::Instance => &self.instance,
            DispatchSide::Class => &self.class,
        }
    }

    pub fn surface_mut(&mut self, side: DispatchSide) -> &mut MemberSurface {
        match side {
            DispatchSide::Instance => &mut self.instance,
            DispatchSide::Class => &mut self.class,
        }
    }

    pub fn add_field(&mut self, side: DispatchSide, name: impl Into<String>, ty: TypeKnowledge) {
        let id = self.id.clone();
        self.surface_mut(side).add_field(id.as_ref(), side, name, ty);
    }

    pub fn get_field(&self, side: DispatchSide, name: &str) -> Option<&TypeKnowledge> {
        self.surface(side).get_field(name)
    }

    pub fn add_callable(&mut self, side: DispatchSide, signature: CallableSignature) {
        let id = self.id.clone();
        self.surface_mut(side).add_callable(id.as_ref(), side, signature);
    }

    pub fn get_callable(&self, side: DispatchSide, selector: &Selector) -> Option<&CallableSignature> {
        self.surface(side).get_callable(selector)
    }
}
