use phalcom_modules::{BuiltinProject, BuiltinProjectSourceProvider, ModuleId, ModulePath, ProjectIdentity};

#[test]
fn builtin_universe_provider_has_virtual_identity_and_exports() {
    let id = ModuleId::builtin(BuiltinProject::Universe, ModulePath::root());
    let provider = BuiltinProjectSourceProvider::new(BuiltinProject::Universe);
    assert_eq!(provider.source_id(&id).unwrap().0.as_ref(), "phalcom://universe/");
    let interface = provider.load_interface(&id).unwrap();
    assert!(interface.exports.contains_key("Object"));
    assert_eq!(id.project, ProjectIdentity::Builtin(BuiltinProject::Universe));
}

#[test]
fn builtin_projects_are_disjoint() {
    let universe = ModuleId::builtin(BuiltinProject::Universe, ModulePath::root());
    let std = ModuleId::builtin(BuiltinProject::Std, ModulePath::root());
    assert_ne!(universe.project, std.project);
}
