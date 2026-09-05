use phalcom_modules::identity::ImportSiteId;
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceModuleSession};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

fn location(path: &Path) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    }
}

#[test]
fn checkpoint_a2_evidence_1_twenty_imports_edit_one_resolves_one_reuses_nineteen() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("pkg");
    fs::create_dir_all(&root).unwrap();

    let mut pkg_content = String::new();
    for i in 1..=21 {
        let name = format!("m{:02}", i);
        let file = root.join(format!("{}.ph", name));
        fs::write(&file, format!("class C{} {{}}\nexport C{}\n", i, i)).unwrap();
        pkg_content.push_str(&format!("export C{}\n", i));
    }
    fs::write(root.join("package.ph"), &pkg_content).unwrap();

    let mut main_content = String::new();
    for i in 1..=20 {
        main_content.push_str(&format!("import .m{:02} as M{:02}\n", i, i));
    }
    main_content.push_str("class Main {}\nexport Main\n");

    let main_file = root.join("main.ph");
    fs::write(&main_file, &main_content).unwrap();

    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(location(&root.join("package.ph")), Arc::from(pkg_content), SourceRevision(1))
        .unwrap();

    // Add 20 modules
    for i in 1..=20 {
        let file = root.join(format!("m{:02}.ph", i));
        session
            .set_overlay(location(&file), Arc::from(format!("class C{} {{}}\nexport C{}\n", i, i)), SourceRevision(1))
            .unwrap();
    }

    // Add main with 20 imports
    let initial_up = session
        .set_overlay(location(&main_file), Arc::from(main_content.clone()), SourceRevision(1))
        .unwrap();

    assert_eq!(initial_up.stats.imports_resolved, 20);
    assert_eq!(initial_up.stats.import_sites_reused, 0);

    // Edit 1 import path in main.ph: change import 20 from .m20 to .m21
    let mut edited_main = String::new();
    for i in 1..=19 {
        edited_main.push_str(&format!("import .m{:02} as M{:02}\n", i, i));
    }
    edited_main.push_str("import .m21 as M21\n");
    edited_main.push_str("class Main {}\nexport Main\n");

    // Add m21.ph first
    session
        .set_overlay(location(&root.join("m21.ph")), Arc::from("class C21 {}\nexport C21\n"), SourceRevision(1))
        .unwrap();

    // Now edit main.ph
    let edit_up = session
        .set_overlay(location(&main_file), Arc::from(edited_main), SourceRevision(2))
        .unwrap();

    // EXACT ACCEPTANCE CRITERIA:
    // "A module with 20 imports, editing 1 import path resolves exactly 1 import and reuses 19"
    assert_eq!(edit_up.stats.imports_resolved, 1, "exactly 1 import must be re-resolved");
    assert_eq!(edit_up.stats.import_resolutions_reused, 19, "19 imports must be reused");
    assert_eq!(edit_up.stats.import_sites_considered, 20);
    assert_eq!(edit_up.stats.import_sites_validated, 20);
    assert_eq!(edit_up.stats.import_sites_reused, 19);
}

#[test]
fn checkpoint_a2_evidence_2_body_only_edit_zero_import_resolutions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("pkg");
    fs::create_dir_all(&root).unwrap();

    fs::write(root.join("package.ph"), "export A\nexport B\n").unwrap();
    let file_a = root.join("a.ph");
    let file_b = root.join("b.ph");

    fs::write(&file_b, "class B { foo() -> Int { 1 } }\nexport B\n").unwrap();
    fs::write(&file_a, "import .b as B\nclass A { bar() -> Int { 2 } }\nexport A\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(location(&root.join("package.ph")), Arc::from("export A\nexport B\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&file_b), Arc::from("class B { foo() -> Int { 1 } }\nexport B\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&file_a), Arc::from("import .b as B\nclass A { bar() -> Int { 2 } }\nexport A\n"), SourceRevision(1))
        .unwrap();

    // Body-only edit in b.ph
    let up_b = session
        .set_overlay(location(&file_b), Arc::from("class B { foo() -> Int { 999 } }\nexport B\n"), SourceRevision(2))
        .unwrap();

    // EXACT ACCEPTANCE CRITERIA:
    // "Body-only edit across multi-module session produces imports_resolved == 0"
    assert_eq!(up_b.stats.imports_resolved, 0, "body-only edit must resolve 0 imports");
    assert!(up_b.stats.import_sites_reused >= 1, "import sites must be reused");

    // Body-only edit in a.ph
    let up_a = session
        .set_overlay(location(&file_a), Arc::from("import .b as B\nclass A { bar() -> Int { 777 } }\nexport A\n"), SourceRevision(2))
        .unwrap();

    assert_eq!(up_a.stats.imports_resolved, 0, "body-only edit must resolve 0 imports");
    assert!(up_a.stats.import_sites_reused >= 1, "import sites must be reused");
}

#[test]
fn checkpoint_a2_evidence_3_export_only_edit_zero_import_resolutions() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("pkg");
    fs::create_dir_all(&root).unwrap();

    fs::write(root.join("package.ph"), "export A\nexport B\n").unwrap();
    let file_a = root.join("a.ph");
    let file_b = root.join("b.ph");

    fs::write(&file_b, "class B1 {}\nexport B1\n").unwrap();
    fs::write(&file_a, "import .b as B\nclass A {}\nexport A\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(location(&root.join("package.ph")), Arc::from("export A\nexport B\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&file_b), Arc::from("class B1 {}\nexport B1\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&file_a), Arc::from("import .b as B\nclass A {}\nexport A\n"), SourceRevision(1))
        .unwrap();

    // Export-only edit in b.ph: adding a new export class B2
    let up = session
        .set_overlay(
            location(&file_b),
            Arc::from("class B1 {}\nclass B2 {}\nexport B1\nexport B2\n"),
            SourceRevision(2),
        )
        .unwrap();

    // EXACT ACCEPTANCE CRITERIA:
    // "Export-only edit across multi-module session produces imports_resolved == 0 for all dependent modules"
    assert_eq!(up.stats.imports_resolved, 0, "export-only edit must not re-resolve unchanged imports in dependents");
    assert!(up.stats.import_sites_reused >= 1, "import sites must be reused");
}

#[test]
fn checkpoint_a2_evidence_4_and_5_negative_resolution_survives_unrelated_and_invalidates_on_candidate() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("pkg");
    fs::create_dir_all(&root).unwrap();

    fs::write(root.join("package.ph"), "export A\nexport B\nexport C\n").unwrap();
    let file_a = root.join("a.ph");
    let file_b = root.join("b.ph");
    let file_c = root.join("c.ph");

    // a imports missing .c
    fs::write(&file_a, "import .c as C\nclass A {}\nexport A\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(location(&root.join("package.ph")), Arc::from("export A\nexport B\nexport C\n"), SourceRevision(1))
        .unwrap();

    let init_up = session
        .set_overlay(location(&file_a), Arc::from("import .c as C\nclass A {}\nexport A\n"), SourceRevision(1))
        .unwrap();

    assert_eq!(init_up.stats.imports_resolved, 1);
    let a_mod = session.module_for_source(&location(&file_a).source_id).unwrap().clone();
    assert!(session.diagnostics().contains_key(&a_mod));

    // EVIDENCE 4:
    // Adding unrelated b.ph when import was for missing c.ph
    // Produces imports_resolved == 0, negative resolution reused.
    let unrelated_up = session
        .set_overlay(location(&file_b), Arc::from("class B {}\nexport B\n"), SourceRevision(1))
        .unwrap();

    assert_eq!(unrelated_up.stats.imports_resolved, 0, "unrelated module addition must not re-resolve absent import");
    assert_eq!(unrelated_up.stats.negative_resolutions_reused, 1, "negative resolution must be reused");
    assert!(session.diagnostics().contains_key(&a_mod));

    // EVIDENCE 5:
    // Adding c.ph re-resolves the site that was waiting for c.
    let relevant_up = session
        .set_overlay(location(&file_c), Arc::from("class C {}\nexport C\n"), SourceRevision(1))
        .unwrap();

    assert_eq!(relevant_up.stats.imports_resolved, 1, "adding candidate module must re-resolve the waiting site");
    let a_diags = session.diagnostics().get(&a_mod).cloned().unwrap_or_default();
    assert!(a_diags.is_empty(), "a.ph must now resolve without error: {:?}", a_diags);
}

#[test]
fn checkpoint_a2_evidence_6_prefix_provenance_compound_imports() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("pkg");
    let sub = root.join("sub");
    fs::create_dir_all(&sub).unwrap();

    fs::write(root.join("package.ph"), "export Sub\nexport Main\n").unwrap();
    fs::write(sub.join("package.ph"), "export Leaf\n").unwrap();
    fs::write(sub.join("leaf.ph"), "class Leaf {}\nexport Leaf\n").unwrap();

    let main_file = root.join("main.ph");
    fs::write(&main_file, "import .sub.leaf as Leaf\nclass Main {}\nexport Main\n").unwrap();

    let mut session = WorkspaceModuleSession::new();
    session
        .set_overlay(location(&root.join("package.ph")), Arc::from("export Sub\nexport Main\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&sub.join("package.ph")), Arc::from("export Leaf\n"), SourceRevision(1))
        .unwrap();
    session
        .set_overlay(location(&sub.join("leaf.ph")), Arc::from("class Leaf {}\nexport Leaf\n"), SourceRevision(1))
        .unwrap();
    let main_up = session
        .set_overlay(location(&main_file), Arc::from("import .sub.leaf as Leaf\nclass Main {}\nexport Main\n"), SourceRevision(1))
        .unwrap();

    assert_eq!(main_up.stats.imports_resolved, 1);
    let main_mod = session.module_for_source(&location(&main_file).source_id).unwrap().clone();
    let site = ImportSiteId::new(main_mod.clone(), phalcom_modules::identity::ImportSiteLocalId::new(0));

    let product = session.import_product(&site).expect("import product exists for site 0");
    assert!(product.target.is_ok());

    assert!(!product.prefixes.is_empty(), "compound import must record prefixes");
    let prefix_names: Vec<_> = product.prefixes.iter().map(|p| p.prefix.as_str()).collect();
    assert!(prefix_names.contains(&".sub"), "prefixes must include '.sub': {:?}", prefix_names);

    let mut delta = phalcom_modules::topology::TopologyDelta::default();
    assert!(!delta.resolution_product_may_have_changed(product));

    let sub_pkg_mod = product.prefixes.iter().find(|p| p.prefix == ".sub").unwrap().module.clone();
    delta.removed_modules.insert(sub_pkg_mod);
    assert!(delta.resolution_product_may_have_changed(product), "removal of intermediate prefix module must invalidate product");
}
