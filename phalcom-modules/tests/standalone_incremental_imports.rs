use std::sync::Arc;

use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceModuleSession};
use tempfile::tempdir;

fn location(path: &std::path::Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn standalone_package_importer_survives_until_relative_sibling_is_discovered() {
    let root = tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let main = location(&root.path().join("main.ph"));
    let shapes = location(&root.path().join("shapes.ph"));

    let mut session = WorkspaceModuleSession::new();

    let first = session
        .set_overlay(main.clone(), Arc::from("import .shapes as shapes\nlet value = 1\n"), SourceRevision(1))
        .expect("temporarily unresolved imports must not roll back the importer");

    let main_module = session
        .module_for_source(&main.source_id)
        .cloned()
        .expect("the open importer must remain registered");
    assert!(first.linked.modules.contains_key(&main_module));

    let second = session
        .set_overlay(shapes.clone(), Arc::from("class Circle {}\n"), SourceRevision(1))
        .expect("discovering the sibling should relink the retained importer");

    let shapes_module = session
        .module_for_source(&shapes.source_id)
        .cloned()
        .expect("the discovered sibling must be registered");
    let linked_main = second.linked.modules.get(&main_module).expect("the importer must remain in the linked program");
    let import_id = linked_main
        .bindings
        .imports
        .get("shapes")
        .expect("the relative module alias should become linked after discovery");
    let read = linked_main
        .linked_reads
        .get(import_id.0 as usize)
        .expect("import binding must reference a linked read");
    assert_eq!(read, &phalcom_modules::linker::LinkedReadSpec::Module(shapes_module));
}

#[test]
fn standalone_module_cannot_relative_import_sibling() {
    let root = tempdir().unwrap();
    // No package.ph! Both are standalone modules.
    let main = location(&root.path().join("main.ph"));
    let shapes = location(&root.path().join("shapes.ph"));

    let mut session = WorkspaceModuleSession::new();

    let _first = session
        .set_overlay(main.clone(), Arc::from("import .shapes as shapes\nlet value = 1\n"), SourceRevision(1))
        .expect("unresolved import in standalone module does not crash session");

    let second = session
        .set_overlay(shapes.clone(), Arc::from("class Circle {}\n"), SourceRevision(1))
        .expect("overlay sets cleanly");

    let main_module = session.module_for_source(&main.source_id).cloned().unwrap();
    assert!(main_module.project.as_synthetic().is_some(), "standalone module must be synthetic");

    let linked_main = second.linked.modules.get(&main_module).expect("importer is present");
    // Without package.ph, the relative import MUST NOT be linked
    assert!(
        linked_main.bindings.imports.get("shapes").is_none(),
        "standalone module without package.ph must not link relative sibling"
    );
}
