use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn run_json_file(label: &str, source: &str) -> Output {
    let path = std::env::temp_dir().join(format!("numeric_diagnostics_{label}_{}.ph", std::process::id()));
    fs::write(&path, source).expect("write numeric traceback fixture");
    let output = Command::new(phalcom_bin())
        .args(["--trace-format", "json", "--color", "never"])
        .arg(&path)
        .output()
        .expect("run phalcom numeric traceback fixture");
    let _ = fs::remove_file(path);
    output
}

fn assert_json_numeric_error(label: &str, source: &str, kind: &str, message: &str) {
    let output = run_json_file(label, source);
    assert!(!output.status.success(), "{label} unexpectedly succeeded");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(r#"{"ev":"traceback""#), "{label} was not JSON:\n{stderr}");
    assert!(
        stderr.contains(&format!(r#""kind":"{kind}""#)),
        "{label} missing error kind `{kind}`:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!(r#""message":"{message}""#)),
        "{label} missing error message `{message}`:\n{stderr}"
    );
    assert!(
        stderr.contains(r#""line":1,"name":"<main>""#),
        "{label} missing source frame location:\n{stderr}"
    );
}

#[test]
fn numeric_errors_have_surface_kind_message_and_source_frame() {
    for (label, source, kind, message) in [
        ("floor_div_zero", "0 ~/ 0\n", "divideByZero", "Division by zero"),
        ("mod_zero", "0 % 0\n", "divideByZero", "Division by zero"),
        ("pow_zero_negative", "0 ** -1\n", "divideByZero", "Division by zero"),
        (
            "floor_div_nonfinite",
            "1e309 ~/ 1\n",
            "nonFiniteNumber",
            "Non-finite number: non-finite operand",
        ),
        (
            "shift_negative",
            "1 << -1\n",
            "invalidShift",
            "Invalid shift count: shift count must be non-negative",
        ),
        (
            "bit_index_negative",
            "1.bitAt(-1)\n",
            "invalidBitIndex",
            "Invalid bit index: bit index must be non-negative",
        ),
        (
            "shift_limit",
            "1 << 8388608\n",
            "numericLimit",
            "Numeric limit exceeded: Left shift exceeds configured bit limit",
        ),
        ("number_new", "Number.new()\n", "abstractClass", "cannot instantiate abstract class Number"),
    ] {
        assert_json_numeric_error(label, source, kind, message);
    }
}

#[test]
fn repl_json_numeric_trace_has_no_fabricated_caret_or_column() {
    let output = Command::new(phalcom_bin())
        .args(["--trace-format", "json", "--color", "never", "-i", "0 ~/ 0"])
        .output()
        .expect("run source-less numeric traceback fixture");
    assert!(!output.status.success(), "source-less numeric expression unexpectedly succeeded");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains(r#""col":"#), "source-less JSON trace fabricated a column:\n{stderr}");
    assert!(!stderr.contains('╰'), "source-less JSON trace fabricated a caret block:\n{stderr}");
}
