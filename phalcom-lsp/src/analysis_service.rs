//! Asynchronous semantic analysis service with latest-wins edit coalescing and background worker thread.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use phalcom_ast::ast::Program;
use tokio::sync::mpsc;
use tower_lsp::lsp_types::Url;

use crate::perf::COUNTERS;
use crate::semantic::{FileRevision, SemanticDb, SemanticGeneration};

/// Pending work batch coalesced by the worker loop before execution.
#[derive(Default)]
pub struct PendingWork {
    /// File update batch indexed by URL (latest revision wins).
    pub file_updates: BTreeMap<Url, (FileRevision, Program)>,
    /// Active core module replacement, if enqueued.
    pub core_update: Option<(FileRevision, Program)>,
    /// Enqueued file removals.
    pub removals: BTreeSet<Url>,
    /// Full workspace re-analysis flag.
    pub full_workspace_rebuild_requested: bool,
    /// Active execution in progress flag.
    pub is_processing: bool,
}

impl PendingWork {
    fn is_idle(&self) -> bool {
        self.file_updates.is_empty()
            && self.core_update.is_none()
            && self.removals.is_empty()
            && !self.full_workspace_rebuild_requested
            && !self.is_processing
    }

    fn is_empty(&self) -> bool {
        self.file_updates.is_empty()
            && self.core_update.is_none()
            && self.removals.is_empty()
            && !self.full_workspace_rebuild_requested
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.file_updates.clear();
        self.core_update = None;
        self.removals.clear();
        self.full_workspace_rebuild_requested = false;
        self.is_processing = false;
    }
}

/// Shared synchronization primitive for the analysis service worker.
pub struct WorkerShared {
    /// Monotonic epoch counter incremented on every enqueued update batch.
    pub epoch: AtomicU64,
    /// Coalesced pending work state.
    pub pending: Mutex<PendingWork>,
    /// Condvar used to signal worker thread when new work arrives or shutdown is requested.
    pub condvar: Condvar,
    /// Nonblocking shutdown signal.
    pub shutdown: AtomicBool,
}

/// Events emitted by the background analysis worker thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisEvent {
    /// A new semantic snapshot generation was published.
    Published {
        /// Published generation counter.
        generation: SemanticGeneration,
    },
    /// Intermediate batch result was discarded due to a higher epoch.
    StaleBatchDiscarded {
        /// Epoch counter at time of discard.
        epoch: u64,
    },
    /// Error encountered during worker execution.
    Error {
        /// Error description message.
        message: String,
    },
}

/// Front-end handle for managing background semantic analysis.
pub struct AnalysisService {
    db: Arc<SemanticDb>,
    shared: Arc<WorkerShared>,

    worker_thread: Option<JoinHandle<()>>,
}

impl AnalysisService {
    /// Standard edit debounce duration before committing a batch to semantic analysis.
    pub const DEBOUNCE_DURATION: Duration = Duration::from_millis(150);

    /// Creates a new `AnalysisService` with a dedicated background worker thread.
    pub fn new(db: Arc<SemanticDb>) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let shared = Arc::new(WorkerShared {
            epoch: AtomicU64::new(0),
            pending: Mutex::new(PendingWork::default()),
            condvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
        });

        let db_clone = db.clone();
        let shared_clone = shared.clone();
        let event_tx_clone = event_tx.clone();

        let worker_thread = thread::Builder::new()
            .name("phalcom-lsp-analyzer".to_string())
            .spawn(move || {
                worker_loop(db_clone, shared_clone, event_tx_clone);
            })
            .expect("failed to spawn phalcom-lsp analyzer thread");

        (
            Self {
                db,
                shared,
                worker_thread: Some(worker_thread),
            },
            event_rx,
        )
    }

    /// Enqueues or updates a source file revision for background semantic processing.
    pub fn enqueue_file_update(&self, uri: Url, revision: FileRevision, program: Program) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        if pending.file_updates.insert(uri, (revision, program)).is_some() {
            COUNTERS.source_updates_coalesced.fetch_add(1, Ordering::Relaxed);
        }
        COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues active core module replacement.
    pub fn enqueue_core_update(&self, revision: FileRevision, program: Program) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        pending.core_update = Some((revision, program));
        COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues file removal.
    pub fn enqueue_file_removal(&self, uri: Url) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        pending.file_updates.remove(&uri);
        pending.removals.insert(uri);
        COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Requests a full workspace re-analysis pass.
    pub fn request_full_workspace_rebuild(&self) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        pending.full_workspace_rebuild_requested = true;
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Flushes and waits for all currently enqueued pending work to be processed by the worker.
    pub fn flush(&self) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        while !pending.is_idle() && !self.shared.shutdown.load(Ordering::SeqCst) {
            pending = self.shared.condvar.wait(pending).expect("worker condvar wait poisoned");
        }
    }

    /// Nonblocking shutdown signal to terminate worker thread.
    pub fn shutdown(&self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Access to the underlying semantic database snapshot handle.
    pub fn db(&self) -> &Arc<SemanticDb> {
        &self.db
    }
}

impl Drop for AnalysisService {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}

/// Worker loop running on the dedicated background thread.
fn worker_loop(
    db: Arc<SemanticDb>,
    shared: Arc<WorkerShared>,
    event_tx: mpsc::UnboundedSender<AnalysisEvent>,
) {
    loop {
        let mut pending = shared.pending.lock().expect("worker pending lock poisoned");

        // Wait for work or shutdown signal
        while pending.is_empty() && !shared.shutdown.load(Ordering::SeqCst) {
            pending = shared.condvar.wait(pending).expect("worker condvar wait poisoned");
        }

        if shared.shutdown.load(Ordering::SeqCst) {
            break;
        }

        // Check if any enqueued file is unindexed (first analysis): if so, analyze immediately without debounce
        let is_first_time = pending.file_updates.keys().any(|uri| db.file_snapshot(uri).is_none())
            || pending.core_update.is_some()
            || pending.full_workspace_rebuild_requested;

        if !is_first_time {
            // Debounce / edit coalescing: wait for edits to settle
            let timeout = AnalysisService::DEBOUNCE_DURATION;
            let result = shared.condvar.wait_timeout(pending, timeout).expect("worker condvar wait_timeout poisoned");
            pending = result.0;
        }

        // Process pending work batch
        while !pending.is_empty() && !shared.shutdown.load(Ordering::SeqCst) {
            // Take snapshot of work batch under lock
            let batch_epoch = shared.epoch.load(Ordering::SeqCst);

            pending.is_processing = true;
            let file_updates = std::mem::take(&mut pending.file_updates);
            let core_update = pending.core_update.take();
            let removals = std::mem::take(&mut pending.removals);
            let _full_rebuild = pending.full_workspace_rebuild_requested;
            pending.full_workspace_rebuild_requested = false;

            // Release lock during heavy semantic execution
            drop(pending);

            COUNTERS.semantic_batches_started.fetch_add(1, Ordering::Relaxed);

            let mut latest_generation = db.generation();

            // Process removals first
            for uri in removals {
                latest_generation = db.remove_file(&uri);
            }

            // Process file updates batch
            if !file_updates.is_empty() {
                let batch = file_updates
                    .into_iter()
                    .map(|(uri, (rev, prog))| (uri, rev, prog))
                    .collect::<Vec<_>>();
                latest_generation = db.update_files_batch(batch);
            }

            // Process core update if enqueued
            if let Some((rev, prog)) = core_update {
                latest_generation = db.update_core(rev, &prog);
            }

            // Epoch staleness check: if newer edits were enqueued during execution, discard intermediate result as stale
            let current_epoch = shared.epoch.load(Ordering::SeqCst);
            if current_epoch > batch_epoch {
                COUNTERS.stale_batches_discarded.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::StaleBatchDiscarded { epoch: batch_epoch });
            } else {
                COUNTERS.semantic_batches_published.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::Published {
                    generation: latest_generation,
                });
            }

            // Re-acquire lock and notify any flush callers
            pending = shared.pending.lock().expect("worker pending lock poisoned");
            pending.is_processing = false;
            shared.condvar.notify_all();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;
    use tower_lsp::lsp_types::Url;

    fn uri(value: &str) -> Url {
        Url::parse(value).unwrap()
    }

    #[test]
    fn analysis_service_coalesces_edits_and_publishes_generation() {
        let db = Arc::new(SemanticDb::new());
        let (service, mut rx) = AnalysisService::new(db.clone());

        let file_uri = uri("file:///test.ph");
        let parse1 = parse("class A { }", 0);
        let parse2 = parse("class A { foo() { } }", 0);

        service.enqueue_file_update(file_uri.clone(), FileRevision(1), parse1.program);
        service.enqueue_file_update(file_uri.clone(), FileRevision(2), parse2.program);

        // Wait for Published event from worker
        let event = rx.blocking_recv().expect("expected event from analysis service");
        assert!(matches!(event, AnalysisEvent::Published { .. }));

        assert!(db.file_snapshot(&file_uri).is_some());
        assert_eq!(db.file_snapshot(&file_uri).unwrap().revision, FileRevision(2));

        service.shutdown();
    }

    #[test]
    fn analysis_service_shutdown_is_nonblocking_and_terminates() {
        let db = Arc::new(SemanticDb::new());
        let (service, _rx) = AnalysisService::new(db);
        service.shutdown();
    }
}
