//! Shared one-module workspace fixture for incremental semantic tests.

use phalcom_modules::identity::ModuleId;
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Build minimal linked input while keeping source parsing identical across
/// every incremental scenario.
pub(crate) fn single_module_input(module: ModuleId, source: &str, generation: u64) -> SemanticWorkspaceInput {
    multi_module_input(vec![(module, source.to_owned())], generation)
}

/// Build minimal linked input for several source modules.
pub(crate) fn multi_module_input(modules: Vec<(ModuleId, String)>, generation: u64) -> SemanticWorkspaceInput {
    let mut linked_modules = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (module, source) in modules {
        let parsed = phalcom_ast::parse(&source, 0);
        let program = Arc::new(parsed.program);
        let local_globals = InterfaceBuilder::build(module.clone(), ModuleKind::Module, &program)
            .map(|interface| {
                interface
                    .declarations
                    .keys()
                    .enumerate()
                    .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
                    .collect()
            })
            .unwrap_or_default();

        linked_modules.insert(
            module.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: module.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals,
                    ..ModuleBindingLayout::default()
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        );
        sources.insert(module.clone(), Arc::new(ParsedModuleUnit::new(module, ModuleKind::Module, None, Arc::from(source), program)));
    }

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: linked_modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: sources.keys().next().cloned().expect("multi-module fixture requires one module"),
        initialization_order: sources.keys().cloned().collect(),
    });

    SemanticWorkspaceInput::new(linked, sources, generation)
}
