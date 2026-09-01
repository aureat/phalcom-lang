use phalcom_modules::{ModuleComponent, ModuleId, ModuleKind, ModulePath, UniverseSourceProvider};

fn path(parts: &[&str]) -> ModulePath {
    ModulePath::from_components(parts.iter().map(|part| ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>())
}

#[test]
fn builtin_roots_are_packages_and_nested_nodes_have_stable_virtual_uris() {
    let universe = UniverseSourceProvider::new();
    let root = ModuleId::universe(ModulePath::root());
    assert_eq!(universe.load_interface(&root).unwrap().kind, ModuleKind::Package);

    let selector = ModuleId::universe(path(&["reflection", "selector"]));
    assert!(universe.contains(&selector.path));
    // ROOT-05
    assert_eq!(universe.source_id(&selector).unwrap().to_string(), "phalcom://universe/reflection/selector");

    let universe_json = ModuleId::universe(path(&["json"]));
    // ROOT-05
    assert_eq!(universe.source_id(&universe_json).unwrap().to_string(), "phalcom://universe/json");
    assert_eq!(universe.load_interface(&universe_json).unwrap().kind, ModuleKind::Package);
}

#[test]
fn test_root_01_and_02_and_05_builtin_providers_source_text_embedded() {
    let universe = UniverseSourceProvider::new();
    let root_id = ModuleId::universe(ModulePath::root());
    let src = universe.source_text(&root_id).unwrap();
    assert!(src.contains("expose .object"));

    let selector_id = ModuleId::universe(path(&["reflection", "selector"]));
    let sel_src = universe.source_text(&selector_id).unwrap();
    assert!(sel_src.contains("First-class dispatch selector representation"));

    let json_id = ModuleId::universe(path(&["json"]));
    let json_src = universe.source_text(&json_id).unwrap();
    assert!(json_src.contains("export parse"));
}
