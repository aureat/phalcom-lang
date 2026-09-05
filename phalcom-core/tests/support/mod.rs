//! Shared helpers for the language acceptance corpus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use phalcom_core::compiler::attributes::CompileMode;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::vm::{BufferedOutput, VM};

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

#[derive(Debug, Clone, Copy)]
struct CorpusCaseOptions {
    compile_mode: CompileMode,
    strip_contract_metadata: bool,
}

fn case_options(path: &Path) -> CorpusCaseOptions {
    let mut options = CorpusCaseOptions {
        compile_mode: CompileMode::Debug,
        strip_contract_metadata: false,
    };
    for flag in extra_flags(path) {
        match flag.as_str() {
            "--release" => options.compile_mode = CompileMode::Release,
            "--unchecked" => options.compile_mode = CompileMode::Unchecked,
            "--strip-contract-metadata" => options.strip_contract_metadata = true,
            _ => {}
        }
    }
    options
}

#[derive(Debug)]
enum CorpusOutcome {
    Success,
    CompileFailure(ProgramCompileError),
    RuntimeFailure(PhError),
}

#[derive(Debug)]
struct CorpusRun {
    stdout: Vec<u8>,
    outcome: CorpusOutcome,
}

fn run_corpus_case(path: &Path) -> CorpusRun {
    let options = case_options(path);
    let program = match ProgramCompiler::compile_entry_selection(EntrySelection::Module(path.to_path_buf())) {
        Ok(program) => program,
        Err(error) => {
            return CorpusRun {
                stdout: Vec::new(),
                outcome: CorpusOutcome::CompileFailure(error),
            };
        }
    };

    let sink = BufferedOutput::new();
    let handle = sink.handle();
    let mut vm = VM::new_with_output(Box::new(sink));
    vm.compile_mode = options.compile_mode;
    vm.strip_contract_metadata = options.strip_contract_metadata;
    let outcome = match vm.run_compiled(&program) {
        Ok(()) => CorpusOutcome::Success,
        Err(error) => CorpusOutcome::RuntimeFailure(error),
    };

    CorpusRun {
        stdout: handle.bytes(),
        outcome,
    }
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/language")
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

fn assert_success(label: &str, run: &CorpusRun) {
    if matches!(&run.outcome, CorpusOutcome::Success) {
        return;
    }
    panic!("{label} failed: {}", outcome_summary(&run.outcome));
}

fn outcome_summary(outcome: &CorpusOutcome) -> String {
    match outcome {
        CorpusOutcome::Success => "success".to_string(),
        CorpusOutcome::CompileFailure(ProgramCompileError::Semantic(diagnostics)) => {
            let details = diagnostics
                .iter()
                .flat_map(|(_, diagnostics)| diagnostics.iter())
                .take(8)
                .map(|diagnostic| format!("[{:?}] {}", diagnostic.code, diagnostic.message))
                .collect::<Vec<_>>();
            if details.is_empty() {
                "semantic compile failure without diagnostics".to_string()
            } else {
                format!("semantic compile failure: {}", details.join("; "))
            }
        }
        CorpusOutcome::CompileFailure(error) => format!("compile failure: {error}"),
        CorpusOutcome::RuntimeFailure(error) => format!("runtime failure: {error}"),
    }
}

fn assert_stdout_exact(label: &str, output: &[u8], expected: &[u8]) {
    let mut actual = output.to_vec();
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

fn assert_negative_output(label: &str, run: &CorpusRun, expected_note: &str) {
    let diagnostic = match &run.outcome {
        CorpusOutcome::Success => panic!("{label} unexpectedly succeeded. stdout:\n{}", String::from_utf8_lossy(&run.stdout)),
        outcome => outcome_summary(outcome),
    };
    assert!(
        diagnostic.contains(expected_note),
        "{label} did not mention the expected diagnostic substring `{expected_note}`.\nphase failure:\n{diagnostic}"
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

    let cases = collect_cases(&dir);
    if pending {
        assert!(!cases.is_empty(), "no pending cases in {}", dir.display());
    }

    for case_path in cases {
        let case_label = case_name(&case_path);
        let expected = expected_path(&case_path);
        assert!(expected.exists(), "missing expected sidecar for {}", case_label);

        let output = run_corpus_case(&case_path);

        if negative {
            let note = fs::read_to_string(&expected).unwrap_or_else(|err| panic!("failed to read {}: {err}", expected.display()));
            assert_negative_output(&case_label, &output, note.trim());
        } else {
            assert_success(&case_label, &output);
            let expected_bytes = fs::read(&expected).unwrap_or_else(|err| panic!("failed to read {}: {err}", expected.display()));
            assert_stdout_exact(&case_label, &output.stdout, &expected_bytes);
        }
    }
}

/// Runs all active PASS cases in `tests/fixtures/language/<label>/`.
pub fn check_pass(label: &str) {
    check_cases(label, false, false);
}

/// Runs all NEGATIVE cases in `tests/fixtures/language/<label>/`.
pub fn check_negative(label: &str) {
    check_cases(label, false, true);
}

/// Runs all PENDING cases in `tests/fixtures/language/<label>/pending/`.
pub fn check_pending(label: &str) {
    check_cases(label, true, false);
}

/// Runs one named PENDING case without opening the rest of a deferred lane.
/// This keeps an independently complete migration gate visible while unrelated
/// pending cases remain intentionally deferred.
pub fn check_pending_case(label: &str, case: &str) {
    let path = corpus_root().join(label).join("pending").join(format!("{case}.ph"));
    assert!(path.exists(), "missing pending corpus case: {}", path.display());
    let expected = expected_path(&path);
    assert!(expected.exists(), "missing expected sidecar for {}", path.display());

    let output = run_corpus_case(&path);
    let case_label = case_name(&path);
    assert_success(&case_label, &output);
    let expected_bytes = fs::read(&expected).unwrap_or_else(|err| panic!("failed to read {}: {err}", expected.display()));
    assert_stdout_exact(&case_label, &output.stdout, &expected_bytes);
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
/// - JumpIfNone is present (the direct immediate-None end check)
/// - isSome / unwrapOr / WrapSome are absent (no option extraction or re-wrapping sends)
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
