use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticSeverity, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId, SemanticRevision, SnapshotId, WorkspaceId};
use crate::signature::CallableSignatureTable;
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_modules::graph::SemanticGraph;
use phalcom_modules::identity::SourceLocation;
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::query::ModuleQueryFacade;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// Retained module query products for pure read queries over immutable snapshot.
#[derive(Clone, Debug)]
pub struct ModuleQueryProducts {
    pub universe: Arc<ProjectUniverse>,
    pub unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
    pub linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
    pub resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
    pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
}

impl ModuleQueryProducts {
    pub fn new(
        universe: Arc<ProjectUniverse>,
        unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
        linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
        resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
        sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
    ) -> Self {
        Self {
            universe,
            unlinked,
            linked,
            resolved_imports,
            sources,
        }
    }

    pub fn empty() -> Self {
        Self {
            universe: Arc::new(ProjectUniverse::new()),
            unlinked: Arc::new(BTreeMap::new()),
            linked: Arc::new(BTreeMap::new()),
            resolved_imports: Arc::new(BTreeMap::new()),
            sources: Arc::new(BTreeMap::new()),
        }
    }
}

impl Default for ModuleQueryProducts {
    fn default() -> Self {
        Self::empty()
    }
}

/// Semantic completeness status of a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotStatus {
    Complete,
    Partial { blocked_modules: u32 },
}

impl SnapshotStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial { .. })
    }
}

/// Immutable semantic snapshot representing a consistent view of the workspace.
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    pub id: SnapshotId,
    pub generation: u64,
    pub store: Arc<TypeStore>,
    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
    pub semantic_graph: Arc<SemanticGraph>,
    pub callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,
    pub module_products: Arc<ModuleQueryProducts>,
    pub status: SnapshotStatus,
}

impl SemanticSnapshot {
    // Snapshot construction intentionally mirrors its immutable field layout.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace: WorkspaceId,
        revision: SemanticRevision,
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
        surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
        dispatch: Arc<SurfaceDispatchResolver>,
        callable_signatures: Arc<CallableSignatureTable>,
        declarations: Arc<DeclarationTypeTable>,
        hierarchy: Arc<MapTypeHierarchy>,
        diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
        semantic_graph: Arc<SemanticGraph>,
    ) -> Self {
        let store_id = store.id();
        let id = SnapshotId::new(workspace, revision, store_id);
        Self {
            id,
            generation,
            store,
            sources,
            surfaces,
            dispatch,
            callable_signatures,
            declarations,
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses: Arc::new(HashMap::new()),
            module_products: Arc::new(ModuleQueryProducts::empty()),
            status: SnapshotStatus::Complete,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_callable_analyses(
        workspace: WorkspaceId,
        revision: SemanticRevision,
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
        surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
        dispatch: Arc<SurfaceDispatchResolver>,
        callable_signatures: Arc<CallableSignatureTable>,
        declarations: Arc<DeclarationTypeTable>,
        hierarchy: Arc<MapTypeHierarchy>,
        diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
        semantic_graph: Arc<SemanticGraph>,
        callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,
    ) -> Self {
        let store_id = store.id();
        let id = SnapshotId::new(workspace, revision, store_id);
        Self {
            id,
            generation,
            store,
            sources,
            surfaces,
            dispatch,
            callable_signatures,
            declarations,
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses,
            module_products: Arc::new(ModuleQueryProducts::empty()),
            status: SnapshotStatus::Complete,
        }
    }

    pub fn with_callable_analyses(mut self, callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>) -> Self {
        self.callable_analyses = callable_analyses;
        self
    }

    pub fn with_module_products(mut self, module_products: Arc<ModuleQueryProducts>) -> Self {
        self.module_products = module_products;
        self
    }

    pub fn module_queries(&self) -> ModuleQueryFacade<'_> {
        ModuleQueryFacade::new(
            &self.module_products.universe,
            &self.module_products.unlinked,
            &self.module_products.linked,
            &self.module_products.resolved_imports,
            &self.module_products.sources,
        )
    }

    pub fn id(&self) -> SnapshotId {
        self.id
    }

    pub fn status(&self) -> &SnapshotStatus {
        &self.status
    }

    pub fn store(&self) -> &Arc<TypeStore> {
        &self.store
    }

    pub fn sources(&self) -> &Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>> {
        &self.sources
    }

    pub fn surfaces(&self) -> &Arc<HashMap<DeclarationId, DeclarationSurface>> {
        &self.surfaces
    }

    pub fn dispatch(&self) -> &Arc<SurfaceDispatchResolver> {
        &self.dispatch
    }

    pub fn callable_signatures(&self) -> &Arc<CallableSignatureTable> {
        &self.callable_signatures
    }

    pub fn declarations(&self) -> &Arc<DeclarationTypeTable> {
        &self.declarations
    }

    pub fn hierarchy(&self) -> &Arc<MapTypeHierarchy> {
        &self.hierarchy
    }

    pub fn diagnostics(&self) -> &Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>> {
        &self.diagnostics
    }

    pub fn semantic_graph(&self) -> &Arc<SemanticGraph> {
        &self.semantic_graph
    }

    pub fn callable_analyses(&self) -> &Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>> {
        &self.callable_analyses
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .values()
            .any(|diags| diags.iter().any(|d| d.severity == DiagnosticSeverity::Error))
    }

    pub fn all_diagnostics(&self) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.diagnostics.values().flat_map(|diags| diags.iter())
    }

    pub fn diagnostics_for(&self, module: &ModuleId) -> Option<&[SemanticDiagnostic]> {
        self.diagnostics.get(module).map(|d| d.as_ref())
    }
}
