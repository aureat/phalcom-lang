use crate::identity::ModuleId;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use phalcom_ast::ast::Program;
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::{LinkedModule, LinkedProgram};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Input to whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub generation: u64,
}

/// The result of whole-workspace semantic analysis.
#[derive(Clone, Debug)]
pub struct SemanticAnalysis {
    pub snapshot: Arc<SemanticSnapshot>,
}

/// Analyzes an entire linked workspace and returns an immutable semantic snapshot for this generation.
pub fn analyze_workspace(input: SemanticWorkspaceInput) -> SemanticAnalysis {
    let mut session = crate::session::SemanticWorkspaceSession::new();
    let update = session.update(input);
    SemanticAnalysis { snapshot: update.snapshot }
}

/// Convenience helper to analyze a single module as a standalone workspace.
pub fn analyze_single_module(module: ModuleId, source: Arc<str>, program: Arc<Program>) -> SemanticAnalysis {
    let unlinked =
        InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program).unwrap_or_else(|_| phalcom_modules::interface::UnlinkedModuleInterface {
            id: module.clone(),
            kind: ModuleKind::Module,
            imports: Vec::new(),
            exports: BTreeMap::new(),
            declarations: BTreeMap::new(),
            exposed_children: std::collections::BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        });

    let universe = Arc::new(phalcom_modules::project::ProjectUniverse::new());
    let mut interfaces = BTreeMap::new();
    interfaces.insert(module.clone(), unlinked);
    let linker = phalcom_modules::linker::ModuleLinker::new(universe.clone(), interfaces);
    let linked = linker
        .link_with_unresolved_imports(module.clone(), &BTreeMap::new())
        .map(Arc::new)
        .unwrap_or_else(|_| {
            let linked_mod = LinkedModule {
                interface: phalcom_modules::interface::LinkedModuleInterface {
                    module: module.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: phalcom_modules::linker::ModuleBindingLayout::default(),
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            };
            let mut modules = BTreeMap::new();
            modules.insert(module.clone(), linked_mod);
            Arc::new(LinkedProgram {
                universe,
                modules,
                graphs: phalcom_modules::graph::ModuleGraphs::default(),
                entry: module.clone(),
                initialization_order: vec![module.clone()],
            })
        });

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, source, program)),
    );

    analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 0,
    })
}
