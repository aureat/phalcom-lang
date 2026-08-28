use crate::advisory::AdvisoryWorkspace;
use crate::checker::incident::InternalSemanticIncident;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticSeverity, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId, SemanticRevision, SnapshotId, SourceSiteId, SourceSiteRef, WorkspaceId};
use crate::presentation::{FormalFactRef, FormalFactSite, FormalSemanticProjection, SemanticSiteView};
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::source::ParsedModuleUnit;
use crate::source_index::{OccurrenceView, SourceSemanticIndex, SourceSite};
use crate::surface::DeclarationSurface;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_modules::graph::SemanticGraph;
use phalcom_modules::identity::{SourceId, SourceLocation};
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::query::ModuleQueryFacade;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

fn collect_internal_incidents(
    callable_analyses: &HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
) -> Arc<[InternalSemanticIncident]> {
    let mut incidents = callable_analyses
        .values()
        .flat_map(|analysis| analysis.internal_incidents.iter().cloned())
        .collect::<Vec<_>>();
    incidents.sort_by_key(|incident| (incident.callable.clone(), incident.id));
    Arc::from(incidents.into_boxed_slice())
}

/// Retained module query products for pure read queries over immutable snapshot.
#[derive(Clone, Debug)]
pub struct ModuleQueryProducts {
    pub universe: Arc<ProjectUniverse>,
    pub unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
    pub linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
    pub resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
    pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
    pub source_modules: Arc<BTreeMap<SourceId, ModuleId>>,
    pub display_path_modules: Arc<BTreeMap<PathBuf, ModuleId>>,
}

impl ModuleQueryProducts {
    pub fn new(
        universe: Arc<ProjectUniverse>,
        unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
        linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
        resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
        sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
    ) -> Self {
        let source_modules = Arc::new(sources.iter().map(|(module, location)| (location.source_id.clone(), module.clone())).collect());
        let display_path_modules = Arc::new(
            sources
                .iter()
                .map(|(module, location)| (location.display_path.clone(), module.clone()))
                .collect(),
        );
        Self {
            universe,
            unlinked,
            linked,
            resolved_imports,
            sources,
            source_modules,
            display_path_modules,
        }
    }

    pub fn empty() -> Self {
        Self {
            universe: Arc::new(ProjectUniverse::new()),
            unlinked: Arc::new(BTreeMap::new()),
            linked: Arc::new(BTreeMap::new()),
            resolved_imports: Arc::new(BTreeMap::new()),
            sources: Arc::new(BTreeMap::new()),
            source_modules: Arc::new(BTreeMap::new()),
            display_path_modules: Arc::new(BTreeMap::new()),
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
    /// Read-only compiler-generated source documents used solely for semantic
    /// provenance and editor presentation. They are never analysis inputs.
    pub presentation_sources: Arc<BTreeMap<ModuleId, Arc<str>>>,
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    pub field_signatures: Arc<FieldSignatureTable>,
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
    pub semantic_graph: Arc<SemanticGraph>,
    pub callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,
    /// Internal analyzer incidents, kept separate from source diagnostics.
    pub internal_incidents: Arc<[InternalSemanticIncident]>,
    pub formal_projection: Arc<FormalSemanticProjection>,
    pub source_index: Arc<SourceSemanticIndex>,
    /// Immutable advisory runtime-shape products for this exact snapshot.
    pub advisory: Arc<AdvisoryWorkspace>,
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
            presentation_sources: Arc::new(BTreeMap::new()),
            surfaces,
            dispatch,
            callable_signatures,
            field_signatures: Arc::new(FieldSignatureTable::new()),
            declarations,
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses: Arc::new(HashMap::new()),
            internal_incidents: Arc::from([]),
            formal_projection: Arc::new(FormalSemanticProjection::default()),
            source_index: Arc::new(SourceSemanticIndex::default()),
            advisory: Arc::new(AdvisoryWorkspace::default()),
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
        let formal_projection = Arc::new(FormalSemanticProjection::from_callable_analyses(&callable_analyses));
        let internal_incidents = collect_internal_incidents(&callable_analyses);
        Self {
            id,
            generation,
            store,
            sources,
            presentation_sources: Arc::new(BTreeMap::new()),
            surfaces,
            dispatch,
            callable_signatures,
            field_signatures: Arc::new(FieldSignatureTable::new()),
            declarations,
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses,
            internal_incidents,
            formal_projection,
            source_index: Arc::new(SourceSemanticIndex::default()),
            advisory: Arc::new(AdvisoryWorkspace::default()),
            module_products: Arc::new(ModuleQueryProducts::empty()),
            status: SnapshotStatus::Complete,
        }
    }

    pub fn with_field_signatures(mut self, field_signatures: Arc<FieldSignatureTable>) -> Self {
        self.field_signatures = field_signatures;
        self
    }

    pub fn with_callable_analyses(mut self, callable_analyses: Arc<HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>) -> Self {
        self.formal_projection = Arc::new(FormalSemanticProjection::from_callable_analyses(&callable_analyses));
        self.internal_incidents = collect_internal_incidents(&callable_analyses);
        self.callable_analyses = callable_analyses;
        self
    }

    /// Attaches read-only compiler presentation sources. These documents
    /// provide source coordinates for canonical declaration provenance but do
    /// not participate in linking or semantic analysis.
    pub fn with_presentation_sources(mut self, sources: Arc<BTreeMap<ModuleId, Arc<str>>>) -> Self {
        self.presentation_sources = sources;
        self
    }

    /// Returns exact compiler-owned presentation text for one virtual source.
    pub fn presentation_source(&self, module: &ModuleId) -> Option<&str> {
        self.presentation_sources.get(module).map(AsRef::as_ref)
    }

    /// Attaches one immutable compiler-owned source semantic index.
    pub fn with_source_index(mut self, source_index: Arc<SourceSemanticIndex>) -> Self {
        self.formal_projection = Arc::new(FormalSemanticProjection::from_callable_analyses_with_source_index(
            &self.callable_analyses,
            Some(&source_index),
        ));
        self.source_index = source_index;
        self
    }

    /// Attaches immutable advisory products built from this snapshot's source
    /// and formal inputs.
    pub fn with_advisory(mut self, advisory: Arc<AdvisoryWorkspace>) -> Self {
        self.advisory = advisory;
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
            &self.module_products.source_modules,
            &self.module_products.display_path_modules,
        )
    }

    /// Returns the protocol-neutral editor query facade for this snapshot.
    pub fn editor(&self) -> crate::editor::EditorSemanticQuery<'_> {
        crate::editor::EditorSemanticQuery::new(self)
    }

    /// Returns the canonical module associated with a source-provider identity.
    pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
        self.module_products.source_modules.get(source)
    }

    /// Returns the canonical module associated with an already-produced display path.
    /// This query performs no filesystem work or path canonicalization.
    pub fn module_for_display_path(&self, path: &std::path::Path) -> Option<&ModuleId> {
        self.module_products.display_path_modules.get(path)
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

    /// Returns source semantic products published with this snapshot.
    pub fn source_index(&self) -> &Arc<SourceSemanticIndex> {
        &self.source_index
    }

    /// Returns machine-readable formal source projection.
    pub fn formal_projection(&self) -> &Arc<FormalSemanticProjection> {
        &self.formal_projection
    }

    /// Returns compiler-owned advisory products for this snapshot.
    pub fn advisory(&self) -> &Arc<AdvisoryWorkspace> {
        &self.advisory
    }

    /// Returns the read-only advisory query facade for this snapshot.
    pub fn advisory_queries(&self) -> crate::advisory::AdvisoryQuery<'_> {
        crate::advisory::AdvisoryQuery::new(&self.advisory)
    }

    /// Retrieves formal expression product by exact callable/expression key.
    pub fn formal_expression(
        &self,
        callable: &crate::identity::CallableId,
        expression: crate::identity::ExpressionId,
    ) -> Option<&crate::checker::ExpressionAnalysis> {
        self.callable_analyses.get(callable)?.expressions.get(&expression)
    }

    /// Retrieves formal binding product by exact callable/binding key.
    pub fn formal_binding(&self, callable: &crate::identity::CallableId, binding: crate::identity::BindingId) -> Option<&crate::checker::BindingState> {
        self.callable_analyses.get(callable)?.bindings.get(&binding)
    }

    /// Retrieves machine-readable formal fact attachment at a source position.
    pub fn formal_fact_at(&self, module: &ModuleId, offset: usize) -> Option<&FormalFactSite> {
        self.formal_projection.fact_at(module, offset)
    }

    /// Retrieves a machine-readable formal site by canonical fact identity.
    pub fn formal_fact(&self, fact: &FormalFactRef) -> Option<&FormalFactSite> {
        self.formal_projection.get(fact)
    }

    /// Resolves a source-site handle only against its owning snapshot.
    pub fn resolve_site_ref(&self, site: &SourceSiteRef) -> Option<&SourceSite> {
        site.resolve_for(self.id).and_then(|site| self.source_site(site))
    }

    /// Returns an immutable source site by canonical snapshot-local identity.
    pub fn source_site(&self, site: &SourceSiteId) -> Option<&SourceSite> {
        self.source_index.source_site(site)
    }

    /// Returns source site selected by indexed byte position.
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        self.source_index.source_site_at(module, offset)
    }

    /// Returns occurrence and exact target selected by indexed byte position.
    pub fn occurrence_at(&self, module: &ModuleId, offset: usize) -> Option<OccurrenceView<'_>> {
        self.source_index.occurrence_at(module, offset)
    }

    /// Returns exact source sites for one canonical target.
    pub fn occurrences_for_target(&self, target: &crate::identity::SemanticTargetId) -> Option<&[SourceSiteId]> {
        self.source_index.occurrences_for_target(target)
    }

    /// Returns formal expression product attached to one source site.
    pub fn formal_expression_at(&self, site: &SourceSiteId) -> Option<&crate::checker::ExpressionAnalysis> {
        self.formal_fact_for_site(site).as_ref().and_then(|fact| match fact {
            FormalFactRef::Expression { callable, expression } => self.formal_expression(callable, *expression),
            _ => None,
        })
    }

    /// Returns formal binding product attached to one source site.
    pub fn formal_binding_at(&self, site: &SourceSiteId) -> Option<&crate::checker::BindingState> {
        self.formal_fact_for_site(site).as_ref().and_then(|fact| match fact {
            FormalFactRef::Binding { callable, binding } => self.formal_binding(callable, *binding),
            _ => None,
        })
    }

    /// Returns advisory expression/binding fact attached to one source site.
    pub fn advisory_fact(&self, site: &SourceSiteId) -> Option<&crate::advisory::AdvisoryFact> {
        self.advisory.expression(site).or_else(|| self.advisory.binding(site))
    }

    /// Returns advisory parameter fact for one canonical callable slot.
    pub fn advisory_binding(&self, slot: &crate::advisory::AdvisoryParameterSlot) -> Option<&crate::advisory::AdvisoryFact> {
        self.advisory.parameter(slot)
    }

    /// Returns advisory callable summary for one canonical callable.
    pub fn advisory_callable(&self, callable: &crate::identity::CallableId) -> Option<&crate::advisory::AdvisoryCallableSummary> {
        self.advisory.callable(callable)
    }

    /// Composes indexed source, formal, and advisory products without
    /// triggering analysis or scanning callable bodies.
    pub fn semantic_site_at(&self, module: &ModuleId, offset: usize) -> SemanticSiteView<'_> {
        let source_site = self.source_index.occurrence_at(module, offset).map(|view| view.occurrence.site.clone());
        let formal = self.formal_projection.fact_at(module, offset);
        let advisory = source_site
            .as_ref()
            .and_then(|site| self.advisory.expression(site).or_else(|| self.advisory.binding(site)));
        let target = source_site.as_ref().and_then(|site| self.advisory.target(site));
        SemanticSiteView {
            source_site,
            formal,
            advisory,
            target,
        }
    }

    pub(crate) fn formal_fact_for_site(&self, site: &SourceSiteId) -> Option<FormalFactRef> {
        for attachment in self.source_index.modules.values().flat_map(|module| module.attachments.values()) {
            if attachment.formal_expressions.values().any(|candidate| candidate == site) {
                let expression = attachment
                    .formal_expressions
                    .iter()
                    .find_map(|(expression, candidate)| (candidate == site).then_some(expression))?;
                return Some(FormalFactRef::Expression {
                    callable: attachment.callable.clone(),
                    expression: *expression,
                });
            }
            if let Some(binding) = attachment
                .formal_bindings
                .iter()
                .find_map(|(binding, candidate)| (candidate == site).then_some(binding))
            {
                return Some(FormalFactRef::Binding {
                    callable: attachment.callable.clone(),
                    binding: *binding,
                });
            }
        }
        None
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
