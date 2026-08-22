//! Semantic declaration and module surface definitions.

use crate::identity::{CallableId, DeclarationId, FieldId};
use crate::types::evidence::TypeKnowledge;
use std::collections::HashMap;

/// Published semantic surface for a module or class declaration.
#[derive(Clone, Debug, Default)]
pub struct DeclarationSurface {
    pub id: Option<DeclarationId>,
    pub fields: HashMap<FieldId, TypeKnowledge>,
    pub callables: HashMap<CallableId, TypeKnowledge>,
}

impl DeclarationSurface {
    pub fn new(id: Option<DeclarationId>) -> Self {
        Self {
            id,
            fields: HashMap::new(),
            callables: HashMap::new(),
        }
    }
}
