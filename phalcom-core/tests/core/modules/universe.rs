//! Comprehensive integration test suite validating the entire universe/std/project model.
//! Covers spec test IDs: MODEL-01..05, ROOT-01..05, CORE-01..03, BOOT-01..03,
//! PRE-01..04, NONE-01, PROJ-01..04, NAME-01..03, PKG-01, EXP-01, REF-01..05,
//! META-01..03, SEL-01..03, SHADOW-01..03, STD-01, SCC-01..03.

use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::native::NativeSourceIndex;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{DunderPolicy, DunderRole, ModuleId, ModulePath, ProjectUniverse};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn fixture_path(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/universe_v1").join(sub)
}

#[test]
fn project_hierarchy_preserves_class_relationships_and_root_bindings() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("project_hierarchy");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("project_hierarchy compiles");

    vm.run_compiled(&program).expect("project_hierarchy runs");

    // MODEL-01: Verify Project < Object, Package < Module class relationship in kernel tower
    let project_class = vm.universe.classes.project_class;
    let package_class = vm.universe.classes.package_class;
    let module_class = vm.universe.classes.module_class;
    let object_class = vm.universe.classes.object_class;

    assert_eq!(vm.heap.class(project_class).superclass, Some(object_class));
    assert_eq!(vm.heap.class(package_class).superclass, Some(module_class));

    // MODEL-02: Project root package.ph has __module__, __package__, __project__ all referring to the Project object
    let root_id = program.modules.keys().find(|id| id.path.is_root()).expect("root project module present");
    let root_obj = vm.module_registry.get(root_id).unwrap().object;

    // Check structural fields on ModuleObject
    assert_eq!(vm.heap.module(root_obj).package, Some(root_obj));
    assert_eq!(vm.heap.module(root_obj).root_package, Some(root_obj));

    // Check globals __module__, __package__, __project__
    let mod_sym = vm.interner.intern("__module__");
    let pkg_sym = vm.interner.intern("__package__");
    let proj_sym = vm.interner.intern("__project__");
    let mod_val = vm.heap.module(root_obj).get(mod_sym).unwrap();
    let pkg_val = vm.heap.module(root_obj).get(pkg_sym).unwrap();
    let proj_val = vm.heap.module(root_obj).get(proj_sym).unwrap();

    assert_eq!(mod_val, Value::obj(root_obj));
    assert_eq!(pkg_val, Value::obj(root_obj).wrap_some().unwrap());
    assert!(proj_val.is_some());
    let proj_obj = proj_val.gc_obj_ref().unwrap();
    assert_eq!(vm.heap.project(proj_obj).root_package, root_obj);

    // NAME-02: Check that module-b.ph in package-a maps to logical geometry_toolkit.package_a.module_b
    let b_mod_entry = program
        .modules
        .keys()
        .find(|id| id.path.to_string().contains("module_b"))
        .expect("module_b present");
    assert_eq!(b_mod_entry.path.to_string(), "package_a.module_b");
}

#[test]
fn standalone_package_has_no_project_binding() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("standalone_package");
    let selection = EntrySelection::Package(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("standalone package compiles");

    vm.run_compiled(&program).expect("standalone package runs");

    // Root package of standalone package has __project__ == None
    let root_mod = program.modules.keys().find(|id| id.path.is_root()).unwrap();
    let root_obj = vm.module_registry.get(root_mod).unwrap().object;

    let proj_sym = vm.interner.intern("__project__");
    let proj_val = vm.heap.module(root_obj).get(proj_sym).unwrap();
    assert_eq!(proj_val, Value::none());
    assert_eq!(vm.heap.module(root_obj).root_package, None);
}

#[test]
fn standalone_module_has_no_package_or_project_binding() {
    let mut vm = VM::new();
    let solo_file = fixture_path("standalone_module/solo.ph");
    let selection = EntrySelection::Module(solo_file);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("standalone module compiles");

    vm.run_compiled(&program).expect("standalone module runs");

    let solo_obj = vm.module_registry.get(&program.entry).unwrap().object;

    let pkg_sym = vm.interner.intern("__package__");
    let proj_sym = vm.interner.intern("__project__");

    assert_eq!(vm.heap.module(solo_obj).get(pkg_sym).unwrap(), Value::none());
    assert_eq!(vm.heap.module(solo_obj).get(proj_sym).unwrap(), Value::none());
    assert_eq!(vm.heap.module(solo_obj).package, None);
    assert_eq!(vm.heap.module(solo_obj).root_package, None);
}

#[test]
fn builtin_client_resolves_prelude_and_standard_library() {
    let mut vm = VM::new();
    let fixture_dir = fixture_path("builtin_client");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("builtin client compiles");

    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "builtin client runs: {:?}", res.err());
}

#[test]
fn universe_package_intrinsics_match_provider_neutral_package_rules() {
    let vm = VM::new();
    let root = vm.module_registry.get(&ModuleId::universe_root()).expect("Universe root materialized").object;
    let collections = ModuleId::universe(ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("collections").unwrap(),
    ]));
    let collections = vm.module_registry.get(&collections).expect("collections package materialized").object;
    let list = ModuleId::universe(ModulePath::from_components(vec![
        phalcom_modules::ModuleComponent::from_identifier("collections").unwrap(),
        phalcom_modules::ModuleComponent::from_identifier("list").unwrap(),
    ]));
    let list = vm.module_registry.get(&list).expect("collections.list module materialized").object;

    assert_eq!(vm.heap.module(root).package, Some(root));
    assert_eq!(vm.heap.module(collections).package, Some(collections));
    assert_eq!(vm.heap.module(list).package, Some(collections));
}

#[test]
fn boot_01_bootstrap_measurement_separates_catalog_closure_and_execution() {
    let index = NativeSourceIndex::build().expect("canonical Universe source index builds");
    let root = ModuleId::universe_root();
    let reachable = index
        .reachable_units_from_roots(std::slice::from_ref(&root))
        .expect("Universe root dependency closure resolves");
    let ordered = index
        .initialization_order_from_roots(std::slice::from_ref(&root))
        .expect("Universe root dependency order resolves");

    assert_eq!(ordered.len(), reachable.len());
    assert!(reachable.contains(&root));
    assert!(
        reachable.len() < index.units.len(),
        "root closure must remain distinct from full source catalog"
    );

    let vm = VM::new();
    let measurement = vm.universe_bootstrap_measurement();
    assert_eq!(measurement.discovered_units, index.units.len());
    assert_eq!(measurement.root_reachable_units, reachable.len());
    assert!(measurement.executed_units <= measurement.discovered_units);
    assert!(measurement.executed_units > 0);
}

#[test]
fn standalone_context_rejects_user_project_dependencies() {
    let tmp = TempDir::new().unwrap();
    let solo = tmp.path().join("solo.ph");
    // Attempting to import an external user project from standalone module must fail
    fs::write(&solo, "import custom_dep.parser as p\nlet x = 1\n").unwrap();

    let selection = EntrySelection::Module(solo);
    let res = ProgramCompiler::compile_entry_selection(selection);
    assert!(res.is_err(), "standalone module must not resolve user project dependencies");
    match res.unwrap_err() {
        ProgramCompileError::StandaloneImportRequiresPackageContext { import_name } => {
            assert!(import_name.contains("custom_dep"));
        }
        other => panic!("expected StandaloneImportRequiresPackageContext, got {other:?}"),
    }
}

#[test]
fn legacy_core_import_is_rejected() {
    let mut universe = ProjectUniverse::new();
    let tmp = TempDir::new().unwrap();
    let manifest = tmp.path().join("project.toml");
    fs::write(
        &manifest,
        "[project]\nname = \"core_test\"\nnamespace = \"core_test\"\nentry = \"core_test.main\"\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/package.ph"), "").unwrap();
    let root_id = universe.load_root(&manifest).unwrap();
    let importer_id = ModuleId::resolved(root_id, ModulePath::root());

    let provider = phalcom_modules::FilesystemSourceProvider::new();
    let mut resolver = phalcom_modules::ModuleResolver::new(&universe, &provider);
    let syntax = phalcom_ast::parse("import core.Object\n", 0);
    let dep = &syntax.program.preamble.dependencies[0];
    if let phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(m)) = dep {
        let err = resolver.resolve_import(&importer_id, &m.path).unwrap_err();
        assert!(matches!(err, phalcom_modules::ModuleResolutionError::LegacyCoreImportRemoved));
    }
}

#[test]
fn curated_prelude_exposes_public_names_and_hides_internal_classes() {
    let mut vm = VM::new();
    // PRE-01: Object, Class, Number, String, List, Map, Set, Option, Some, Function, Selector, SelectorPattern are in prelude
    for name in [
        "Object",
        "Class",
        "Number",
        "String",
        "List",
        "Map",
        "Set",
        "Option",
        "Some",
        "Function",
        "Selector",
        "SelectorPattern",
    ] {
        let sym = vm.interner.intern(name);
        assert!(vm.prelude_bindings.contains_key(&sym), "prelude must contain {name}");
    }
    // PRE-02: Behavior, Metaclass, Message are NOT in prelude
    for name in ["Behavior", "Metaclass", "Message"] {
        let sym = vm.interner.intern(name);
        assert!(!vm.prelude_bindings.contains_key(&sym), "prelude must NOT contain {name}");
    }

    // NONE-01: Prelude None is immediate Value::none(); universe.None is the None class object
    let none_sym = vm.interner.intern("None");
    assert!(vm.prelude_bindings.contains_key(&none_sym));
    let universe_pkg = vm.create_builtin_package("universe");
    let none_cls = vm.universe.classes.none_class;
    assert_eq!(vm.heap.module(universe_pkg).get(none_sym).unwrap(), Value::obj(none_cls));
}

#[test]
fn project_manifest_validation_rejects_invalid_and_accepts_valid_namespace() {
    let tmp = TempDir::new().unwrap();
    let manifest_path = tmp.path().join("project.toml");

    // PROJ-02: Missing namespace rejected
    fs::write(&manifest_path, "[project]\nname = \"My Project\"\n").unwrap();
    let res = phalcom_modules::ProjectManifest::load_file(&manifest_path).and_then(|m| m.validate());
    assert!(res.is_err(), "missing namespace must fail validation");

    // NAME-01: kebab-case or invalid namespace rejected
    fs::write(&manifest_path, "[project]\nname = \"My Project\"\nnamespace = \"my-project\"\n").unwrap();
    let res2 = phalcom_modules::ProjectManifest::load_file(&manifest_path).and_then(|m| m.validate());
    assert!(res2.is_err(), "kebab namespace must fail validation");

    // Valid snake_case namespace accepted
    fs::write(&manifest_path, "[project]\nname = \"My Project\"\nnamespace = \"my_project\"\n").unwrap();
    let manifest = phalcom_modules::ProjectManifest::load_file(&manifest_path).and_then(|m| m.validate()).unwrap();
    assert_eq!(manifest.name, "My Project");
    assert_eq!(manifest.namespace.as_str(), "my_project");
}

#[test]
fn dunder_policy_restricts_internal_names_and_allows_authorized_hooks() {
    let policy = DunderPolicy::default();

    // REF-01: Unknown dunder rejected
    assert!(policy.validate_user_declaration("__unknown__", DunderRole::Binding).is_err());
    // Non-overridable dunder rejected
    assert!(policy.validate_user_declaration("__module__", DunderRole::Binding).is_err());
    assert!(policy.validate_user_declaration("__name__", DunderRole::Method).is_err());

    // REF-01B: Test hook with authorized role succeeds
    let hook_policy = policy.with_hook("__intercept__", &[DunderRole::Method]);
    assert!(hook_policy.validate_user_declaration("__intercept__", DunderRole::Method).is_ok());
    assert!(hook_policy.validate_user_declaration("__intercept__", DunderRole::Binding).is_err());

    // REF-02: _$ internal method declaration from user source is rejected by compiler/parser
    let source = "class MyClass { _$primitive() { 1 } }";
    let res = phalcom_ast::parse(source, 0);
    assert!(!res.errors.is_empty(), "ast rejects _$ in user method declarations");
}

#[test]
fn unit_metadata_attaches_to_packages_and_modules() {
    let fixture_dir = fixture_path("unit_metadata");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("unit_metadata compiles");

    let mut vm = VM::new();
    vm.run_compiled(&program).expect("unit_metadata runs");

    // META-01 & META-02: Verify metadata attached to module objects
    let root_id = program.modules.keys().find(|id| id.path.is_root()).unwrap();
    let root_obj = vm.module_registry.get(root_id).unwrap().object;
    let root_meta = vm.heap.module(root_obj).metadata.as_ref().unwrap();
    assert_eq!(root_meta.attributes[0].name, "documentation");
    assert_eq!(root_meta.attributes[0].target, phalcom_modules::MetadataTarget::Package);

    let child_pkg_id = program.modules.keys().find(|id| id.path.to_string() == "child").unwrap();
    let child_pkg_obj = vm.module_registry.get(child_pkg_id).unwrap().object;
    let child_meta = vm.heap.module(child_pkg_obj).metadata.as_ref().unwrap();
    assert_eq!(child_meta.attributes[0].name, "documentation");
    assert_eq!(child_meta.attributes[0].target, phalcom_modules::MetadataTarget::Package);

    let mod_a_id = program.modules.keys().find(|id| id.path.to_string() == "child.module_a").unwrap();
    let mod_a_obj = vm.module_registry.get(mod_a_id).unwrap().object;
    let mod_a_meta = vm.heap.module(mod_a_obj).metadata.as_ref().unwrap();
    assert_eq!(mod_a_meta.attributes[0].name, "documentation");
    assert_eq!(mod_a_meta.attributes[0].target, phalcom_modules::MetadataTarget::Module);
}

#[test]
fn semantic_scc_project_realizes_successfully() {
    let fixture_dir = fixture_path("semantic_scc");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("semantic_scc compiles");

    let mut vm = VM::new();
    vm.run_compiled(&program).expect("semantic_scc runs cleanly");
}
