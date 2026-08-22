//! Bounded runtime typing overlay for dynamically constructed types and kinds.

use crate::heap::ClassId;
use crate::typing::handle::{RuntimeOverlayKindId, RuntimeOverlayTypeId, RuntimeTypeRef};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeOverlayKindNode {
    Type,
    Arrow {
        parameters: Box<[crate::typing::handle::RuntimeKindRef]>,
        result: Box<crate::typing::handle::RuntimeKindRef>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeOverlayTypeNode {
    Nominal { class: ClassId },
    Applied { origin: RuntimeTypeRef, arguments: Box<[RuntimeTypeRef]> },
    Union(Box<[RuntimeTypeRef]>),
}

/// Bounded overlay for explicitly constructed type forms.
#[derive(Clone, Debug, Default)]
pub struct RuntimeTypingOverlay {
    pub kinds: Vec<RuntimeOverlayKindNode>,
    pub types: Vec<RuntimeOverlayTypeNode>,
    pub kind_interner: HashMap<RuntimeOverlayKindNode, RuntimeOverlayKindId>,
    pub type_interner: HashMap<RuntimeOverlayTypeNode, RuntimeOverlayTypeId>,
    pub bytes_used: usize,
}

impl RuntimeTypingOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_type(&mut self, node: RuntimeOverlayTypeNode) -> RuntimeOverlayTypeId {
        if let Some(&id) = self.type_interner.get(&node) {
            return id;
        }
        let id = RuntimeOverlayTypeId(self.types.len() as u32);
        self.types.push(node.clone());
        self.type_interner.insert(node, id);
        id
    }

    pub fn type_ref(&mut self, node: RuntimeOverlayTypeNode) -> RuntimeTypeRef {
        RuntimeTypeRef::Overlay(self.intern_type(node))
    }

    pub fn type_node(&self, id: RuntimeOverlayTypeId) -> Option<&RuntimeOverlayTypeNode> {
        self.types.get(id.0 as usize)
    }
}
