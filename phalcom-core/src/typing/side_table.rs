//! Method to semantic signature side table.
//! Keeps runtime `MethodObject`s compact while allowing reflection to inspect static typing metadata.

use crate::heap::ObjRef;
use crate::typing::handle::MetadataPoolId;
use phalcom_type_meta::declaration::CallableRecordId;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCallableRef {
    pub pool: MetadataPoolId,
    pub record: CallableRecordId,
}

/// VM-owned mapping from live MethodObject `ObjRef` to stable callable metadata.
#[derive(Clone, Debug, Default)]
pub struct MethodSemanticIndex {
    by_method: HashMap<ObjRef, RuntimeCallableRef>,
}

impl MethodSemanticIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, method: ObjRef, callable: RuntimeCallableRef) {
        self.by_method.insert(method, callable);
    }

    pub fn get(&self, method: ObjRef) -> Option<RuntimeCallableRef> {
        self.by_method.get(&method).copied()
    }

    pub fn remove(&mut self, method: ObjRef) {
        self.by_method.remove(&method);
    }

    pub fn clear(&mut self) {
        self.by_method.clear();
    }
}
