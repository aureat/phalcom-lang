//! Performance instrumentation and deterministic counters for Phalcom LSP.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global performance counters for semantic analysis and server operations.
#[derive(Debug, Default)]
pub struct PerfCounters {
    /// Count of source updates enqueued for analysis.
    pub source_updates_enqueued: AtomicU64,
    /// Count of intermediate source updates coalesced before processing.
    pub source_updates_coalesced: AtomicU64,
    /// Count of source mutations rejected as stale or duplicate.
    pub source_updates_discarded: AtomicU64,
    /// Count of semantic analysis batches started.
    pub semantic_batches_started: AtomicU64,
    /// Count of semantic analysis generations published.
    pub semantic_batches_published: AtomicU64,
    /// Count of workspace scan batches that published semantic state.
    pub scan_batches_published: AtomicU64,
    /// Count of completed analysis batches discarded because a newer epoch superseded them.
    pub stale_batches_discarded: AtomicU64,
    /// Count of workspace files discovered by scanner.
    pub workspace_files_discovered: AtomicU64,
    /// Count of workspace files parsed.
    pub workspace_files_parsed: AtomicU64,
    /// Count of flow analysis passes executed over AST surfaces.
    pub flow_passes: AtomicU64,
    /// Count of solver fixed-point rounds executed.
    pub solver_rounds: AtomicU64,
    /// Count of individual callables analyzed.
    pub callables_analyzed: AtomicU64,
}

impl PerfCounters {
    /// Creates a new zero-initialized set of performance counters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all counter values to zero.
    pub fn reset(&self) {
        self.source_updates_enqueued.store(0, Ordering::Relaxed);
        self.source_updates_coalesced.store(0, Ordering::Relaxed);
        self.source_updates_discarded.store(0, Ordering::Relaxed);
        self.semantic_batches_started.store(0, Ordering::Relaxed);
        self.semantic_batches_published.store(0, Ordering::Relaxed);
        self.scan_batches_published.store(0, Ordering::Relaxed);
        self.stale_batches_discarded.store(0, Ordering::Relaxed);
        self.workspace_files_discovered.store(0, Ordering::Relaxed);
        self.workspace_files_parsed.store(0, Ordering::Relaxed);
        self.flow_passes.store(0, Ordering::Relaxed);
        self.solver_rounds.store(0, Ordering::Relaxed);
        self.callables_analyzed.store(0, Ordering::Relaxed);
    }

    /// Captures a snapshot of current counter values.
    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            source_updates_enqueued: self.source_updates_enqueued.load(Ordering::Relaxed),
            source_updates_coalesced: self.source_updates_coalesced.load(Ordering::Relaxed),
            source_updates_discarded: self.source_updates_discarded.load(Ordering::Relaxed),
            semantic_batches_started: self.semantic_batches_started.load(Ordering::Relaxed),
            semantic_batches_published: self.semantic_batches_published.load(Ordering::Relaxed),
            scan_batches_published: self.scan_batches_published.load(Ordering::Relaxed),
            stale_batches_discarded: self.stale_batches_discarded.load(Ordering::Relaxed),
            workspace_files_discovered: self.workspace_files_discovered.load(Ordering::Relaxed),
            workspace_files_parsed: self.workspace_files_parsed.load(Ordering::Relaxed),
            flow_passes: self.flow_passes.load(Ordering::Relaxed),
            solver_rounds: self.solver_rounds.load(Ordering::Relaxed),
            callables_analyzed: self.callables_analyzed.load(Ordering::Relaxed),
        }
    }
}

/// Immutable snapshot of performance counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterSnapshot {
    /// Count of source updates enqueued.
    pub source_updates_enqueued: u64,
    /// Count of source updates coalesced.
    pub source_updates_coalesced: u64,
    /// Count of source mutations rejected as stale or duplicate.
    pub source_updates_discarded: u64,
    /// Count of semantic batches started.
    pub semantic_batches_started: u64,
    /// Count of semantic batches published.
    pub semantic_batches_published: u64,
    /// Count of workspace scan batches that published semantic state.
    pub scan_batches_published: u64,
    /// Count of stale batches discarded.
    pub stale_batches_discarded: u64,
    /// Count of workspace files discovered.
    pub workspace_files_discovered: u64,
    /// Count of workspace files parsed.
    pub workspace_files_parsed: u64,
    /// Count of flow passes.
    pub flow_passes: u64,
    /// Count of solver rounds.
    pub solver_rounds: u64,
    /// Count of callables analyzed.
    pub callables_analyzed: u64,
}

/// Global shared performance counters instance.
pub static COUNTERS: LazyLock<PerfCounters> = LazyLock::new(PerfCounters::new);

static PERF_ENABLED: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("PHALCOM_LSP_PERF")
        .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
});

/// Returns whether `PHALCOM_LSP_PERF` logging is enabled.
pub fn is_perf_enabled() -> bool {
    *PERF_ENABLED
}

/// Lightweight RAII timer span for reporting operation durations when `PHALCOM_LSP_PERF=1`.
pub struct PerfSpan {
    name: &'static str,
    start: Instant,
}

impl PerfSpan {
    /// Starts timing a named performance span.
    pub fn start(name: &'static str) -> Self {
        Self { name, start: Instant::now() }
    }
}

impl Drop for PerfSpan {
    fn drop(&mut self) {
        if is_perf_enabled() {
            let elapsed = self.start.elapsed();
            eprintln!("[phalcom-lsp perf] span={} elapsed_ms={}", self.name, elapsed.as_millis());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_counters_snapshot_and_reset() {
        COUNTERS.reset();
        assert_eq!(
            COUNTERS.snapshot(),
            CounterSnapshot {
                source_updates_enqueued: 0,
                source_updates_coalesced: 0,
                source_updates_discarded: 0,
                semantic_batches_started: 0,
                semantic_batches_published: 0,
                scan_batches_published: 0,
                stale_batches_discarded: 0,
                workspace_files_discovered: 0,
                workspace_files_parsed: 0,
                flow_passes: 0,
                solver_rounds: 0,
                callables_analyzed: 0,
            }
        );

        COUNTERS.source_updates_enqueued.fetch_add(5, Ordering::Relaxed);
        COUNTERS.flow_passes.fetch_add(12, Ordering::Relaxed);

        let snap = COUNTERS.snapshot();
        assert_eq!(snap.source_updates_enqueued, 5);
        assert_eq!(snap.flow_passes, 12);

        COUNTERS.reset();
        assert_eq!(COUNTERS.snapshot().source_updates_enqueued, 0);
        assert_eq!(COUNTERS.snapshot().flow_passes, 0);
    }
}
