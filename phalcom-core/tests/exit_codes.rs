use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn run_bin(args: &[&str]) -> Output {
    Command::new(phalcom_bin())
        .args(args)
        .env_remove("RUST_LOG")
        .env_remove("RUST_LOG_STYLE")
        .output()
        .expect("failed to spawn the `phalcom` binary")
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1B' {
            in_esc = true;
        } else if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn test_missing_file_exit_code() {
    let output = run_bin(&["non_existent_file_xyz.ph"]);
    assert_eq!(output.status.code(), Some(66));
}

#[test]
fn test_syntax_error_exit_code() {
    let file_path = std::env::temp_dir().join(format!("syntax_{}.ph", std::process::id()));
    fs::write(&file_path, "class Point {\n  var x\n  var y\n  construct new(x, y) {\n    _x = x\n    _y = y\n  }\n}\nlet p = Point.new(1, 2)\n1 + ").unwrap();

    let output = run_bin(&[file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);
    assert_eq!(output.status.code(), Some(65));
}

#[test]
fn test_runtime_error_exit_code() {
    let file_path = std::env::temp_dir().join(format!("runtime_{}.ph", std::process::id()));
    fs::write(&file_path, "1.foo").unwrap();

    let output = run_bin(&[file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);
    assert_eq!(output.status.code(), Some(70));
}

#[test]
fn test_check_vs_run_syntax_diagnostic() {
    let file_path = std::env::temp_dir().join(format!("check_vs_run_{}.ph", std::process::id()));
    fs::write(&file_path, "1 + ").unwrap();

    let run_output = run_bin(&[file_path.to_str().unwrap()]);
    let check_output = run_bin(&["check", file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    assert_eq!(run_output.status.code(), Some(65));
    assert_eq!(check_output.status.code(), Some(65));

    let run_err = strip_ansi(&String::from_utf8_lossy(&run_output.stderr));
    let check_err = strip_ansi(&String::from_utf8_lossy(&check_output.stderr));

    if run_err != check_err {
        eprintln!("run_output status: {:?}", run_output.status);
        eprintln!("run_output stdout: {}", String::from_utf8_lossy(&run_output.stdout));
        eprintln!("run_output stderr: {}", String::from_utf8_lossy(&run_output.stderr));
        eprintln!("check_output status: {:?}", check_output.status);
        eprintln!("check_output stdout: {}", String::from_utf8_lossy(&check_output.stdout));
        eprintln!("check_output stderr: {}", String::from_utf8_lossy(&check_output.stderr));
    }

    assert_eq!(run_err, check_err);
}
