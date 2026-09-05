use phalcom_modules::{
    ModuleId, NullDependencyProvider, ProjectUniverse, SourceId, SourceLocation, SourceRevision, WorkspaceModuleSession, WorkspaceSourceBatchMutation,
};
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
fn standalone_move_is_remove_then_add_identity_transition() {
    let temp = TempDir::new().unwrap();
    let old_file = temp.path().join("old.ph");
    let new_file = temp.path().join("new.ph");
    fs::write(&old_file, "class Moved {}\n").unwrap();
    fs::write(&new_file, "class Moved {}\n").unwrap();
    let old_source = location(&old_file);
    let new_source = location(&new_file);
    let mut session = WorkspaceModuleSession::new();

    let first = session
        .set_overlay(old_source.clone(), Arc::from("class Moved {}\n"), SourceRevision(1))
        .unwrap();
    let old_module = first.sources.keys().next().cloned().unwrap();

    let removed = session.remove_source(old_source.source_id.clone()).unwrap();
    assert!(removed.removed_modules.contains(&old_module));
    assert!(removed.identity_changes.contains(&old_module));

    let added = session
        .set_overlay(new_source.clone(), Arc::from("class Moved {}\n"), SourceRevision(1))
        .unwrap();
    let new_module = session.module_for_source(&new_source.source_id).unwrap().clone();
    assert!(!added.sources.contains_key(&old_module));
    assert_ne!(new_module, old_module, "logical path move must not preserve old module identity");
    assert!(session.module_for_source(&old_source.source_id).is_none());
}

#[test]
fn standalone_package_supports_relative_children() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "").unwrap();
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
    let main_mod = session.module_for_source(&SourceId(main.to_string_lossy().into())).unwrap();
    assert!(main_mod.project.as_resolved().is_some());
    assert!(!session.resolved_imports().is_empty());
}

#[test]
fn standalone_sibling_files_do_not_form_package() {
    let temp = TempDir::new().unwrap();
    let mover = temp.path().join("mover.ph");
    let main = temp.path().join("main.ph");
    fs::write(&mover, "class Mover {}\nexport Mover\n").unwrap();
    fs::write(&main, "import .mover as MoverModule\n").unwrap();
    let mut session = WorkspaceModuleSession::new();
    let _update = session
        .set_overlays([
            (location(&mover), Arc::from("class Mover {}\nexport Mover\n"), SourceRevision(1)),
            (location(&main), Arc::from("import .mover as MoverModule\n"), SourceRevision(1)),
        ])
        .unwrap();
    let mover_mod = session.module_for_source(&SourceId(mover.to_string_lossy().into())).unwrap();
    let main_mod = session.module_for_source(&SourceId(main.to_string_lossy().into())).unwrap();
    assert!(mover_mod.project.as_synthetic().is_some());
    assert!(main_mod.project.as_synthetic().is_some());
    assert!(session.resolved_imports().is_empty(), "standalone modules without package.ph must not resolve sibling imports");
}

#[test]
fn package_marker_addition_and_removal_reclassifies_open_sources() {
    let temp = TempDir::new().unwrap();
    let main = temp.path().join("main.ph");
    let helper = temp.path().join("helper.ph");
    fs::write(&main, "import .helper as Helper\n").unwrap();
    fs::write(&helper, "class Helper {}\nexport Helper\n").unwrap();
    let main_source = location(&main);
    let helper_source = location(&helper);
    let mut session = WorkspaceModuleSession::new();

    session
        .set_overlays([
            (main_source.clone(), Arc::from("import .helper as Helper\n"), SourceRevision(1)),
            (helper_source.clone(), Arc::from("class Helper {}\nexport Helper\n"), SourceRevision(1)),
        ])
        .unwrap();
    let old_main = session.module_for_source(&main_source.source_id).unwrap().clone();
    let old_helper = session.module_for_source(&helper_source.source_id).unwrap().clone();
    assert!(old_main.project.as_synthetic().is_some());
    assert!(old_helper.project.as_synthetic().is_some());

    let package = temp.path().join("package.ph");
    fs::write(&package, "").unwrap();
    let added = session
        .apply_batch([WorkspaceSourceBatchMutation::SetDiskSnapshot {
            source: location(&package),
            text: Arc::from(""),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .unwrap();
    let new_main = session.module_for_source(&main_source.source_id).unwrap().clone();
    let new_helper = session.module_for_source(&helper_source.source_id).unwrap().clone();
    assert!(new_main.project.as_resolved().is_some());
    assert_eq!(new_main.project, new_helper.project);
    assert_ne!(new_main, old_main);
    assert_ne!(new_helper, old_helper);
    assert!(added.removed_modules.contains(&old_main));
    assert!(added.removed_modules.contains(&old_helper));
    assert!(added.stats.topology_invalidations > 0);

    fs::remove_file(&package).unwrap();
    let removed = session.remove_source(SourceId(package.to_string_lossy().into())).unwrap();
    let final_main = session.module_for_source(&main_source.source_id).unwrap();
    assert!(final_main.project.as_synthetic().is_some());
    assert!(removed.identity_changes.iter().any(|module| module == &new_main));
}

#[test]
fn direct_file_inside_standalone_package_uses_package_identity() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "").unwrap();
    let util = temp.path().join("util.ph");
    fs::write(&util, "class Util {}\n").unwrap();
    let mut session = WorkspaceModuleSession::new();
    let loc = location(&util);
    session.set_overlay(loc.clone(), Arc::from("class Util {}\n"), SourceRevision(1)).unwrap();
    let module = session.module_for_source(&loc.source_id).unwrap().clone();
    let project_id = module.project.as_resolved().expect("must be resolved project");
    let proj = session.universe().get_project(project_id).expect("project exists");
    assert!(proj.is_standalone_package());
}

#[test]
fn intermediate_directory_without_package_ph_is_not_package() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "").unwrap();
    let sub = temp.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    let leaf = sub.join("leaf.ph");
    fs::write(&leaf, "class Leaf {}\n").unwrap();
    let mut universe = ProjectUniverse::new();
    let ownership = phalcom_modules::classify_entry_ownership(&leaf, &mut universe).unwrap();
    assert!(matches!(ownership, phalcom_modules::EntryOwnership::StandaloneModule { .. }));
}

#[test]
fn nested_standalone_package_ownership_is_preserved() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "").unwrap();
    let nested = temp.path().join("tools");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("package.ph"), "").unwrap();
    let run = nested.join("run.ph");
    fs::write(&run, "let value = 1\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let ownership = phalcom_modules::classify_entry_ownership(&run, &mut universe).unwrap();
    let expected_root = temp.path().canonicalize().unwrap();
    assert!(matches!(
        ownership,
        phalcom_modules::EntryOwnership::StandalonePackageOwned { package_root } if package_root == expected_root
    ));
}

#[test]
fn persistent_project_precedes_ancestor_standalone_package() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("package.ph"), "").unwrap();
    let project = temp.path().join("project");
    let source_root = project.join("src");
    fs::create_dir_all(&source_root).unwrap();
    fs::write(
        project.join("project.toml"),
        "[project]\nname = \"nested-project\"\nnamespace = \"nested_project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(source_root.join("package.ph"), "").unwrap();
    let main = source_root.join("main.ph");
    fs::write(&main, "let value = 1\n").unwrap();

    let mut universe = ProjectUniverse::new();
    let ownership = phalcom_modules::classify_entry_ownership(&main, &mut universe).unwrap();
    assert!(matches!(ownership, phalcom_modules::EntryOwnership::ProjectOwned { .. }));
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

#[test]
fn disk_snapshot_is_not_an_open_overlay() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("disk.ph");
    fs::write(&file, "class DiskOnDisk {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();

    let update = session
        .apply_batch([WorkspaceSourceBatchMutation::SetDiskSnapshot {
            source: source.clone(),
            text: Arc::from("class DiskOnDisk {}\n"),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .unwrap();
    let module = update.sources.keys().next().unwrap();

    assert!(!session.source(module).unwrap().open_overlay);
    assert_eq!(session.source(module).unwrap().text.as_ref(), "class DiskOnDisk {}\n");
}

#[test]
fn scanner_style_disk_snapshot_replaces_overlay_without_opening_it() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("refresh.ph");
    fs::write(&file, "class Disk {}").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();
    let first = session.set_overlay(source.clone(), Arc::from("class Overlay {}\n"), SourceRevision(1)).unwrap();
    let module = first.sources.keys().next().unwrap().clone();

    session
        .apply_batch([WorkspaceSourceBatchMutation::SetDiskSnapshot {
            source,
            text: Arc::from("class DiskRefresh {}\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .unwrap();

    let state = session.source(&module).unwrap();
    assert!(!state.open_overlay);
    assert_eq!(state.text.as_ref(), "class DiskRefresh {}\n");
}

#[test]
fn mixed_overlay_and_disk_batch_rebuilds_once() {
    let temp = TempDir::new().unwrap();
    let overlay_file = temp.path().join("overlay.ph");
    let disk_file = temp.path().join("disk.ph");
    fs::write(&overlay_file, "class Overlay {}").unwrap();
    fs::write(&disk_file, "class Disk {}").unwrap();
    let mut session = WorkspaceModuleSession::new();
    let generation = session.generation();

    session
        .apply_batch([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&overlay_file),
                text: Arc::from("class OverlayLive {}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetDiskSnapshot {
                source: location(&disk_file),
                text: Arc::from("class DiskSnapshot {}\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .unwrap();

    assert_eq!(session.generation(), generation + 1);
    assert_eq!(session.sources().len(), 2);
    assert!(session.sources().values().any(|state| state.open_overlay));
    assert!(session.sources().values().any(|state| !state.open_overlay));
}

#[test]
fn body_only_edit_preserves_interface_fingerprint_and_stops_propagation() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"demo\"\nnamespace = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/package.ph"), "expose a, b\n").unwrap();
    let file_a = root.join("src/a.ph");
    let file_b = root.join("src/b.ph");
    fs::write(&file_a, "class A { compute() -> Int { 1 } }\nexport A\n").unwrap();
    fs::write(&file_b, "from demo.a import A\nclass B { getA() -> A { A() } }\nexport B\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    let src_a = location(&file_a);
    let src_b = location(&file_b);

    let up1 = session
        .apply_batch([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: src_a.clone(),
                text: Arc::from("class A { compute() -> Int { 1 } }\nexport A\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: src_b.clone(),
                text: Arc::from("from demo.a import A\nclass B { getA() -> A { A() } }\nexport B\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .unwrap();
    assert!(up1.stats.interfaces_built >= 2);
    assert!(up1.stats.imports_resolved >= 1);

    let mod_a = session.module_for_source(&src_a.source_id).cloned().unwrap();
    let _mod_b = session.module_for_source(&src_b.source_id).cloned().unwrap();

    let (_iface_a_v1, fp_a_v1) = session.interfaces().get(&mod_a).cloned().unwrap();
    let linked_v1 = session.linked().cloned().unwrap();
    let import_prods_v1 = session.import_products().clone();

    // Body-only edit in A: change method body `1` -> `42`
    let up2 = session
        .apply_batch([WorkspaceSourceBatchMutation::SetOverlay {
            source: src_a.clone(),
            text: Arc::from("class A { compute() -> Int { 42 } }\nexport A\n"),
            revision: SourceRevision(2),
            recovered_program: None,
        }])
        .unwrap();

    let (_iface_a_v2, fp_a_v2) = session.interfaces().get(&mod_a).cloned().unwrap();
    assert_eq!(fp_a_v1, fp_a_v2, "interface fingerprint must be stable across body edit");

    // Propagation stopped: linked program was reused directly without relinking
    let linked_v2 = session.linked().cloned().unwrap();
    assert!(Arc::ptr_eq(&linked_v1, &linked_v2), "body-only edit must reuse linked program directly");
    assert_eq!(session.import_products().len(), import_prods_v1.len());
    assert!(up2.changed_modules.contains(&mod_a));
    assert_eq!(up2.stats.interfaces_built, 1);
    assert!(up2.stats.interfaces_reused >= 1);
    assert_eq!(up2.stats.imports_resolved, 0);
    assert!(up2.stats.linked_modules_reused >= 2);
    assert_eq!(up2.stats.linked_components, 0);
}

#[test]
fn failed_transaction_does_not_mutate_committed_state() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("demo.ph");
    fs::write(&file, "class Initial {}\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();

    let _up1 = session
        .set_overlay(source.clone(), Arc::from("class Initial {}\n"), SourceRevision(1))
        .unwrap();
    let mod_id = session.module_for_source(&source.source_id).cloned().unwrap();
    let initial_gen = session.generation();
    let initial_text = session.source(&mod_id).unwrap().text.clone();

    // Attempt mutation with syntax error
    let err = session.set_overlay(
        source.clone(),
        Arc::from("class Broken { !@#$ }\n"),
        SourceRevision(2),
    );
    assert!(err.is_err(), "parse failure must return Err");

    // Committed state must be 100% untouched
    assert_eq!(session.generation(), initial_gen);
    assert_eq!(session.source(&mod_id).unwrap().text, initial_text);
    assert_eq!(session.source(&mod_id).unwrap().revision, SourceRevision(1));
}

#[test]
fn cache_negative_result_resolves_when_source_is_added() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"demo\"\nnamespace = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/package.ph"), "expose a, b\n").unwrap();
    let file_b = root.join("src/b.ph");
    fs::write(&file_b, "from demo.a import A\nclass B {}\nexport B\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    let src_b = location(&file_b);

    // Initial update: a.ph does not exist yet; import resolution produces negative result
    let _up1 = session
        .set_overlay(
            src_b.clone(),
            Arc::from("from demo.a import A\nclass B {}\nexport B\n"),
            SourceRevision(1),
        )
        .unwrap();

    let mod_b = session.module_for_source(&src_b.source_id).cloned().unwrap();
    assert!(session.resolved_imports().get(&(mod_b.clone(), "demo.a".into())).is_none());

    // Now add a.ph to the workspace
    let file_a = root.join("src/a.ph");
    fs::write(&file_a, "class A {}\nexport A\n").unwrap();
    let src_a = location(&file_a);

    let _up2 = session
        .set_overlay(
            src_a.clone(),
            Arc::from("class A {}\nexport A\n"),
            SourceRevision(1),
        )
        .unwrap();

    let mod_a = session.module_for_source(&src_a.source_id).cloned().unwrap();
    // Negative resolution was invalidated and successfully resolved on the next update
    assert_eq!(
        session.resolved_imports().get(&(mod_b, "demo.a".into())),
        Some(&mod_a),
        "previously missing import must resolve after source addition"
    );
}

#[test]
fn remove_source_hard_purges_module_products() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("demo.ph");
    fs::write(&file, "class Demo {}\nexport Demo\n").unwrap();
    let source = location(&file);
    let mut session = WorkspaceModuleSession::new();

    let up = session
        .set_overlay(source.clone(), Arc::from("class Demo {}\nexport Demo\n"), SourceRevision(1))
        .unwrap();
    let mod_id = up.sources.keys().next().cloned().unwrap();

    assert!(session.interfaces().contains_key(&mod_id));
    assert!(session.linked_modules().contains_key(&mod_id));

    let removed = session.remove_source(source.source_id).unwrap();

    assert!(!session.interfaces().contains_key(&mod_id));
    assert!(!session.linked_modules().contains_key(&mod_id));
    assert!(session.source(&mod_id).is_none());
    assert!(removed.stats.purged_products >= 2);
}
