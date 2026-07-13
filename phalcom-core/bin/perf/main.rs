//! `phalcom-perf` — combined correctness + performance runner.
//!
//! Runs the language acceptance corpus (`tests/lang/**`) and the Wren-suite
//! benchmarks (`benchmarks/**`) through the built `phalcom` binary, timing
//! every case with [`std::time::Instant`], printing a slowest-first summary
//! table, and appending a machine-readable JSON-lines log under
//! `target/perf-logs/` (git-ignored via the existing `*.log` rule) so runs
//! can be diffed for regressions later.
//!
//! This is a report tool, not a test gate: `cargo test` remains the
//! correctness gate (`phalcom-core/tests/lang.rs`); this binary answers "is it
//! still fast" and "how long did the green suite take" in one pass, which
//! `cargo test`'s libtest harness does not surface on stable Rust.
//!
//! ```sh
//! cargo build -r -p phalcom-core --bin phalcom      # release binary perf measures
//! cargo run -r -p phalcom-core --bin phalcom-perf    # corpus + benchmarks
//! cargo run -r -p phalcom-core --bin phalcom-perf -- --bench-only
//! cargo run -r -p phalcom-core --bin phalcom-perf -- --label concurrency --pending
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;

/// CLI surface for `phalcom-perf`.
#[derive(Parser)]
#[command(name = "phalcom-perf", about = "Run the corpus + benchmarks with timing, save a log")]
struct Cli {
    /// Run only the `tests/lang` acceptance corpus (skip `benchmarks/`).
    #[arg(long)]
    corpus_only: bool,

    /// Run only `benchmarks/` (skip the acceptance corpus).
    #[arg(long)]
    bench_only: bool,

    /// Also run PENDING corpus fixtures (known-not-yet-passing spec targets).
    #[arg(long)]
    pending: bool,

    /// Restrict the corpus run to one top-level label directory (e.g. `concurrency`).
    #[arg(long)]
    label: Option<String>,

    /// Use the debug binary even if a release binary is present. Debug
    /// timings are not representative of real performance — use only for
    /// quick correctness iteration.
    #[arg(long)]
    debug: bool,

    /// Print the slowest N cases in the summary table (default 20).
    #[arg(long, default_value_t = 20)]
    top: usize,
}

/// Which corpus lane a case belongs to, mirroring `tests/support/mod.rs`'s
/// PASS/NEGATIVE/PENDING model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lane {
    /// Exact-stdout acceptance case.
    Pass,
    /// Must exit non-zero and mention a diagnostic substring, never panic.
    Negative,
    /// Spec target not yet implemented; run for visibility, not gating.
    Pending,
    /// A `benchmarks/` program: timed only, no correctness lane.
    Bench,
}

/// Outcome of running and (where applicable) verifying one case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Pass,
    Fail,
    /// Panicked (exit 101 or a `panicked at` stderr line) — always a bug.
    Error,
    /// Bench case: ran to completion, no correctness check performed.
    Ran,
}

/// One timed case, ready for the summary table and the JSON log line.
struct CaseResult {
    lane: Lane,
    label: String,
    name: String,
    status: Status,
    ms: f64,
    note: String,
}

fn workspace_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    root.canonicalize().unwrap_or(root)
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang")
}

fn benchmarks_root() -> PathBuf {
    workspace_root().join("benchmarks")
}

/// Locates the built `phalcom` binary, preferring release for representative
/// timings. Exits with a clear message rather than a confusing spawn error if
/// neither profile has been built.
fn find_phalcom_binary(force_debug: bool) -> PathBuf {
    let target = workspace_root().join("target");
    let release = target.join("release/phalcom");
    let debug = target.join("debug/phalcom");

    if !force_debug && release.exists() {
        return release;
    }
    if debug.exists() {
        if !force_debug {
            eprintln!("warning: no release binary at {} — using debug (timings not representative); run `cargo build -r -p phalcom-core --bin phalcom` first", release.display());
        }
        return debug;
    }
    eprintln!(
        "error: no phalcom binary found at {} or {}\nbuild one first: cargo build -r -p phalcom-core --bin phalcom",
        release.display(),
        debug.display()
    );
    std::process::exit(1);
}

/// Recursively finds every `.ph` file under `dir` that has a sibling
/// `.expected` file — the same "is this a runnable case" rule
/// `tests/support/mod.rs` applies per-directory, generalized across the
/// whole subtree so nested `pending/`/`negative/` dirs are picked up in one
/// walk. Import-fixture `lib/` files have no `.expected` sibling and are
/// skipped by this rule without a special case.
fn collect_corpus_cases(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_ph_files(dir, &mut out);
    out.retain(|p| p.with_extension("expected").exists());
    out.sort();
    out
}

/// Recursively finds every `.ph` file under `dir` (benchmarks have no
/// `.expected` sidecar — timed, not verified).
fn collect_bench_cases(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_ph_files(dir, &mut out);
    out.sort();
    out
}

fn walk_ph_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_ph_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ph") {
            out.push(path);
        }
    }
}

/// Classifies a corpus case's lane from its path, mirroring the directory
/// convention documented in `tests/lang/MANIFEST.md`: any `pending/`
/// ancestor is PENDING; any `negative/` ancestor, or a top-level label that
/// is wholly negative (`runtime-errors`/`syntax-errors`/`compile-errors`),
/// is NEGATIVE; everything else is PASS.
fn classify_lane(path: &Path, corpus_root: &Path) -> Lane {
    let rel = path.strip_prefix(corpus_root).unwrap_or(path);
    let components: Vec<&str> = rel.components().filter_map(|c| c.as_os_str().to_str()).collect();

    if components.iter().any(|c| *c == "pending") {
        return Lane::Pending;
    }
    if components.iter().any(|c| *c == "negative") {
        return Lane::Negative;
    }
    match components.first() {
        Some(&("runtime-errors" | "syntax-errors" | "compile-errors")) => Lane::Negative,
        _ => Lane::Pass,
    }
}

fn label_of(path: &Path, corpus_root: &Path) -> String {
    path.strip_prefix(corpus_root)
        .ok()
        .and_then(|rel| rel.components().next())
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("?")
        .to_string()
}

fn case_name(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn run_timed(bin: &Path, path: &Path) -> (Output, Duration) {
    let start = Instant::now();
    let output = Command::new(bin).arg(path).output().expect("failed to spawn the phalcom binary");
    (output, start.elapsed())
}

fn looks_panicked(output: &Output) -> bool {
    output.status.code() == Some(101) || String::from_utf8_lossy(&output.stderr).contains("panicked at")
}

/// Verifies a PASS/PENDING case: exact stdout match against `.expected`
/// (trailing-newline-insensitive, same rule as `assert_stdout_exact`).
fn verify_pass(output: &Output, expected_path: &Path) -> (Status, String) {
    if looks_panicked(output) {
        return (Status::Error, "panicked".to_string());
    }
    let mut expected = fs::read(expected_path).unwrap_or_default();
    let mut actual = output.stdout.clone();
    for buf in [&mut expected, &mut actual] {
        if buf.ends_with(b"\n") {
            buf.pop();
            if buf.ends_with(b"\r") {
                buf.pop();
            }
        }
    }
    if actual == expected {
        (Status::Pass, String::new())
    } else {
        (Status::Fail, "stdout mismatch".to_string())
    }
}

/// Verifies a NEGATIVE case: non-zero exit, no panic, diagnostic substring
/// present in stdout+stderr — same rule as `assert_negative_output`.
fn verify_negative(output: &Output, expected_path: &Path) -> (Status, String) {
    if looks_panicked(output) {
        return (Status::Error, "panicked".to_string());
    }
    if output.status.code() == Some(0) {
        return (Status::Fail, "unexpectedly succeeded".to_string());
    }
    let note = fs::read_to_string(expected_path).unwrap_or_default();
    let note = note.trim();
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if combined.contains(note) {
        (Status::Pass, String::new())
    } else {
        (Status::Fail, format!("missing diagnostic substring `{note}`"))
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn lane_str(lane: Lane) -> &'static str {
    match lane {
        Lane::Pass => "pass",
        Lane::Negative => "negative",
        Lane::Pending => "pending",
        Lane::Bench => "bench",
    }
}

fn status_str(status: Status) -> &'static str {
    match status {
        Status::Pass => "pass",
        Status::Fail => "fail",
        Status::Error => "error",
        Status::Ran => "ran",
    }
}

fn git_short_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "nogit".to_string())
}

fn main() {
    let cli = Cli::parse();
    let run_corpus = !cli.bench_only;
    let run_bench = !cli.corpus_only;
    let bin = find_phalcom_binary(cli.debug);

    let mut results: Vec<CaseResult> = Vec::new();

    if run_corpus {
        let root = corpus_root();
        for path in collect_corpus_cases(&root) {
            let lane = classify_lane(&path, &root);
            if lane == Lane::Pending && !cli.pending {
                continue;
            }
            let label = label_of(&path, &root);
            if let Some(filter) = &cli.label {
                if &label != filter {
                    continue;
                }
            }
            let name = case_name(&path, &root);
            let expected = path.with_extension("expected");
            let (output, elapsed) = run_timed(&bin, &path);
            let (status, note) = match lane {
                Lane::Negative => verify_negative(&output, &expected),
                _ => verify_pass(&output, &expected),
            };
            results.push(CaseResult {
                lane,
                label,
                name,
                status,
                ms: elapsed.as_secs_f64() * 1000.0,
                note,
            });
        }
    }

    if run_bench {
        let root = benchmarks_root();
        for path in collect_bench_cases(&root) {
            let label = label_of(&path, &root);
            if let Some(filter) = &cli.label {
                if &label != filter {
                    continue;
                }
            }
            let name = case_name(&path, &root);
            let (output, elapsed) = run_timed(&bin, &path);
            let status = if looks_panicked(&output) {
                Status::Error
            } else if output.status.success() {
                Status::Ran
            } else {
                Status::Fail
            };
            let note = if status == Status::Ran {
                String::new()
            } else {
                String::from_utf8_lossy(&output.stderr).lines().next().unwrap_or("").to_string()
            };
            results.push(CaseResult {
                lane: Lane::Bench,
                label,
                name,
                status,
                ms: elapsed.as_secs_f64() * 1000.0,
                note,
            });
        }
    }

    print_summary(&results, cli.top);
    if let Err(err) = write_log(&results) {
        eprintln!("warning: failed to write perf log: {err}");
    }

    let hard_failures = results
        .iter()
        .filter(|r| r.lane != Lane::Bench && r.lane != Lane::Pending && r.status != Status::Pass)
        .count();
    if hard_failures > 0 {
        std::process::exit(1);
    }
}

fn print_summary(results: &[CaseResult], top: usize) {
    let total_ms: f64 = results.iter().map(|r| r.ms).sum();

    println!("\n=== slowest {top} cases ===");
    let mut by_time: Vec<&CaseResult> = results.iter().collect();
    by_time.sort_by(|a, b| b.ms.partial_cmp(&a.ms).unwrap());
    for r in by_time.iter().take(top) {
        println!(
            "{:>8.2} ms  [{:<8}] {:<9} {}{}",
            r.ms,
            lane_str(r.lane),
            status_str(r.status),
            r.name,
            if r.note.is_empty() { String::new() } else { format!("  ({})", r.note) }
        );
    }

    println!("\n=== per-label totals ===");
    let mut labels: Vec<&str> = results.iter().map(|r| r.label.as_str()).collect();
    labels.sort();
    labels.dedup();
    for label in labels {
        let group: Vec<&CaseResult> = results.iter().filter(|r| r.label == label).collect();
        let group_ms: f64 = group.iter().map(|r| r.ms).sum();
        let passed = group.iter().filter(|r| r.status == Status::Pass || r.status == Status::Ran).count();
        println!("{label:<20} {:>4} cases  {:>4} ok  {:>9.2} ms total  {:>8.2} ms avg", group.len(), passed, group_ms, group_ms / group.len().max(1) as f64);
    }

    let total = results.len();
    let passed = results.iter().filter(|r| r.status == Status::Pass || r.status == Status::Ran).count();
    let failed = results.iter().filter(|r| r.status == Status::Fail).count();
    let errored = results.iter().filter(|r| r.status == Status::Error).count();
    println!(
        "\n=== totals ===\n{total} cases  {passed} ok  {failed} fail  {errored} error  {:.2} ms wall",
        total_ms
    );
}

fn write_log(results: &[CaseResult]) -> std::io::Result<PathBuf> {
    let dir = workspace_root().join("target/perf-logs");
    fs::create_dir_all(&dir)?;
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let path = dir.join(format!("{ts}_{}.log", git_short_sha()));

    let mut body = String::new();
    for r in results {
        body.push_str(&format!(
            "{{\"lane\":\"{}\",\"label\":\"{}\",\"case\":\"{}\",\"status\":\"{}\",\"ms\":{:.3},\"note\":\"{}\"}}\n",
            lane_str(r.lane),
            json_escape(&r.label),
            json_escape(&r.name),
            status_str(r.status),
            r.ms,
            json_escape(&r.note),
        ));
    }
    let total_ms: f64 = results.iter().map(|r| r.ms).sum();
    let passed = results.iter().filter(|r| r.status == Status::Pass || r.status == Status::Ran).count();
    body.push_str(&format!(
        "{{\"summary\":true,\"total_cases\":{},\"passed\":{},\"total_ms\":{:.3},\"timestamp\":{},\"git_sha\":\"{}\"}}\n",
        results.len(),
        passed,
        total_ms,
        ts,
        git_short_sha()
    ));

    fs::write(&path, body)?;
    println!("\nlog: {}", path.display());
    Ok(path)
}
