use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use serde_json::json;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[path = "perf_workloads.rs"]
mod perf_workloads;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name} to identify this profile's source state"))
}

fn profile_standalone_compile() -> Result<(), String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/universe_v1/standalone_module/solo.ph");
    ProgramCompiler::compile_entry_selection(EntrySelection::Module(path))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn profile_multi_module_project() -> Result<(), String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/universe_v1/project_hierarchy");
    ProgramCompiler::compile_entry_selection(EntrySelection::Project(path))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "semantic-baseline".to_owned());
    if !matches!(
        mode.as_str(),
        "semantic-baseline" | "session-init" | "standalone-compile" | "multi-module-project" | "feature-dense" | "incremental-updates"
    ) {
        panic!(
            "unknown feature-disabled workload {mode}; use semantic-baseline, session-init, standalone-compile, multi-module-project, feature-dense, or incremental-updates"
        );
    }

    if mode == "session-init" {
        // The measured session gets a warm, process-shared canonical baseline.
        phalcom_semantic::canonical_baseline::canonical_universe_semantic_baseline();
    }
    let mut incremental_workspace = if mode == "incremental-updates" {
        Some(perf_workloads::IncrementalWorkspace::prepare())
    } else {
        None
    };
    let (workload, lifecycle) = match mode.as_str() {
        "semantic-baseline" => ("canonical_semantic_baseline", "process_cold"),
        "session-init" => ("semantic_session_instantiation", "session_warm"),
        "standalone-compile" => ("standalone_module_compile", "process_cold"),
        "multi-module-project" => ("multi_module_project_compile", "process_cold"),
        "feature-dense" => ("feature_dense_semantic_package", "process_cold"),
        "incremental-updates" => ("incremental_update_variants", "session_warm"),
        _ => unreachable!(),
    };

    let run_id = format!(
        "perf-profile-disabled-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
    );
    let started_unix_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let started = Instant::now();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match mode.as_str() {
        "semantic-baseline" => {
            phalcom_semantic::canonical_baseline::canonical_universe_semantic_baseline();
            Ok(())
        }
        "session-init" => {
            drop(phalcom_semantic::SemanticWorkspaceSession::new());
            Ok(())
        }
        "standalone-compile" => profile_standalone_compile(),
        "multi-module-project" => profile_multi_module_project(),
        "feature-dense" => perf_workloads::run_feature_dense(),
        "incremental-updates" => match incremental_workspace.as_mut().expect("workspace prepared") {
            Ok(workspace) => workspace.run_variants(),
            Err(error) => Err(error.clone()),
        },
        _ => unreachable!(),
    }))
    .unwrap_or_else(|_| Err(format!("{mode} workload panicked")));
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let report = json!({
        "schema_version": 3,
        "metadata": {
            "run_id": run_id,
            "started_unix_ns": started_unix_ns,
            "observability_level": "off",
            "lifecycle": lifecycle,
            "baseline_head": required_env("PHALCOM_PERF_BASELINE_HEAD"),
            "execution_tree_head": required_env("PHALCOM_PERF_EXECUTION_TREE_HEAD"),
            "source_revision": required_env("PHALCOM_PERF_SOURCE_REVISION"),
            "branch": std::env::var("PHALCOM_PERF_BRANCH").unwrap_or_else(|_| "branch-not-recorded".to_owned()),
            "dirty": std::env::var("PHALCOM_PERF_DIRTY").is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
            "toolchain": std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_else(|_| "toolchain-not-recorded".to_owned()),
            "workload": workload
        },
        "elapsed_ns": elapsed_ns,
        "spans": [],
        "counters": {},
        "query_metrics": [],
        "distributions": {}
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("serialize feature-disabled profile"));
    if let Err(error) = outcome {
        eprintln!("Feature-disabled {mode} workload failed after emitting elapsed time: {error}");
        std::process::exit(1);
    }
}
