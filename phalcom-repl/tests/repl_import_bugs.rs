//! Regression tests for REPL import system bugs (Bugs 1, 2, 3).

use phalcom_repl::repl::{CellOutcome, ReplSession};

#[test]
fn selective_import_missing_name_raises() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    let outcome = session.eval("from universe.reflection import NonExistentName");
    // Must fail, not silently succeed with None
    assert!(matches!(outcome, CellOutcome::Failed));
}

#[test]
fn selective_import_exposed_child_not_exported_raises() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    // universe.reflection exposes .selector as a child module, not as a symbol export
    let outcome = session.eval("from universe.reflection import selector");
    // Should fail: selector is a child module path, not an exported binding
    assert!(matches!(outcome, CellOutcome::Failed));
}

#[test]
fn selective_import_selector_class_is_non_none() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    let outcome1 = session.eval("from universe.errors.unsupported import unsupported");
    assert!(!matches!(outcome1, CellOutcome::Failed));

    let outcome2 = session.eval("unsupported");
    match outcome2 {
        CellOutcome::Value(v) => assert!(!v.is_none(), "unsupported must be a value, not None"),
        other => panic!("expected a value, got {:?}", other),
    }
}

#[test]
fn direct_path_import_package_info_resolves() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    // Must not produce ModuleNotFound
    let outcome = session.eval("import universe.reflection.package_info");
    assert!(!matches!(outcome, CellOutcome::Failed));
}

#[test]
fn module_import_selector_property_access_is_non_none() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    let outcome1 = session.eval("import universe.errors.unsupported");
    assert!(!matches!(outcome1, CellOutcome::Failed));

    let outcome2 = session.eval("unsupported.unsupported");
    match outcome2 {
        CellOutcome::Value(v) => assert!(!v.is_none(), "unsupported.unsupported must be a value, not None"),
        other => panic!("expected a value, got {:?}", other),
    }
}

#[test]
fn universe_root_exports_package_info() {
    let mut session = ReplSession::start(std::env::current_dir().unwrap());
    let outcome1 = session.eval("from universe import PackageInfo");
    assert!(!matches!(outcome1, CellOutcome::Failed));

    let outcome2 = session.eval("PackageInfo");
    match outcome2 {
        CellOutcome::Value(v) => assert!(!v.is_none(), "PackageInfo must be a class, not None"),
        other => panic!("expected a value, got {:?}", other),
    }
}
