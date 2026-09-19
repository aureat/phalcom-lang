use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::vm::{BufferedOutput, VM};
use phalcom_semantic::DiagnosticCode;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_logical_imports_execute_without_physical_paths() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/logical_imports");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("logical imports should compile and link");

    let sink = BufferedOutput::new();
    let handle = sink.handle();
    let mut vm = VM::new_native_with_output(Box::new(sink));
    vm.run_compiled(&program).expect("logical imports should execute");

    assert_eq!(handle.bytes(), b"42\n3\n4\n3\n<True>\n42\n1\n99\n3\n");
}

#[test]
fn time_p1_ordinary_package_uses_grouped_public_surface_and_temporal_prelude() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/time_p1");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("time_p1 should compile through the ordinary project linker");
    assert!(
        program.modules.keys().all(|id| !id.project.is_universe()),
        "ordinary package must not own canonical Universe runtime modules"
    );

    let sink = BufferedOutput::new();
    let handle = sink.handle();
    let mut vm = VM::new_with_output(Box::new(sink));
    vm.run_compiled(&program).expect("time_p1 should execute");

    assert_eq!(handle.bytes(), b"time_p1 ok\n");
}

#[test]
fn time_p1_ordinary_package_keeps_non_prelude_temporal_names_explicit() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/time_p1");
    let selection = EntrySelection::Project(fixture_dir);
    let program = ProgramCompiler::compile_entry_selection(selection).expect("time_p1 should compile through the ordinary project linker");

    let mut vm = VM::new();
    vm.run_compiled(&program).expect("time_p1 should initialize");
    for name in [
        "Clock",
        "SystemClock",
        "Instant",
        "clock",
        "WallClock",
        "SystemWallClock",
        "wallClock",
        "Date",
        "Time",
        "DateTime",
        "Period",
        "UtcOffset",
    ] {
        assert!(
            !vm.prelude_bindings.contains_key(&vm.interner.intern(name)),
            "{name} must not be an implicit prelude binding"
        );
    }
}

#[test]
fn time_p1_ordinary_package_cannot_cross_instant_or_clock_visibility_boundaries() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules_v1/time_p1_access");
    let selection = EntrySelection::Project(fixture_dir);
    let error = ProgramCompiler::compile_entry_selection(selection).expect_err("ordinary package must not access internal/protected time members");
    let ProgramCompileError::Semantic(diagnostics) = error else {
        panic!("expected semantic visibility diagnostics, got {error:?}");
    };
    let codes = diagnostics.by_module.values().flatten().map(|diagnostic| diagnostic.code).collect::<Vec<_>>();
    assert!(
        codes.iter().filter(|code| **code == DiagnosticCode::MemberInaccessible).count() >= 2,
        "expected both visibility violations: {codes:?}"
    );
}

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

    // Attempting to re-initialize program reproduces sticky failure without re-running
    let res2 = vm.initialize_program(&program);
    assert!(res2.is_err());
    let err2 = res2.unwrap_err();
    match err2 {
        PhError::ModuleInitialization(init_err) => {
            let rendered = format!("{init_err}");
            assert!(rendered.contains("ModuleInitializationError:"));
            assert!(rendered.contains("Caused by error in module"));
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
    let res = vm.send_dynamic(phalcom_core::value::Value::obj(base_obj), config_sym, &[]);
    assert!(res.is_ok(), "send_dynamic on module export should succeed: {:?}", res.err());
    let val = res.unwrap();
    assert!(val.as_obj().is_some());
}
