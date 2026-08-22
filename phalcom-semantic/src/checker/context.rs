//! Checking context and scope environments.

use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{DeclarationId, ModuleId};
use crate::types::evidence::TypeKnowledge;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use crate::types::annotation::TypeResolver;
use std::collections::HashMap;

/// Environment of local bindings in current lexical block/scope.
#[derive(Clone, Debug, Default)]
pub struct LocalEnv {
    bindings: HashMap<String, TypeKnowledge>,
}

impl LocalEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, knowledge: TypeKnowledge) {
        self.bindings.insert(name.into(), knowledge);
    }

    pub fn get(&self, name: &str) -> Option<&TypeKnowledge> {
        self.bindings.get(name)
    }
}

/// The active context during semantic type checking.
pub struct CheckingContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub current_module: ModuleId,
    pub current_class: Option<DeclarationId>,
    pub expected_return: Option<TypeKnowledge>,
    pub local_envs: Vec<LocalEnv>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> CheckingContext<'a> {
    pub fn new(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        current_module: ModuleId,
    ) -> Self {
        Self {
            store,
            hierarchy,
            resolver,
            current_module,
            current_class: None,
            expected_return: None,
            local_envs: vec![LocalEnv::new()],
            diagnostics: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.local_envs.push(LocalEnv::new());
    }

    pub fn pop_scope(&mut self) {
        self.local_envs.pop();
    }

    pub fn bind_local(&mut self, name: impl Into<String>, knowledge: TypeKnowledge) {
        if let Some(env) = self.local_envs.last_mut() {
            env.insert(name, knowledge);
        }
    }

    pub fn lookup_local(&self, name: &str) -> Option<&TypeKnowledge> {
        for env in self.local_envs.iter().rev() {
            if let Some(k) = env.get(name) {
                return Some(k);
            }
        }
        None
    }
}
