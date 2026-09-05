use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::{SemanticAnalysis, SemanticWorkspaceInput, analyze_workspace};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Small builder for real multi-module semantic inputs used by integration tests.
pub struct WorkspaceFixture {
    modules: BTreeMap<String, Arc<str>>,
    entry: Option<String>,
}

/// Analyzed workspace plus stable names for its module identities.
pub struct WorkspaceAnalysisFixture {
    pub analysis: SemanticAnalysis,
    modules: BTreeMap<String, ModuleId>,
}

impl WorkspaceFixture {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            entry: None,
        }
    }

    pub fn module(mut self, name: impl Into<String>, source: impl Into<Arc<str>>) -> Self {
        self.modules.insert(name.into(), source.into());
        self
    }

    pub fn entry(mut self, name: impl Into<String>) -> Self {
        self.entry = Some(name.into());
        self
    }

    pub fn analyze(self) -> WorkspaceAnalysisFixture {
        assert!(!self.modules.is_empty(), "workspace fixture needs at least one module");
        let project = ResolvedProjectId::from_raw(1);
        let module_ids = self
            .modules
            .keys()
            .map(|name| (name.clone(), module_id(project, name)))
            .collect::<BTreeMap<_, _>>();

        let mut sources = BTreeMap::new();
        let mut linked_modules = BTreeMap::new();
        for (name, source) in &self.modules {
            let module = module_ids.get(name).expect("module id created above").clone();
            let parsed = phalcom_ast::parse(source, 0);
            assert!(parsed.errors.is_empty(), "parse errors in `{name}`: {:#?}", parsed.errors);
            sources.insert(
                module.clone(),
                Arc::new(ParsedModuleUnit::new(
                    module.clone(),
                    ModuleKind::Module,
                    None,
                    source.clone(),
                    Arc::new(parsed.program),
                )),
            );
            linked_modules.insert(
                module.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module,
                        kind: ModuleKind::Module,
                        exports: BTreeMap::new(),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout::default(),
                    linked_reads: Vec::new(),
                    runtime_dependencies: Vec::new(),
                },
            );
        }

        let entry_name = self.entry.or_else(|| self.modules.keys().next().cloned()).expect("non-empty modules");
        let entry = module_ids.get(&entry_name).expect("entry module exists").clone();
        let initialization_order = module_ids.values().cloned().collect();
        let linked = Arc::new(LinkedProgram {
            universe: Arc::new(ProjectUniverse::new()),
            modules: linked_modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry,
            initialization_order,
        });

        let analysis = analyze_workspace(SemanticWorkspaceInput::new(
            linked,
            sources,
            1,
        ));
        assert!(
            analysis.snapshot.internal_incidents.is_empty(),
            "semantic analyzer produced internal incidents: {:#?}",
            analysis.snapshot.internal_incidents
        );

        WorkspaceAnalysisFixture { analysis, modules: module_ids }
    }
}

impl WorkspaceAnalysisFixture {
    pub fn module(&self, name: &str) -> &ModuleId {
        self.modules.get(name).unwrap_or_else(|| panic!("missing workspace module `{name}`"))
    }

    pub fn decl(&self, module: &str, name: &str) -> DeclarationId {
        DeclarationId::new(self.module(module).clone(), name.into())
    }
}

fn module_id(project: ResolvedProjectId, name: &str) -> ModuleId {
    let components: Vec<ModuleComponent> = name
        .split('.')
        .map(|component| ModuleComponent::from_identifier(component).expect("valid fixture module component"))
        .collect();
    ModuleId::resolved(project, ModulePath::from_components(components))
}
