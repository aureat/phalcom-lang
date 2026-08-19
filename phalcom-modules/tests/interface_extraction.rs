use phalcom_ast::parser::parse;
use phalcom_modules::{
    BuiltinProject, ImportSurface, InterfaceBuilder, InterfaceError, ModuleComponent, ModuleId, ModuleKind, ModulePath, UnlinkedExportTarget,
};

fn make_test_id(name: &str) -> ModuleId {
    ModuleId::builtin(
        BuiltinProject::Universe,
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

/// IFACE-01 — Declaration becomes exportable
#[test]
fn iface_01_declaration_becomes_exportable() {
    let id = make_test_id("test_mod");
    let src = "class Foo {}\nexport Foo\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let iface = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program).expect("build interface");
    assert!(iface.declarations.contains_key("Foo"));
    assert_eq!(
        iface.exports.get("Foo").map(|e| &e.target),
        Some(&UnlinkedExportTarget::Local("Foo".to_string()))
    );
}

/// IFACE-02 — Module-import binding collision with declaration is rejected
#[test]
fn iface_02_module_import_declaration_collision_rejected() {
    let id = make_test_id("test_mod");
    let src = "import other\nlet other = 1\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let result = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program);
    assert!(
        matches!(result, Err(InterfaceError::DuplicateBinding { ref name, .. }) if name == "other"),
        "expected DuplicateBinding for 'other', got {:?}",
        result
    );
}

/// IFACE-03 — Selective import items appear in namespace but not in exports unless re-exported
#[test]
fn iface_03_selective_import_not_in_exports_or_declarations() {
    let id = make_test_id("test_mod");
    let src = "from other import Foo\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let iface = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program).expect("build interface");
    assert_eq!(iface.imports.len(), 1);
    assert!(matches!(iface.imports[0], ImportSurface::Selective(_)));
    assert!(iface.exports.is_empty());
    assert!(!iface.declarations.contains_key("Foo"));
}

/// IFACE-04 — expose on a non-package module is rejected
#[test]
fn iface_04_expose_on_non_package_rejected() {
    let id = make_test_id("test_mod");
    let src = "expose .child\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let result = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program);
    assert!(
        matches!(result, Err(InterfaceError::ExposeOutsidePackage(_))),
        "expected ExposeOutsidePackage, got {:?}",
        result
    );
}

/// IFACE-05 — expose on a Package accumulates exposed_children
#[test]
fn iface_05_expose_on_package_accumulates_children() {
    let id = make_test_id("test_pkg");
    let src = "expose .alpha\nexpose .beta\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let iface = InterfaceBuilder::build(id, ModuleKind::Package, &parse_result.program).expect("build interface");
    let alpha = ModuleComponent::from_identifier("alpha").unwrap();
    let beta = ModuleComponent::from_identifier("beta").unwrap();
    assert!(iface.exposed_children.contains(&alpha));
    assert!(iface.exposed_children.contains(&beta));
    assert_eq!(iface.exposed_children.len(), 2);
}

/// IFACE-06 — Re-export produces export surface with ReExport target
#[test]
fn iface_06_reexport_produces_reexport_target() {
    let id = make_test_id("test_mod");
    let src = "export Foo from other\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let iface = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program).expect("build interface");
    let export_foo = iface.exports.get("Foo").expect("Foo export exists");
    match &export_foo.target {
        UnlinkedExportTarget::ReExport { remote, .. } => {
            assert_eq!(remote, "Foo");
        }
        other => panic!("expected ReExport target, got {:?}", other),
    }
    assert!(iface.imports.iter().any(|imp| matches!(imp, ImportSurface::ReExport(_))));
}

/// IFACE-07 — Duplicate export name is rejected
#[test]
fn iface_07_duplicate_export_rejected() {
    let id = make_test_id("test_mod");
    let src = "class Foo {}\nclass Bar {}\nexport Foo\nexport Foo\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let result = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program);
    assert!(
        matches!(result, Err(InterfaceError::DuplicateExport { ref name, .. }) if name == "Foo"),
        "expected DuplicateExport for 'Foo', got {:?}",
        result
    );
}

/// IFACE-08 — Export of undeclared name is rejected
#[test]
fn iface_08_export_undeclared_rejected() {
    let id = make_test_id("test_mod");
    let src = "export GhostName\n";
    let parse_result = parse(src, 0);
    assert!(parse_result.errors.is_empty());

    let result = InterfaceBuilder::build(id, ModuleKind::Module, &parse_result.program);
    assert!(
        matches!(result, Err(InterfaceError::UnknownExport { ref name, .. }) if name == "GhostName"),
        "expected UnknownExport for 'GhostName', got {:?}",
        result
    );
}
