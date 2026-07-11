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
#[ignore = "spec target: lexical"]
fn lexical_pending() {
    support::check_pending("lexical");
}

#[test]
fn arithmetic() {
    support::check_pass("arithmetic");
}

#[test]
fn booleans() {
    // U5: `and`/`or` are lazy sends over a block argument (control-flow.md
    // §2) — graduated from PENDING.
    support::check_pass("booleans");
}

#[test]
fn bindings() {
    support::check_pass("bindings");
}

#[test]
#[ignore = "spec target: bindings"]
fn bindings_pending() {
    support::check_pending("bindings");
}

#[test]
fn messages() {
    support::check_pass("messages");
}

#[test]
#[ignore = "spec target: messages"]
fn messages_pending() {
    support::check_pending("messages");
}

#[test]
fn dispatch() {
    support::check_pass("dispatch");
}

#[test]
#[ignore = "spec target: dispatch"]
fn dispatch_pending() {
    support::check_pending("dispatch");
}

#[test]
fn classes() {
    support::check_pass("classes");
}

#[test]
#[ignore = "spec target: classes"]
fn classes_pending() {
    support::check_pending("classes");
}

#[test]
fn control_flow() {
    support::check_pass("control-flow");
}

#[test]
#[ignore = "spec target: control-flow"]
fn control_flow_pending() {
    support::check_pending("control-flow");
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
fn absence() {
    // U6: absence is `Option` (`Some`/`None`) — the shared `None` singleton,
    // `Some.new(_)` construction, and the `match(some:none:)` eliminator.
    support::check_pass("absence");
}

#[test]
#[ignore = "spec target: absence — prettier printString + Some(x) sugar are U-STD"]
fn absence_pending() {
    support::check_pending("absence");
}

#[test]
fn compile_errors() {
    // U6: compile-time diagnostics — surface `nil` is undefined, `let` requires
    // an initializer and rejects reassignment (ADR-0014), and a literal
    // `Option` condition has no truth value (BD-U6-1 Option A).
    support::check_negative("compile-errors");
}

#[test]
fn metaclass() {
    support::check_pass("metaclass");
}

#[test]
#[ignore = "PENDING: metaclass tower — U2"]
fn metaclass_pending() {
    support::check_pending("metaclass");
}

#[test]
fn blocks() {
    support::check_pass("blocks");
}

#[test]
#[ignore = "spec target: blocks"]
fn blocks_pending() {
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
fn system() {
    support::check_pass("system");
}

#[test]
#[ignore = "PENDING: System/IO — later"]
fn system_pending() {
    support::check_pending("system");
}

#[test]
#[ignore = "spec target: concurrency"]
fn concurrency_pending() {
    support::check_pending("concurrency");
}

#[test]
fn list() {
    // U-LIST: kernel `List` — native array storage, `.ph`-defined
    // at(_:)/size/add(_:)/each(_:) protocol over the floor primitives.
    support::check_pass("list");
}
