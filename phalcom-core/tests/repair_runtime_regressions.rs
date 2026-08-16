use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_runtime_materialization_idempotent() {
    let mut vm = VM::new();
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/diamond_app");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("diamond_app should compile");

    // First materialization and run
    vm.run_compiled(&program).expect("first run succeeds");

    let entry_obj1 = vm.module_registry.get(&program.entry).unwrap().object;

    // Second materialization of the same program
    vm.materialize_program(&program).expect("second materialization succeeds");
    let entry_obj2 = vm.module_registry.get(&program.entry).unwrap().object;

    // Objects are reused and identity is preserved
    assert_eq!(entry_obj1, entry_obj2);
}

#[test]
fn test_whole_module_reexport_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src/sub")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nentry = \"app.main\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "expose .sub\n").unwrap();
    fs::write(proj_dir.join("src/sub/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/sub/calculator.ph"), "let add = |a, b| { a + b }\nexport add\n").unwrap();
    // facade re-exports sub.calculator
    fs::write(proj_dir.join("src/facade.ph"), "import .sub.calculator as calc\nexport calc\n").unwrap();
    // main calls through facade
    fs::write(proj_dir.join("src/main.ph"), "import .facade as f\nlet res = f.calc.add(10, 20)\n").unwrap();

    let mut vm = VM::new();
    let selection = EntrySelection::Project(proj_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("whole-module reexport app compiles");
    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "whole-module reexport app runs: {:?}", res.err());
}

#[test]
fn test_export_callable_arguments_dispatch() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nentry = \"app.main\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/ops.ph"), "let divide = |a, b| { a / b }\nexport divide\n").unwrap();
    fs::write(proj_dir.join("src/main.ph"), "import .ops as ops\nlet result = ops.divide(20, 4)\n").unwrap();

    let mut vm = VM::new();
    let selection = EntrySelection::Project(proj_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("labeled export app compiles");
    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "callable export send succeeds: {:?}", res.err());
}

#[test]
fn test_export_name_shadows_module_methods() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nentry = \"app.main\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "").unwrap();
    fs::write(proj_dir.join("src/meta-mod.ph"), "let name = \"custom_module_name\"\nexport name\n").unwrap();
    fs::write(proj_dir.join("src/main.ph"), "import .meta_mod as m\nlet n = m.name\n").unwrap();

    let mut vm = VM::new();
    let selection = EntrySelection::Project(proj_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("export name collision app compiles");
    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "export shadows module method: {:?}", res.err());
}
