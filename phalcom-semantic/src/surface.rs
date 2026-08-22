//! Semantic declaration and module surface definitions.

use crate::dispatch::CallableSignature;
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use crate::types::evidence::TypeKnowledge;
use phalcom_common::selector::Selector;
use std::collections::HashMap;

/// Published semantic surface for a module or class declaration.
#[derive(Clone, Debug, Default)]
pub struct DeclarationSurface {
    pub id: Option<DeclarationId>,
    pub fields: HashMap<String, TypeKnowledge>,
    pub field_ids: HashMap<FieldId, TypeKnowledge>,
    pub callables: HashMap<CallableId, TypeKnowledge>,
    pub callable_signatures: HashMap<Selector, CallableSignature>,
}

impl DeclarationSurface {
    pub fn new(id: Option<DeclarationId>) -> Self {
        Self {
            id,
            fields: HashMap::new(),
            field_ids: HashMap::new(),
            callables: HashMap::new(),
            callable_signatures: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: impl Into<String>, ty: TypeKnowledge) {
        let name_str = name.into();
        if let Some(ref id) = self.id {
            let field_id = FieldId::new(id.clone(), name_str.clone(), DispatchSide::Instance);
            self.field_ids.insert(field_id, ty.clone());
        }
        self.fields.insert(name_str, ty);
    }

    pub fn get_field(&self, name: &str) -> Option<&TypeKnowledge> {
        self.fields.get(name)
    }

    pub fn add_callable(&mut self, signature: CallableSignature) {
        if let Some(ref id) = self.id {
            let callable_id = CallableId::new(id.clone(), signature.selector.clone(), DispatchSide::Instance);
            self.callables.insert(callable_id, signature.return_type.clone());
        }
        self.callable_signatures.insert(signature.selector.clone(), signature);
    }

    pub fn get_callable(&self, selector: &Selector) -> Option<&CallableSignature> {
        self.callable_signatures.get(selector)
    }
}
