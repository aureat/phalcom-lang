use phalcom_modules::identity::{BuiltinPackage, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::resolver::ModuleResolver;
use phalcom_modules::source::{FilesystemSourceProvider, ModuleKind};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn parsed_module_caching_and_interface_projection() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "expose .math\n").unwrap();
    fs::write(proj_dir.join("src/math.ph"), "class Math { add(a, b) { a + b } }\nexport Math\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(proj_dir.join("project.toml")).expect("universe load succeeds");
    assert_eq!(root_id, ResolvedProjectId::from_raw(1));

    let fs_provider = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &fs_provider);

    let mod_path = ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("math").unwrap()]);
    let mod_id = ModuleId::resolved(root_id, mod_path);

    // 1. load_parsed reads and parses source once
    let parsed1 = resolver.load_parsed(&mod_id).expect("successful parse");
    assert_eq!(parsed1.kind, ModuleKind::Module);
    assert_eq!(parsed1.id, mod_id);

    // 2. second load_parsed returns identical Arc pointer without reparsing
    let parsed2 = resolver.load_parsed(&mod_id).expect("successful second load");
    assert!(Arc::ptr_eq(&parsed1, &parsed2));

    // 3. load_interface projects from parsed cache without reparsing
    let iface = resolver.load_interface(&mod_id).expect("successful interface");
    assert!(iface.declarations.contains_key("Math"));
}

#[test]
fn builtin_parsed_module_caching() {
    let universe = ProjectUniverse::new();
    let fs = FilesystemSourceProvider::new();
    let mut resolver = ModuleResolver::new(&universe, &fs);

    let list_id = ModuleId::builtin(
        BuiltinPackage::Universe,
        ModulePath::from_components(vec![
            phalcom_modules::identity::ModuleComponent::from_identifier("collections").unwrap(),
            phalcom_modules::identity::ModuleComponent::from_identifier("list").unwrap(),
        ]),
    );

    let parsed1 = resolver.load_parsed(&list_id).expect("parsed builtin list");
    let parsed2 = resolver.load_parsed(&list_id).expect("second load parsed builtin list");
    assert!(Arc::ptr_eq(&parsed1, &parsed2));

    let iface = resolver.load_interface(&list_id).expect("builtin list interface");
    assert!(iface.declarations.contains_key("List"));
}
