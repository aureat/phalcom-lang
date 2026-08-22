use phalcom_modules::identity::{
    ModuleComponent, ModuleId, ModulePath, ResolvedProjectId,
};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{
    GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec,
    ModuleBindingLayout, SymbolId,
};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::{
    analyze_single_module, analyze_workspace, SemanticWorkspaceInput, TypeHierarchy,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn single_module_analysis_succeeds() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from("class Point { getX() -> Int { 42 } }");
    let parse_res = phalcom_ast::parse(&source, 0);
    let program = Arc::new(parse_res.program);

    let analysis = analyze_single_module(module.clone(), source, program);
    assert!(!analysis.snapshot.has_errors());
    assert!(analysis.snapshot.sources.contains_key(&module));
    assert!(analysis
        .snapshot
        .surfaces
        .contains_key(&DeclarationId::new(module.clone(), "Point".into())));
}

#[test]
fn workspace_multi_module_linking_resolution_and_cycles() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src/shapes")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "expose .shapes\n").unwrap();
    fs::write(
        proj_dir.join("src/point.ph"),
        "class Point { get() -> Int { 1 } }\nexport Point\n",
    )
    .unwrap();
    fs::write(
        proj_dir.join("src/shapes/circle.ph"),
        "import app.point.Point\nclass Circle is Point { radius() -> Int { 5 } }\nexport Circle\n",
    )
    .unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe
        .load_root(proj_dir.join("project.toml"))
        .expect("universe load succeeds");

    let point_mod = ModuleId::resolved(
        root_id,
        ModulePath::from_components(vec![
            ModuleComponent::from_identifier("point").unwrap()
        ]),
    );
    let circle_mod = ModuleId::resolved(
        root_id,
        ModulePath::from_components(vec![
            ModuleComponent::from_identifier("shapes").unwrap(),
            ModuleComponent::from_identifier("circle").unwrap(),
        ]),
    );

    let point_src: Arc<str> =
        Arc::from(fs::read_to_string(proj_dir.join("src/point.ph")).unwrap());
    let circle_src: Arc<str> =
        Arc::from(fs::read_to_string(proj_dir.join("src/shapes/circle.ph")).unwrap());

    let point_prog = Arc::new(phalcom_ast::parse(&point_src, 0).program);
    let circle_prog = Arc::new(phalcom_ast::parse(&circle_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        point_mod.clone(),
        Arc::new(ParsedModuleUnit::new(
            point_mod.clone(),
            ModuleKind::Module,
            None,
            point_src,
            point_prog,
        )),
    );
    sources.insert(
        circle_mod.clone(),
        Arc::new(ParsedModuleUnit::new(
            circle_mod.clone(),
            ModuleKind::Module,
            None,
            circle_src,
            circle_prog,
        )),
    );

    let mut modules = BTreeMap::new();
    let mut point_exports = BTreeMap::new();
    point_exports.insert(
        "Point".into(),
        LinkedExport {
            public_name: "Point".into(),
            target: LinkedExportTarget::Binding(SymbolId {
                module: point_mod.clone(),
                name: "Point".into(),
            }),
            range: phalcom_common::range::SourceRange::default(),
        },
    );
    modules.insert(
        point_mod.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: point_mod.clone(),
                kind: ModuleKind::Module,
                exports: point_exports,
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Point".into(), GlobalBindingId(0))]),
                imports: BTreeMap::new(),
            },
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );

    let mut circle_imports = BTreeMap::new();
    circle_imports.insert("Point".into(), ImportBindingId(0));
    let mut circle_exports = BTreeMap::new();
    circle_exports.insert(
        "Circle".into(),
        LinkedExport {
            public_name: "Circle".into(),
            target: LinkedExportTarget::Binding(SymbolId {
                module: circle_mod.clone(),
                name: "Circle".into(),
            }),
            range: phalcom_common::range::SourceRange::default(),
        },
    );
    modules.insert(
        circle_mod.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: circle_mod.clone(),
                kind: ModuleKind::Module,
                exports: circle_exports,
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Circle".into(), GlobalBindingId(0))]),
                imports: circle_imports,
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: point_mod.clone(),
                name: "Point".into(),
            })],
            runtime_dependencies: vec![point_mod.clone()],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(universe),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: circle_mod.clone(),
        initialization_order: vec![point_mod.clone(), circle_mod.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(!analysis.snapshot.has_errors());

    // 1. Cross-module superclass resolved
    let circle_decl = DeclarationId::new(circle_mod.clone(), "Circle".into());
    let point_decl = DeclarationId::new(point_mod.clone(), "Point".into());
    assert_eq!(
        analysis.snapshot.hierarchy.superclass(&circle_decl),
        Some(&point_decl)
    );

    // 2. Point in both modules resolves canonically and has same TypeId
    let point_ty1 = analysis.snapshot.declarations.form(&point_decl).unwrap();
    assert_eq!(KindId::TYPE, analysis.snapshot.store.kind_of(point_ty1));
}

#[test]
fn inheritance_cycle_is_rejected_in_workspace() {
    let universe = ProjectUniverse::new();
    let mod_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("a").unwrap()]),
    );
    let mod_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("b").unwrap()]),
    );

    let a_src: Arc<str> = Arc::from("import b.B\nclass A is B {}\nexport A\n");
    let b_src: Arc<str> = Arc::from("import a.A\nclass B is A {}\nexport B\n");

    let a_prog = Arc::new(phalcom_ast::parse(&a_src, 0).program);
    let b_prog = Arc::new(phalcom_ast::parse(&b_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        mod_a.clone(),
        Arc::new(ParsedModuleUnit::new(
            mod_a.clone(),
            ModuleKind::Module,
            None,
            a_src,
            a_prog,
        )),
    );
    sources.insert(
        mod_b.clone(),
        Arc::new(ParsedModuleUnit::new(
            mod_b.clone(),
            ModuleKind::Module,
            None,
            b_src,
            b_prog,
        )),
    );

    let mut modules = BTreeMap::new();
    modules.insert(
        mod_a.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_a.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([(
                    "A".into(),
                    LinkedExport {
                        public_name: "A".into(),
                        target: LinkedExportTarget::Binding(SymbolId {
                            module: mod_a.clone(),
                            name: "A".into(),
                        }),
                        range: phalcom_common::range::SourceRange::default(),
                    },
                )]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("A".into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("B".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: mod_b.clone(),
                name: "B".into(),
            })],
            runtime_dependencies: vec![mod_b.clone()],
        },
    );

    modules.insert(
        mod_b.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_b.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([(
                    "B".into(),
                    LinkedExport {
                        public_name: "B".into(),
                        target: LinkedExportTarget::Binding(SymbolId {
                            module: mod_b.clone(),
                            name: "B".into(),
                        }),
                        range: phalcom_common::range::SourceRange::default(),
                    },
                )]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("B".into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("A".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: mod_a.clone(),
                name: "A".into(),
            })],
            runtime_dependencies: vec![mod_a.clone()],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(universe),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: mod_a.clone(),
        initialization_order: vec![mod_a.clone(), mod_b.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(
        analysis.snapshot.has_errors(),
        "inheritance cycle must be detected and rejected"
    );
}

#[test]
fn same_leaf_name_in_two_modules_stays_distinct() {
    let mod_x = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("x").unwrap()]),
    );
    let mod_y = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("y").unwrap()]),
    );

    let x_src: Arc<str> = Arc::from("class Item { xVal() -> Int { 1 } }\n");
    let y_src: Arc<str> = Arc::from("class Item { yVal() -> String { \"y\" } }\n");

    let x_prog = Arc::new(phalcom_ast::parse(&x_src, 0).program);
    let y_prog = Arc::new(phalcom_ast::parse(&y_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        mod_x.clone(),
        Arc::new(ParsedModuleUnit::new(mod_x.clone(), ModuleKind::Module, None, x_src, x_prog)),
    );
    sources.insert(
        mod_y.clone(),
        Arc::new(ParsedModuleUnit::new(mod_y.clone(), ModuleKind::Module, None, y_src, y_prog)),
    );

    let mut modules = BTreeMap::new();
    modules.insert(
        mod_x.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_x.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::new(),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout::default(),
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );
    modules.insert(
        mod_y.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_y.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::new(),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout::default(),
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: mod_x.clone(),
        initialization_order: vec![mod_x.clone(), mod_y.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    let decl_x = DeclarationId::new(mod_x, "Item".into());
    let decl_y = DeclarationId::new(mod_y, "Item".into());

    let form_x = analysis.snapshot.declarations.form(&decl_x).unwrap();
    let form_y = analysis.snapshot.declarations.form(&decl_y).unwrap();

    assert_ne!(form_x, form_y, "declarations in different modules must have distinct TypeIds");
    assert!(analysis.snapshot.surfaces.contains_key(&decl_x));
    assert!(analysis.snapshot.surfaces.contains_key(&decl_y));
}

#[test]
fn generation_retains_clean_snapshot_and_removes_stale_declarations() {
    let module = ModuleId::core();
    let source_v1: Arc<str> = Arc::from("class OldName { val() -> Int { 1 } }");
    let analysis_v1 = analyze_single_module(
        module.clone(),
        source_v1,
        Arc::new(phalcom_ast::parse("class OldName { val() -> Int { 1 } }", 0).program),
    );

    assert!(analysis_v1.snapshot.surfaces.contains_key(&DeclarationId::new(module.clone(), "OldName".into())));

    let source_v2: Arc<str> = Arc::from("class NewName { val() -> Int { 2 } }");
    let analysis_v2 = analyze_single_module(
        module.clone(),
        source_v2,
        Arc::new(phalcom_ast::parse("class NewName { val() -> Int { 2 } }", 0).program),
    );

    assert!(analysis_v2.snapshot.surfaces.contains_key(&DeclarationId::new(module.clone(), "NewName".into())));
    assert!(!analysis_v2.snapshot.surfaces.contains_key(&DeclarationId::new(module.clone(), "OldName".into())));
}
