use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::vm::VM;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_diamond_module_materialization_and_execution() {
    let mut vm = VM::new();
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/diamond_app");

    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("diamond_app should compile and link");

    assert!(program.modules.len() >= 4); // a, b, base, main
    assert!(program.initialization_order.len() >= 4);

    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "diamond_app execution should succeed: {:?}", res.err());
}

#[test]
fn test_sticky_failure_propagation() {
    let mut vm = VM::new();
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/failure_app");

    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("failure_app should compile and link");

    let res = vm.run_compiled(&program);
    assert!(res.is_err());
    let err = res.unwrap_err();

    match err {
        PhError::ModuleInitialization(init_err) => {
            let rendered = format!("{init_err}");
            assert!(rendered.contains("ModuleInitializationError:"));
            assert!(rendered.contains("Caused by error in module"));
        }
        other => panic!("expected ModuleInitializationError, got {other:?}"),
    }

    // Attempting to initialize the dependent module reproduces sticky failure with dependency chain
    let main_id = program.entry.clone();
    let res2 = vm.initialize_single_module(&program, &main_id, &mut Vec::new());
    assert!(res2.is_err());
    let err2 = res2.unwrap_err();
    match err2 {
        PhError::ModuleInitialization(init_err) => {
            let rendered = format!("{init_err}");
            assert!(rendered.contains("Dependency chain:"));
        }
        other => panic!("expected ModuleInitializationError, got {other:?}"),
    }
}

#[test]
fn test_context_free_inline_import_rejection() {
    let source: Arc<str> = "from .math import add\nlet x = 1\n".into();
    let selection = EntrySelection::Inline(source);
    let res = ProgramCompiler::compile_entry_selection(selection);

    assert!(matches!(res, Err(ProgramCompileError::ReplImportRequiresProjectContext { .. })));
}

#[test]
fn test_module_export_send_dispatch() {
    let mut vm = VM::new();
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/diamond_app");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("diamond_app should compile");
    vm.run_compiled(&program).expect("diamond_app initializes");

    // Find the base module in registry
    let (base_id, _) = program.modules.iter().find(|(id, _)| id.path.to_string().contains("base")).unwrap();
    let base_obj = vm.module_registry.get(base_id).unwrap().object;

    // Dispatch Config export on base module
    let config_sym = vm.interner.intern("Config");
    let res = vm.send_dynamic(phalcom_core::value::Value::Obj(base_obj), config_sym, &[]);
    assert!(res.is_ok(), "send_dynamic on module export should succeed: {:?}", res.err());
    let val = res.unwrap();
    assert!(matches!(val, phalcom_core::value::Value::Obj(_)));
}

#[test]
fn test_core_new_execution() {
    let mut vm = VM::new();
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples/core_new.ph")
        .canonicalize()
        .unwrap();
    let selection = EntrySelection::Module(file);
    let program = ProgramCompiler::compile_entry_selection(selection).unwrap();
    let res = vm.run_compiled(&program);
    assert!(res.is_ok(), "run_compiled error: {:?}", res.err());
}
