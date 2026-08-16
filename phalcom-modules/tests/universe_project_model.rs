use phalcom_modules::{BuiltinProject, BuiltinProjectSourceProvider, ModuleComponent, ModuleId, ModuleKind, ModulePath};

fn path(parts: &[&str]) -> ModulePath {
    ModulePath::from_components(parts.iter().map(|part| ModuleComponent::from_identifier(part).unwrap()).collect::<Vec<_>>())
}

#[test]
fn builtin_roots_are_projects_and_nested_nodes_have_stable_virtual_uris() {
    let universe = BuiltinProjectSourceProvider::new(BuiltinProject::Universe);
    let root = ModuleId::builtin(BuiltinProject::Universe, ModulePath::root());
    assert_eq!(universe.load_interface(&root).unwrap().kind, ModuleKind::ProjectRoot);

    let selector = ModuleId::builtin(BuiltinProject::Universe, path(&["reflection", "selector"]));
    assert!(universe.contains(&selector.path));
    assert_eq!(universe.source_id(&selector).unwrap().to_string(), "phalcom://universe/reflection/selector");

    let std_json = ModuleId::builtin(BuiltinProject::Std, path(&["json"]));
    let std_provider = BuiltinProjectSourceProvider::new(BuiltinProject::Std);
    assert_eq!(std_provider.source_id(&std_json).unwrap().to_string(), "phalcom://std/json");
    assert_eq!(std_provider.load_interface(&std_json).unwrap().kind, ModuleKind::Module);
}
