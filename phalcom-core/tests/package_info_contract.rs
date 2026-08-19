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
fn test_package_reflection_contract() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("project_hierarchy");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("compile succeeds");
    vm.run_compiled(&program).expect("runs successfully");

    // Root package
    let root_id = program.modules.keys().find(|id| id.path.is_root()).expect("root package present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;
    let root_val = Value::obj(root_obj);

    // 1. isRoot, parentPackage, rootPackage
    let is_root_sym = vm.interner.intern("isRoot");
    let parent_pkg_sym = vm.interner.intern("parentPackage");
    let root_pkg_sym = vm.interner.intern("rootPackage");

    let is_root_val = vm.send_dynamic(root_val, is_root_sym, &[]).expect("isRoot send succeeds");
    assert_eq!(is_root_val.as_bool(), Some(true));

    let parent_val = vm.send_dynamic(root_val, parent_pkg_sym, &[]).expect("parentPackage send succeeds");
    assert!(parent_val.is_none());

    let root_pkg_val = vm.send_dynamic(root_val, root_pkg_sym, &[]).expect("rootPackage send succeeds");
    assert_eq!(root_pkg_val, Value::obj(root_obj));

    // 2. children and ChildModuleTable
    let children_sym = vm.interner.intern("children");
    let children_val = vm.send_dynamic(root_val, children_sym, &[]).expect("children send succeeds");
    assert!(children_val.is_obj());

    let names_sym = vm.interner.intern("names");
    let names_val = vm.send_dynamic(children_val, names_sym, &[]).expect("names send succeeds");
    assert!(names_val.is_obj());

    let contains_sym = vm.interner.intern("contains(_)");
    let child_sym = Value::symbol(vm.interner.intern("package_a"));
    let contains_val = vm.send_dynamic(children_val, contains_sym, &[child_sym]).expect("contains send succeeds");
    assert_eq!(contains_val.as_bool(), Some(true));

    let get_sym = vm.interner.intern("get(_)");
    let get_val = vm.send_dynamic(children_val, get_sym, &[child_sym]).expect("get send succeeds");
    assert!(get_val.is_some());

    // 3. dunder methods: __parent__, __children__, __version__, __namespace__
    let dunder_parent_sym = vm.interner.intern("__parent__");
    let dunder_children_sym = vm.interner.intern("__children__");
    let dunder_version_sym = vm.interner.intern("__version__");
    let dunder_namespace_sym = vm.interner.intern("__namespace__");

    let dunder_parent_val = vm.send_dynamic(root_val, dunder_parent_sym, &[]).expect("__parent__ succeeds");
    assert_eq!(dunder_parent_val, parent_val);

    let dunder_children_val = vm.send_dynamic(root_val, dunder_children_sym, &[]).expect("__children__ succeeds");
    assert_eq!(dunder_children_val, children_val);

    let dunder_version_val = vm.send_dynamic(root_val, dunder_version_sym, &[]).expect("__version__ succeeds");
    assert!(dunder_version_val.is_none());

    let dunder_namespace_val = vm.send_dynamic(root_val, dunder_namespace_sym, &[]).expect("__namespace__ succeeds");
    assert_eq!(dunder_namespace_val.as_symbol().unwrap(), vm.interner.intern("geometry_toolkit"));

    // 4. packageInfo fields
    let pkg_info_sym = vm.interner.intern("packageInfo");
    let pkg_info_val = vm.send_dynamic(root_val, pkg_info_sym, &[]).expect("packageInfo send succeeds");
    let _info_obj = pkg_info_val.as_obj().expect("PackageInfo object");

    let info_name_sym = vm.interner.intern("name");
    let info_name_val = vm.send_dynamic(pkg_info_val, info_name_sym, &[]).expect("info.name succeeds");
    assert_eq!(vm.heap.string(info_name_val.as_obj().unwrap()).as_str(), "Geometry Toolkit");

    let info_authors_sym = vm.interner.intern("authors");
    let info_authors_val = vm.send_dynamic(pkg_info_val, info_authors_sym, &[]).expect("info.authors succeeds");
    assert!(info_authors_val.is_obj());

    let info_identity_sym = vm.interner.intern("identity");
    let info_identity_val = vm.send_dynamic(pkg_info_val, info_identity_sym, &[]).expect("info.identity succeeds");
    assert!(info_identity_val.is_obj());
}
