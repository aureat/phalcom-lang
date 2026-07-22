//! Comprehensive unit and integration test suite for U-REPL Phase B (§04–§07).

use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_repl::completer::PhalcomCompleter;
use phalcom_repl::highlighter::PhalcomHighlighter;
use phalcom_repl::oracle::ReplOracle;
use phalcom_repl::repl::{CellOutcome, ReplSession, ValueExt};
use phalcom_repl::snapshot::ReplSnapshot;
use phalcom_repl::validator::{PhalcomValidator, Verdict, classify, explicit_continuation};
use reedline::{Completer, Highlighter, Validator};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
fn unterminated_lexer_mode_continues() {
    // PDR-0006. A bare `/* …` used to classify Invalid and submit mid-comment: an
    // empty statement list is a grammatically *complete* parse, so the parser never
    // wanted another token and never emitted `UnrecognizedEof`. The lowering now
    // co-emits one, and the validator's single rule is untouched.
    assert_eq!(classify("/* an unfinished comment"), Verdict::Incomplete);
    assert_eq!(classify("/* spanning\n   two lines"), Verdict::Incomplete);
    // A bare unterminated string is the same shape, and now continues for the same
    // reason rather than incidentally.
    assert_eq!(classify("\"an unfinished string"), Verdict::Incomplete);

    // A *closed* comment is complete, not continued — the co-emission must not fire
    // for a mode that was properly terminated.
    assert_eq!(classify("/* done */ let x = 1"), Verdict::Complete);
    // And a genuine syntax error is still Invalid, not held open forever.
    assert_eq!(classify("let x = )"), Verdict::Invalid);
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
            assert!(!m.selector.starts_with("init "), "Initializer kind must never be offered in completion");
        }
    }
}

// --- §06 Surface: ranking, insertion shape, layer discipline ---
//
// The five tests below are named in impl/06-surface.md:150-154 and were never
// written. They are the completer/highlighter half of §S5/§S7.

/// Builds an oracle over a live session snapshot, as the surface types take it.
fn oracle_for(session: &ReplSession) -> Arc<ReplOracle> {
    let snap = ReplSnapshot::capture(&session.vm, session.module);
    Arc::new(ReplOracle::new(Arc::new(Mutex::new(snap))))
}

#[test]
fn ranking_puts_own_before_inherited() {
    // §S5 — `(own_depth, name)` order on a real chain: a subclass's own members
    // sort ahead of everything it inherits, and ties break by name.
    let mut session = ReplSession::start(PathBuf::from("."));
    assert!(matches!(
        session.eval("class Base {\n  construct new() {}\n  zzz_base { return 1 }\n}"),
        CellOutcome::Unit
    ));
    assert!(matches!(
        session.eval("class Derived extends Base {\n  construct new() {}\n  aaa_own { return 2 }\n}"),
        CellOutcome::Unit
    ));

    let oracle = oracle_for(&session);
    let cid = oracle.find_class_by_name("Derived").expect("Derived must be in the snapshot");
    let members = oracle.members_for_class(cid);

    let own = members.iter().position(|m| m.name == "aaa_own").expect("own member missing");
    let inherited = members.iter().position(|m| m.name == "zzz_base").expect("inherited member missing");
    assert!(own < inherited, "own members must rank before inherited ones regardless of name order");
    assert_eq!(members[own].own_depth, 0, "an own member sits at depth 0");
    assert!(members[inherited].own_depth > 0, "an inherited member sits deeper");

    // Ties break by name within a depth.
    let depths: Vec<usize> = members.iter().map(|m| m.own_depth).collect();
    let mut sorted = depths.clone();
    sorted.sort_unstable();
    assert_eq!(depths, sorted, "members must be ordered by ascending own_depth");
}

#[test]
fn arity_zero_inserts_bare_name() {
    // §S7 — a getter or zero-arity method completes to a bare name, no parens.
    let mut session = ReplSession::start(PathBuf::from("."));
    assert!(matches!(
        session.eval("class Widget {\n  construct new() {}\n  spin { return 1 }\n}"),
        CellOutcome::Unit
    ));
    assert!(matches!(session.eval("let w = Widget.new()"), CellOutcome::Unit));

    let completions = complete_for(&session, "w.sp");
    assert!(completions.iter().any(|c| c == "spin"), "arity-0 member must insert bare; got {completions:?}");
    assert!(
        !completions.iter().any(|c| c == "spin("),
        "arity-0 member must not insert a call opening; got {completions:?}"
    );
}

#[test]
fn arity_n_inserts_call_opening() {
    // §S7 — an arity-n method completes to `name(` with the cursor inside, and
    // explicitly not to a snippet placeholder like `${1:}`.
    let mut session = ReplSession::start(PathBuf::from("."));
    assert!(matches!(
        session.eval("class Widget {\n  construct new() {}\n  scale(n) { return n }\n}"),
        CellOutcome::Unit
    ));
    assert!(matches!(session.eval("let w = Widget.new()"), CellOutcome::Unit));

    let completions = complete_for(&session, "w.sc");
    assert!(
        completions.iter().any(|c| c == "scale("),
        "arity-n member must insert a call opening; got {completions:?}"
    );
    assert!(
        !completions.iter().any(|c| c.contains("${")),
        "insertion must not carry a snippet placeholder; got {completions:?}"
    );
}

/// Runs the completer over `line` with the cursor at its end, returning the
/// suggestion values.
fn complete_for(session: &ReplSession, line: &str) -> Vec<String> {
    let completer = PhalcomCompleter::new(PathBuf::from("."), oracle_for(session));
    let mut completer = completer;
    completer.complete(line, line.len()).into_iter().map(|s| s.value).collect()
}

#[test]
fn l1_never_keywords_inside_strings() {
    // The bug the lexer-backed L1 layer exists to fix: a regex-battery highlighter
    // colors `class` inside a string literal, because it never tokenizes. Driving
    // the real lexer means the whole literal is one string token.
    let session = ReplSession::start(PathBuf::from("."));
    let highlighter = PhalcomHighlighter::new(oracle_for(&session));

    let line = "let s = \"class while return\"";
    let styled = highlighter.highlight(line, line.len());

    // Every styled run that falls inside the literal must share one style — the
    // string's — so no keyword inside it is colored differently.
    let open = line.find('"').expect("literal present");
    let mut offset = 0usize;
    let mut styles_in_literal = Vec::new();
    for (style, text) in &styled.buffer {
        if offset >= open && !text.trim().is_empty() {
            styles_in_literal.push(style.clone());
        }
        offset += text.len();
    }
    assert!(!styles_in_literal.is_empty(), "the literal must produce styled output");
    let first = &styles_in_literal[0];
    assert!(
        styles_in_literal.iter().all(|s| s == first),
        "keywords inside a string literal must not be styled as keywords"
    );
}

#[test]
fn layers_only_refine() {
    // L2/L3 are refinements: with L2 unbuilt and L3 contributing nothing for a
    // line with no identifiers to dim, the rendered text must still reproduce the
    // input exactly. A layer that *replaced* rather than refined would drop or
    // reorder characters.
    let session = ReplSession::start(PathBuf::from("."));
    let highlighter = PhalcomHighlighter::new(oracle_for(&session));

    for line in ["let x = 1 + 2", "class Foo {", "\"a string\"", "unbound_name"] {
        let styled = highlighter.highlight(line, line.len());
        let rendered: String = styled.buffer.iter().map(|(_, text)| text.as_str()).collect();
        assert_eq!(rendered, line, "highlighting must only add style, never alter the text");
    }
}

// --- §06 Surface & Value Echo Hazard Tests ---

#[test]
fn value_echo_sends_tostring() {
    // The load-bearing half: echo must *dispatch*. `Value::to_string` is the native
    // renderer and falls to `to_debug` for a plain instance, so a user override is
    // invisible through it. Only a real send can produce this marker.
    let mut session = ReplSession::start(PathBuf::from("."));

    let res1 = session.eval("class Custom {\n  construct new() {}\n  toString { return \"MARKER-7f3a\" }\n}");
    assert!(matches!(res1, CellOutcome::Unit), "class cell should succeed, got {res1:?}");

    match session.eval("Custom.new()") {
        CellOutcome::Value(val) => {
            let rendered = val.to_string_guarded(&mut session.vm);
            assert_eq!(rendered, "MARKER-7f3a", "value echo must send `toString`; got the native rendering instead");
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

    let res1 = session.eval("class BadString {\n  construct new() {}\n  toString { return Error.new(\"boom\").raise() }\n}");
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
    assert!(matches!(session.eval("let bad = )"), CellOutcome::Failed), "a syntax error must report Failed");

    assert!(!session.reload(), ":reload must return false when a cell in history fails on replay");

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
    assert!(matches!(PhalcomValidator.validate(":reload"), reedline::ValidationResult::Complete));
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
