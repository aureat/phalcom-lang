use phalcom_modules::UNIVERSE_NODES;
use phalcom_modules::builtin::UniverseSourceProvider;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{FilesystemSourceProvider, ModuleKind, SourceProvider};
use std::path::PathBuf;

fn path(parts: &[&str]) -> ModulePath {
    ModulePath::from_components(
        parts
            .iter()
            .map(|part| ModuleComponent::from_identifier(part).expect("valid identifier"))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn test_builtin_universe_root_is_package() {
    let provider = UniverseSourceProvider::new();
    let universe_root_id = ModuleId::universe(ModulePath::root());
    let iface = provider.load_interface(&universe_root_id).expect("builtin universe root must load interface");
    assert_eq!(iface.kind, ModuleKind::Package);
}

#[test]
fn test_universe_platform_children_are_packages() {
    let provider = UniverseSourceProvider::new();
    for child in [
        "io",
        "fs",
        "path",
        "text",
        "regex",
        "json",
        "math",
        "random",
        "time",
        "process",
        "net",
        "concurrent",
        "testing",
    ] {
        let id = ModuleId::universe(path(&[child]));
        assert_eq!(provider.load_interface(&id).unwrap().kind, ModuleKind::Package);
    }
}

#[test]
fn test_project_root_package_is_package_not_project() {
    let mut universe = ProjectUniverse::new();
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../phalcom-core/tests/fixtures/universe_v1/project_hierarchy/project.toml")
        .canonicalize()
        .expect("fixture project.toml exists");

    let root_project_id = universe.load_root(&manifest_path).expect("project loads");
    let project = universe.get_project(root_project_id).unwrap();

    let fs_provider = FilesystemSourceProvider::new();
    let root_unit = fs_provider.locate(project, &ModulePath::root()).expect("root package locates");

    assert_eq!(root_unit.kind, ModuleKind::Package);
}

#[test]
fn test_all_universe_nodes_have_consistent_source_kind() {
    let provider = UniverseSourceProvider::new();
    for node in UNIVERSE_NODES {
        let id = ModuleId::universe(path(node.path));
        let iface = provider
            .load_interface(&id)
            .unwrap_or_else(|e| panic!("failed to load interface for {id}: {e}"));
        let _src = provider.source_text(&id).unwrap_or_else(|e| panic!("missing source text for {id}: {e}"));
        assert_eq!(iface.kind, node.kind, "node {id} must retain catalog kind");
    }
}
