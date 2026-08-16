use phalcom_ast::ast::{ImportPath, ImportRoot, PathSegment};
use phalcom_ast::parser::parse;
use phalcom_modules::{
    BuiltinProject, InterfaceBuilder, InterfaceError, LinkedExportTarget, ModuleComponent, ModuleId, ModuleKind, ModuleLinker,
    ModuleLoadError, ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectIdentity, ProjectManifest,
    ProjectSourceProvider, ProjectUniverse, SessionSourceProvider, SourceProvider,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

fn write_project(dir: &std::path::Path, display: &str, namespace: &str, extra_manifest: &str) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/package.ph"), "").unwrap();
    fs::write(
        dir.join("project.toml"),
        format!(
            "[project]\nname = {display:?}\nnamespace = {namespace:?}\n{extra_manifest}"
        ),
    )
    .unwrap();
}

#[test]
fn real_project_identity_is_disjoint_from_builtin_identity() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    write_project(&proj_dir, "Core Library", "corelib", "");
    fs::write(proj_dir.join("src/core.ph"), "class MyCore {}\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let module_id = ModuleId::resolved(
        root_id,
        ModulePath::from_components(vec![ModuleComponent::from_identifier("core").unwrap()]),
    );
    assert!(matches!(module_id.project, ProjectIdentity::Resolved(_)));
    assert!(matches!(ModuleId::core().project, ProjectIdentity::Builtin(BuiltinProject::Universe)));
    assert_ne!(module_id, ModuleId::core());
}

#[test]
fn distinct_synthetic_sessions_have_distinct_identities_even_for_same_logical_name() {
    let syn1 = ModuleId::synthetic("same_name");
    let syn2 = ModuleId::synthetic("same_name");
    assert_ne!(syn1.project, syn2.project);
    assert_eq!(syn1.path, syn2.path);
}

#[test]
fn export_before_class_declaration_succeeds_order_independent() {
    let program = parse("export Foo\nclass Foo {}\n", 0).program;
    let interface = InterfaceBuilder::build(ModuleId::synthetic("test"), ModuleKind::Module, &program)
        .expect("export before class should succeed");
    assert!(interface.exports.contains_key("Foo"));
    assert!(interface.declarations.contains_key("Foo"));
}

#[test]
fn duplicate_body_declarations_are_rejected() {
    let program = parse("let value = 1\nlet value = 2\n", 0).program;
    let result = InterfaceBuilder::build(ModuleId::synthetic("test"), ModuleKind::Module, &program);
    assert!(matches!(result, Err(InterfaceError::DuplicateDeclaration { .. })));
}

#[test]
fn import_body_declaration_collision_is_rejected() {
    let program = parse("import .other\nclass other {}\n", 0).program;
    let result = InterfaceBuilder::build(ModuleId::synthetic("test"), ModuleKind::Module, &program);
    assert!(matches!(result, Err(InterfaceError::DuplicateImportBinding { .. })));
}

#[test]
fn whole_module_reexport_links_to_module_target() {
    let a = ModuleId::synthetic("a");
    let owner = a.project;
    let b = ModuleId {
        project: owner,
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("b").unwrap()]),
    };
    let prog_a = parse("class Helper {}\nexport Helper\n", 0).program;
    let iface_a = InterfaceBuilder::build(a.clone(), ModuleKind::Module, &prog_a).unwrap();
    let prog_b = parse("import .a as sub\nexport sub\n", 0).program;
    let iface_b = InterfaceBuilder::build(b.clone(), ModuleKind::Module, &prog_b).unwrap();
    let interfaces = BTreeMap::from([(a.clone(), iface_a), (b.clone(), iface_b)]);
    let resolved = BTreeMap::from([((b.clone(), ".a".to_string()), a.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces);
    let linked = linker.link(b.clone(), &resolved).expect("whole-module export should link");
    assert_eq!(linked.modules[&b].interface.exports["sub"].target, LinkedExportTarget::Module(a));
}

#[test]
fn manifest_requires_explicit_namespace_and_rejects_mixed_dependency_forms() {
    let missing_namespace = ProjectManifest::parse("[project]\nname = \"Display Only\"\n").unwrap();
    assert!(matches!(missing_namespace.validate(), Err(ProjectError::MissingProjectNamespace)));

    let mixed = ProjectManifest::parse(
        "[project]\nname = \"Bad\"\nnamespace = \"bad\"\n[dependencies]\nfoo = { path = \"../foo\", package = \"foo\", version = \"1.0\" }\n",
    )
    .unwrap();
    assert!(matches!(mixed.validate(), Err(ProjectError::InvalidProjectManifest(_))));
}

#[test]
fn manifest_cycle_diagnostic_closes_on_repeated_node() {
    let tmp = TempDir::new().unwrap();
    let a_dir = tmp.path().join("a");
    let b_dir = tmp.path().join("b");
    let c_dir = tmp.path().join("c");
    write_project(&a_dir, "A", "a", "[dependencies]\nb = { path = \"../b\" }\n");
    write_project(&b_dir, "B", "b", "[dependencies]\nc = { path = \"../c\" }\n");
    write_project(&c_dir, "C", "c", "[dependencies]\nb = { path = \"../b\" }\n");

    let mut universe = ProjectUniverse::new();
    let err = universe.load_root(a_dir.join("project.toml")).unwrap_err();
    match err {
        ProjectError::ProjectDependencyCycle { chain } => {
            assert!(chain.contains("B → C → B"), "chain was: {chain}");
            assert!(!chain.starts_with("A → B → C → B"), "cycle should start at repeated node: {chain}");
        }
        other => panic!("expected ProjectDependencyCycle, got {other:?}"),
    }
}

#[test]
fn canonical_physical_kebab_name_is_enforced() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    write_project(&proj_dir, "Project", "proj", "");
    fs::write(proj_dir.join("src/private_tool.ph"), "class Secret {}\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let provider = ProjectSourceProvider::new(&universe);
    let id = ModuleId::resolved(
        proj_id,
        ModulePath::from_components(vec![ModuleComponent::from_identifier("private_tool").unwrap()]),
    );
    let err = provider.locate(&id).unwrap_err();
    assert!(matches!(
        err,
        ModuleLoadError::Resolution(ModuleResolutionError::NonCanonicalPhysicalName { .. })
    ));
}

#[test]
fn standalone_module_cannot_discover_sibling_source() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("main.ph"), "import .helper\n").unwrap();
    fs::write(tmp.path().join("helper.ph"), "let value = 1\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let synthetic = universe.allocate_synthetic_id();
    let provider = SessionSourceProvider::standalone_module(synthetic, tmp.path().join("main.ph")).unwrap();
    let entry = provider.entry_id().unwrap().clone();
    let mut resolver = ModuleResolver::new(&universe, &provider);
    let import = ImportPath {
        root: ImportRoot::Relative { dots: 1, range: (0..1).into() },
        segments: vec![PathSegment { name: "helper".to_string(), range: (1..7).into() }],
        range: (0..7).into(),
    };
    let err = resolver.resolve_import(&entry, &import).unwrap_err();
    assert!(matches!(
        err,
        ModuleLoadError::Resolution(ModuleResolutionError::StandaloneSiblingImport { .. })
    ));
}

#[test]
fn core_alias_and_universe_root_share_one_builtin_identity() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    write_project(&proj_dir, "Project", "proj", "");

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let provider = SessionSourceProvider::project(&universe);
    let mut resolver = ModuleResolver::new(&universe, &provider);
    let importer = ModuleId::resolved(proj_id, ModulePath::root());

    for root_name in ["core", "universe"] {
        let import = ImportPath {
            root: ImportRoot::Absolute(PathSegment { name: root_name.to_string(), range: (0..root_name.len()).into() }),
            segments: Vec::new(),
            range: (0..root_name.len()).into(),
        };
        let resolved = resolver.resolve_import(&importer, &import).expect("builtin root should resolve");
        assert_eq!(resolved.id, ModuleId::universe());
        assert_eq!(resolved.kind, ModuleKind::Package);
    }
}
