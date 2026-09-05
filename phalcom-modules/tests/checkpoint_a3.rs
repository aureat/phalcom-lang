use phalcom_modules::identity::{ModuleId, SourceId, SourceLocation};
use phalcom_modules::interface::UnlinkedModuleInterface;
use phalcom_modules::linker::ModuleLinker;
use phalcom_modules::session::{WorkspaceModuleSession, WorkspaceSourceBatchMutation};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn location(path: &PathBuf) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.clone(),
    }
}

/// Verification Fixture:
/// Component 1 (package ab): ab/package.ph, ab/a.ph <-> ab/b.ph (mutual imports / cycle)
/// Component 2 (package cd): cd/package.ph, cd/c.ph -> cd/d.ph
/// Component 3 (package e):  e/package.ph, e/e.ph
fn setup_three_component_fixture() -> (TempDir, WorkspaceModuleSession, SourceLocation, SourceLocation, SourceLocation, SourceLocation, SourceLocation) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let ab_dir = root.join("ab");
    let cd_dir = root.join("cd");
    let e_dir = root.join("e");

    fs::create_dir_all(&ab_dir).unwrap();
    fs::create_dir_all(&cd_dir).unwrap();
    fs::create_dir_all(&e_dir).unwrap();

    let file_ab_pkg = ab_dir.join("package.ph");
    let file_a = ab_dir.join("a.ph");
    let file_b = ab_dir.join("b.ph");

    let file_cd_pkg = cd_dir.join("package.ph");
    let file_c = cd_dir.join("c.ph");
    let file_d = cd_dir.join("d.ph");

    let file_e_pkg = e_dir.join("package.ph");
    let file_e = e_dir.join("e.ph");

    fs::write(&file_ab_pkg, "expose .a\nexpose .b\n").unwrap();
    fs::write(&file_a, "class A { compute() -> Int { 1 } }\nexport A\n").unwrap();
    fs::write(&file_b, "from .a import A\nclass B { getA() -> A { A() } }\nexport B\n").unwrap();

    fs::write(&file_cd_pkg, "expose .c\nexpose .d\n").unwrap();
    fs::write(&file_c, "from .d import D\nclass C { getD() -> D { D() } }\nexport C\n").unwrap();
    fs::write(&file_d, "class D { val() -> Int { 10 } }\nexport D\n").unwrap();

    fs::write(&file_e_pkg, "expose .e\n").unwrap();
    fs::write(&file_e, "class E { tag() -> String { \"E\" } }\nexport E\n").unwrap();

    let loc_ab_pkg = location(&file_ab_pkg);
    let loc_a = location(&file_a);
    let loc_b = location(&file_b);

    let loc_cd_pkg = location(&file_cd_pkg);
    let loc_c = location(&file_c);
    let loc_d = location(&file_d);

    let loc_e_pkg = location(&file_e_pkg);
    let loc_e = location(&file_e);

    let mut session = WorkspaceModuleSession::new();
    let _up1 = session
        .apply_batch([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_ab_pkg,
                text: Arc::from("expose .a\nexpose .b\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_a.clone(),
                text: Arc::from("class A { compute() -> Int { 1 } }\nexport A\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_b.clone(),
                text: Arc::from("from .a import A\nclass B { getA() -> A { A() } }\nexport B\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_cd_pkg,
                text: Arc::from("expose .c\nexpose .d\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_c.clone(),
                text: Arc::from("from .d import D\nclass C { getD() -> D { D() } }\nexport C\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_d.clone(),
                text: Arc::from("class D { val() -> Int { 10 } }\nexport D\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_e_pkg,
                text: Arc::from("expose .e\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_e.clone(),
                text: Arc::from("class E { tag() -> String { \"E\" } }\nexport E\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .unwrap();

    (temp, session, loc_a, loc_b, loc_c, loc_d, loc_e)
}

#[test]
fn checkpoint_a3_evidence_1_body_only_edit_reuses_all_components() {
    let (_temp, mut session, _loc_a, loc_b, _loc_c, _loc_d, _loc_e) = setup_three_component_fixture();

    assert_eq!(session.retained_components().len(), 3, "fixture should form 3 components");

    // Edit B body only
    let up = session
        .set_overlay(
            loc_b.clone(),
            Arc::from("from .a import A\nclass B { getA() -> A { A() } compute() -> Int { 99 } }\nexport B\n"),
            phalcom_modules::SourceRevision(2),
        )
        .unwrap();

    assert_eq!(up.stats.linked_components_recomputed, 0, "body edit should recompute zero components");
    assert_eq!(up.stats.linked_components_reused, 3, "body edit should reuse all 3 components");
}

#[test]
fn checkpoint_a3_evidence_2_public_interface_edit_recomputes_only_affected_component() {
    let (_temp, mut session, _loc_a, loc_b, _loc_c, _loc_d, _loc_e) = setup_three_component_fixture();

    let mod_c = session.module_for_source(&_loc_c.source_id).cloned().unwrap();
    let mod_e = session.module_for_source(&_loc_e.source_id).cloned().unwrap();

    let comp_c_before = session.module_components().get(&mod_c).cloned().unwrap();
    let comp_e_before = session.module_components().get(&mod_e).cloned().unwrap();

    let retained_c_before = session.retained_components().get(&comp_c_before).cloned().unwrap();
    let retained_e_before = session.retained_components().get(&comp_e_before).cloned().unwrap();

    // Edit B public interface: export BExtra
    let up = session
        .set_overlay(
            loc_b.clone(),
            Arc::from("from .a import A\nclass B { getA() -> A { A() } }\nclass BExtra {}\nexport B\nexport BExtra\n"),
            phalcom_modules::SourceRevision(2),
        )
        .unwrap();

    assert_eq!(up.stats.linked_components_recomputed, 1, "editing B interface should recompute exactly 1 component");
    assert_eq!(up.stats.linked_components_reused, 2, "C/D and E components must be retained");

    let retained_c_after = session.retained_components().get(&comp_c_before).cloned().unwrap();
    let retained_e_after = session.retained_components().get(&comp_e_before).cloned().unwrap();

    assert!(Arc::ptr_eq(&retained_c_before, &retained_c_after), "Component C product must be structurally reused");
    assert!(Arc::ptr_eq(&retained_e_before, &retained_e_after), "Component E product must be structurally reused");
}

#[test]
fn checkpoint_a3_evidence_3_import_target_change_affects_only_target_component() {
    let (_temp, mut session, _loc_a, _loc_b, loc_c, _loc_d, _loc_e) = setup_three_component_fixture();

    // Change C's import target from cd.d to e.e
    let up = session
        .set_overlay(
            loc_c.clone(),
            Arc::from("from e.e import E\nclass C { getE() -> E { E() } }\nexport C\n"),
            phalcom_modules::SourceRevision(2),
        )
        .unwrap();

    // C now imports e.e, so cd and e merge into one component!
    // Component ab is unaffected and reused.
    assert_eq!(up.stats.linked_components_recomputed, 1, "only C's merged component should recompute");
    assert!(up.stats.linked_components_reused >= 1, "A/B component must be reused");
}

#[test]
fn checkpoint_a3_evidence_4_private_dependency_fingerprint_split() {
    let (_temp, mut session, _loc_a, loc_b, _loc_c, _loc_d, _loc_e) = setup_three_component_fixture();

    let mod_b = session.module_for_source(&loc_b.source_id).cloned().unwrap();
    let public_fp_v1 = session.linked_modules().get(&mod_b).unwrap().1;
    let private_fp_v1 = *session.linked_dependency_fingerprints().get(&mod_b).unwrap();

    // Change B private implementation to add a private top-level binding (keeping public export surface stable)
    let _up = session
        .set_overlay(
            loc_b.clone(),
            Arc::from("from .a import A\nlet private_val = 42\nclass B { getA() -> A { A() } }\nexport B\n"),
            phalcom_modules::SourceRevision(2),
        )
        .unwrap();

    let public_fp_v2 = session.linked_modules().get(&mod_b).unwrap().1;
    let private_fp_v2 = *session.linked_dependency_fingerprints().get(&mod_b).unwrap();

    assert_eq!(public_fp_v1, public_fp_v2, "public linked interface fingerprint must remain stable");
    assert_ne!(private_fp_v1, private_fp_v2, "private dependency fingerprint must change when internal linkage reads change");
}

#[test]
fn checkpoint_a3_evidence_5_strict_and_tolerant_linking_parity() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let file_pkg = root.join("package.ph");
    let file_a = root.join("a.ph");
    let file_b = root.join("b.ph");

    fs::write(&file_pkg, "expose .a\nexpose .b\n").unwrap();
    fs::write(&file_a, "class A { compute() -> Int { 1 } }\nexport A\n").unwrap();
    fs::write(&file_b, "from .a import A\nclass B { getA() -> A { A() } }\nexport B\n").unwrap();

    let loc_pkg = location(&file_pkg);
    let loc_a = location(&file_a);
    let loc_b = location(&file_b);

    let mut session = WorkspaceModuleSession::new();
    let _up = session
        .apply_batch([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_pkg,
                text: Arc::from("expose .a\nexpose .b\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_a.clone(),
                text: Arc::from("class A { compute() -> Int { 1 } }\nexport A\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_b.clone(),
                text: Arc::from("from .a import A\nclass B { getA() -> A { A() } }\nexport B\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .unwrap();

    let unlinked: std::collections::BTreeMap<ModuleId, UnlinkedModuleInterface> = session
        .interfaces()
        .iter()
        .map(|(id, (iface, _))| (id.clone(), (**iface).clone()))
        .collect();

    let linker = ModuleLinker::new(session.universe().clone().into(), unlinked);
    let entry = session.module_for_source(&loc_a.source_id).cloned().unwrap();

    let strict_program = linker.link(entry.clone(), session.resolved_imports()).unwrap();
    let tolerant_res = linker.link_component_tolerant(entry.clone(), session.resolved_imports());

    assert_eq!(strict_program.modules, tolerant_res.program.modules, "strict and tolerant linking must produce identical modules");
    assert_eq!(strict_program.initialization_order, tolerant_res.program.initialization_order, "strict and tolerant linking must produce identical initialization order");
    assert!(tolerant_res.diagnostics.is_empty(), "valid program tolerant linking has zero diagnostics");
    assert!(tolerant_res.blocked_modules.is_empty(), "valid program tolerant linking has zero blocked modules");
}

#[test]
fn checkpoint_a3_evidence_6_cycle_survivors_retain_initialization_order() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let cyc_dir = root.join("cyc");
    let ind_dir = root.join("ind");
    fs::create_dir_all(&cyc_dir).unwrap();
    fs::create_dir_all(&ind_dir).unwrap();

    // Cyclic Component 1 (cyc): X <-> Y
    // Independent Component 2 (ind): Z -> W
    let file_cyc_pkg = cyc_dir.join("package.ph");
    let file_x = cyc_dir.join("x.ph");
    let file_y = cyc_dir.join("y.ph");

    let file_ind_pkg = ind_dir.join("package.ph");
    let file_z = ind_dir.join("z.ph");
    let file_w = ind_dir.join("w.ph");

    fs::write(&file_cyc_pkg, "expose .x\nexpose .y\n").unwrap();
    fs::write(&file_x, "import .y\nlet x = 1\nexport x\n").unwrap();
    fs::write(&file_y, "import .x\nlet y = 2\nexport y\n").unwrap();

    fs::write(&file_ind_pkg, "expose .z\nexpose .w\n").unwrap();
    fs::write(&file_z, "from .w import w\nlet z = w\nexport z\n").unwrap();
    fs::write(&file_w, "let w = 42\nexport w\n").unwrap();

    let loc_cyc_pkg = location(&file_cyc_pkg);
    let loc_x = location(&file_x);
    let loc_y = location(&file_y);

    let loc_ind_pkg = location(&file_ind_pkg);
    let loc_z = location(&file_z);
    let loc_w = location(&file_w);

    let mut session = WorkspaceModuleSession::new();
    let up = session
        .apply_batch([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_cyc_pkg,
                text: Arc::from("expose .x\nexpose .y\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_x,
                text: Arc::from("import .y\nlet x = 1\nexport x\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_y,
                text: Arc::from("import .x\nlet y = 2\nexport y\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_ind_pkg,
                text: Arc::from("expose .z\nexpose .w\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_z,
                text: Arc::from("from .w import w\nlet z = w\nexport z\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: loc_w,
                text: Arc::from("let w = 42\nexport w\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .unwrap();

    // Cycling modules X and Y are blocked in tolerant mode
    assert!(up.blocked_modules.len() >= 2, "cycle modules X and Y should be blocked");

    // Surviving modules Z and W retain valid initialization order (W before Z)
    let init_order = &up.linked.initialization_order;
    assert!(!init_order.is_empty(), "surviving modules must produce initialization order");
    let mod_z = session.module_for_source(&SourceId(file_z.to_string_lossy().into())).cloned().unwrap();
    let mod_w = session.module_for_source(&SourceId(file_w.to_string_lossy().into())).cloned().unwrap();

    let pos_z = init_order.iter().position(|m| m == &mod_z).unwrap();
    let pos_w = init_order.iter().position(|m| m == &mod_w).unwrap();
    assert!(pos_w < pos_z, "dependency W must initialize before Z");
}

