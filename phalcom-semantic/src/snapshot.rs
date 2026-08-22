//! Semantic snapshot for immutable incremental query execution.

use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticSeverity, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId};
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_modules::graph::SemanticGraph;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Immutable semantic snapshot representing a consistent view of the workspace.
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    pub generation: u64,
    pub store: Arc<TypeStore>,
    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
    pub semantic_graph: Arc<SemanticGraph>,
}

impl SemanticSnapshot {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .values()
            .any(|diags| diags.iter().any(|d| d.severity == DiagnosticSeverity::Error))
    }

    pub fn all_diagnostics(&self) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.diagnostics.values().flat_map(|diags| diags.iter())
    }
}
