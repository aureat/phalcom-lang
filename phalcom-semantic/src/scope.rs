//! Semantic scope model for declaration symbols and local bindings.

use crate::identity::BindingId;
use phalcom_common::range::SourceRange;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ScopeTable {
    bindings: HashMap<String, (BindingId, SourceRange)>,
}

impl ScopeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, id: BindingId, range: SourceRange) {
        self.bindings.insert(name.into(), (id, range));
    }

    pub fn get(&self, name: &str) -> Option<(BindingId, SourceRange)> {
        self.bindings.get(name).copied()
    }
}
