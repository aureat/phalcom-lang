use std::path::PathBuf;
use std::process::{Command, Output};

const SOURCE: &str = r#"
class CellNum {
  @constructor new() {}
}

class Probe {
  @class
  run() {
    let value: Int = CellNum.new()
  }
}
"#;

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

fn run(args: &[&str]) -> Output {
    Command::new(phalcom_bin())
        .args(args)
        .env_remove("RUST_LOG")
        .env_remove("RUST_LOG_STYLE")
        .output()
        .expect("spawn phalcom")
}

#[test]
fn test_compact_mode() {
    let output = run(&["--diagnostic-detail", "compact", "--color", "never", "--plain", "check", "--source", SOURCE]);
    assert_eq!(output.status.code(), Some(65));

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("initializer conflicts with declared type"));
    assert!(!stderr.contains("\x1b["));
    assert!(!stderr.contains("[e0]"));
}

#[test]
fn test_explain_mode() {
    let output = run(&["--diagnostic-detail", "explain", "--color", "never", "--plain", "check", "--source", SOURCE]);
    assert_eq!(output.status.code(), Some(65));

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("initializer conflicts with declared type"));
    assert!(stderr.contains("@constructor") || stderr.contains("constructor") || stderr.contains("CellNum") || stderr.contains("Int"));
    assert!(!stderr.contains("\x1b["));
}

#[test]
fn test_trace_mode() {
    let output = run(&["--diagnostic-detail", "trace", "--color", "never", "--plain", "check", "--source", SOURCE]);
    assert_eq!(output.status.code(), Some(65));

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("[e"));
    assert!(!stderr.contains("\x1b["));
}

#[test]
fn test_json_structure() {
    let output = run(&["--diagnostic-detail", "trace", "check", "--format", "json", "--source", SOURCE]);
    assert_eq!(output.status.code(), Some(65));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut found_diagnostic = false;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).expect("valid json line");
        if v.get("code").is_some() {
            found_diagnostic = true;
            assert!(v.get("severity").is_some());
            assert!(v.get("code").is_some());
            assert!(v.get("message").is_some());
            assert!(v.get("module").is_some());
            assert!(v.get("range").is_some());
            assert!(v.get("labels").is_some());
            assert!(v.get("explanation").is_some());
            assert!(v.get("guidance").is_some());
            assert!(v.get("context").is_some());
            assert!(v.get("trace").is_some());
            assert!(v.get("fixes").is_some());

            let trace = v["trace"].as_array().expect("trace is array");
            assert!(!trace.is_empty());

            let relation_node = trace.iter().find(|node| node["rule"] == "type_relation").expect("found type_relation rule");

            assert!(relation_node["status"] == "established" || relation_node["status"] == "assumed");

            // Assert machine fields are not Rust debug spellings
            for node in trace {
                if let Some(rule) = node["rule"].as_str() {
                    assert_ne!(rule, "TypeRelation");
                    assert_ne!(rule, "CallableSelection");
                }
                if let Some(origin) = node["origin"].as_str() {
                    assert_ne!(origin, "ConstructorSemantics");
                }
            }
        }
    }
    assert!(found_diagnostic, "at least one diagnostic in json output");
}

#[test]
fn test_detail_projection_monotonicity() {
    let get_json_diag = |detail: &str| -> serde_json::Value {
        let output = run(&["--diagnostic-detail", detail, "check", "--format", "json", "--source", SOURCE]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if v.get("code").is_some() {
                    return v;
                }
            }
        }
        panic!("no diagnostic found for detail {detail}");
    };

    let compact = get_json_diag("compact");
    let explain = get_json_diag("explain");
    let trace = get_json_diag("trace");

    assert_eq!(compact["code"], explain["code"]);
    assert_eq!(explain["code"], trace["code"]);

    let compact_exp_len = compact["explanation"].as_array().map_or(0, |a| a.len());
    let explain_exp_len = explain["explanation"].as_array().map_or(0, |a| a.len());
    assert!(compact_exp_len <= explain_exp_len);

    let compact_trace_len = compact["trace"].as_array().map_or(0, |a| a.len());
    let explain_trace_len = explain["trace"].as_array().map_or(0, |a| a.len());
    let trace_trace_len = trace["trace"].as_array().map_or(0, |a| a.len());

    assert_eq!(compact_trace_len, 0);
    assert_eq!(explain_trace_len, 0);
    assert!(trace_trace_len > 0);
}
