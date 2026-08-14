//! Performance instrumentation and deterministic counters for Phalcom LSP.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

/// Performance counters owned by one analysis service and semantic database.
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
    /// Count of callable bodies seeded into incremental solving.
    pub dirty_callables_seeded: AtomicU64,
    /// Count of callable worklist entries actually visited.
    pub solver_callables_visited: AtomicU64,
    /// Count of visited callables whose summaries changed.
    pub solver_callables_changed: AtomicU64,
    /// Count of speculative semantic candidate state clones.
    pub semantic_candidate_state_clones: AtomicU64,
    /// Count of published file products reused by pointer identity.
    pub published_file_products_reused: AtomicU64,
    /// Count of published class products reused by pointer identity.
    pub published_class_products_reused: AtomicU64,
    /// Count of published callable summaries reused by pointer identity.
    pub published_summary_products_reused: AtomicU64,
    /// Whole class tables materialized by query code; should stay zero.
    pub query_class_table_materializations: AtomicU64,
    /// Whole summary tables materialized by query code; should stay zero.
    pub query_summary_table_materializations: AtomicU64,
    /// Filesystem canonicalization calls made from query paths.
    pub query_filesystem_canonicalizations: AtomicU64,
    /// Disk reads made from query paths.
    pub query_disk_reads: AtomicU64,
    /// Product-specific inlay refresh requests.
    pub inlay_refresh_requests: AtomicU64,
    /// Product-specific semantic-token refresh requests.
    pub semantic_token_refresh_requests: AtomicU64,
    /// Count of source contribution replacements.
    pub parameter_sources_replaced: AtomicU64,
    /// Count of parameter slots touched by replacements.
    pub parameter_slots_touched: AtomicU64,
    /// Count of parameter slots whose joined value changed.
    pub parameter_slots_changed: AtomicU64,
    /// Count of directory entries consumed by progressive scanning.
    pub scan_directory_entries_consumed: AtomicU64,
    /// Count of scan results rejected after source freshness checks.
    pub scan_results_discarded_as_stale: AtomicU64,
    /// Count of scan results rejected because a live document won the race.
    pub scan_results_discarded_for_open_document: AtomicU64,
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
        self.dirty_callables_seeded.store(0, Ordering::Relaxed);
        self.solver_callables_visited.store(0, Ordering::Relaxed);
        self.solver_callables_changed.store(0, Ordering::Relaxed);
        self.semantic_candidate_state_clones.store(0, Ordering::Relaxed);
        self.published_file_products_reused.store(0, Ordering::Relaxed);
        self.published_class_products_reused.store(0, Ordering::Relaxed);
        self.published_summary_products_reused.store(0, Ordering::Relaxed);
        self.query_class_table_materializations.store(0, Ordering::Relaxed);
        self.query_summary_table_materializations.store(0, Ordering::Relaxed);
        self.query_filesystem_canonicalizations.store(0, Ordering::Relaxed);
        self.query_disk_reads.store(0, Ordering::Relaxed);
        self.inlay_refresh_requests.store(0, Ordering::Relaxed);
        self.semantic_token_refresh_requests.store(0, Ordering::Relaxed);
        self.parameter_sources_replaced.store(0, Ordering::Relaxed);
        self.parameter_slots_touched.store(0, Ordering::Relaxed);
        self.parameter_slots_changed.store(0, Ordering::Relaxed);
        self.scan_directory_entries_consumed.store(0, Ordering::Relaxed);
        self.scan_results_discarded_as_stale.store(0, Ordering::Relaxed);
        self.scan_results_discarded_for_open_document.store(0, Ordering::Relaxed);
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
            dirty_callables_seeded: self.dirty_callables_seeded.load(Ordering::Relaxed),
            solver_callables_visited: self.solver_callables_visited.load(Ordering::Relaxed),
            solver_callables_changed: self.solver_callables_changed.load(Ordering::Relaxed),
            semantic_candidate_state_clones: self.semantic_candidate_state_clones.load(Ordering::Relaxed),
            published_file_products_reused: self.published_file_products_reused.load(Ordering::Relaxed),
            published_class_products_reused: self.published_class_products_reused.load(Ordering::Relaxed),
            published_summary_products_reused: self.published_summary_products_reused.load(Ordering::Relaxed),
            query_class_table_materializations: self.query_class_table_materializations.load(Ordering::Relaxed),
            query_summary_table_materializations: self.query_summary_table_materializations.load(Ordering::Relaxed),
            query_filesystem_canonicalizations: self.query_filesystem_canonicalizations.load(Ordering::Relaxed),
            query_disk_reads: self.query_disk_reads.load(Ordering::Relaxed),
            inlay_refresh_requests: self.inlay_refresh_requests.load(Ordering::Relaxed),
            semantic_token_refresh_requests: self.semantic_token_refresh_requests.load(Ordering::Relaxed),
            parameter_sources_replaced: self.parameter_sources_replaced.load(Ordering::Relaxed),
            parameter_slots_touched: self.parameter_slots_touched.load(Ordering::Relaxed),
            parameter_slots_changed: self.parameter_slots_changed.load(Ordering::Relaxed),
            scan_directory_entries_consumed: self.scan_directory_entries_consumed.load(Ordering::Relaxed),
            scan_results_discarded_as_stale: self.scan_results_discarded_as_stale.load(Ordering::Relaxed),
            scan_results_discarded_for_open_document: self.scan_results_discarded_for_open_document.load(Ordering::Relaxed),
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
    /// Count of callable bodies seeded into incremental solving.
    pub dirty_callables_seeded: u64,
    /// Count of callable worklist entries actually visited.
    pub solver_callables_visited: u64,
    /// Count of visited callables whose summaries changed.
    pub solver_callables_changed: u64,
    /// Count of speculative semantic candidate state clones.
    pub semantic_candidate_state_clones: u64,
    /// Count of published file products reused by pointer identity.
    pub published_file_products_reused: u64,
    /// Count of published class products reused by pointer identity.
    pub published_class_products_reused: u64,
    /// Count of published callable summaries reused by pointer identity.
    pub published_summary_products_reused: u64,
    /// Whole class tables materialized by query code.
    pub query_class_table_materializations: u64,
    /// Whole summary tables materialized by query code.
    pub query_summary_table_materializations: u64,
    /// Query-path filesystem canonicalization calls.
    pub query_filesystem_canonicalizations: u64,
    /// Query-path disk reads.
    pub query_disk_reads: u64,
    /// Inlay refresh requests.
    pub inlay_refresh_requests: u64,
    /// Semantic-token refresh requests.
    pub semantic_token_refresh_requests: u64,
    /// Count of source contribution replacements.
    pub parameter_sources_replaced: u64,
    /// Count of parameter slots touched by replacements.
    pub parameter_slots_touched: u64,
    /// Count of parameter slots whose joined value changed.
    pub parameter_slots_changed: u64,
    /// Count of directory entries consumed by progressive scanning.
    pub scan_directory_entries_consumed: u64,
    /// Count of scan results rejected after source freshness checks.
    pub scan_results_discarded_as_stale: u64,
    /// Count of scan results rejected because a live document won the race.
    pub scan_results_discarded_for_open_document: u64,
}

/// Shared counter handle passed between the service, worker, and semantic passes.
pub type PerfCountersHandle = Arc<PerfCounters>;

/// Compatibility counter set for callers that used the original global API.
///
/// Production analysis owns counters through [`crate::semantic::SemanticDb`]
/// and does not increment this set. Keeping this handle avoids breaking tools
/// that imported `COUNTERS` while preventing their resets from affecting live
/// services or parallel tests.
pub static COUNTERS: LazyLock<PerfCountersHandle> = LazyLock::new(|| Arc::new(PerfCounters::new()));

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
    counters: Option<PerfCountersHandle>,
    context: Option<PerfContext>,
}

/// Optional generation and source-epoch context for a performance span.
#[derive(Debug, Clone, Copy)]
pub struct PerfContext {
    /// Semantic generation observed when span started.
    pub generation: Option<u64>,
    /// Source epoch associated with work batch.
    pub epoch: Option<u64>,
}

impl PerfSpan {
    /// Starts timing a named performance span.
    pub fn start(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            counters: None,
            context: None,
        }
    }

    /// Starts timing a span and reports the supplied counter owner's snapshot.
    pub fn start_with_counters(name: &'static str, counters: PerfCountersHandle) -> Self {
        Self {
            name,
            start: Instant::now(),
            counters: Some(counters),
            context: None,
        }
    }

    /// Starts timing a span and emits its context and legacy counter snapshot.
    pub fn start_with_context(name: &'static str, context: PerfContext) -> Self {
        Self::start_with_context_and_counters(name, context, COUNTERS.clone())
    }

    /// Starts timing a span with generation/epoch context and an owned counter set.
    pub fn start_with_context_and_counters(name: &'static str, context: PerfContext, counters: PerfCountersHandle) -> Self {
        Self {
            name,
            start: Instant::now(),
            counters: Some(counters),
            context: Some(context),
        }
    }
}

impl Drop for PerfSpan {
    fn drop(&mut self) {
        if is_perf_enabled() {
            let elapsed = self.start.elapsed();
            eprint!("[phalcom-lsp perf] span={} elapsed_ms={}", self.name, elapsed.as_millis());
            if let Some(context) = self.context {
                if let Some(generation) = context.generation {
                    eprint!(" generation={generation}");
                }
                if let Some(epoch) = context.epoch {
                    eprint!(" epoch={epoch}");
                }
            }
            if let Some(counters) = &self.counters {
                let snapshot = counters.snapshot();
                eprintln!(
                    " updates={}/{}/{} batches={}/{}/{} scan={}/{}/{} flow={} solve={} callables={} candidates={} reused={}/{}/{}",
                    snapshot.source_updates_enqueued,
                    snapshot.source_updates_coalesced,
                    snapshot.source_updates_discarded,
                    snapshot.semantic_batches_started,
                    snapshot.semantic_batches_published,
                    snapshot.stale_batches_discarded,
                    snapshot.scan_batches_published,
                    snapshot.workspace_files_discovered,
                    snapshot.workspace_files_parsed,
                    snapshot.flow_passes,
                    snapshot.solver_rounds,
                    snapshot.callables_analyzed,
                    snapshot.semantic_candidate_state_clones,
                    snapshot.published_file_products_reused,
                    snapshot.published_class_products_reused,
                    snapshot.published_summary_products_reused
                );
            } else {
                eprintln!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_counters_snapshot_and_reset() {
        let counters = PerfCounters::new();
        assert_eq!(
            counters.snapshot(),
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
                dirty_callables_seeded: 0,
                solver_callables_visited: 0,
                solver_callables_changed: 0,
                semantic_candidate_state_clones: 0,
                published_file_products_reused: 0,
                published_class_products_reused: 0,
                published_summary_products_reused: 0,
                query_class_table_materializations: 0,
                query_summary_table_materializations: 0,
                query_filesystem_canonicalizations: 0,
                query_disk_reads: 0,
                inlay_refresh_requests: 0,
                semantic_token_refresh_requests: 0,
                parameter_sources_replaced: 0,
                parameter_slots_touched: 0,
                parameter_slots_changed: 0,
                scan_directory_entries_consumed: 0,
                scan_results_discarded_as_stale: 0,
                scan_results_discarded_for_open_document: 0,
            }
        );

        counters.source_updates_enqueued.fetch_add(5, Ordering::Relaxed);
        counters.flow_passes.fetch_add(12, Ordering::Relaxed);

        let snap = counters.snapshot();
        assert_eq!(snap.source_updates_enqueued, 5);
        assert_eq!(snap.flow_passes, 12);

        counters.reset();
        assert_eq!(counters.snapshot().source_updates_enqueued, 0);
        assert_eq!(counters.snapshot().flow_passes, 0);
    }

    #[test]
    fn counter_sets_are_independent_for_parallel_services() {
        let first = PerfCounters::new();
        let second = PerfCounters::new();

        first.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);

        assert_eq!(first.snapshot().source_updates_enqueued, 1);
        assert_eq!(second.snapshot().source_updates_enqueued, 0);
    }
}
