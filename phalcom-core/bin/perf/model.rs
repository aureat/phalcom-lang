use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkRun {
    pub schema_version: u32,
    pub run_id: String,
    pub timestamp: u64,
    pub git: GitMetadata,
    pub build: BuildMetadata,
    pub host: HostMetadata,
    pub layouts: BTreeMap<String, TypeLayout>,
    pub command: RunCommandMetadata,
    pub suite: SuiteMetadata,
    pub resource_quality: ResourceQuality,
    pub cases: Vec<CaseResult>,
    pub summary: RunSummary,
}

impl BenchmarkRun {
    pub fn from_json_str(s: &str) -> Result<Self, String> {
        let run: BenchmarkRun = serde_json::from_str(s).map_err(|e| e.to_string())?;
        if run.schema_version != SCHEMA_VERSION {
            return Err(format!("unsupported schema version {}; expected {}", run.schema_version, SCHEMA_VERSION));
        }
        Ok(run)
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitMetadata {
    pub sha: String,
    pub short_sha: String,
    pub branch: String,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildMetadata {
    pub binary_path: String,
    pub binary_size_bytes: u64,
    pub profile: String,
    pub target_triple: String,
    pub rustc_version: String,
    pub cargo_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostMetadata {
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub cpu_model: String,
    pub logical_cpus: usize,
    pub host_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeLayout {
    pub size_bytes: usize,
    pub align_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunCommandMetadata {
    pub subcommand: String,
    pub args_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuiteMetadata {
    pub name: String,
    pub path: String,
    pub case_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceQuality {
    Full,
    WallOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sample {
    pub index: usize,
    pub order: Option<SampleOrder>,
    pub wall_ns: u64,
    pub user_ns: Option<u64>,
    pub sys_ns: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub exit_code: i32,
    pub status: SampleStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SampleStatus {
    Ok,
    CorrectnessFailure,
    ProcessError,
    Contaminated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SampleOrder {
    BaselineThenCandidate,
    CandidateThenBaseline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricSummary {
    pub n: usize,
    pub min: f64,
    pub median: f64,
    pub max: f64,
    pub mad: f64,
    pub p90: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseAggregate {
    pub id: String,
    pub tag: String,
    pub wall: MetricSummary,
    pub user_ns: Option<MetricSummary>,
    pub sys_ns: Option<MetricSummary>,
    pub peak_rss_bytes: Option<MetricSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseResult {
    pub id: String,
    pub path: String,
    pub verification_result: String,
    pub samples: Vec<Sample>,
    pub aggregate: Option<CaseAggregate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub contaminated: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparisonRun {
    pub schema_version: u32,
    pub comparison_id: String,
    pub baseline_run_id: String,
    pub candidate_run_id: String,
    pub compatible: bool,
    pub incompatibility_reasons: Vec<String>,
    pub layout_delta: BTreeMap<String, (TypeLayout, TypeLayout)>,
    pub cases: Vec<CaseComparison>,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaseComparison {
    pub id: String,
    pub baseline_agg: Option<CaseAggregate>,
    pub candidate_agg: Option<CaseAggregate>,
    pub delta_abs: Option<f64>,
    pub delta_pct: Option<f64>,
    pub gate_threshold: Option<f64>,
    pub gate_result: String,
    pub paired_stats: Option<PairedStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedStats {
    pub pairs: usize,
    pub faster: usize,
    pub slower: usize,
    pub ties: usize,
    pub median_ratio: f64,
    pub inconclusive: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Regressions,
    Improvements,
    Neutral,
    Inconclusive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_schema_v1() {
        let mut layouts = BTreeMap::new();
        layouts.insert(
            "Value".into(),
            TypeLayout {
                size_bytes: 16,
                align_bytes: 8,
            },
        );
        let run = BenchmarkRun {
            schema_version: 1,
            run_id: "test-run-1".into(),
            timestamp: 1000000000,
            git: GitMetadata {
                sha: "abcdef".into(),
                short_sha: "abcd".into(),
                branch: "main".into(),
                dirty: false,
            },
            build: BuildMetadata {
                binary_path: "/tmp/phalcom".into(),
                binary_size_bytes: 1024,
                profile: "release".into(),
                target_triple: "x86_64-apple-darwin".into(),
                rustc_version: "1.85".into(),
                cargo_version: "1.85".into(),
            },
            host: HostMetadata {
                os: "macos".into(),
                os_version: "14.0".into(),
                arch: "aarch64".into(),
                cpu_model: "Apple M1".into(),
                logical_cpus: 8,
                host_key: "macos-aarch64".into(),
            },
            layouts,
            command: RunCommandMetadata {
                subcommand: "run".into(),
                args_summary: "--suite representation".into(),
            },
            suite: SuiteMetadata {
                name: "representation".into(),
                path: "benchmarks/suites/representation.json".into(),
                case_count: 1,
            },
            resource_quality: ResourceQuality::Full,
            cases: vec![],
            summary: RunSummary {
                total_cases: 0,
                passed: 0,
                failed: 0,
                contaminated: 0,
            },
        };

        let json = run.to_json_string().unwrap();
        let loaded = BenchmarkRun::from_json_str(&json).unwrap();
        assert_eq!(run, loaded);
    }

    #[test]
    fn reject_unknown_schema() {
        let json = r#"{"schema_version": 99, "run_id": "foo"}"#;
        assert!(BenchmarkRun::from_json_str(json).is_err());
    }
}
