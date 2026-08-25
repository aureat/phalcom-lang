use phalcom_modules::{ModuleId, NullDependencyProvider, SourceId, SourceLocation, SourceRevision, WorkspaceModuleSession};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

fn location(path: &std::path::Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn standalone_overlay_has_stable_identity_and_disk_fallback() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("demo.ph");
    fs::write(&file, "class Disk {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();

    let first = session.set_overlay(source.clone(), Arc::from("class Overlay {}\n"), SourceRevision(1)).unwrap();
    let module = first.sources.keys().next().cloned().unwrap();
    assert!(session.source(&module).unwrap().open_overlay);
    assert_eq!(&*session.source(&module).unwrap().text, "class Overlay {}\n");

    let second = session
        .set_overlay(source.clone(), Arc::from("class OverlayAgain {}\n"), SourceRevision(2))
        .unwrap();
    assert_eq!(second.sources.keys().find(|candidate| candidate.project == module.project), Some(&module));
    assert_eq!(session.source(&module).unwrap().revision, SourceRevision(2));

    session.remove_overlay(source.source_id.clone()).unwrap();
    let state = session.source(&module).unwrap();
    assert!(!state.open_overlay);
    assert_eq!(&*state.text, "class Disk {}\n");

    session.remove_source(source.source_id).unwrap();
    assert!(session.source(&module).is_none());
    assert!(session.module_for_source(&SourceId(file.to_string_lossy().into())).is_none());
}

#[test]
fn project_edits_reuse_resolved_project_and_module_identity() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"demo\"\nnamespace = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/package.ph"), "").unwrap();
    let file = root.join("src/main.ph");
    fs::write(&file, "class Main {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();

    session.set_overlay(source.clone(), Arc::from("class Main {}\n"), SourceRevision(1)).unwrap();
    let module = session.module_for_source(&source.source_id).unwrap().clone();
    let project = module.project.as_resolved().unwrap();
    assert_eq!(session.universe().projects().len(), 1);

    session.set_overlay(source, Arc::from("class Main { run() {} }\n"), SourceRevision(2)).unwrap();
    assert_eq!(session.module_for_source(&SourceId(file.to_string_lossy().into())), Some(&module));
    assert_eq!(
        session
            .module_for_source(&SourceId(file.to_string_lossy().into()))
            .unwrap()
            .project
            .as_resolved(),
        Some(project)
    );
    assert_eq!(session.universe().projects().len(), 1);
}

#[test]
fn removed_source_reports_identity_change() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("remove.ph");
    fs::write(&file, "class Remove {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();
    let update = session.set_overlay(source.clone(), Arc::from("class Remove {}\n"), SourceRevision(1)).unwrap();
    let module: ModuleId = update.sources.keys().next().cloned().unwrap();
    let removed = session.remove_source(source.source_id).unwrap();
    assert!(removed.removed_modules.contains(&module));
    assert!(removed.identity_changes.contains(&module));
}

#[test]
fn standalone_relative_imports_resolve_from_registered_sources() {
    let temp = TempDir::new().unwrap();
    let mover = temp.path().join("mover.ph");
    let main = temp.path().join("main.ph");
    fs::write(&mover, "class Mover {}\nexport Mover\n").unwrap();
    fs::write(&main, "import .mover as MoverModule\n").unwrap();
    let mut session = WorkspaceModuleSession::new();
    let update = session
        .set_overlays([
            (location(&mover), Arc::from("class Mover {}\nexport Mover\n"), SourceRevision(1)),
            (location(&main), Arc::from("import .mover as MoverModule\n"), SourceRevision(1)),
        ])
        .unwrap();
    assert_eq!(update.sources.len(), 2);
    assert_eq!(update.linked.modules.len(), 2);
}

#[test]
fn workspace_root_reset_rebuilds_project_graph_and_preserves_open_overlay() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"demo\"\nnamespace = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/package.ph"), "").unwrap();
    let file = root.join("src/main.ph");
    fs::write(&file, "class Main {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(source.clone(), Arc::from("class Main { open() {} }\n"), SourceRevision(1))
        .unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"demo\"\nnamespace = \"demo\"\nversion = \"0.2.0\"\n",
    )
    .unwrap();

    let update = session.set_workspace_roots(&[root], &NullDependencyProvider).unwrap();
    let module = session.module_for_source(&source.source_id).unwrap().clone();
    assert!(!update.removed_modules.is_empty());
    assert_eq!(&*session.source(&module).unwrap().text, "class Main { open() {} }\n");
    assert!(session.source(&module).unwrap().open_overlay);
}
