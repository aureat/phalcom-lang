//! Semantic snapshot for immutable incremental query execution.

use crate::identity::ModuleId;
use crate::source::ParsedSourceUnit;
use crate::types::store::TypeStore;
use std::collections::HashMap;
use std::sync::Arc;

/// Immutable semantic snapshot representing a consistent view of the workspace.
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    pub generation: u64,
    pub store: Arc<TypeStore>,
    pub sources: Arc<HashMap<ModuleId, Arc<ParsedSourceUnit>>>,
}

impl SemanticSnapshot {
    pub fn new(
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<HashMap<ModuleId, Arc<ParsedSourceUnit>>>,
    ) -> Self {
        Self {
            generation,
            store,
            sources,
        }
    }
}
