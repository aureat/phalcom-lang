use crate::PerfAttribution;
use serde::{Deserialize, Serialize};

/// A stable terminal or lifecycle event for one semantic query kind.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfQueryEvent {
    Request,
    DirectReuse,
    RevalidatedReuse,
    Recomputed,
    Invalidated,
    Blocked,
    Cancelled,
    BudgetExceeded,
    Failed,
}

/// Aggregated lifecycle data for one query kind and optional detailed attribution.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PerfQueryMetrics {
    pub query_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<PerfAttribution>,
    pub requests: u64,
    pub direct_reuse: u64,
    pub revalidated_reuse: u64,
    pub recomputed: u64,
    pub invalidated: u64,
    pub blocked: u64,
    pub cancelled: u64,
    pub budget_exceeded: u64,
    pub failed: u64,
    /// Sum of observed recompute-attempt durations, including failed attempts.
    /// Missing start observations do not contribute a synthetic zero-duration
    /// sample.
    pub recompute_duration_ns: u64,
    pub recompute_duration_max_ns: u64,
}

impl PerfQueryMetrics {
    pub(crate) fn record(&mut self, event: PerfQueryEvent, duration_ns: Option<u64>) {
        let counter = match event {
            PerfQueryEvent::Request => &mut self.requests,
            PerfQueryEvent::DirectReuse => &mut self.direct_reuse,
            PerfQueryEvent::RevalidatedReuse => &mut self.revalidated_reuse,
            PerfQueryEvent::Recomputed => &mut self.recomputed,
            PerfQueryEvent::Invalidated => &mut self.invalidated,
            PerfQueryEvent::Blocked => &mut self.blocked,
            PerfQueryEvent::Cancelled => &mut self.cancelled,
            PerfQueryEvent::BudgetExceeded => &mut self.budget_exceeded,
            PerfQueryEvent::Failed => &mut self.failed,
        };
        *counter = counter.saturating_add(1);
        if let Some(duration_ns) = duration_ns {
            self.recompute_duration_ns = self.recompute_duration_ns.saturating_add(duration_ns);
            self.recompute_duration_max_ns = self.recompute_duration_max_ns.max(duration_ns);
        }
    }
}
