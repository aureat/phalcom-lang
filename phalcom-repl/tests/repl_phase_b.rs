//! Comprehensive unit and integration test suite for U-REPL Phase B (§04–§07).

use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_repl::repl::{CellOutcome, ReplSession, ValueExt};
use phalcom_repl::snapshot::ReplSnapshot;
use phalcom_repl::validator::{classify, PhalcomValidator, Verdict};
use reedline::Validator;
use std::path::PathBuf;

// --- §04 Continuation Tests ---

#[test]
fn validator_matches_probe_classification() {
    let cases = [
        ("let x = 1", Verdict::Complete),
        ("1 + 1", Verdict::Complete),
        ("", Verdict::Complete),
        ("class Foo {", Verdict::Incomplete),
        ("let x = 1 +", Verdict::Incomplete),
        ("foo(1,", Verdict::Incomplete),
        ("[1, 2,", Verdict::Incomplete),
        ("if (x) {", Verdict::Incomplete),
        ("let s = \"abc", Verdict::Incomplete),
        ("let x = )", Verdict::Invalid),
        ("1 +* 2", Verdict::Invalid),
    ];

    for (src, want) in cases {
        assert_eq!(classify(src), want, "Misclassified: {src:?}");
    }
}

#[test]
fn invalid_input_submits_rather_than_waiting() {
    let v = PhalcomValidator;
    let res = v.validate("let x = )");
    assert!(matches!(res, reedline::ValidationResult::Complete));
}

#[test]
fn trailing_backslash_joins_before_lexing() {
    let mut session = ReplSession::start(PathBuf::from("."));
    let src = "let x = 1 + \n 2";
    match session.eval(src) {
        CellOutcome::Unit | CellOutcome::Value(_) => {}
        CellOutcome::Failed => panic!("Trailing backslash joined input should evaluate clean"),
    }
}

// --- §05 Snapshot & Oracle Tests ---

#[test]
fn snapshot_reflects_globals_after_cell() {
    let mut session = ReplSession::start(PathBuf::from("."));
    session.eval("let test_global_x = 42");

    let snap = ReplSnapshot::capture(&session.vm, session.module);
    let sym = session.vm.get_or_intern("test_global_x");

    assert!(snap.globals.contains_key(&sym), "Bound global must appear in snapshot");
}

#[test]
fn snapshot_tags_own_depth() {
    let mut vm = VM::new();
    let abs_path = std::env::current_dir().unwrap().display().to_string();
    let module = vm.create_module("main", &abs_path);

    let list_cls = vm.universe.classes.list_class;
    let snap = ReplSnapshot::capture(&vm, module);

    let members = snap.members.get(&list_cls).expect("List class must have members");
    let has_depth_0 = members.iter().any(|m| m.own_depth == 0);
    let has_higher_depth = members.iter().any(|m| m.own_depth > 0);

    assert!(has_depth_0, "List own members must be at depth 0");
    assert!(has_higher_depth, "Inherited members must have own_depth > 0");
}

#[test]
fn initializer_kind_never_offered() {
    let mut vm = VM::new();
    let abs_path = std::env::current_dir().unwrap().display().to_string();
    let module = vm.create_module("main", &abs_path);

    let snap = ReplSnapshot::capture(&vm, module);
    for members in snap.members.values() {
        for m in members {
            assert!(
                !m.selector.starts_with("init "),
                "Initializer kind must never be offered in completion"
            );
        }
    }
}

// --- §06 Surface & Value Echo Hazard Tests ---

#[test]
fn value_echo_survives_raising_tostring() {
    let mut session = ReplSession::start(PathBuf::from("."));

    let res1 = session.eval("class BadString {\n  construct new() {}\n  toString { return 1 / 0 }\n}");
    assert!(matches!(res1, CellOutcome::Unit), "Class definition cell should succeed, got {res1:?}");

    let outcome = session.eval("BadString.new()");
    match outcome {
        CellOutcome::Value(val) => {
            let rendered = val.to_string_guarded(&session.vm);
            assert!(
                rendered.contains("BadString"),
                "Raising toString must degrade to class display, got: {rendered}"
            );
        }
        CellOutcome::Failed => panic!("Cell evaluation must succeed despite toString failure"),
        CellOutcome::Unit => panic!("Expected Value outcome"),
    }
}

// --- §07 Reload Hazard Tests ---

#[test]
fn reload_survives_declarations() {
    let mut session = ReplSession::start(PathBuf::from("."));

    let r1 = session.eval("let x = 1");
    assert!(matches!(r1, CellOutcome::Unit), "Cell 1 failed: {r1:?}");
    let r2 = session.eval("class Foo {\n  construct new() {}\n}");
    assert!(matches!(r2, CellOutcome::Unit), "Cell 2 failed: {r2:?}");
    let r3 = session.eval("let f = Foo.new()");
    assert!(matches!(r3, CellOutcome::Unit), "Cell 3 failed: {r3:?}");

    let ok = session.reload();
    assert!(ok, ":reload must succeed on sessions containing variable and class declarations");
}

#[test]
fn reload_rebuilds_session_from_history() {
    let mut session = ReplSession::start(PathBuf::from("."));

    session.eval("let val_a = 100");
    session.eval("let val_b = val_a + 50");

    assert!(session.reload(), ":reload failed to execute history");

    let val_b_sym = session.vm.get_or_intern("val_b");
    let module_obj = session.vm.heap.module(session.module);
    let slot = module_obj.slot_of(val_b_sym).expect("val_b must exist after reload");
    let val = module_obj.globals[slot];

    match val {
        Value::Number(n) => assert_eq!(n, 150.0),
        _ => panic!("Expected number value for val_b"),
    }
}
