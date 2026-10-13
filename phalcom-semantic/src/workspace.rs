use crate::identity::ModuleId;
use crate::session::SemanticUpdateFailure;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::request::SemanticProductRequest;
use phalcom_ast::ast::Program;
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::LinkedProgram;
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_modules::diagnostic::ModuleDiagnostic;

/// Input to whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
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
    pub product_request: SemanticProductRequest,
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
        let interfaces = unlinked
            .iter()
            .map(|(module, interface)| (module.clone(), Arc::new(interface.clone())))
            .collect();
        let topology = Arc::new(phalcom_modules::topology::ModuleTopology::from_parts(
            phalcom_modules::stabilization::ResolverGeneration(generation),
            &linked.universe,
            &unlinked,
            &source_locations,
        ));
        Self {
            linked,
            sources,
            interfaces: Arc::new(interfaces),
            import_products: Arc::new(BTreeMap::new()),
            module_graph_changed: false,
            import_sites_by_module: Arc::new(BTreeMap::new()),
            require_canonical_import_products: false,
            diagnostics: BTreeMap::new(),
            blocked_modules: BTreeSet::new(),
            generation,
            topology: Some(topology),
            reverse_imports: None,
            product_request: SemanticProductRequest::default(),
        }
    }

    pub fn with_product_request(mut self, product_request: SemanticProductRequest) -> Self {
        self.product_request = product_request;
        self
    }

    pub fn with_topology(mut self, topology: Arc<phalcom_modules::topology::ModuleTopology>) -> Self {
        self.topology = Some(topology);
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
pub fn analyze_workspace(input: SemanticWorkspaceInput) -> Result<SemanticAnalysis, SemanticUpdateFailure> {
    let mut session = crate::session::SemanticWorkspaceSession::new().with_product_request(input.product_request);
    let update = session.update(input)?;
    Ok(SemanticAnalysis { snapshot: update.snapshot })
}

/// Convenience helper to analyze a single module as a standalone workspace.
pub fn analyze_single_module(module: ModuleId, source: Arc<str>, program: Arc<Program>) -> Result<SemanticAnalysis, SemanticUpdateFailure> {
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

    analyze_workspace(SemanticWorkspaceInput {
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
        product_request: SemanticProductRequest::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::SemanticWorkspaceInput;
    use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
    use phalcom_modules::interface::InterfaceBuilder;
    use phalcom_modules::linker::ModuleLinker;
    use phalcom_modules::project::ProjectUniverse;
    use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[test]
    fn new_retains_interfaces_used_to_build_topology() {
        let source: Arc<str> = Arc::from("class Subject {}\n");
        let parsed = phalcom_ast::parse(&source, 0);
        assert!(parsed.errors.is_empty());
        let module = ModuleId::resolved(ResolvedProjectId::from_raw(1), ModulePath::root());
        let interface = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &parsed.program).expect("interface builds");
        let linked = Arc::new(
            ModuleLinker::new(Arc::new(ProjectUniverse::new()), BTreeMap::from([(module.clone(), interface)]))
                .link(module.clone(), &BTreeMap::new())
                .expect("module links"),
        );
        let parsed = Arc::new(ParsedModuleUnit::new(
            module.clone(),
            ModuleKind::Module,
            None,
            source,
            Arc::new(parsed.program),
        ));

        let input = SemanticWorkspaceInput::new(linked, BTreeMap::from([(module.clone(), parsed)]), 1);

        assert!(input.interfaces.contains_key(&module));
        assert!(input.interfaces[&module].declarations.contains_key("Subject"));
    }
}
