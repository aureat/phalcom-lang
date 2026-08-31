use phalcom_modules::{BuiltinPackage, BuiltinProjectSourceProvider, ModuleComponent, ModuleId, ModuleKind, ModulePath};

fn path(parts: &[&str]) -> ModulePath {
    ModulePath::from_components(parts.iter().map(|part| ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>())
}

#[test]
fn builtin_roots_are_packages_and_nested_nodes_have_stable_virtual_uris() {
    let universe = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let root = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    assert_eq!(universe.load_interface(&root).unwrap().kind, ModuleKind::Package);

    let selector = ModuleId::builtin(BuiltinPackage::Universe, path(&["reflection", "selector"]));
    assert!(universe.contains(&selector.path));
    // ROOT-05
    assert_eq!(universe.source_id(&selector).unwrap().to_string(), "phalcom://universe/reflection/selector");

    let std_json = ModuleId::builtin(BuiltinPackage::Std, path(&["json"]));
    let std_provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Std);
    // ROOT-05
    assert_eq!(std_provider.source_id(&std_json).unwrap().to_string(), "phalcom://std/json");
    assert_eq!(std_provider.load_interface(&std_json).unwrap().kind, ModuleKind::Package);
}

#[test]
fn test_root_01_and_02_and_05_builtin_providers_source_text_embedded() {
    let universe = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let root_id = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    let src = universe.source_text(&root_id).unwrap();
    assert!(src.contains("expose .object"));

    let selector_id = ModuleId::builtin(BuiltinPackage::Universe, path(&["reflection", "selector"]));
    let sel_src = universe.source_text(&selector_id).unwrap();
    assert!(sel_src.contains("First-class dispatch selector representation"));

    let std_provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Std);
    let json_id = ModuleId::builtin(BuiltinPackage::Std, path(&["json"]));
    let json_src = std_provider.source_text(&json_id).unwrap();
    assert!(json_src.contains("export parse"));
}
