//! Shared one-module workspace fixture for incremental semantic tests.

use phalcom_modules::identity::ModuleId;
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface};
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Build minimal linked input while keeping source parsing identical across
/// every incremental scenario.
pub(crate) fn single_module_input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    let program = Arc::new(parsed.program);
    let _ = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program);

    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_module);

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });
    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)),
    );

    SemanticWorkspaceInput { linked, sources, generation }
}
