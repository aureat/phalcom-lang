use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{BenchmarkRun, ComparisonRun};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaselineIndex {
    pub baselines: BTreeMap<String, String>,
}

pub fn save_local(workspace_root: &Path, run: &BenchmarkRun, summary_txt: Option<&str>) -> Result<PathBuf, String> {
    let run_dir = workspace_root.join("target/perf/runs").join(&run.run_id);
    fs::create_dir_all(&run_dir).map_err(|e| e.to_string())?;

    let run_file = run_dir.join("run.json");
    let json = run.to_json_string()?;
    fs::write(&run_file, json).map_err(|e| e.to_string())?;

    if let Some(txt) = summary_txt {
        let summary_file = run_dir.join("summary.txt");
        let _ = fs::write(summary_file, txt);
    }

    Ok(run_dir)
}

pub fn save_comparison(workspace_root: &Path, comparison: &ComparisonRun) -> Result<PathBuf, String> {
    let run_dir = workspace_root.join("target/perf/runs").join(&comparison.comparison_id);
    fs::create_dir_all(&run_dir).map_err(|e| e.to_string())?;

    let comp_file = run_dir.join("comparison.json");
    let json = serde_json::to_string_pretty(comparison).map_err(|e| e.to_string())?;
    fs::write(&comp_file, json).map_err(|e| e.to_string())?;

    Ok(comp_file)
}

pub fn load_run(workspace_root: &Path, run_ref: &str) -> Result<BenchmarkRun, String> {
    let path = resolve_run_path(workspace_root, run_ref)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    BenchmarkRun::from_json_str(&content)
}

fn resolve_run_path(workspace_root: &Path, run_ref: &str) -> Result<PathBuf, String> {
    if run_ref == "latest" {
        return find_latest_run(workspace_root);
    }

    if let Some(baseline_name) = run_ref.strip_prefix("baseline:") {
        let index = load_baseline_index(workspace_root)?;
        let run_id = index
            .baselines
            .get(baseline_name)
            .ok_ok_or_else(|| format!("baseline '{baseline_name}' not found in baselines.json"))?;
        return resolve_run_path(workspace_root, run_id);
    }

    // Direct run ID check: history first, then local
    let history_path = workspace_root.join("benchmarks/results/history").join(format!("{run_ref}.json"));
    if history_path.exists() {
        return Ok(history_path);
    }

    let local_path = workspace_root.join("target/perf/runs").join(run_ref).join("run.json");
    if local_path.exists() {
        return Ok(local_path);
    }

    Err(format!("could not resolve run reference '{run_ref}'"))
}

fn find_latest_run(workspace_root: &Path) -> Result<PathBuf, String> {
    let runs_dir = workspace_root.join("target/perf/runs");
    if !runs_dir.exists() {
        return Err("no local runs found under target/perf/runs".into());
    }

    let mut latest_path = None;
    let mut latest_mtime = std::time::SystemTime::UNIX_EPOCH;

    let entries = fs::read_dir(&runs_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let run_json = entry.path().join("run.json");
        if run_json.exists() {
            if let Ok(meta) = fs::metadata(&run_json) {
                if let Ok(mtime) = meta.modified() {
                    if mtime >= latest_mtime {
                        latest_mtime = mtime;
                        latest_path = Some(run_json);
                    }
                }
            }
        }
    }

    latest_path.ok_or_else(|| "no valid run.json files found in target/perf/runs".into())
}

pub fn promote(workspace_root: &Path, run_id: &str, name: &str) -> Result<PathBuf, String> {
    let run = load_run(workspace_root, run_id)?;

    if run.summary.contaminated > 0 {
        return Err(format!("cannot promote run '{run_id}': run is marked contaminated (machine contention)"));
    }

    let history_dir = workspace_root.join("benchmarks/results/history");
    fs::create_dir_all(&history_dir).map_err(|e| e.to_string())?;

    let dest_path = history_dir.join(format!("{}.json", run.run_id));
    let json = run.to_json_string()?;
    fs::write(&dest_path, json).map_err(|e| e.to_string())?;

    let mut index = load_baseline_index(workspace_root).unwrap_or_default();
    index.baselines.insert(name.to_string(), run.run_id.clone());
    save_baseline_index(workspace_root, &index)?;

    Ok(dest_path)
}

pub fn load_baseline_index(workspace_root: &Path) -> Result<BaselineIndex, String> {
    let path = workspace_root.join("benchmarks/results/baselines.json");
    if !path.exists() {
        return Ok(BaselineIndex::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_baseline_index(workspace_root: &Path, index: &BaselineIndex) -> Result<(), String> {
    let results_dir = workspace_root.join("benchmarks/results");
    fs::create_dir_all(&results_dir).map_err(|e| e.to_string())?;
    let path = results_dir.join("baselines.json");
    let json = serde_json::to_string_pretty(index).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn list_runs(workspace_root: &Path, limit: usize) -> Vec<(String, u64, String, usize)> {
    let mut runs = Vec::new();
    let runs_dir = workspace_root.join("target/perf/runs");

    if let Ok(entries) = fs::read_dir(&runs_dir) {
        for entry in entries.flatten() {
            let run_json = entry.path().join("run.json");
            if let Ok(content) = fs::read_to_string(&run_json) {
                if let Ok(run) = BenchmarkRun::from_json_str(&content) {
                    runs.push((run.run_id, run.timestamp, run.suite.name, run.cases.len()));
                }
            }
        }
    }

    runs.sort_by_key(|r| std::cmp::Reverse(r.1));
    runs.truncate(limit);
    runs
}

trait OptionExt<T> {
    fn ok_ok_or_else<F: FnOnce() -> String>(self, f: F) -> Result<T, String>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_ok_or_else<F: FnOnce() -> String>(self, f: F) -> Result<T, String> {
        self.ok_or_else(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn dummy_run(id: &str, contaminated: bool) -> BenchmarkRun {
        BenchmarkRun {
            schema_version: 1,
            run_id: id.into(),
            timestamp: 1000,
            git: GitMetadata {
                sha: "a".into(),
                short_sha: "a".into(),
                branch: "b".into(),
                dirty: false,
            },
            build: BuildMetadata {
                binary_path: "p".into(),
                binary_size_bytes: 10,
                profile: "release".into(),
                target_triple: "t".into(),
                rustc_version: "r".into(),
                cargo_version: "c".into(),
            },
            host: HostMetadata {
                os: "macos".into(),
                os_version: "1.0".into(),
                arch: "aarch64".into(),
                cpu_model: "m1".into(),
                logical_cpus: 8,
                host_key: "key".into(),
            },
            layouts: Default::default(),
            command: RunCommandMetadata {
                subcommand: "run".into(),
                args_summary: "".into(),
            },
            suite: SuiteMetadata {
                name: "test".into(),
                path: "".into(),
                case_count: 0,
            },
            resource_quality: ResourceQuality::WallOnly,
            cases: vec![],
            summary: RunSummary {
                total_cases: 0,
                passed: 0,
                failed: 0,
                contaminated: if contaminated { 1 } else { 0 },
            },
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let run = dummy_run("run-1", false);
        save_local(root, &run, None).unwrap();

        let loaded = load_run(root, "run-1").unwrap();
        assert_eq!(loaded.run_id, "run-1");

        let latest = load_run(root, "latest").unwrap();
        assert_eq!(latest.run_id, "run-1");
    }

    #[test]
    fn promote_and_resolve_baseline() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let run = dummy_run("run-1", false);
        save_local(root, &run, None).unwrap();

        promote(root, "run-1", "main").unwrap();

        let baseline_run = load_run(root, "baseline:main").unwrap();
        assert_eq!(baseline_run.run_id, "run-1");
    }

    #[test]
    fn reject_contaminated_promotion() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let run = dummy_run("run-bad", true);
        save_local(root, &run, None).unwrap();

        let res = promote(root, "run-bad", "main");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("run is marked contaminated"));
    }
}
