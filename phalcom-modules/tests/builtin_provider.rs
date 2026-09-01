use phalcom_modules::{ModuleId, ModulePath, ProjectIdentity, SyntheticProjectIdAllocator, UniverseSourceProvider};

#[test]
fn builtin_universe_provider_has_virtual_identity_and_exports() {
    let id = ModuleId::universe(ModulePath::root());
    let provider = UniverseSourceProvider::new();
    assert_eq!(provider.source_id(&id).unwrap().0.as_ref(), "phalcom://universe/");
    let interface = provider.load_interface(&id).unwrap();
    assert!(interface.exports.contains_key("Object"));
    assert_eq!(id.project, ProjectIdentity::Universe);
}

#[test]
fn universe_identity_is_disjoint_from_resolved_and_synthetic() {
    let mut allocator = SyntheticProjectIdAllocator;
    assert_ne!(ModuleId::universe_root().project, ProjectIdentity::from(phalcom_modules::ResolvedProjectId::from_raw(1)));
    assert_ne!(ModuleId::universe_root().project, ProjectIdentity::from(allocator.allocate()));
}

#[test]
fn builtin_universe_reflection_children_load() {
    let provider = UniverseSourceProvider::new();
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
        let id = ModuleId::universe(path);
        assert!(provider.contains(&id.path), "module path {id} should exist in provider");
        let iface = provider.load_interface(&id).unwrap_or_else(|_| panic!("interface for {id} should load"));
        assert_eq!(iface.kind, phalcom_modules::ModuleKind::Module);
        let src = provider.source_text(&id).unwrap_or_else(|_| panic!("source text for {id} should load"));
        assert!(!src.is_empty(), "source for {id} should not be empty");
    }
}
