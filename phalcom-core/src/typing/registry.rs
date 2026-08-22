//! VM-owned RuntimeTypingRegistry.

use crate::heap::ClassId;
use crate::typing::handle::MetadataPoolId;
use crate::typing::loader::{LoadedSemanticMetadata, RuntimeNominalBindingTable};
use crate::typing::side_table::MethodSemanticIndex;
use phalcom_type_meta::identity::StableDeclarationRef;
use std::sync::Arc;

/// Central runtime typing registry owned by the VM.
#[derive(Clone, Debug, Default)]
pub struct RuntimeTypingRegistry {
    pools: Vec<Arc<LoadedSemanticMetadata>>,
    nominal_bindings: RuntimeNominalBindingTable,
    pub method_semantics: MethodSemanticIndex,
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
        self.nominal_bindings.insert(decl, class);
    }

    pub fn resolve_nominal(&self, decl: &StableDeclarationRef) -> Option<ClassId> {
        self.nominal_bindings.get(decl)
    }
}
