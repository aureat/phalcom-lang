use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::{SemanticWorkspaceInput, analyze_workspace};
use std::collections::BTreeMap;
use std::sync::Arc;

fn component(name: &str) -> ModuleComponent {
    ModuleComponent::from_identifier(name).expect("valid module component")
}

fn module(project: u32, path: &[&str]) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(project),
        ModulePath::from_components(path.iter().map(|part| component(part)).collect::<Vec<_>>()),
    )
}

#[allow(clippy::too_many_arguments)]
fn analyze_imported_constructor(
    data_project: u32,
    data_path: &[&str],
    data_kind: ModuleKind,
    consumer_project: u32,
    consumer_path: &[&str],
    package_paths: &[(u32, &[&str], ModuleKind)],
    declaration: &str,
    reexport: bool,
) -> phalcom_semantic::SemanticAnalysis {
    let data_module = module(data_project, data_path);
    let consumer_module = module(consumer_project, consumer_path);
    let mut sources = BTreeMap::new();
    let mut linked_modules = BTreeMap::new();

    for (project, path, kind) in package_paths {
        let package_module = module(*project, path);
        let source: Arc<str> = Arc::from("");
        let program = Arc::new(phalcom_ast::parse(&source, 0).program);
        sources.insert(
            package_module.clone(),
            Arc::new(ParsedModuleUnit::new(package_module.clone(), *kind, None, source, program)),
        );
        linked_modules.insert(
            package_module.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: package_module,
                    kind: *kind,
                    exports: BTreeMap::new(),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout::default(),
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        );
    }

    let data_source: Arc<str> = Arc::from(format!("{declaration}\nexport Secret\n"));
    let data_program = Arc::new(phalcom_ast::parse(&data_source, 0).program);
    sources.insert(
        data_module.clone(),
        Arc::new(ParsedModuleUnit::new(data_module.clone(), data_kind, None, data_source, data_program)),
    );
    linked_modules.insert(
        data_module.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: data_module.clone(),
                kind: data_kind,
                exports: BTreeMap::from([(
                    "Secret".into(),
                    LinkedExport {
                        public_name: "Secret".into(),
                        target: LinkedExportTarget::Binding(SymbolId {
                            module: data_module.clone(),
                            name: "Secret".into(),
                        }),
                        range: Default::default(),
                    },
                )]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Secret".into(), GlobalBindingId(0))]),
                imports: BTreeMap::new(),
            },
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );

    let import_path = data_path.join(".");
    let reexport_source = if reexport { "export Secret\n" } else { "" };
    let consumer_source: Arc<str> = Arc::from(format!(
        "import {import_path}.Secret\n{reexport_source}class Caller {{ @class make() -> Secret {{ Secret(42) }} }}\n"
    ));
    let consumer_program = Arc::new(phalcom_ast::parse(&consumer_source, 0).program);
    sources.insert(
        consumer_module.clone(),
        Arc::new(ParsedModuleUnit::new(
            consumer_module.clone(),
            ModuleKind::Module,
            None,
            consumer_source,
            consumer_program,
        )),
    );
    linked_modules.insert(
        consumer_module.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: consumer_module.clone(),
                kind: ModuleKind::Module,
                exports: if reexport {
                    BTreeMap::from([(
                        "Secret".into(),
                        LinkedExport {
                            public_name: "Secret".into(),
                            target: LinkedExportTarget::Binding(SymbolId {
                                module: module(data_project, data_path),
                                name: "Secret".into(),
                            }),
                            range: Default::default(),
                        },
                    )])
                } else {
                    BTreeMap::new()
                },
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::new(),
                imports: BTreeMap::from([("Secret".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: data_module,
                name: "Secret".into(),
            })],
            runtime_dependencies: vec![module(data_project, data_path)],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: linked_modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: consumer_module,
        initialization_order: sources.keys().cloned().collect(),
    });
    analyze_workspace(SemanticWorkspaceInput::new(linked, sources, 1)).expect("semantic test analysis should succeed")
}

#[test]
fn package_constructor_access_uses_canonical_topology() {
    let analysis = analyze_imported_constructor(
        1,
        &["pkg", "model"],
        ModuleKind::Module,
        1,
        &["pkg", "client"],
        &[(1, &["pkg"], ModuleKind::Package)],
        "data Secret package(_ value: Int)",
        false,
    );
    assert!(
        !analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "same-package construction should be allowed: {:?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn module_constructor_access_requires_the_exact_defining_module() {
    let same_module = phalcom_modules::identity::ModuleId::universe_root();
    let source: Arc<str> = Arc::from("data Secret module(_ value: Int)\nclass Caller { @class make() -> Secret { Secret(42) } }\n");
    let parsed = phalcom_ast::parse(&source, 0);
    let same_analysis = phalcom_semantic::analyze_single_module(same_module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");
    assert!(
        !same_analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "same-module construction should be allowed: {:?}",
        same_analysis.snapshot.diagnostics
    );

    let other_analysis = analyze_imported_constructor(1, &["api"], ModuleKind::Module, 1, &["client"], &[], "data Secret module(_ value: Int)", false);
    assert!(
        other_analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "other-module construction should be denied: {:?}",
        other_analysis.snapshot.diagnostics
    );
}

#[test]
fn private_constructor_access_requires_the_exact_declaring_owner() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("data Secret private(_ value: Int)\nclass Caller { @class make() -> Secret { Secret(42) } }\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);

    let analysis = phalcom_semantic::analyze_single_module(module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "a different declaration in the same module must not call a private data constructor: {:?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn package_constructor_access_rejects_nested_and_sibling_packages() {
    for (consumer_path, packages) in [
        (
            &["pkg", "nested", "client"][..],
            vec![(1, &["pkg"][..], ModuleKind::Package), (1, &["pkg", "nested"][..], ModuleKind::Package)],
        ),
        (
            &["other", "client"][..],
            vec![(1, &["pkg"][..], ModuleKind::Package), (1, &["other"][..], ModuleKind::Package)],
        ),
    ] {
        let analysis = analyze_imported_constructor(
            1,
            &["pkg", "model"],
            ModuleKind::Module,
            1,
            consumer_path,
            &packages,
            "data Secret package(_ value: Int)",
            false,
        );
        assert!(
            analysis
                .snapshot
                .all_diagnostics()
                .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
            "package boundary should deny {:?}: {:?}",
            consumer_path,
            analysis.snapshot.diagnostics
        );
    }
}

#[test]
fn internal_constructor_access_requires_the_same_project_identity() {
    let same_project = analyze_imported_constructor(
        1,
        &["api"],
        ModuleKind::Module,
        1,
        &["client"],
        &[],
        "data Secret internal(_ value: Int)",
        false,
    );
    assert!(
        !same_project
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "same-project construction should be allowed: {:?}",
        same_project.snapshot.diagnostics
    );

    let other_project = analyze_imported_constructor(
        1,
        &["api"],
        ModuleKind::Module,
        2,
        &["client"],
        &[],
        "data Secret internal(_ value: Int)",
        false,
    );
    assert!(
        other_project
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "other-project construction should be denied: {:?}",
        other_project.snapshot.diagnostics
    );
}

#[test]
fn reexported_data_name_does_not_widen_constructor_access() {
    let analysis = analyze_imported_constructor(1, &["api"], ModuleKind::Module, 1, &["facade"], &[], "data Secret private(_ value: Int)", true);
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "re-exported name must not widen constructor access: {:?}",
        analysis.snapshot.diagnostics
    );
}
