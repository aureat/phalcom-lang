use crate::identity::ModuleId;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::session::SemanticUpdateFailure;
use phalcom_ast::ast::Program;
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedProgram, SemanticLinkedProgram};
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_modules::diagnostic::ModuleDiagnostic;

/// Input to whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    /// Symbolic import authority used by semantic resolution. During the
    /// migration this is optional for compatibility callers; production
    /// compilation supplies it before executable finalization.
    pub semantic_linked: Option<Arc<SemanticLinkedProgram>>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub interfaces: Arc<BTreeMap<ModuleId, Arc<phalcom_modules::interface::UnlinkedModuleInterface>>>,
    /// Exact canonical import products. Production module-session updates
    /// retain this immutable root across semantic publication.
    pub import_products: Arc<BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
    /// Exact module-layer graph/import delta, when supplied by the persistent
    /// module session. Direct compatibility inputs leave this false.
    pub module_graph_changed: bool,
    /// Exact importer-owned import-site worklists retained by Plan A.
    pub import_sites_by_module: Arc<BTreeMap<ModuleId, BTreeSet<phalcom_modules::identity::ImportSiteId>>>,
    /// Production module updates require canonical import products; direct
    /// source-index compatibility callers may retain path-map fallback.
    pub require_canonical_import_products: bool,
    pub diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    pub blocked_modules: BTreeSet<ModuleId>,
    pub generation: u64,
    pub topology: Option<Arc<phalcom_modules::topology::ModuleTopology>>,
    pub reverse_imports: Option<Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>>,
}

impl SemanticWorkspaceInput {
    pub fn new(linked: Arc<LinkedProgram>, sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>, generation: u64) -> Self {
        let mut unlinked = BTreeMap::new();
        let mut source_locations = BTreeMap::new();
        for (module, unit) in &sources {
            if let Ok(interface) = InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
                unlinked.insert(module.clone(), interface);
            }
            if let Some(source) = &unit.source {
                source_locations.insert(module.clone(), source.clone());
            }
        }
        let topology = Arc::new(phalcom_modules::topology::ModuleTopology::from_parts(
            phalcom_modules::stabilization::ResolverGeneration(generation),
            &linked.universe,
            &unlinked,
            &source_locations,
        ));
        Self {
            linked,
            semantic_linked: None,
            sources,
            interfaces: Arc::new(BTreeMap::new()),
            import_products: Arc::new(BTreeMap::new()),
            module_graph_changed: false,
            import_sites_by_module: Arc::new(BTreeMap::new()),
            require_canonical_import_products: false,
            diagnostics: BTreeMap::new(),
            blocked_modules: BTreeSet::new(),
            generation,
            topology: Some(topology),
            reverse_imports: None,
        }
    }

    pub fn with_topology(mut self, topology: Arc<phalcom_modules::topology::ModuleTopology>) -> Self {
        self.topology = Some(topology);
        self
    }

    pub fn with_semantic_linked(mut self, semantic_linked: Arc<SemanticLinkedProgram>) -> Self {
        self.semantic_linked = Some(semantic_linked);
        self
    }

    pub fn with_reverse_imports(mut self, reverse_imports: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>) -> Self {
        self.reverse_imports = Some(reverse_imports);
        self
    }

    pub fn with_interfaces(mut self, interfaces: BTreeMap<ModuleId, Arc<phalcom_modules::interface::UnlinkedModuleInterface>>) -> Self {
        self.interfaces = Arc::new(interfaces);
        self
    }

    pub fn with_import_sites_by_module(mut self, import_sites_by_module: Arc<BTreeMap<ModuleId, BTreeSet<phalcom_modules::identity::ImportSiteId>>>) -> Self {
        self.import_sites_by_module = import_sites_by_module;
        self
    }

    pub fn with_import_products(
        mut self,
        import_products: Arc<BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
    ) -> Self {
        self.import_products = import_products;
        self
    }

    pub fn with_canonical_import_products(mut self) -> Self {
        self.require_canonical_import_products = true;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_blocked_modules(mut self, blocked_modules: BTreeSet<ModuleId>) -> Self {
        self.blocked_modules = blocked_modules;
        self
    }
}

/// The result of whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticAnalysis {
    pub snapshot: Arc<SemanticSnapshot>,
}

/// Analyzes an entire linked workspace and returns an immutable semantic snapshot for this generation.
pub fn analyze_workspace(input: SemanticWorkspaceInput) -> SemanticAnalysis {
    try_analyze_workspace(input).unwrap_or_else(|error| panic!("semantic workspace analysis failed: {error}"))
}

/// Fallible workspace analysis entry point for executable and service consumers.
pub fn try_analyze_workspace(input: SemanticWorkspaceInput) -> Result<SemanticAnalysis, SemanticUpdateFailure> {
    let mut session = crate::session::SemanticWorkspaceSession::new();
    let update = session.try_update(input)?;
    Ok(SemanticAnalysis { snapshot: update.snapshot })
}

/// Convenience helper to analyze a single module as a standalone workspace.
pub fn analyze_single_module(module: ModuleId, source: Arc<str>, program: Arc<Program>) -> SemanticAnalysis {
    try_analyze_single_module(module, source, program).unwrap_or_else(|error| panic!("single-module semantic analysis failed: {error}"))
}

/// Fallible single-module analysis that preserves interface and linking failures.
pub fn try_analyze_single_module(module: ModuleId, source: Arc<str>, program: Arc<Program>) -> Result<SemanticAnalysis, SemanticUpdateFailure> {
    let unlinked = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program)
        .map_err(|error| SemanticUpdateFailure::Failed(format!("single-module interface construction failed: {error}")))?;

    let universe = Arc::new(phalcom_modules::project::ProjectUniverse::new());
    let mut interfaces = BTreeMap::new();
    interfaces.insert(module.clone(), unlinked.clone());
    let linker = phalcom_modules::linker::ModuleLinker::new(universe.clone(), interfaces);
    let linked = Arc::new(
        linker
            .link_with_unresolved_imports(module.clone(), &BTreeMap::new())
            .map_err(|error| SemanticUpdateFailure::Failed(format!("single-module linking failed: {error}")))?,
    );

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(module.clone(), ModuleKind::Module, None, source, program)),
    );

    let mut input_interfaces = BTreeMap::new();
    input_interfaces.insert(module, Arc::new(unlinked));

    try_analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        interfaces: Arc::new(input_interfaces),
        import_products: Arc::new(BTreeMap::new()),
        import_sites_by_module: Arc::new(BTreeMap::new()),
        require_canonical_import_products: false,
        diagnostics: BTreeMap::new(),
        blocked_modules: BTreeSet::new(),
        generation: 0,
        topology: None,
        reverse_imports: None,
        module_graph_changed: false,
    })
}
