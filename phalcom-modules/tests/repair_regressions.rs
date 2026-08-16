use phalcom_ast::ast::{ImportPath, ImportRoot, PathSegment};
use phalcom_ast::parser::parse;
use phalcom_modules::{
    FilesystemSourceProvider, InterfaceBuilder, InterfaceError, LinkedExportTarget, ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModuleLoadError,
    ModulePath, ModuleResolutionError, ModuleResolver, ProjectError, ProjectIdentity, ProjectManifest, ProjectUniverse, ResolvedProjectId, SourceProvider,
    SyntheticProjectIdAllocator,
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
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"corelib\"\nversion = \"0.1.0\"\nnamespace = \"corelib\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/core.ph"), "class MyCore {}\n").unwrap();

    let root_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    assert!(matches!(ProjectIdentity::from(root_id), ProjectIdentity::Resolved(_)));

    let module_id = ModuleId {
        project: root_id.into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("core").unwrap()]),
    };
    assert_ne!(module_id, ModuleId::core());
    assert!(matches!(ModuleId::core().project, ProjectIdentity::Builtin(_)));
}

#[test]
fn test_distinct_same_named_synthetic_modules_have_distinct_identities() {
    let mut ids = SyntheticProjectIdAllocator::default();
    let syn1 = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let syn2 = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    assert_ne!(syn1, syn2, "display/logical names must not manufacture semantic identity");
}

#[test]
fn test_export_before_class_declaration_succeeds_order_independent() {
    let source = "export Foo\nclass Foo {}\n";
    let program = parse(source, 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let module_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let interface = InterfaceBuilder::build(module_id, ModuleKind::Module, &program).expect("export before class should succeed");
    assert!(interface.exports.contains_key("Foo"));
    assert!(interface.declarations.contains_key("Foo"));
}

#[test]
fn test_import_body_declaration_collision_rejected() {
    let source = "import .other\nclass other {}\n";
    let program = parse(source, 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let module_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let res = InterfaceBuilder::build(module_id, ModuleKind::Module, &program);
    assert!(matches!(res, Err(InterfaceError::DuplicateBinding { .. })));
}

#[test]
fn test_duplicate_class_declaration_rejected() {
    let program = parse("class Foo {}\nclass Foo {}\n", 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let res = InterfaceBuilder::build(ModuleId::synthetic(ids.allocate(), ModulePath::root()), ModuleKind::Module, &program);
    assert!(matches!(res, Err(InterfaceError::DuplicateDeclaration { ref name, .. }) if name == "Foo"));
}

#[test]
fn test_duplicate_let_declaration_rejected() {
    let program = parse("let value = 1\nlet value = 2\n", 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let res = InterfaceBuilder::build(ModuleId::synthetic(ids.allocate(), ModulePath::root()), ModuleKind::Module, &program);
    assert!(matches!(res, Err(InterfaceError::DuplicateDeclaration { ref name, .. }) if name == "value"));
}

#[test]
fn test_cross_kind_duplicate_declaration_rejected() {
    let program = parse("class Foo {}\nlet Foo = 1\n", 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let res = InterfaceBuilder::build(ModuleId::synthetic(ids.allocate(), ModulePath::root()), ModuleKind::Module, &program);
    assert!(matches!(res, Err(InterfaceError::DuplicateDeclaration { ref name, .. }) if name == "Foo"));
}

#[test]
fn test_project_display_name_remains_distinct_from_namespace() {
    let manifest = ProjectManifest::parse(
        "[project]\nname = \"geometry-toolkit\"\nnamespace = \"geometry_toolkit\"\n",
    )
    .unwrap();
    let validated = manifest.validate().unwrap();
    assert_eq!(validated.name, "geometry-toolkit");
    assert_eq!(validated.namespace.as_str(), "geometry_toolkit");
}

#[test]
fn test_module_load_error_preserves_parse_error_and_span() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    let invalid_source = "class {\n";
    let pkg_file = proj_dir.join("src/package.ph");
    fs::write(&pkg_file, invalid_source).unwrap();

    let expected = parse(invalid_source, 0).errors.into_iter().next().expect("fixture must contain a parse error");
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let source_provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &source_provider);
    let module_id = ModuleId { project: project_id.into(), path: ModulePath::root() };

    let err = resolver.load_interface(&module_id).unwrap_err();
    match err {
        ModuleLoadError::Parse { source, error, .. } => {
            assert_eq!(source, fs::canonicalize(&pkg_file).unwrap());
            assert_eq!(error, expected);
        }
        other => panic!("expected typed parse error, got {other:?}"),
    }
}

#[test]
fn duplicate_body_declarations_are_rejected_instead_of_overwriting() {
    let source = "class Thing {}\nclass Thing {}\n";
    let program = parse(source, 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let module_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let err = InterfaceBuilder::build(module_id, ModuleKind::Module, &program).unwrap_err();
    assert!(matches!(err, InterfaceError::DuplicateDeclaration { ref name, .. } if name == "Thing"));
}

#[test]
fn class_and_let_share_one_checked_module_namespace() {
    let source = "class Thing {}\nlet Thing = 1\n";
    let program = parse(source, 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let module_id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let err = InterfaceBuilder::build(module_id, ModuleKind::Module, &program).unwrap_err();
    assert!(matches!(err, InterfaceError::DuplicateDeclaration { ref name, .. } if name == "Thing"));
}

#[test]
fn test_whole_module_reexport_links_to_module_target() {
    let a = ModuleId {
        project: ResolvedProjectId::from_raw(1).into(),
        path: ModulePath::from_components(vec![ModuleComponent::from_identifier("a").unwrap()]),
    };
    let b = ModuleId {
        project: ResolvedProjectId::from_raw(1).into(),
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
namespace = "bad_manifest"

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

    fs::write(a_dir.join("project.toml"), "[project]\nname = \"a\"\nnamespace = \"a\"\n[dependencies]\nb = { path = \"../b\" }\n").unwrap();
    fs::write(b_dir.join("project.toml"), "[project]\nname = \"b\"\nnamespace = \"b\"\n[dependencies]\nc = { path = \"../c\" }\n").unwrap();
    fs::write(c_dir.join("project.toml"), "[project]\nname = \"c\"\nnamespace = \"c\"\n[dependencies]\nb = { path = \"../b\" }\n").unwrap();

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
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    // Valid root package
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(proj_id).unwrap();

    let provider = FilesystemSourceProvider::new();
    let unit = provider.locate(project, &ModulePath::root()).unwrap();
    assert_eq!(unit.kind, ModuleKind::Package);
}

#[cfg(unix)]
#[test]
fn test_symlinked_root_package_escaping_source_root_is_rejected() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    let outside = tmp.path().join("outside-package.ph");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(&outside, "").unwrap();
    symlink(&outside, proj_dir.join("src/package.ph")).unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(proj_id).unwrap();
    let provider = FilesystemSourceProvider::new();
    let err = provider.locate(project, &ModulePath::root()).unwrap_err();
    assert!(matches!(err, ModuleResolutionError::ImportOutsideSourceRoot(_, _)));
}

#[cfg(unix)]
#[test]
fn test_duplicate_source_identity_detected_through_symlink_alias() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/alpha.ph"), "class Alpha {}\n").unwrap();
    symlink(proj_dir.join("src/alpha.ph"), proj_dir.join("src/beta.ph")).unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(proj_id).unwrap();
    let provider = FilesystemSourceProvider::new();
    let alpha = ModulePath::from_components(vec![ModuleComponent::from_identifier("alpha").unwrap()]);
    let beta = ModulePath::from_components(vec![ModuleComponent::from_identifier("beta").unwrap()]);
    provider.locate(project, &alpha).unwrap();
    let err = provider.locate(project, &beta).unwrap_err();
    assert!(matches!(err, ModuleResolutionError::DuplicateSourceIdentity(_)));
}

#[test]
fn test_duplicate_source_identity_detected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
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
fn canonical_logical_and_physical_component_spellings_are_one_to_one() {
    let logical = ModuleComponent::from_identifier("module_b").unwrap();
    assert_eq!(logical.to_kebab(), "module-b");
    assert_eq!(ModuleComponent::from_kebab("module-b").unwrap(), logical);
    assert!(ModuleComponent::from_identifier("module-b").is_err());
    assert!(ModuleComponent::from_identifier("Module_b").is_err());
    assert!(ModuleComponent::from_kebab("module_b").is_err());
    assert!(ModuleComponent::from_kebab("Module-b").is_err());
}

#[test]
fn noncanonical_physical_snake_case_module_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/private_tool.ph"), "class Tool {}\n").unwrap();
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(project_id).unwrap();
    let provider = FilesystemSourceProvider::new();
    let logical = ModulePath::from_components(vec![ModuleComponent::from_identifier("private_tool").unwrap()]);
    assert!(matches!(provider.locate(project, &logical), Err(ModuleResolutionError::NonCanonicalPhysicalName { .. })));
}

#[test]
fn resolver_negative_cache_is_generation_scoped() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(project_id).unwrap();
    let provider = FilesystemSourceProvider::new();
    let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("later_mod").unwrap()]);

    assert!(provider.locate(project, &path).is_err());
    fs::write(proj_dir.join("src/later-mod.ph"), "let value = 1\n").unwrap();
    assert!(provider.locate(project, &path).is_err(), "same generation must keep its negative cache stable");
    let before = provider.generation();
    provider.clear_cache();
    assert!(provider.generation() > before);
    assert!(provider.locate(project, &path).is_ok(), "new generation must not retain stale negative results");
}

#[cfg(unix)]
#[test]
fn canonical_source_confinement_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    let outside = tmp.path().join("outside.ph");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(&outside, "let escaped = true\n").unwrap();
    symlink(&outside, proj_dir.join("src/escape.ph")).unwrap();
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let project = universe.get_project(project_id).unwrap();
    let provider = FilesystemSourceProvider::new();
    let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("escape").unwrap()]);
    assert!(matches!(provider.locate(project, &path), Err(ModuleResolutionError::ImportOutsideSourceRoot(_, _))));
}

#[test]
fn unknown_unit_metadata_is_inert_on_ordinary_modules() {
    let program = parse("@!package_root(\"opaque\")\nlet value = 1\n", 0).program;
    let mut ids = SyntheticProjectIdAllocator::default();
    let id = ModuleId::synthetic(ids.allocate(), ModulePath::root());
    let interface = InterfaceBuilder::build(id, ModuleKind::Module, &program).unwrap();
    assert_eq!(interface.metadata.attributes.len(), 1);
    assert_eq!(interface.metadata.attributes[0].name, "package_root");
}

#[test]
fn package_surface_preserves_typed_parse_failure() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src/broken")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/broken/package.ph"), "let =\n").unwrap();
    let mut universe = ProjectUniverse::new();
    let project_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &provider);
    let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("broken").unwrap()]);
    assert!(matches!(
        resolver.load_package_surface(project_id, &path),
        Err(ModuleResolutionError::PackageSurface(ref boxed)) if matches!(**boxed, ModuleLoadError::Parse { .. })
    ));
}

#[test]
fn legacy_core_import_is_deliberately_not_a_public_import_root() {
    let tmp = TempDir::new().unwrap();
    let proj_dir = tmp.path().join("proj");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(proj_dir.join("project.toml"), "[project]\nname = \"proj\"\nnamespace = \"proj\"\n").unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();

    let mut universe = ProjectUniverse::new();
    let proj_id = universe.load_root(proj_dir.join("project.toml")).unwrap();
    let source_provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &source_provider);
    let importer_id = ModuleId { project: proj_id.into(), path: ModulePath::root() };
    let import_core = ImportPath {
        root: ImportRoot::Absolute(PathSegment { name: "core".to_string(), range: (0..4).into() }),
        segments: Vec::new(),
        range: (0..4).into(),
    };

    assert!(resolver.resolve_import(&importer_id, &import_core).is_err());
}
