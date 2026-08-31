use phalcom_modules::{BuiltinPackage, BuiltinProjectSourceProvider, ModuleId, ModulePath, ProjectIdentity};

#[test]
fn builtin_universe_provider_has_virtual_identity_and_exports() {
    let id = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    assert_eq!(provider.source_id(&id).unwrap().0.as_ref(), "phalcom://universe/");
    let interface = provider.load_interface(&id).unwrap();
    assert!(interface.exports.contains_key("Object"));
    assert_eq!(id.project, ProjectIdentity::Builtin(BuiltinPackage::Universe));
}

#[test]
fn builtin_projects_are_disjoint() {
    let universe = ModuleId::builtin(BuiltinPackage::Universe, ModulePath::root());
    let std = ModuleId::builtin(BuiltinPackage::Std, ModulePath::root());
    assert_ne!(universe.project, std.project);
}

#[test]
fn builtin_universe_reflection_children_load() {
    let provider = BuiltinProjectSourceProvider::new(BuiltinPackage::Universe);
    let children = [
        "module",
        "package_object",
        "project",
        "project_manifest",
        "package_info",
        "package_author",
        "package_requirement",
        "resolved_project_dependency",
        "module_dependency",
        "export_table",
        "export",
        "export_kind",
        "child_module_table",
        "module_identity",
        "package_identity",
        "project_identity",
        "uri",
        "selector",
        "message",
        "attribute",
    ];

    for child in children {
        let path = ModulePath::from_components(vec![
            phalcom_modules::ModuleComponent::from_identifier("reflection").unwrap(),
            phalcom_modules::ModuleComponent::from_identifier(child).unwrap(),
        ]);
        let id = ModuleId::builtin(BuiltinPackage::Universe, path);
        assert!(provider.contains(&id.path), "module path {id} should exist in provider");
        let iface = provider.load_interface(&id).unwrap_or_else(|_| panic!("interface for {id} should load"));
        assert_eq!(iface.kind, phalcom_modules::ModuleKind::Module);
        let src = provider.source_text(&id).unwrap_or_else(|_| panic!("source text for {id} should load"));
        assert!(!src.is_empty(), "source for {id} should not be empty");
    }
}
