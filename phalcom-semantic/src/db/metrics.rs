//! Query and invalidation execution metrics.

use std::sync::atomic::{AtomicU64, Ordering};

/// Operational metrics and counters for query execution.
#[derive(Debug, Default)]
pub struct QueryMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub invalidations: AtomicU64,
    pub cancellations: AtomicU64,
    pub budget_exhaustions: AtomicU64,
    pub relation_steps: AtomicU64,
    pub scc_iterations: AtomicU64,
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_invalidation(&self) {
        self.invalidations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cancellation(&self) {
        self.cancellations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_budget_exhaustion(&self) {
        self.budget_exhaustions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_relation_step(&self) {
        self.relation_steps.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_scc_iteration(&self) {
        self.scc_iterations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    pub fn total_misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }
}
