//! Language acceptance corpus.
//!
//! One `#[test]` per feature label so the suite can be filtered by label with
//! `cargo test -p phalcom-core --test lang <label>`.

mod support;

#[test]
fn lexical() {
    support::check_pass("lexical");
}

#[test]
fn arithmetic() {
    support::check_pass("arithmetic");
}

#[test]
#[ignore = "PENDING: boolean short-circuit semantics"]
fn booleans() {
    support::check_pending("booleans");
}

#[test]
fn bindings() {
    support::check_pass("bindings");
}

#[test]
fn messages() {
    support::check_pass("messages");
}

#[test]
fn dispatch() {
    support::check_pass("dispatch");
}

#[test]
fn classes() {
    support::check_pass("classes");
}

#[test]
fn control_flow() {
    support::check_pass("control-flow");
}

#[test]
fn syntax_errors() {
    support::check_negative("syntax-errors");
}

#[test]
fn runtime_errors() {
    support::check_negative("runtime-errors");
}

#[test]
#[ignore = "PENDING: absence/Option — U6"]
fn absence() {
    support::check_pending("absence");
}

#[test]
#[ignore = "PENDING: metaclass tower — U2"]
fn metaclass() {
    support::check_pending("metaclass");
}

#[test]
#[ignore = "PENDING: blocks/closures — U4"]
fn blocks() {
    support::check_pending("blocks");
}

#[test]
#[ignore = "PENDING: functions — later"]
fn functions() {
    support::check_pending("functions");
}

#[test]
#[ignore = "PENDING: errors/Result — later"]
fn errors() {
    support::check_pending("errors");
}

#[test]
#[ignore = "PENDING: System/IO — later"]
fn system() {
    support::check_pending("system");
}
