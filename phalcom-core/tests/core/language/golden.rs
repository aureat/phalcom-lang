//! Golden `.ph` corpus runner.
//!
//! Executes known-good Phalcom programs through the real `phalcom` CLI
//! binary (as a subprocess, exercising the full lex/parse/compile/run
//! pipeline exactly as a user would) and asserts:
//!   1. the process does not panic (no exit-101 / "panicked at" on stderr),
//!   2. stdout matches a fixed, hand-verified string.
//!
//! This is a *regression* gate, not a behavior spec: it only proves "this
//! program's output hasn't silently changed", not "this program does what
//! the spec says it should".
//!
//! ## Why only four of `examples/*.ph` are here
//!
//! `examples/core_new.ph`, `examples/person2.ph`, `examples/person.ph`, and
//! `examples/calculator.ph` run to completion without panicking. The remaining
//! examples (`person3.ph`, `simple.ph`) use syntax the current grammar does not
//! yet accept (labeled/named constructor params, `@construct` decorators, etc.)
//! and stay excluded until that syntax lands.
//!
//! Historically *all* real `.ph` files also tripped a second defect: the
//! grammar rejected a *trailing newline* at end-of-input (F10), and hitting
//! *any* parse error panicked unconditionally because `SyntaxError`'s `Display`
//! impl was `todo!()` (F9). U0 fixed both — a trailing newline now parses and a
//! parse error renders a diagnostic and exits non-zero — which is what
//! unblocked `person.ph`/`calculator.ph` here.
//!
//! The two extra `tests/fixtures/golden/*.ph` fixtures give a little more
//! coverage (string/number printing, arithmetic, `let` bindings). They predate
//! the F10 fix and deliberately have **no trailing newline**; that is no longer
//! required, but they are left as-is since their goldens are already pinned.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn run(path: &Path) -> Output {
    Command::new(phalcom_bin()).arg(path).output().expect("failed to spawn the `phalcom` binary")
}

/// Runs `path` (resolved relative to the `phalcom-core` crate root) and
/// asserts it doesn't panic and its stdout equals `expected_stdout` exactly.
fn assert_golden(path: &str, expected_stdout: &str) {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    assert!(full.exists(), "golden fixture missing: {}", full.display());

    let output = run(&full);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.code() != Some(101), "{path} panicked (exit 101, Rust panic). stderr:\n{stderr}");
    assert!(!stderr.contains("panicked at"), "{path} panicked. stderr:\n{stderr}");
    assert_eq!(stdout, expected_stdout, "{path} produced unexpected stdout");
}

#[test]
fn example_core_new() {
    // `System.new()` is disallowed. `System` prints as `System` — the class's
    // own name via `Object#toString`'s class-receiver case (ADR-0015), which
    // `System.print` now agrees with (U-ERR-FIX PRINT-TOSTRING routes the
    // print path through a `toString` send for any object with no bespoke
    // native renderer).
    assert_golden("../examples/core_new.ph", "System\n");
}

#[test]
fn example_person2() {
    // `Person.new()` (zero-arg static ctor) -> `.init(name)` never called ->
    // `_name` field is unset -> `name` getter reads back the surface `None`
    // value (U6: surface `nil` is gone; an unset field surfaces as `None` via
    // the private-sentinel boundary — ADR-0007/ADR-0010). `None` prints as
    // `None` (U-CORE-4's `Option#toString` display override).
    assert_golden("../examples/person2.ph", "None\n");
}

#[test]
fn example_person() {
    // The current example constructs a named person and exercises its
    // accessors plus the custom `toString` presentation.
    assert_golden("../examples/person.ph", "Person(name: Bob, age: 30)\n");
}

#[test]
fn example_calculator() {
    // Unblocked by the U0 trailing-newline fix (the file ends in `\n`).
    // Exercises arithmetic method calls and number/float printing.
    assert_golden("../examples/calculator.ph", "8\n6\n30\n3.1415\n");
}

#[test]
fn fixture_hello_world() {
    assert_golden("tests/fixtures/golden/hello.ph", "hello, world\n");
}

#[test]
fn fixture_arithmetic() {
    assert_golden("tests/fixtures/golden/arithmetic.ph", "7\n30\nab\n");
}

#[test]
fn fixture_blocks_map_reduce() {
    // A map-style unary block and a reduce-style binary block, both invoked
    // via `call` (functions.md §1-2).
    assert_golden("tests/fixtures/golden/blocks_map_reduce.ph", "25\n7\n");
}

#[test]
fn fixture_blocks_escaping_counter() {
    // The classic escaping-closure case: `count` is captured as an open
    // upvalue, then promoted to a heap-owned closed cell when `makeCounter`'s
    // frame returns, so the counter keeps incrementing correctly afterward
    // (ADR-0013).
    assert_golden("tests/fixtures/golden/blocks_escaping_counter.ph", "1\n2\n3\n");
}
