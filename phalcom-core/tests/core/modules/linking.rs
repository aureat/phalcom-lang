use phalcom_core::modules::RuntimeLinkedRead;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{LinkError, LinkedReadSpec};
use std::path::PathBuf;
use std::sync::Arc;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_linker").join(name)
}

/// COMP-01 — Single-module file compiles without error
#[test]
fn comp_01_single_module_inline_compiles() {
    let source: Arc<str> = "let x = 42\n".into();
    let selection = EntrySelection::Inline(source);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");
    assert_eq!(program.modules.len(), 1);
}

/// COMP-02 — Selective import of non-exported name fails at link time
#[test]
fn comp_02_selective_import_non_exported_fails_link() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.importer\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .exporter\nexpose .importer\n").unwrap();
    std::fs::write(src_dir.join("exporter.ph"), "class Foo {}\n").unwrap(); // not exported
    std::fs::write(src_dir.join("importer.ph"), "from .exporter import Foo\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let result = ProgramCompiler::compile_entry_selection(selection);
    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::MissingExport { ref name, .. })) if name == "Foo"),
        "expected MissingExport for Foo, got {:?}",
        result
    );
}

/// COMP-03 — Module-import without alias uses last path segment as local name
#[test]
fn comp_03_module_import_uses_last_segment() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "import universe.reflection.selector\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");

    let (entry_id, entry_mod) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("entry"))
        .expect("entry module present");

    let linked_entry = program.linked.modules.get(entry_id).expect("linked entry present");
    assert!(linked_entry.bindings.imports.contains_key("selector"));
    assert!(
        entry_mod
            .linked_reads
            .iter()
            .any(|spec| matches!(spec, LinkedReadSpec::Module(id) if id.path.to_string().contains("selector")))
    );
}

/// COMP-04 — Builtin selective import of a real export compiles successfully
#[test]
fn comp_04_builtin_selective_import_real_export_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "from universe.errors.unsupported import unsupported\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile should succeed");

    let (entry_id, entry_mod) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("entry"))
        .expect("entry module present");

    let linked_entry = program.linked.modules.get(entry_id).expect("linked entry present");
    assert!(linked_entry.bindings.imports.contains_key("unsupported"));
    assert!(
        entry_mod
            .linked_reads
            .iter()
            .any(|spec| matches!(spec, LinkedReadSpec::Binding(sym) if &*sym.name == "unsupported"))
    );
}

/// COMP-05 — Builtin selective import of a non-exported name fails
#[test]
fn comp_05_builtin_selective_import_non_exported_fails() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"test_pkg\"\nnamespace = \"test_pkg\"\nversion = \"0.1.0\"\nentry = \"test_pkg.entry\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .entry\n").unwrap();
    std::fs::write(src_dir.join("entry.ph"), "from universe.reflection import NonExistentExport\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let result = ProgramCompiler::compile_entry_selection(selection);
    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::MissingExport { ref name, .. })) if name == "NonExistentExport"),
        "expected MissingExport for NonExistentExport, got {:?}",
        result
    );
}

/// COMP-06 — Initialization order puts dependencies before dependents
#[test]
fn comp_06_chain_abc_initialization_order() {
    let selection = EntrySelection::Project(fixture_path("chain_abc"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile chain_abc");

    let order = &program.initialization_order;
    let pos_c = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".c") || id.path.to_string() == "c")
        .expect("c found");
    let pos_b = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".b") || id.path.to_string() == "b")
        .expect("b found");
    let pos_a = order
        .iter()
        .position(|id| id.path.to_string().ends_with(".a") || id.path.to_string() == "a")
        .expect("a found");

    assert!(pos_c < pos_b, "c must precede b");
    assert!(pos_b < pos_a, "b must precede a");
}

/// COMP-07 — Cyclic module dependency is detected and rejected
#[test]
fn comp_07_cycle_ab_detected_and_rejected() {
    let selection = EntrySelection::Project(fixture_path("cycle_ab"));
    let result = ProgramCompiler::compile_entry_selection(selection);

    assert!(
        matches!(result, Err(ProgramCompileError::Link(LinkError::RuntimeCycle(_)))),
        "expected RuntimeCycle error, got {:?}",
        result
    );
}

/// COMP-08 — EntrySelection::Module for a standalone .ph file succeeds
#[test]
fn comp_08_standalone_module_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("standalone.ph");
    std::fs::write(&file, "let x = 1\n").unwrap();

    let selection = EntrySelection::Module(file);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("standalone compile");
    assert_eq!(program.modules.len(), 1);
}

/// MAT-01 — Module export slots are populated after run_compiled
#[test]
fn mat_01_export_slots_populated() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (exporter_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("exporter"))
        .expect("exporter module present");

    let exporter_obj = vm.module_registry.get(exporter_id).unwrap().object;
    let widget_sym = vm.interner.intern("Widget");
    let widget_val = vm.heap.module(exporter_obj).get(widget_sym).expect("Widget slot exists");
    assert!(!widget_val.is_none(), "Widget slot must not be None");
    assert!(widget_val.is_obj(), "Widget slot must be an object");
}

/// MAT-02 — __module__ intrinsic resolves to the module object itself
#[test]
fn mat_02_module_intrinsic_resolves_to_self() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let mod_sym = vm.interner.intern("__module__");
    for id in program.modules.keys() {
        let obj = vm.module_registry.get(id).unwrap().object;
        let val = vm.heap.module(obj).get(mod_sym).expect("__module__ present");
        assert_eq!(val, Value::obj(obj), "__module__ must match own object reference for {id}");
    }
}

/// MAT-03 — __package__ for a root package is Some(self)
#[test]
fn mat_03_root_package_intrinsic_is_some_self() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (root_id, _) = program.modules.iter().find(|(id, _)| id.path.is_root()).expect("root package present");

    let root_obj = vm.module_registry.get(root_id).unwrap().object;
    let pkg_sym = vm.interner.intern("__package__");
    let val = vm.heap.module(root_obj).get(pkg_sym).expect("__package__ present");
    assert_eq!(val, Value::obj(root_obj).wrap_some().unwrap());
}

/// MAT-04 — __package__ for a leaf module points to nearest parent package object
#[test]
fn mat_04_leaf_package_points_to_parent() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (root_id, _) = program.modules.iter().find(|(id, _)| id.path.is_root()).expect("root package present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;

    let (exporter_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("exporter"))
        .expect("exporter present");
    let exporter_obj = vm.module_registry.get(exporter_id).unwrap().object;

    let pkg_sym = vm.interner.intern("__package__");
    let val = vm.heap.module(exporter_obj).get(pkg_sym).expect("__package__ present");
    assert_eq!(val, Value::obj(root_obj).wrap_some().unwrap());
}

/// MAT-05 — Selective import value is non-None after full run_compiled
#[test]
fn mat_05_selective_import_value_non_none() {
    let mut vm = VM::new();
    let selection = EntrySelection::Project(fixture_path("simple_export"));
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile simple_export");
    vm.run_compiled(&program).expect("run simple_export");

    let (importer_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("importer"))
        .expect("importer present");
    let importer_obj = vm.module_registry.get(importer_id).unwrap().object;

    // Check linked_read slot 0
    let linked_read = vm.heap.module(importer_obj).linked_reads.first().copied().expect("linked read 0");
    let value = match linked_read {
        RuntimeLinkedRead::Module(m) => Value::obj(m),
        RuntimeLinkedRead::Binding(b) => vm.heap.module(b.module).get_by_slot(b.slot as usize).expect("slot value"),
    };
    assert!(!value.is_none(), "linked read for Widget must be non-None");
    assert!(value.is_obj(), "linked read for Widget must be an object");
}

/// MAT-06 — Module-import binding resolves to the module object
#[test]
fn mat_06_module_import_binding_resolves_to_module() {
    let mut vm = VM::new();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"mod_import_test\"\nnamespace = \"mod_import_test\"\nversion = \"0.1.0\"\nentry = \"mod_import_test.importer\"\n",
    )
    .unwrap();
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("package.ph"), "expose .importer\n").unwrap();
    std::fs::write(src_dir.join("importer.ph"), "import universe.reflection.selector\n").unwrap();

    let selection = EntrySelection::Project(root.to_path_buf());
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("run succeeds");

    let (importer_id, _) = program
        .modules
        .iter()
        .find(|(id, _)| id.path.to_string().contains("importer"))
        .expect("importer present");
    let importer_obj = vm.module_registry.get(importer_id).unwrap().object;

    let linked_read = vm.heap.module(importer_obj).linked_reads.first().copied().expect("linked read 0");
    match linked_read {
        RuntimeLinkedRead::Module(m) => {
            assert!(vm.heap.module(m).name.contains("reflection.selector"));
        }
        RuntimeLinkedRead::Binding(_) => panic!("expected Module linked read"),
    }
}
