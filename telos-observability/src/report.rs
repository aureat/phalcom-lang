use crate::{PerfAttribution, PerfCounter, PerfOutcome, PerfPhase, PerfQueryMetrics};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PerfRunMetadata {
    pub run_id: String,
    pub started_unix_ns: u128,
    pub observability_level: crate::ObservabilityLevel,
    /// Process/session cache state for the measured operation when the harness
    /// can classify it deterministically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<PerfLifecycle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_head: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_tree_head: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dirty: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
}

/// Reproducible cache/process boundary for one characterization sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfLifecycle {
    ProcessCold,
    ProcessWarm,
    SessionCold,
    SessionWarm,
    VmColdInsideWarmProcess,
}

impl PerfRunMetadata {
    pub fn new(run_id: impl Into<String>, observability_level: crate::ObservabilityLevel) -> Self {
        let started_unix_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        Self {
            run_id: run_id.into(),
            started_unix_ns,
            observability_level,
            lifecycle: None,
            baseline_head: None,
            execution_tree_head: None,
            source_revision: None,
            branch: None,
            dirty: None,
            toolchain: None,
            workload: None,
        }
    }

    pub fn with_lifecycle(mut self, lifecycle: PerfLifecycle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    pub fn with_baseline_head(mut self, head: impl Into<String>) -> Self {
        self.baseline_head = Some(head.into());
        self
    }

    pub fn with_execution_tree_head(mut self, head: impl Into<String>) -> Self {
        self.execution_tree_head = Some(head.into());
        self
    }

    pub fn with_source_revision(mut self, revision: impl Into<String>) -> Self {
        self.source_revision = Some(revision.into());
        self
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.dirty = Some(dirty);
        self
    }

    pub fn with_toolchain(mut self, toolchain: impl Into<String>) -> Self {
        self.toolchain = Some(toolchain.into());
        self
    }

    pub fn with_workload(mut self, workload: impl Into<String>) -> Self {
        self.workload = Some(workload.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PerfSpanRecord {
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u64>,
    pub phase: PerfPhase,
    pub start_offset_ns: u64,
    pub duration_ns: u64,
    pub outcome: PerfOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<PerfAttribution>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PerfReport {
    pub schema_version: u32,
    pub metadata: PerfRunMetadata,
    pub elapsed_ns: u64,
    pub spans: Vec<PerfSpanRecord>,
    pub counters: BTreeMap<PerfCounter, u64>,
    /// Query lifecycle aggregates. Empty when no semantic query instrumentation
    /// ran in the profiled process.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_metrics: Vec<PerfQueryMetrics>,
    /// Bounded count/sum/max distributions, collected only in Detailed/Trace.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub distributions: BTreeMap<String, PerfDistribution>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PerfDistribution {
    pub count: u64,
    pub sum: u64,
    pub max: u64,
}

impl PerfDistribution {
    pub(crate) fn record(&mut self, sample: u64) {
        self.count = self.count.saturating_add(1);
        self.sum = self.sum.saturating_add(sample);
        self.max = self.max.max(sample);
    }
}
