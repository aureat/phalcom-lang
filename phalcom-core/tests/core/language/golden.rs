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
//! The checked-in `tests/fixtures/golden/*.ph` fixtures give coverage for
//! string/number printing, arithmetic, `let` bindings, and escaping closures.

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
