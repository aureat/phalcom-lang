use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/language").join(path)
}

fn run(path: &Path) -> Output {
    Command::new(phalcom_bin()).arg(path).output().expect("phalcom subprocess starts")
}

#[test]
fn successful_file_run_preserves_stdout_and_status() {
    let path = fixture("system/system_print_string.ph");
    let expected = std::fs::read(path.with_extension("expected")).expect("stdout expected sidecar");
    let output = run(&path);
    assert!(
        output.status.success(),
        "status: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let mut actual = output.stdout;
    if actual.ends_with(b"\n") {
        actual.pop();
        if actual.ends_with(b"\r") {
            actual.pop();
        }
    }
    assert_eq!(actual, expected);
}

#[test]
fn syntax_failure_preserves_compile_status_and_diagnostic() {
    let path = fixture("syntax-errors/syntax_missing_paren.ph");
    let expected = std::fs::read_to_string(path.with_extension("expected")).expect("syntax expected sidecar");
    let output = run(&path);
    assert_eq!(output.status.code(), Some(65));
    assert!(String::from_utf8_lossy(&output.stderr).contains(expected.trim()));
}

#[test]
fn missing_file_preserves_io_status() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/language/does-not-exist.ph");
    let output = run(&path);
    assert_eq!(output.status.code(), Some(66));
}
