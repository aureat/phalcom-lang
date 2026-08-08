//! Shared helpers for the language acceptance corpus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

/// Extra CLI flags a golden `.ph` case can request, via a `// flags: <args>`
/// comment line anywhere in the source (U-ANNOT-CONTRACTS stripping goldens
/// need to select `--release`/`--unchecked`, which the default `phalcom
/// <path>` invocation has no other way to express per-file). Returns `None`
/// for the overwhelming majority of cases with no such line, which run
/// exactly as before this helper existed.
fn extra_flags(path: &Path) -> Vec<String> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix("// flags:"))
        .map(|rest| rest.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

fn run(path: &Path) -> Output {
    Command::new(phalcom_bin())
        .args(extra_flags(path))
        .arg(path)
        .output()
        .expect("failed to spawn the `phalcom` binary")
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang")
}

fn case_name(path: &Path) -> String {
    path.strip_prefix(corpus_root()).unwrap_or(path).display().to_string()
}

fn collect_cases(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        panic!("missing corpus directory: {}", dir.display());
    }

    let mut cases = Vec::new();
    for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display())) {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read directory entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("ph") {
            cases.push(path);
        }
    }

    cases.sort();
    cases
}

fn expected_path(case_path: &Path) -> PathBuf {
    case_path.with_extension("expected")
}

fn stdout_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.split_inclusive('\n').collect();
    let actual_lines: Vec<&str> = actual.split_inclusive('\n').collect();
    let max_len = expected_lines.len().max(actual_lines.len());

    let mut diff = String::new();
    for index in 0..max_len {
        match (expected_lines.get(index), actual_lines.get(index)) {
            (Some(exp), Some(act)) if exp == act => {}
            (Some(exp), Some(act)) => {
                diff.push_str(&format!(
                    "first stdout mismatch at line {}\n- expected: {:?}\n+ actual:   {:?}\n",
                    index + 1,
                    exp,
                    act
                ));
                break;
            }
            (Some(exp), None) => {
                diff.push_str(&format!(
                    "stdout ended early at line {}\n- expected: {:?}\n+ actual:   <missing>\n",
                    index + 1,
                    exp
                ));
                break;
            }
            (None, Some(act)) => {
                diff.push_str(&format!(
                    "stdout had an unexpected extra line {}\n- expected: <missing>\n+ actual:   {:?}\n",
                    index + 1,
                    act
                ));
                break;
            }
            (None, None) => break,
        }
    }

    diff
}

fn assert_no_panic(label: &str, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(101), "{label} panicked (exit 101). stderr:\n{stderr}");
    assert!(!stderr.contains("panicked at"), "{label} panicked. stderr:\n{stderr}");
}

fn assert_success(label: &str, output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{label} failed with {}. stdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
}

fn assert_stdout_exact(label: &str, output: &Output, expected: &[u8]) {
    let mut actual = output.stdout.clone();
    let mut expected = expected.to_vec();

    if actual.ends_with(b"\n") {
        actual.pop();
        if actual.ends_with(b"\r") {
            actual.pop();
        }
    }
    if expected.ends_with(b"\n") {
        expected.pop();
        if expected.ends_with(b"\r") {
            expected.pop();
        }
    }

    if actual == expected {
        return;
    }

    let expected_text = String::from_utf8_lossy(&expected);
    let actual_text = String::from_utf8_lossy(&actual);
    panic!(
        "{label} produced unexpected stdout\n{}expected stdout:\n{expected_text}\nactual stdout:\n{actual_text}",
        stdout_diff(&expected_text, &actual_text)
    );
}

fn assert_negative_output(label: &str, output: &Output, expected_note: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        output.status.code() != Some(0),
        "{label} unexpectedly succeeded. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_panic(label, output);
    assert!(
        combined.contains(expected_note),
        "{label} did not mention the expected diagnostic substring `{expected_note}`.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn check_cases(label: &str, pending: bool, negative: bool) {
    let root = corpus_root().join(label);
    let dir = if negative {
        root.clone()
    } else if pending {
        root.join("pending")
    } else {
        root
    };

    for case_path in collect_cases(&dir) {
        let case_label = case_name(&case_path);
        let expected = expected_path(&case_path);
        assert!(expected.exists(), "missing expected sidecar for {}", case_label);

        let output = run(&case_path);
        assert_no_panic(&case_label, &output);

        if negative {
            let note = fs::read_to_string(&expected).unwrap_or_else(|err| panic!("failed to read {}: {err}", expected.display()));
            assert_negative_output(&case_label, &output, note.trim());
        } else {
            assert_success(&case_label, &output);
            let expected_bytes = fs::read(&expected).unwrap_or_else(|err| panic!("failed to read {}: {err}", expected.display()));
            assert_stdout_exact(&case_label, &output, &expected_bytes);
        }
    }
}

/// Runs all active PASS cases in `tests/lang/<label>/`.
pub fn check_pass(label: &str) {
    check_cases(label, false, false);
}

/// Runs all NEGATIVE cases in `tests/lang/<label>/`.
pub fn check_negative(label: &str) {
    check_cases(label, false, true);
}

/// Runs all PENDING cases in `tests/lang/<label>/pending/`.
pub fn check_pending(label: &str) {
    check_cases(label, true, false);
}

/// Disassembles the `for`-loop fixture at `rel_path` (relative to the corpus
/// root) and asserts its taken path is a direct jump loop with no materialized
/// block / `block_call` (C-ITER-4, the U-ITER §7.1 preclusion guard).
///
/// A `for` lowered to `coll.each { … }` would materialize the body as a
/// [`Closure`] and interpose `f.call(_)` (a native frame) — which would raise
/// `CannotYieldAcrossNativeFrame` for a fiber-backed generator. This proves the
/// direct-jump lowering (D-ITER-2): the chunk must contain a `Loop(` back-edge
/// and **no** `Closure(` opcode.
pub fn check_for_no_block_call(rel_path: &str) {
    let path = corpus_root().join(rel_path);
    assert!(path.exists(), "missing disasm fixture: {}", path.display());
    let output = Command::new(phalcom_bin())
        .arg("disasm")
        .arg(&path)
        .output()
        .expect("failed to spawn the `phalcom` binary for disassembly");
    assert_no_panic(rel_path, &output);
    assert!(
        output.status.success(),
        "disasm of {rel_path} failed. stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("Loop("),
        "{rel_path}: expected a `Loop(` back-edge (a direct jump loop) in the disassembly, got:\n{text}"
    );
    assert!(
        !text.contains("Closure("),
        "{rel_path}: the `for` chunk must emit NO `Closure(` (no materialized block / block_call on \
         the taken path, C-ITER-4), but the disassembly contains one:\n{text}"
    );
}

/// Asserts the U-ITERABLE Route B zero-allocation loop properties:
/// - JumpIfNone is present (the end-sentinel identity check)
/// - isSome / unwrapOr / WrapSome are absent (no option reification / extraction sends)
pub fn check_for_zero_alloc_loop(rel_path: &str) {
    let path = corpus_root().join(rel_path);
    assert!(path.exists(), "missing disasm fixture: {}", path.display());
    let output = Command::new(phalcom_bin())
        .arg("disasm")
        .arg(&path)
        .output()
        .expect("failed to spawn the `phalcom` binary for disassembly");
    assert_no_panic(rel_path, &output);
    assert!(
        output.status.success(),
        "disasm of {rel_path} failed. stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("JumpIfNone("),
        "{rel_path}: expected `JumpIfNone(` in the disassembly, got:\n{text}"
    );
    assert!(!text.contains("isSome"), "{rel_path}: expected NO `isSome` in the disassembly, got:\n{text}");
    assert!(
        !text.contains("unwrapOr"),
        "{rel_path}: expected NO `unwrapOr` in the disassembly, got:\n{text}"
    );
    assert!(
        !text.contains("WrapSome"),
        "{rel_path}: expected NO `WrapSome` in the disassembly, got:\n{text}"
    );
}

/// Asserts the disassembly of `rel_path` contains NO `WrapSome` opcode.
pub fn check_for_no_wrapsome(rel_path: &str) {
    let path = corpus_root().join(rel_path);
    assert!(path.exists(), "missing disasm fixture: {}", path.display());
    let output = Command::new(phalcom_bin())
        .arg("disasm")
        .arg(&path)
        .output()
        .expect("failed to spawn the `phalcom` binary for disassembly");
    assert_no_panic(rel_path, &output);
    assert!(
        output.status.success(),
        "disasm of {rel_path} failed. stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        !text.contains("WrapSome"),
        "{rel_path}: expected NO `WrapSome` in the disassembly, got:\n{text}"
    );
}
