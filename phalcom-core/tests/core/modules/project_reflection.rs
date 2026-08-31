use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("universe_v1")
        .join(name)
}

#[test]
fn test_project_reflection_contract() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("project_hierarchy");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("runs successfully");

    // Root package
    let root_id = program.modules.keys().find(|id| id.path.is_root()).expect("root package present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;

    // Read __project__ global
    let proj_sym = vm.interner.intern("__project__");
    let proj_val = vm.heap.module(root_obj).get(proj_sym).unwrap();
    assert!(proj_val.is_some());

    let proj_inner = Value::obj(proj_val.gc_obj_ref().expect("Project object"));

    // 1. Project fields
    let name_sym = vm.interner.intern("name");
    let namespace_sym = vm.interner.intern("namespace");
    let manifest_sym = vm.interner.intern("manifest");
    let root_pkg_sym = vm.interner.intern("rootPackage");
    let deps_sym = vm.interner.intern("dependencies");
    let _dev_entry_sym = vm.interner.intern("developmentEntry");
    let id_sym = vm.interner.intern("identity");

    let name_val = vm.send_dynamic(proj_inner, name_sym, &[]).expect("proj.name succeeds");
    assert_eq!(vm.heap.string(name_val.as_obj().unwrap()).as_str(), "Geometry Toolkit");

    let namespace_val = vm.send_dynamic(proj_inner, namespace_sym, &[]).expect("proj.namespace succeeds");
    assert_eq!(namespace_val.as_symbol().unwrap(), vm.interner.intern("geometry_toolkit"));

    let manifest_val = vm.send_dynamic(proj_inner, manifest_sym, &[]).expect("proj.manifest succeeds");
    assert!(manifest_val.is_obj());

    let root_pkg_val = vm.send_dynamic(proj_inner, root_pkg_sym, &[]).expect("proj.rootPackage succeeds");
    assert_eq!(root_pkg_val, Value::obj(root_obj));

    let deps_val = vm.send_dynamic(proj_inner, deps_sym, &[]).expect("proj.dependencies succeeds");
    assert!(deps_val.is_obj());

    let id_val = vm.send_dynamic(proj_inner, id_sym, &[]).expect("proj.identity succeeds");
    assert!(id_val.is_obj());

    // 2. ProjectManifest fields
    let m_name_val = vm.send_dynamic(manifest_val, name_sym, &[]).expect("manifest.name succeeds");
    assert_eq!(vm.heap.string(m_name_val.as_obj().unwrap()).as_str(), "Geometry Toolkit");

    let authors_sym = vm.interner.intern("authors");
    let authors_val = vm.send_dynamic(manifest_val, authors_sym, &[]).expect("manifest.authors succeeds");
    assert!(authors_val.is_obj());

    let source_sym = vm.interner.intern("source");
    let source_val = vm.send_dynamic(manifest_val, source_sym, &[]).expect("manifest.source succeeds");
    assert!(source_val.is_obj());

    let dep_decls_sym = vm.interner.intern("dependencyDeclarations");
    let dep_decls_val = vm
        .send_dynamic(manifest_val, dep_decls_sym, &[])
        .expect("manifest.dependencyDeclarations succeeds");
    assert!(dep_decls_val.is_obj());
}

#[test]
fn test_standalone_has_no_project() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("standalone_package");
    let selection = EntrySelection::Package(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("runs successfully");

    let root_id = program.modules.keys().find(|id| id.path.is_root()).expect("root package present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;

    let proj_sym = vm.interner.intern("__project__");
    let proj_val = vm.heap.module(root_obj).get(proj_sym).unwrap();
    assert!(proj_val.is_none());
}
