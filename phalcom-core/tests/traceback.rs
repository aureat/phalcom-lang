use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn run_bin(args: &[&str]) -> Output {
    Command::new(phalcom_bin()).args(args).output().expect("failed to spawn the `phalcom` binary")
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
fn test_json_traceback_format() {
    let file_path = std::env::temp_dir().join(format!("traceback_json_{}.ph", std::process::id()));
    fs::write(
        &file_path,
        "class Test {\n  @constructor new() {}\n  foo { return self.bar }\n  bar { return 1.missing }\n}\nTest.new().foo\n",
    )
    .unwrap();

    let output = run_bin(&["--trace-format", "json", file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(r#"{"ev":"traceback""#), "Must be JSON: {}", stderr);
    assert!(stderr.contains(r#""name":"foo""#), "Must contain method foo: {}", stderr);
    assert!(stderr.contains(r#""name":"bar""#), "Must contain method bar: {}", stderr);
}

#[test]
fn test_list_spread_traceback_includes_spread_site() {
    let file_path = std::env::temp_dir().join(format!("traceback_list_spread_{}.ph", std::process::id()));
    fs::write(
        &file_path,
        "class Broken {\n  iterate(_ cursor) { return 0 }\n  iteratorValue(_ cursor) { throw Error.new(\"iterator value boom\") }\n}\n[*Broken.new()]\n",
    )
    .unwrap();

    let output = run_bin(&["--trace-format", "json", file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("iterator value boom"), "Must contain iterator error: {stderr}");
    assert!(stderr.contains(r#""name":"iteratorValue(_)""#), "Must contain iteratorValue frame: {stderr}");
    assert!(stderr.contains(r#""line":5"#), "Must contain List spread caller line: {stderr}");
}

#[test]
fn test_labeled_spread_traceback_includes_spread_site() {
    let file_path = std::env::temp_dir().join(format!("traceback_labeled_spread_{}.ph", std::process::id()));
    fs::write(
        &file_path,
        "class BadKey {\n  @constructor new() { _enabled = false }\n  enable() { _enabled = true }\n  hash { if (_enabled) { throw Error.new(\"hash boom\") }; 1 }\n}\nconst key = BadKey.new()\nconst source = { [key]: 1 }\nkey.enable()\n{ **source }\n",
    )
    .unwrap();

    let output = run_bin(&["--trace-format", "json", file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Map key in ** expansion must be Symbol; got object"),
        "Must contain labeled-spread key diagnostic: {stderr}"
    );
    assert!(stderr.contains(r#""line":9"#), "Must contain Map spread caller line: {stderr}");
}

#[test]
fn test_recursion_collapse() {
    let file_path = std::env::temp_dir().join(format!("traceback_collapse_{}.ph", std::process::id()));
    // Create a loop recursion where go calls itself repeatedly.
    // In Phalcom, MAX_CALL_DEPTH is 256. 256 > 3, so repeat collapse should trigger.
    fs::write(
        &file_path,
        "class Boom {\n  @constructor new() {}\n  go(_ n) { return self.go(n + 1) }\n}\nBoom.new().go(0)\n",
    )
    .unwrap();

    let output = run_bin(&[file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    assert!(stderr.contains("[previous frame repeated"), "Must collapse repeating frames: {}", stderr);
}

#[test]
fn test_budget_elision() {
    let file_path = std::env::temp_dir().join(format!("traceback_budget_{}.ph", std::process::id()));
    let mut code = String::new();
    code.push_str("class Boom {\n  @constructor new() {}\n");
    for i in 0..45 {
        if i == 44 {
            code.push_str(&format!("  f{}(_ n) {{ return 1.missing }}\n", i));
        } else {
            code.push_str(&format!("  f{}(_ n) {{ return self.f{}(n) }}\n", i, i + 1));
        }
    }
    code.push_str("}\nBoom.new().f0(0)\n");
    fs::write(&file_path, code).unwrap();

    let output = run_bin(&[file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    assert!(stderr.contains("frames elided"), "Must elide middle frames: {}", stderr);
}

#[test]
fn test_native_frame_rendering() {
    let file_path = std::env::temp_dir().join(format!("traceback_native_{}.ph", std::process::id()));
    // trigger a runtime error inside a native method send, e.g. Number.missing
    fs::write(&file_path, "1.missing\n").unwrap();

    let output = run_bin(&[file_path.to_str().unwrap()]);
    let _ = fs::remove_file(&file_path);

    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    // It should synthesize a native frame for the dynamic send target
    assert!(stderr.contains("[native]"), "Must render native frame line: {}", stderr);
}

fn assert_color_invariance(source: &str, extra_args: &[&str]) {
    let file_path = std::env::temp_dir().join(format!("color_invariance_{}_{}.ph", std::process::id(), {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
    }));
    fs::write(&file_path, source).unwrap();

    let file_str = file_path.to_str().unwrap();
    let mut args_colored = vec!["--color", "always"];
    args_colored.extend(extra_args);
    args_colored.push(file_str);

    let mut args_plain = vec!["--color", "never"];
    args_plain.extend(extra_args);
    args_plain.push(file_str);

    let output_colored = run_bin(&args_colored);
    let output_plain = run_bin(&args_plain);
    let _ = fs::remove_file(&file_path);

    let stderr_colored = String::from_utf8_lossy(&output_colored.stderr);
    let stderr_plain = String::from_utf8_lossy(&output_plain.stderr);

    let stripped_colored = strip_ansi(&stderr_colored);
    assert_eq!(
        stripped_colored, stderr_plain,
        "Color-off invariance failure on stderr!\n\n=== Stripped Colored ===\n{}\n\n=== Never Color ===\n{}",
        stripped_colored, stderr_plain
    );

    let stdout_colored = String::from_utf8_lossy(&output_colored.stdout);
    let stdout_plain = String::from_utf8_lossy(&output_plain.stdout);
    let stripped_stdout = strip_ansi(&stdout_colored);
    assert_eq!(
        stripped_stdout, stdout_plain,
        "Color-off invariance failure on stdout!\n\n=== Stripped Colored ===\n{}\n\n=== Never Color ===\n{}",
        stripped_stdout, stdout_plain
    );
}

#[test]
fn test_color_invariance_runtime_base() {
    assert_color_invariance(
        "class Test {\n  @constructor new() {}\n  foo { return self.bar }\n  bar { return 1.missing }\n}\nTest.new().foo\n",
        &[],
    );
}

#[test]
fn test_color_invariance_runtime_fiber() {
    assert_color_invariance("let worker = Fiber.new {\n  1.missing\n}\nworker.call()\n", &[]);
}

#[test]
fn test_color_invariance_syntax_error() {
    assert_color_invariance("1 + \n", &[]);
}

#[test]
fn test_color_invariance_compile_error() {
    assert_color_invariance("break\n", &[]);
}

#[test]
fn test_color_invariance_disasm() {
    let source = "1 + 2\n";
    let output_colored = Command::new(phalcom_bin())
        .args(["--color", "always", "disasm", "--source", source])
        .output()
        .expect("failed to spawn `phalcom` binary");
    let output_plain = Command::new(phalcom_bin())
        .args(["--color", "never", "disasm", "--source", source])
        .output()
        .expect("failed to spawn `phalcom` binary");

    let stdout_colored = String::from_utf8_lossy(&output_colored.stdout);
    let stdout_plain = String::from_utf8_lossy(&output_plain.stdout);

    let stripped_colored = strip_ansi(&stdout_colored);
    assert_eq!(
        stripped_colored, stdout_plain,
        "Color-off invariance failure on disasm stdout!\n\n=== Stripped Colored ===\n{}\n\n=== Never Color ===\n{}",
        stripped_colored, stdout_plain
    );
}
