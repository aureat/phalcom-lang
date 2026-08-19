use phalcom_modules::builtin::BuiltinProjectSourceProvider;
use phalcom_modules::identity::{BuiltinPackage, ModuleComponent, ModuleId, ModulePath};
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
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let universe_root_id = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    let iface = provider.load_interface(&universe_root_id).expect("builtin universe root must load interface");
    assert_eq!(iface.kind, ModuleKind::Package);
}

#[test]
fn test_std_root_is_package() {
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Std);
    let std_root_id = ModuleId::builtin(BuiltinPackage::Std, ModulePath::root());
    let iface = provider.load_interface(&std_root_id).expect("builtin std root must load interface");
    assert_eq!(iface.kind, ModuleKind::Package);
}

#[test]
fn test_std_json_fs_path_are_packages_because_backed_by_package_ph() {
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Std);

    let json_id = ModuleId::builtin(BuiltinPackage::Std, path(&["json"]));
    let json_iface = provider.load_interface(&json_id).expect("builtin std.json must load interface");
    assert_eq!(json_iface.kind, ModuleKind::Package);

    let fs_id = ModuleId::builtin(BuiltinPackage::Std, path(&["fs"]));
    let fs_iface = provider.load_interface(&fs_id).expect("builtin std.fs must load interface");
    assert_eq!(fs_iface.kind, ModuleKind::Package);

    let path_id = ModuleId::builtin(BuiltinPackage::Std, path(&["path"]));
    let path_iface = provider.load_interface(&path_id).expect("builtin std.path must load interface");
    assert_eq!(path_iface.kind, ModuleKind::Package);
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
fn test_all_builtin_nodes_have_consistent_source_kind() {
    let universe_provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let std_provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Std);

    for (pkg, provider, node_paths) in [
        (
            BuiltinPackage::Universe,
            universe_provider,
            vec![
                vec![],
                vec!["object"],
                vec!["object", "object"],
                vec!["object", "behavior"],
                vec!["object", "class"],
                vec!["object", "metaclass"],
                vec!["scalar"],
                vec!["scalar", "number"],
                vec!["scalar", "string"],
                vec!["scalar", "bool"],
                vec!["scalar", "symbol"],
                vec!["callable"],
                vec!["callable", "function"],
                vec!["callable", "closure"],
                vec!["callable", "method"],
                vec!["callable", "family"],
                vec!["option"],
                vec!["collections"],
                vec!["collections", "iterable"],
                vec!["collections", "list"],
                vec!["collections", "map"],
                vec!["collections", "set"],
                vec!["collections", "tuple"],
                vec!["collections", "record"],
                vec!["collections", "range"],
                vec!["collections", "bytes"],
                vec!["errors"],
                vec!["errors", "error"],
                vec!["errors", "argument"],
                vec!["errors", "indexing"],
                vec!["errors", "contracts"],
                vec!["reflection"],
                vec!["reflection", "module"],
                vec!["reflection", "package_object"],
                vec!["reflection", "project"],
                vec!["reflection", "selector"],
                vec!["reflection", "message"],
                vec!["reflection", "attribute"],
                vec!["concurrency"],
                vec!["concurrency", "fiber"],
            ],
        ),
        (
            BuiltinPackage::Std,
            std_provider,
            vec![
                vec![],
                vec!["io"],
                vec!["fs"],
                vec!["path"],
                vec!["text"],
                vec!["regex"],
                vec!["json"],
                vec!["math"],
                vec!["random"],
                vec!["time"],
                vec!["process"],
                vec!["net"],
                vec!["concurrent"],
                vec!["testing"],
            ],
        ),
    ] {
        for parts in node_paths {
            let mpath = path(&parts);
            let id = ModuleId::builtin(pkg, mpath.clone());
            let iface = provider
                .load_interface(&id)
                .unwrap_or_else(|e| panic!("failed to load interface for {id}: {e}"));
            let _src = provider.source_text(&id).unwrap_or_else(|e| panic!("missing source text for {id}: {e}"));

            // Check that package directories/roots are ModuleKind::Package and leaf .ph are ModuleKind::Module
            let is_package = parts.is_empty() || (pkg == BuiltinPackage::Universe && (parts.len() == 1)) || (pkg == BuiltinPackage::Std && (parts.len() == 1));

            if is_package {
                assert_eq!(iface.kind, ModuleKind::Package, "node {id} must be ModuleKind::Package");
            } else {
                assert_eq!(iface.kind, ModuleKind::Module, "node {id} must be ModuleKind::Module");
            }
        }
    }
}
