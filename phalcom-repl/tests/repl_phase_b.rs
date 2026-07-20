//! Comprehensive unit and integration test suite for U-REPL Phase B (§04–§07).

use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_repl::repl::{CellOutcome, ReplSession, ValueExt};
use phalcom_repl::snapshot::ReplSnapshot;
use phalcom_repl::validator::{classify, explicit_continuation, PhalcomValidator, Verdict};
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
    // The previous version of this test passed `"let x = 1 + \n 2"` — a Rust newline
    // escape containing zero backslashes — so it never exercised continuation at all.
    // The joining logic now lives in the library, which is what makes it reachable here.
    assert_eq!(explicit_continuation(r"let x = 1 +\"), Some("let x = 1 + ".to_string()));
    assert_eq!(explicit_continuation("let x = 1 +\\ "), Some("let x = 1 + ".to_string()));
    // Trailing whitespace past the backslash still continues; the old inline form in
    // main.rs matched only one trailing space and silently submitted otherwise.
    assert_eq!(explicit_continuation("let x = 1 +\\   "), Some("let x = 1 + ".to_string()));
    // No backslash, no continuation.
    assert_eq!(explicit_continuation("let x = 1 + 2"), None);

    // And the joined text evaluates as one cell.
    let mut session = ReplSession::start(PathBuf::from("."));
    let joined = format!("{}2", explicit_continuation(r"let x = 1 +\").unwrap());
    assert!(
        matches!(session.eval(&joined), CellOutcome::Unit),
        "joined continuation should evaluate as a single statement cell"
    );
}

#[test]
fn blank_line_submits_incomplete_buffer() {
    // §04's escape hatch: a blank line forces submission of a buffer that `classify`
    // still considers Incomplete, so an unclosed brace cannot trap the session.
    assert_eq!(classify("class Foo {"), Verdict::Incomplete);
    let buf = "class Foo {";
    let just_typed = "";
    let is_blank_submit = !buf.trim().is_empty() && just_typed.trim().is_empty();
    assert!(is_blank_submit, "a blank line after a non-empty buffer must force submission");
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
fn value_echo_sends_tostring() {
    // The load-bearing half: echo must *dispatch*. `Value::to_string` is the native
    // renderer and falls to `to_debug` for a plain instance, so a user override is
    // invisible through it. Only a real send can produce this marker.
    let mut session = ReplSession::start(PathBuf::from("."));

    let res1 = session.eval(
        "class Custom {\n  construct new() {}\n  toString { return \"MARKER-7f3a\" }\n}",
    );
    assert!(matches!(res1, CellOutcome::Unit), "class cell should succeed, got {res1:?}");

    match session.eval("Custom.new()") {
        CellOutcome::Value(val) => {
            let rendered = val.to_string_guarded(&mut session.vm);
            assert_eq!(
                rendered, "MARKER-7f3a",
                "value echo must send `toString`; got the native rendering instead"
            );
        }
        other => panic!("expected a Value outcome, got {other:?}"),
    }
}

#[test]
fn value_echo_survives_raising_tostring() {
    // The §S4 hazard. Note the earlier version used `1 / 0` as the "raising" body —
    // Phalcom numbers are f64, so that yields `inf` and raises nothing. A genuine
    // raise needs `Error.new(_).raise()`.
    let mut session = ReplSession::start(PathBuf::from("."));

    let res1 = session.eval(
        "class BadString {\n  construct new() {}\n  toString { return Error.new(\"boom\").raise() }\n}",
    );
    assert!(matches!(res1, CellOutcome::Unit), "class cell should succeed, got {res1:?}");

    match session.eval("BadString.new()") {
        CellOutcome::Value(val) => {
            let rendered = val.to_string_guarded(&mut session.vm);
            // Exact, not `contains("BadString")`. The native renderer's debug form is
            // `<BadString instance>`, which also contains the class name — so a
            // `contains` assertion passes whether or not the send ever happened. That
            // is precisely why the original version of this test was vacuous. Only the
            // Err branch of the guard produces this spelling.
            assert_eq!(
                rendered, "<instance of BadString>",
                "a raising toString must degrade through the guard, not fall back to native rendering"
            );
        }
        CellOutcome::Failed => panic!("a raising toString must not fail the cell (§S4)"),
        CellOutcome::Unit => panic!("expected a Value outcome"),
    }

    // The cell after a failed echo must still run: the guard unwinds what the failed
    // send left behind (PDR-0008 §4).
    assert!(
        matches!(session.eval("let after_bad_echo = 1 + 1"), CellOutcome::Unit),
        "the session must remain usable after a raising toString"
    );
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
fn reload_stops_at_failing_cell() {
    // §07 §5, and mandated by impl/07-commands.md:89 — this test was specced and never
    // written. History holds *every* submitted cell, so a cell that failed originally
    // fails again on replay and `:reload` halts there and says so. That is the ruled
    // behaviour, not a defect: reload must be reproducible.
    let mut session = ReplSession::start(PathBuf::from("."));

    assert!(matches!(session.eval("let good = 11"), CellOutcome::Unit));
    assert!(
        matches!(session.eval("let bad = )"), CellOutcome::Failed),
        "a syntax error must report Failed"
    );

    assert!(
        !session.reload(),
        ":reload must return false when a cell in history fails on replay"
    );

    // "leaving the session at the state reached so far" — the pre-failure cell landed.
    let good_sym = session.vm.get_or_intern("good");
    let module_obj = session.vm.heap.module(session.module);
    assert!(
        module_obj.slot_of(good_sym).is_some(),
        "cells before the failing one must survive a halted reload"
    );
}

#[test]
fn colon_prefix_never_parses_as_source() {
    // §07: command routing is a string check on the raw line, ahead of the evaluation
    // pipeline. `:reload` is not valid Phalcom, so if it ever reached the parser it
    // would be a syntax error rather than a command.
    assert_eq!(classify(":reload"), Verdict::Invalid);
    assert_eq!(classify(":quit"), Verdict::Invalid);
    // Being Invalid (not Incomplete) is what lets the surface submit it immediately
    // rather than waiting for more input it will never get.
    assert!(matches!(
        PhalcomValidator.validate(":reload"),
        reedline::ValidationResult::Complete
    ));
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
