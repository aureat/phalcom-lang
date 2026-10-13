use serde_json::json;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name} to identify this profile's source state"))
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "semantic-baseline".to_owned());
    if mode != "semantic-baseline" {
        panic!("the feature-disabled profile currently supports only semantic-baseline");
    }

    let run_id = format!(
        "perf-profile-disabled-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
    );
    let started_unix_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let started = Instant::now();
    let outcome = std::panic::catch_unwind(|| {
        phalcom_semantic::canonical_baseline::canonical_universe_semantic_baseline();
    });
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let report = json!({
        "schema_version": 3,
        "metadata": {
            "run_id": run_id,
            "started_unix_ns": started_unix_ns,
            "observability_level": "off",
            "lifecycle": "process_cold",
            "baseline_head": required_env("PHALCOM_PERF_BASELINE_HEAD"),
            "execution_tree_head": required_env("PHALCOM_PERF_EXECUTION_TREE_HEAD"),
            "source_revision": required_env("PHALCOM_PERF_SOURCE_REVISION"),
            "branch": std::env::var("PHALCOM_PERF_BRANCH").unwrap_or_else(|_| "branch-not-recorded".to_owned()),
            "dirty": std::env::var("PHALCOM_PERF_DIRTY").is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
            "toolchain": std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "toolchain-not-recorded".to_owned()),
            "workload": "canonical_semantic_baseline"
        },
        "elapsed_ns": elapsed_ns,
        "spans": [],
        "counters": {},
        "query_metrics": [],
        "distributions": {}
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("serialize feature-disabled profile"));
    if outcome.is_err() {
        eprintln!("Feature-disabled semantic-baseline workload panicked after emitting elapsed time");
        std::process::exit(1);
    }
}
