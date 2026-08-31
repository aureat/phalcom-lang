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
fn test_module_reflection_contract() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("project_hierarchy");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("runs successfully");

    // Find module_b in package_a
    let mod_id = program
        .modules
        .keys()
        .find(|id| id.path.to_string().contains("module_b"))
        .expect("module_b present");
    let mod_obj = vm.module_registry.get(mod_id).unwrap().object;
    let mod_val = Value::obj(mod_obj);

    // 1. name, namespace, uri, identity
    let name_sym = vm.interner.intern("name");
    let namespace_sym = vm.interner.intern("namespace");
    let uri_sym = vm.interner.intern("uri");
    let id_sym = vm.interner.intern("identity");

    let name_val = vm.send_dynamic(mod_val, name_sym, &[]).expect("name send succeeds");
    assert_eq!(vm.resolve_symbol(name_val.as_symbol().unwrap()), "proj#1:package_a.module_b");

    let namespace_val = vm.send_dynamic(mod_val, namespace_sym, &[]).expect("namespace send succeeds");
    assert!(namespace_val.is_some());
    assert_eq!(namespace_val, Value::symbol(vm.interner.intern("geometry_toolkit")).wrap_some().unwrap());

    let uri_val = vm.send_dynamic(mod_val, uri_sym, &[]).expect("uri send succeeds");
    let uri_obj = uri_val.as_obj().expect("uri object");
    assert!(vm.heap.uri(uri_obj).uri_str.starts_with("file://"));

    let id_val = vm.send_dynamic(mod_val, id_sym, &[]).expect("identity send succeeds");
    let id_obj = id_val.as_obj().expect("identity object");
    assert!(vm.heap.module_identity(id_obj).id_str.starts_with("mod:"));

    // 2. dunder methods: __name__, __path__, __id__, __uri__
    let dunder_name_sym = vm.interner.intern("__name__");
    let dunder_path_sym = vm.interner.intern("__path__");
    let dunder_id_sym = vm.interner.intern("__id__");
    let dunder_uri_sym = vm.interner.intern("__uri__");

    let dunder_name_val = vm.send_dynamic(mod_val, dunder_name_sym, &[]).expect("__name__ succeeds");
    assert_eq!(vm.resolve_symbol(dunder_name_val.as_symbol().unwrap()), "proj#1:package_a.module_b");

    let dunder_path_val = vm.send_dynamic(mod_val, dunder_path_sym, &[]).expect("__path__ succeeds");
    assert_eq!(vm.heap.string(dunder_path_val.as_obj().unwrap()).as_str(), "package_a.module_b");

    let dunder_id_val = vm.send_dynamic(mod_val, dunder_id_sym, &[]).expect("__id__ succeeds");
    assert_eq!(dunder_id_val, id_val);

    let dunder_uri_val = vm.send_dynamic(mod_val, dunder_uri_sym, &[]).expect("__uri__ succeeds");
    assert_eq!(dunder_uri_val, uri_val);

    // 3. package, rootPackage, packageInfo
    let pkg_sym = vm.interner.intern("package");
    let root_pkg_sym = vm.interner.intern("rootPackage");
    let pkg_info_sym = vm.interner.intern("packageInfo");

    let pkg_val = vm.send_dynamic(mod_val, pkg_sym, &[]).expect("package send succeeds");
    assert!(pkg_val.is_some());

    let root_pkg_val = vm.send_dynamic(mod_val, root_pkg_sym, &[]).expect("rootPackage send succeeds");
    assert!(root_pkg_val.is_some());

    let pkg_info_val = vm.send_dynamic(mod_val, pkg_info_sym, &[]).expect("packageInfo send succeeds");
    let pkg_info_obj = pkg_info_val.gc_obj_ref().expect("PackageInfo object");
    assert_eq!(vm.heap.package_info(pkg_info_obj).name, "Geometry Toolkit");

    // 4. exports and ExportTable
    let exports_sym = vm.interner.intern("exports");
    let exports_val = vm.send_dynamic(mod_val, exports_sym, &[]).expect("exports send succeeds");
    let _exports_obj = exports_val.as_obj().expect("ExportTable object");

    let names_sym = vm.interner.intern("names");
    let names_val = vm.send_dynamic(exports_val, names_sym, &[]).expect("names send succeeds");
    assert!(names_val.is_obj());

    let contains_sym = vm.interner.intern("contains(_)");
    let contains_arg = Value::symbol(vm.interner.intern("calculate"));
    let contains_val = vm.send_dynamic(exports_val, contains_sym, &[contains_arg]).expect("contains send succeeds");
    assert_eq!(contains_val.as_bool(), Some(true));

    let get_sym = vm.interner.intern("get(_)");
    let get_val = vm.send_dynamic(exports_val, get_sym, &[contains_arg]).expect("get send succeeds");
    assert!(get_val.is_some());

    // 5. __exports__, __export__, __understands__
    let dunder_exports_sym = vm.interner.intern("__exports__");
    let dunder_exports_val = vm.send_dynamic(mod_val, dunder_exports_sym, &[]).expect("__exports__ succeeds");
    assert_eq!(dunder_exports_val, exports_val);

    let dunder_export_sym = vm.interner.intern("__export__(_)");
    let dunder_export_val = vm.send_dynamic(mod_val, dunder_export_sym, &[contains_arg]).expect("__export__ succeeds");
    assert_eq!(dunder_export_val, get_val);

    let dunder_understands_sym = vm.interner.intern("__understands__(_)");
    let dunder_understands_val = vm
        .send_dynamic(mod_val, dunder_understands_sym, &[contains_arg])
        .expect("__understands__ succeeds");
    assert_eq!(dunder_understands_val.as_bool(), Some(true));

    // 6. dependencies and ModuleDependency
    let deps_sym = vm.interner.intern("dependencies");
    let deps_val = vm.send_dynamic(mod_val, deps_sym, &[]).expect("dependencies send succeeds");
    assert!(deps_val.is_obj());

    let dunder_deps_sym = vm.interner.intern("__dependencies__");
    let dunder_deps_val = vm.send_dynamic(mod_val, dunder_deps_sym, &[]).expect("__dependencies__ succeeds");
    assert!(dunder_deps_val.is_obj());
}
