//! Semantic snapshot for immutable incremental query execution.

use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId};
use crate::source::ParsedSourceUnit;
use crate::surface::DeclarationSurface;
use crate::types::store::TypeStore;
use std::collections::HashMap;
use std::sync::Arc;

/// Immutable semantic snapshot representing a consistent view of the workspace.
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    pub generation: u64,
    pub store: Arc<TypeStore>,
    pub sources: Arc<HashMap<ModuleId, Arc<ParsedSourceUnit>>>,
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
}

impl SemanticSnapshot {
    pub fn new(generation: u64, store: Arc<TypeStore>, sources: Arc<HashMap<ModuleId, Arc<ParsedSourceUnit>>>) -> Self {
        Self {
            generation,
            store,
            sources,
            surfaces: Arc::new(HashMap::new()),
            dispatch: Arc::new(SurfaceDispatchResolver::new()),
        }
    }

    pub fn with_surfaces(
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<HashMap<ModuleId, Arc<ParsedSourceUnit>>>,
        surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
        dispatch: Arc<SurfaceDispatchResolver>,
    ) -> Self {
        Self {
            generation,
            store,
            sources,
            surfaces,
            dispatch,
        }
    }
}
