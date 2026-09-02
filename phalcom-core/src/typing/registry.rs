//! VM-owned RuntimeTypingRegistry.

use crate::heap::ClassId;
use crate::typing::handle::MetadataPoolId;
use crate::typing::loader::{LoadedSemanticMetadata, RuntimeNominalBindingTable};
use crate::typing::side_table::{MethodImplementationIndex, MethodSemanticIndex};
use phalcom_type_meta::identity::StableDeclarationRef;
use std::sync::Arc;

/// Central runtime typing registry owned by the VM.
#[derive(Clone, Debug, Default)]
pub struct RuntimeTypingRegistry {
    pools: Vec<Arc<LoadedSemanticMetadata>>,
    nominal_bindings: RuntimeNominalBindingTable,
    runtime_declarations: std::collections::HashMap<ClassId, StableDeclarationRef>,
    declaration_identities: std::collections::HashMap<(phalcom_modules::ModuleId, Box<str>), StableDeclarationRef>,
    pub method_semantics: MethodSemanticIndex,
    pub method_implementations: MethodImplementationIndex,
}

impl RuntimeTypingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_pool(&mut self, pool: LoadedSemanticMetadata) -> MetadataPoolId {
        let id = MetadataPoolId(self.pools.len() as u32);
        self.pools.push(Arc::new(pool));
        id
    }

    pub fn get_pool(&self, id: MetadataPoolId) -> Option<&Arc<LoadedSemanticMetadata>> {
        self.pools.get(id.0 as usize)
    }

    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    pub fn pools(&self) -> &[Arc<LoadedSemanticMetadata>] {
        &self.pools
    }

    pub fn register_nominal_binding(&mut self, decl: StableDeclarationRef, class: ClassId) {
        self.nominal_bindings.insert(decl.clone(), class);
        self.runtime_declarations.insert(class, decl);
    }

    /// Records the stable declaration corresponding to one runtime module
    /// symbol before its class is created by module initialization.
    pub fn register_declaration_identity(&mut self, module: phalcom_modules::ModuleId, name: Box<str>, declaration: StableDeclarationRef) {
        self.declaration_identities.insert((module, name), declaration);
    }

    pub fn declaration_identity(&self, module: &phalcom_modules::ModuleId, name: &str) -> Option<&StableDeclarationRef> {
        self.declaration_identities.get(&(module.clone(), name.into()))
    }

    pub fn resolve_nominal(&self, decl: &StableDeclarationRef) -> Option<ClassId> {
        self.nominal_bindings.get(decl)
    }

    /// Returns the exact metadata declaration previously associated with a
    /// runtime class. Class display names are presentation only and cannot
    /// identify this declaration.
    pub fn declaration_for_nominal(&self, class: ClassId) -> Option<&StableDeclarationRef> {
        self.runtime_declarations.get(&class)
    }
}
