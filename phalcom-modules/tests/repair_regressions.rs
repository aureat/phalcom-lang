use phalcom_ast::ast::{ImportPath, ImportRoot, PathSegment};
use phalcom_ast::parser::parse;
use phalcom_modules::{
    FilesystemSourceProvider, InterfaceBuilder, InterfaceError, LinkedExportTarget, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModulePath,
    ModuleResolutionError, ModuleResolver, ProjectError, ProjectManifest, ProjectUniverse, ResolvedProjectId, SourceProvider,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_real_project_cannot_equal_core_identity() {
    let mut universe = ProjectUniverse::new();
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"corelib\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/core.ph"), "class MyCore {}\n").unwrap();

    let root_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    assert_ne!(root_id, ResolvedProjectId::RESERVED);

    let module_id = ModuleId {
        project: root_id,
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("core").unwrap()]),
    };
    assert_ne!(module_id, ModuleId::core());
}

#[test]
fn test_distinct_synthetic_modules_have_distinct_identities() {
    let syn1 = ModuleId::synthetic("mod1");
    let syn2 = ModuleId::synthetic("mod2");
    assert_ne!(syn1, syn2);
}

#[test]
fn test_export_before_class_declaration_succeeds_order_independent() {
    let source = "export Foo\nclass Foo {}\n";
    let program = parse(source, 0).program;
    let module_id = ModuleId::synthetic("test");
    let interface = InterfaceBuilder::build(module_id, ModuleKind::Module, &program).expect("export before class should succeed");
    assert!(interface.exports.contains_key("Foo"));
    assert!(interface.declarations.contains_key("Foo"));
}

#[test]
fn test_import_body_declaration_collision_rejected() {
    let source = "import .other\nclass other {}\n";
    let program = parse(source, 0).program;
    let module_id = ModuleId::synthetic("test");
    let res = InterfaceBuilder::build(module_id, ModuleKind::Module, &program);
    assert!(matches!(res, Err(InterfaceError::DuplicateImportBinding { .. })));
}

#[test]
fn test_whole_module_reexport_links_to_module_target() {
    let a = ModuleId {
        project: ResolvedProjectId::from_raw(1),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("a").unwrap()]),
    };
    let b = ModuleId {
        project: ResolvedProjectId::from_raw(1),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("b").unwrap()]),
    };

    let prog_a = parse("class Helper {}\nexport Helper\n", 0).program;
    let iface_a = InterfaceBuilder::build(a.clone(), ModuleKind::Module, &prog_a).unwrap();

    // Module B imports A and re-exports A
    let prog_b = parse("import .a as sub\nexport sub\n", 0).program;
    let iface_b = InterfaceBuilder::build(b.clone(), ModuleKind::Module, &prog_b).unwrap();

    let mut interfaces = BTreeMap::new();
    interfaces.insert(a.clone(), iface_a);
    interfaces.insert(b.clone(), iface_b);

    let resolved = BTreeMap::from([((b.clone(), ".a".to_string()), a.clone())]);
    let linker = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces);
    let linked = linker.link(b.clone(), &resolved).expect("linking whole-module export should succeed");

    let sub_export = &linked.modules[&b].interface.exports["sub"];
    assert_eq!(sub_export.target, LinkedExportTarget::Module(a));
}

#[test]
fn test_manifest_with_both_path_and_package_rejected() {
    let toml = r#"
[project]
name = "bad-manifest"

[dependencies]
foo = { path = "../foo", package = "foo", version = "1.0" }
"#;
    let manifest = ProjectManifest::parse(toml).unwrap();
    let err = manifest.validate().unwrap_err();
    assert!(matches!(err, ProjectError::InvalidProjectManifest(_)));
}

#[test]
fn test_manifest_cycle_diagnostic_closes_on_repeated_node() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // A -> B -> C -> B (cycle is B -> C -> B)
    let a_dir = root.join("a");
    let b_dir = root.join("b");
    let c_dir = root.join("c");

    fs::create_dir_all(a_dir.join("src")).unwrap();
    fs::create_dir_all(b_dir.join("src")).unwrap();
    fs::create_dir_all(c_dir.join("src")).unwrap();

    fs::write(a_dir.join("src/package.ph"), "").unwrap();
    fs::write(b_dir.join("src/package.ph"), "").unwrap();
    fs::write(c_dir.join("src/package.ph"), "").unwrap();

    fs::write(a_dir.join("project.toml"), "[project]\nname = \"a\"\n[dependencies]\nb = { path = \"../b\" }\n").unwrap();
    fs::write(b_dir.join("project.toml"), "[project]\nname = \"b\"\n[dependencies]\nc = { path = \"../c\" }\n").unwrap();
    fs::write(c_dir.join("project.toml"), "[project]\nname = \"c\"\n[dependencies]\nb = { path = \"../b\" }\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let err = universe.load_root(a_dir.join("project.toml")).unwrap_err();
    if let ProjectError::ProjectDependencyCycle { chain } = err {
        assert!(chain.contains("b → c → b"), "chain was: {chain}");
        assert!(!chain.starts_with("a → b → c → b"), "chain should close on B: {chain}");
    } else {
        panic!("expected ProjectDependencyCycle, got {err:?}");
    }
}

#[test]
fn test_confinement_violation_for_root_package() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\n").unwrap();
    // Valid root package
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(proj_id).unwrap();

    let provider = FilesystemSourceProvider::new();
    let unit = provider.locate(project, &ModulePath::root()).unwrap();
    assert_eq!(unit.kind, ModuleKind::Package);
}

#[test]
fn test_duplicate_source_identity_detected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/alpha.ph"), "class Alpha {}\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(proj_id).unwrap();

    let provider = FilesystemSourceProvider::new();
    let alpha_path = ModulePath::from_components(vec![ModuleComponent::from_identifier("alpha").unwrap()]);
    let unit1 = provider.locate(project, &alpha_path).unwrap();
    assert_eq!(unit1.kind, ModuleKind::Module);

    // If another logical path points to the same canonical file, DuplicateSourceIdentity is returned
    // We can simulate this by resolving through a different project/id or path alias
    let project2_id = ResolvedProjectId::from_raw(99);
    let mut project2 = project.clone();
    project2.id = project2_id;
    let res = provider.locate(&project2, &alpha_path);
    assert!(matches!(res, Err(ModuleResolutionError::DuplicateSourceIdentity(_))));
}

#[test]
fn test_import_core_resolves_reserved_root() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let source_provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &source_provider);

    let importer_id = ModuleId {
        project: proj_id,
        path: ModulePath::root(),
    };
    let import_core = ImportPath {
        root: ImportRoot::Absolute(PathSegment {
            name: "core".to_string(),
            range: (0..4).into(),
        }),
        segments: Vec::new(),
        range: (0..4).into(),
    };

    let resolved = resolver.resolve_import(&importer_id, &import_core).expect("import core should resolve");
    assert_eq!(resolved.id, ModuleId::core());
}
