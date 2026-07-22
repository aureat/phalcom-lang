//! T7 Observability — fiber trace integration tests.
//!
//! Exercises `--trace=fibers` with both `--trace-format json` and text format
//! by running the real `phalcom` binary as a subprocess (via temp `.ph` files)
//! and asserting that the expected trace events appear on stderr.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn phalcom_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_phalcom"))
}

/// Write source to a temp file and run it with `--trace=fibers`.
fn run_trace(source: &str, format: &str) -> Output {
    let path = std::env::temp_dir().join(format!(
        "fiber_trace_{}_{}.ph",
        std::process::id(),
        // unique per-call so parallel tests don't collide on same-length sources
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
        }
    ));
    fs::write(&path, source).expect("write temp file");
    let out = Command::new(phalcom_bin())
        .args(["--trace=fibers", &format!("--trace-format={}", format), path.to_str().unwrap()])
        .output()
        .expect("failed to spawn `phalcom` binary");
    let _ = fs::remove_file(&path);
    out
}

fn run_no_trace(source: &str) -> Output {
    let path = std::env::temp_dir().join(format!("fiber_no_trace_{}_{}.ph", std::process::id(), {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
    }));
    fs::write(&path, source).expect("write temp file");
    let out = Command::new(phalcom_bin())
        .arg(path.to_str().unwrap())
        .output()
        .expect("failed to spawn `phalcom` binary");
    let _ = fs::remove_file(&path);
    out
}

/// Fiber that yields once then returns — spawn + yield + switch + done.
const YIELD_SRC: &str = "const f = Fiber.new {\n  Fiber.yield(42)\n}\nf.call()\n";

/// Fiber that errors — spawn + switch + fail.
const FAIL_SRC: &str = "const f = Fiber.new {\n  1.noSuchMethod\n}\nf.try()\n";

/// Fiber that returns immediately — spawn + switch + done.
const DONE_SRC: &str = "const f = Fiber.new {\n  99\n}\nf.call()\n";

// ── JSON format ───────────────────────────────────────────────────────────────

#[test]
fn json_trace_spawn() {
    let out = run_trace(YIELD_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(r#""ev":"spawn""#), "expected spawn; stderr:\n{stderr}");
}

#[test]
fn json_trace_yield() {
    let out = run_trace(YIELD_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(r#""ev":"yield""#), "expected yield; stderr:\n{stderr}");
}

#[test]
fn json_trace_switch() {
    let out = run_trace(YIELD_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(r#""ev":"switch""#), "expected switch; stderr:\n{stderr}");
}

#[test]
fn json_trace_done() {
    let out = run_trace(DONE_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(r#""ev":"done""#), "expected done; stderr:\n{stderr}");
}

#[test]
fn json_trace_fail() {
    let out = run_trace(FAIL_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(r#""ev":"fail""#), "expected fail; stderr:\n{stderr}");
}

#[test]
fn json_trace_objects_are_braced() {
    // Every trace event line must be `{…}` shaped.
    let out = run_trace(YIELD_SRC, "json");
    let stderr = String::from_utf8_lossy(&out.stderr);
    for line in stderr.lines() {
        let Some(pos) = line.find('{') else { continue };
        let obj = &line[pos..];
        assert!(obj.ends_with('}'), "trace line not a JSON object: {line:?}");
    }
}

// ── Text format ───────────────────────────────────────────────────────────────

#[test]
fn text_trace_spawn() {
    let out = run_trace(YIELD_SRC, "text");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("[fiber] spawn"), "expected '[fiber] spawn'; stderr:\n{stderr}");
}

#[test]
fn text_trace_yield() {
    let out = run_trace(YIELD_SRC, "text");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("[fiber] yield"), "expected '[fiber] yield'; stderr:\n{stderr}");
}

#[test]
fn text_trace_switch() {
    let out = run_trace(YIELD_SRC, "text");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("[fiber] switch"), "expected '[fiber] switch'; stderr:\n{stderr}");
}

// ── No trace by default ───────────────────────────────────────────────────────

#[test]
fn no_trace_flag_no_output() {
    let out = run_no_trace(YIELD_SRC);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("[fiber]") && !stderr.contains(r#""ev":"#),
        "unexpected trace output without --trace; stderr:\n{stderr}"
    );
}
