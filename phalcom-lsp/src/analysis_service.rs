//! Asynchronous semantic analysis service with latest-wins edit coalescing and background worker thread.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use phalcom_ast::ast::Program;
use tokio::sync::mpsc;
use tower_lsp::lsp_types::Url;

use crate::analysis_log::{AnalysisLogEvent, AnalysisLogLevel};
use crate::analysis_status::{AnalysisPhase, AnalysisStatus, AnalysisStep, StatusTracker};
use crate::line_index::LineIndex;
use crate::perf::{PerfContext, PerfCountersHandle, PerfSpan};
use crate::publication::SemanticPublication;
use crate::source_transport::source_location_for_uri;
use crate::workspace_scan::{AnalysisMode, ExcludeMatcher, ScanBudget, WorkspaceScanState};
use phalcom_modules::SourceRevision;

/// Closed-file source metadata populated by the worker before it publishes
/// semantic state. Query handlers can read this cache without waiting for the
/// asynchronous event notification task.
#[derive(Clone, Debug)]
pub(crate) struct CachedSource {
    pub(crate) text: Arc<str>,
    pub(crate) program: Arc<Program>,
    pub(crate) line_index: Arc<LineIndex>,
}

pub(crate) type SourceCache = Arc<RwLock<BTreeMap<Url, CachedSource>>>;

/// One parsed live-document update waiting for canonical semantic analysis.
#[derive(Clone, Debug)]
pub(crate) struct PendingSourceUpdate {
    pub(crate) revision: SourceRevision,
    pub(crate) text: Arc<str>,
    pub(crate) program: Arc<Program>,
}

/// Latest workspace discovery configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceScanRequest {
    /// Filesystem roots to discover.
    pub roots: Vec<PathBuf>,
    /// Deep-analysis policy for discovered files.
    pub mode: AnalysisMode,
    /// User-configured path exclusions.
    pub excludes: Vec<String>,
}

/// Pending work batch coalesced by the worker loop before execution.
#[derive(Default)]
pub(crate) struct PendingWork {
    /// File update batch indexed by URL (latest revision wins).
    pub(crate) file_updates: BTreeMap<Url, PendingSourceUpdate>,
    /// Enqueued file removals.
    pub(crate) removals: BTreeSet<Url>,
    /// Closed-file disk refreshes waiting for worker-owned I/O and parsing.
    pub(crate) disk_refreshes: BTreeSet<Url>,
    /// Full workspace re-analysis flag.
    pub(crate) full_workspace_rebuild_requested: bool,
    /// Active execution in progress flag.
    pub(crate) is_processing: bool,
    /// Latest workspace scan request. New roots/config replace older requests.
    pub(crate) workspace_scan: Option<WorkspaceScanRequest>,
}

impl PendingWork {
    fn is_idle(&self) -> bool {
        self.file_updates.is_empty()
            && self.removals.is_empty()
            && self.disk_refreshes.is_empty()
            && !self.full_workspace_rebuild_requested
            && !self.is_processing
            && self.workspace_scan.is_none()
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.file_updates.clear();
        self.removals.clear();
        self.disk_refreshes.clear();
        self.full_workspace_rebuild_requested = false;
        self.is_processing = false;
        self.workspace_scan = None;
    }
}

/// One closed-source disk refresh requested by an LSP handler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiskRefresh {
    /// URI to read and parse on the analysis worker.
    pub uri: Url,
}

/// Shared synchronization primitive for the analysis service worker.
pub(crate) struct WorkerShared {
    /// Monotonic epoch counter incremented on every enqueued update batch.
    pub(crate) epoch: AtomicU64,
    /// Coalesced pending work state.
    pub(crate) pending: Mutex<PendingWork>,
    /// Condvar used to signal worker thread when new work arrives or shutdown is requested.
    pub(crate) condvar: Condvar,
    /// Nonblocking shutdown signal.
    pub(crate) shutdown: AtomicBool,
    /// Open documents whose live buffers have priority over disk discovery.
    pub(crate) open_documents: Mutex<BTreeSet<Url>>,
    /// Per-URI source epochs used to reject stale scan and refresh results.
    pub(crate) source_epochs: Mutex<BTreeMap<Url, u64>>,
    /// True while a configured workspace scan still has undiscovered work.
    pub(crate) scan_in_progress: AtomicBool,
    /// Counter set owned by the semantic database served by this worker.
    pub(crate) counters: PerfCountersHandle,
    #[cfg(test)]
    test_batch_gate: Mutex<Option<Arc<TestBatchGate>>>,
    #[cfg(test)]
    test_scan_gate: Mutex<Option<Arc<TestScanGate>>>,
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestBatchGate {
    state: Mutex<TestBatchGateState>,
    condvar: Condvar,
}

#[cfg(test)]
#[derive(Default)]
struct TestBatchGateState {
    before_entered: bool,
    before_released: bool,
}

#[cfg(test)]
impl TestBatchGate {
    pub(crate) fn wait_until_before_entered(&self) {
        let mut state = self.state.lock().expect("test gate lock poisoned");
        while !state.before_entered {
            state = self.condvar.wait(state).expect("test gate condvar poisoned");
        }
    }

    pub(crate) fn release_before(&self) {
        self.state.lock().expect("test gate lock poisoned").before_released = true;
        self.condvar.notify_all();
    }

    fn wait_before(&self) {
        let mut state = self.state.lock().expect("test gate lock poisoned");
        state.before_entered = true;
        self.condvar.notify_all();
        while !state.before_released {
            state = self.condvar.wait(state).expect("test gate condvar poisoned");
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestScanGate {
    state: Mutex<TestScanGateState>,
    condvar: Condvar,
}

#[cfg(test)]
#[derive(Default)]
struct TestScanGateState {
    entered: bool,
    released: bool,
}

#[cfg(test)]
impl TestScanGate {
    pub(crate) fn wait_until_entered(&self) {
        let mut state = self.state.lock().expect("test scan gate lock poisoned");
        while !state.entered {
            state = self.condvar.wait(state).expect("test scan gate condvar poisoned");
        }
    }

    pub(crate) fn release(&self) {
        self.state.lock().expect("test scan gate lock poisoned").released = true;
        self.condvar.notify_all();
    }

    fn wait(&self) {
        let mut state = self.state.lock().expect("test scan gate lock poisoned");
        state.entered = true;
        self.condvar.notify_all();
        while !state.released {
            state = self.condvar.wait(state).expect("test scan gate condvar poisoned");
        }
    }
}

/// Events emitted by the background analysis worker thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisEvent {
    /// A new semantic snapshot generation was published.
    Published {
        /// Published generation counter.
        generation: u64,
        /// Editor products changed in this publication.
        effects: PublicationEffects,
    },
    /// Progress or phase status update.
    Status(AnalysisStatus),
    /// Structured log event emitted during analysis.
    Log(Box<crate::analysis_log::AnalysisLogEvent>),
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
    /// A source file was shallow-indexed by progressive workspace discovery.
    WorkspaceFileIndexed {
        /// URL of the discovered file.
        uri: Url,
        /// Source text retained for closed-file LSP metadata queries.
        text: Arc<str>,
        /// Revision assigned to the cached source snapshot.
        revision: SourceRevision,
    },
    /// A worker-owned disk refresh found no source file at its URI.
    WorkspaceFileRemoved {
        /// URI removed from shallow source products.
        uri: Url,
    },
}

/// Product-level effects of one semantic publication.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PublicationEffects {
    /// Inlay values or policies may have changed.
    pub inlay_hints_changed: bool,
    /// Source occurrence/token classification may have changed.
    pub semantic_tokens_changed: bool,
}

/// Front-end handle for managing background semantic analysis.
pub struct AnalysisService {
    publication: Arc<SemanticPublication>,
    counters: PerfCountersHandle,
    shared: Arc<WorkerShared>,

    worker_thread: Option<JoinHandle<()>>,
}

impl AnalysisService {
    /// Standard edit debounce duration before committing a batch to semantic analysis.
    pub const DEBOUNCE_DURATION: Duration = Duration::from_millis(150);

    /// Creates a new `AnalysisService` with a dedicated background worker thread.
    pub fn new() -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
        Self::new_with_source_cache(None)
    }

    /// Creates an analysis service with a worker-owned closed-source cache.
    pub(crate) fn new_with_source_cache(source_cache: Option<SourceCache>) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
        let publication = Arc::new(SemanticPublication::new());
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let counters = Arc::new(crate::perf::PerfCounters::new());
        let shared = Arc::new(WorkerShared {
            epoch: AtomicU64::new(0),
            pending: Mutex::new(PendingWork::default()),
            condvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
            open_documents: Mutex::new(BTreeSet::new()),
            source_epochs: Mutex::new(BTreeMap::new()),
            scan_in_progress: AtomicBool::new(false),
            counters: counters.clone(),
            #[cfg(test)]
            test_batch_gate: Mutex::new(None),
            #[cfg(test)]
            test_scan_gate: Mutex::new(None),
        });

        let publication_clone = publication.clone();
        let source_cache_clone = source_cache.clone();
        let shared_clone = shared.clone();
        let event_tx_clone = event_tx.clone();

        let worker_thread = thread::Builder::new()
            .name("phalcom-lsp-analyzer".to_string())
            .spawn(move || {
                worker_loop(publication_clone, source_cache_clone, shared_clone, event_tx_clone);
            })
            .expect("failed to spawn phalcom-lsp analyzer thread");

        (
            Self {
                publication,
                counters,
                shared,
                worker_thread: Some(worker_thread),
            },
            event_rx,
        )
    }

    /// Enqueues or updates a source file revision for background semantic processing.
    pub fn enqueue_file_update(&self, uri: Url, revision: SourceRevision, text: Arc<str>, program: Arc<Program>) {
        self.enqueue_file_update_with_source(uri, revision, text, program);
    }

    /// Enqueues one parsed source and its already-ingested text.
    pub(crate) fn enqueue_file_update_with_source(&self, uri: Url, revision: SourceRevision, text: Arc<str>, program: Arc<Program>) {
        self.bump_source_epoch(&uri);
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        if !self.accepts_revision(&pending, &uri, revision) {
            self.counters.source_updates_discarded.fetch_add(1, Ordering::Relaxed);
            return;
        }
        pending.removals.remove(&uri);
        if pending.file_updates.insert(uri, PendingSourceUpdate { revision, text, program }).is_some() {
            self.counters.source_updates_coalesced.fetch_add(1, Ordering::Relaxed);
        }
        self.counters.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues one coalesced batch of source replacements.
    pub fn enqueue_file_updates(&self, updates: Vec<(Url, SourceRevision, Arc<str>, Arc<Program>)>) {
        if updates.is_empty() {
            return;
        }
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        for (uri, revision, text, program) in updates {
            self.bump_source_epoch(&uri);
            if !self.accepts_revision(&pending, &uri, revision) {
                self.counters.source_updates_discarded.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            pending.removals.remove(&uri);
            if pending.file_updates.insert(uri, PendingSourceUpdate { revision, text, program }).is_some() {
                self.counters.source_updates_coalesced.fetch_add(1, Ordering::Relaxed);
            }
            self.counters.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        }
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    fn accepts_revision(&self, pending: &PendingWork, uri: &Url, revision: SourceRevision) -> bool {
        pending.file_updates.get(uri).is_none_or(|queued| queued.revision < revision)
    }

    /// Enqueues file removal.
    pub fn enqueue_file_removal(&self, uri: Url) {
        self.enqueue_file_mutations(vec![uri], Vec::new());
    }

    /// Enqueues one coalesced batch of source removals.
    pub fn enqueue_file_removals(&self, uris: Vec<Url>) {
        if uris.is_empty() {
            return;
        }
        self.enqueue_file_mutations(uris, Vec::new());
    }

    /// Enqueues one logical batch of disk refreshes and removals.
    pub(crate) fn enqueue_file_mutations(&self, removals: Vec<Url>, refreshes: Vec<DiskRefresh>) {
        if removals.is_empty() && refreshes.is_empty() {
            return;
        }
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        for uri in removals {
            self.bump_source_epoch(&uri);
            pending.file_updates.remove(&uri);
            pending.disk_refreshes.remove(&uri);
            pending.removals.insert(uri);
            self.counters.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        }
        for refresh in refreshes {
            self.bump_source_epoch(&refresh.uri);
            if pending.removals.contains(&refresh.uri) {
                continue;
            }
            pending.disk_refreshes.insert(refresh.uri);
            self.counters.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        }
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues one worker-owned closed-file disk refresh.
    pub(crate) fn enqueue_disk_refresh(&self, uri: Url) {
        self.enqueue_file_mutations(Vec::new(), vec![DiskRefresh { uri }]);
    }

    /// Requests a full workspace re-analysis pass.
    pub fn request_full_workspace_rebuild(&self) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        pending.full_workspace_rebuild_requested = true;
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Replaces pending workspace discovery with the newest configuration.
    pub fn configure_workspace(&self, request: WorkspaceScanRequest) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        pending.workspace_scan = Some(request);
        self.shared.scan_in_progress.store(true, Ordering::SeqCst);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Marks a document as live so disk discovery cannot overwrite its
    /// unsaved shallow index or semantic contribution.
    pub fn mark_open(&self, uri: Url) {
        self.bump_source_epoch(&uri);
        self.shared.open_documents.lock().expect("open document lock poisoned").insert(uri);
    }

    /// Allows subsequent disk discovery for a closed document.
    pub fn mark_closed(&self, uri: &Url) {
        self.bump_source_epoch(uri);
        self.shared.open_documents.lock().expect("open document lock poisoned").remove(uri);
    }

    fn bump_source_epoch(&self, uri: &Url) {
        let mut epochs = self.shared.source_epochs.lock().expect("source epoch lock poisoned");
        let next = epochs.get(uri).copied().unwrap_or_default().saturating_add(1);
        epochs.insert(uri.clone(), next);
    }

    /// Flushes and waits for all currently enqueued pending work to be processed by the worker.
    pub fn flush(&self) {
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        while (!pending.is_idle() || self.shared.scan_in_progress.load(Ordering::SeqCst)) && !self.shared.shutdown.load(Ordering::SeqCst) {
            pending = self.shared.condvar.wait(pending).expect("worker condvar wait poisoned");
        }
    }

    /// Nonblocking shutdown signal to terminate worker thread.
    pub fn shutdown(&self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    #[cfg(test)]
    pub(crate) fn install_test_batch_gate(&self, gate: Arc<TestBatchGate>) {
        *self.shared.test_batch_gate.lock().expect("test gate lock poisoned") = Some(gate);
    }

    #[cfg(test)]
    pub(crate) fn install_test_scan_gate(&self, gate: Arc<TestScanGate>) {
        *self.shared.test_scan_gate.lock().expect("test scan gate lock poisoned") = Some(gate);
    }

    /// Clones the immutable canonical publication handle for event consumers.
    pub(crate) fn publication_handle(&self) -> Arc<SemanticPublication> {
        self.publication.clone()
    }

    /// Returns the latest immutable canonical snapshot.
    pub fn snapshot(&self) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
        self.publication.load()
    }

    /// Returns an opaque read-only handle to source coherence in the latest
    /// canonical semantic publication.
    pub(crate) fn semantic_publication_handle(&self) -> crate::publication::SemanticPublicationHandle {
        crate::publication::SemanticPublicationHandle::new(self.publication.clone())
    }

    /// Returns the counter set shared by this service and its semantic database.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.counters.clone()
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
    publication: Arc<SemanticPublication>,
    source_cache: Option<SourceCache>,
    shared: Arc<WorkerShared>,
    event_tx: mpsc::UnboundedSender<AnalysisEvent>,
) {
    // Worker owns one persistent canonical semantic session. The publication
    // cell only exposes immutable snapshots to concurrent request readers.
    let mut compiler_workspace_state = CompilerWorkspaceState::default();
    let mut scanner = None;
    let mut discovered_files = BTreeSet::new();
    let mut status_tracker = StatusTracker::new(AnalysisMode::Local);
    let _ = event_tx.send(AnalysisEvent::Status(status_tracker.snapshot()));

    loop {
        let mut pending = shared.pending.lock().expect("worker pending lock poisoned");

        while !has_analysis_work(&pending) && pending.workspace_scan.is_none() && scanner.is_none() && !shared.shutdown.load(Ordering::SeqCst) {
            pending = shared.condvar.wait(pending).expect("worker condvar wait poisoned");
        }

        if shared.shutdown.load(Ordering::SeqCst) {
            break;
        }

        if let Some(request) = pending.workspace_scan.take() {
            let mut next = WorkspaceScanState::new(request.mode, ExcludeMatcher::new(&request.excludes));
            next.set_roots(request.roots.clone());
            scanner = Some(next);
            let status = status_tracker.increment_session(request.mode);
            let _ = event_tx.send(AnalysisEvent::Status(status.clone()));
            let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                session: status.session,
                sequence: status.sequence,
                level: AnalysisLogLevel::Info,
                phase: status.phase,
                event: "workspace.session.started".to_string(),
                epoch: Some(shared.epoch.load(Ordering::Acquire)),
                generation: None,
                uri: None,
                revision: None,
                batch_size: None,
                duration_ms: None,
                message: Some(format!("workspace session {} started in {:?} mode", status.session, request.mode)),
                counters: Some(shared.counters.snapshot()),
            })));
        }

        // Interactive semantic work always wins over one background scan chunk.
        if !has_analysis_work(&pending) {
            drop(pending);
            if let Some(scan) = scanner.as_mut() {
                let status = status_tracker.transition(AnalysisPhase::Indexing, Some(AnalysisStep::Discovering));
                let _ = event_tx.send(AnalysisEvent::Status(status));
                let batch = scan.step_with_counters(ScanBudget::default(), Some(&shared.counters));
                let scan_complete = !scan.has_work();
                let scan_env = ScanEnv {
                    source_cache: source_cache.as_ref(),
                    shared: &shared,
                    event_tx: &event_tx,
                };
                process_scan_batch(&scan_env, &mut compiler_workspace_state, scan.mode, batch, &mut discovered_files);
                let snap = shared.counters.snapshot();
                let status = status_tracker.update_counts(
                    snap.workspace_files_discovered,
                    discovered_files.len() as u64,
                    compiler_workspace_state.session.module_session().sources().len() as u64,
                );
                let _ = event_tx.send(AnalysisEvent::Status(status.clone()));
                let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                    session: status.session,
                    sequence: status.sequence,
                    level: AnalysisLogLevel::Verbose,
                    phase: AnalysisPhase::Indexing,
                    event: "scan.batch.completed".to_string(),
                    epoch: Some(shared.epoch.load(Ordering::Acquire)),
                    generation: None,
                    uri: None,
                    revision: None,
                    batch_size: None,
                    duration_ms: None,
                    message: Some(format!("indexed files total: {}", discovered_files.len())),
                    counters: Some(snap),
                })));
                if scan_complete {
                    scanner = None;
                    shared.scan_in_progress.store(false, Ordering::SeqCst);
                    shared.condvar.notify_all();
                }
            }
            if scanner.is_none() && status_tracker.snapshot().phase != AnalysisPhase::Ready {
                let status = status_tracker.transition(AnalysisPhase::Ready, None);
                let _ = event_tx.send(AnalysisEvent::Status(status));
            }
            continue;
        }

        // Shallow discovery feeds hover/navigation and import-closure
        // resolution. Give it one bounded turn before deep analysis of an
        // open document; otherwise a large first semantic batch can leave
        // cross-file declarations unavailable to interactive queries.
        if scanner.is_some() && !pending.file_updates.is_empty() {
            drop(pending);
            if let Some(scan) = scanner.as_mut() {
                let batch = scan.step_with_counters(ScanBudget::default(), Some(&shared.counters));
                let scan_complete = !scan.has_work();
                let scan_env = ScanEnv {
                    source_cache: source_cache.as_ref(),
                    shared: &shared,
                    event_tx: &event_tx,
                };
                process_scan_batch(&scan_env, &mut compiler_workspace_state, scan.mode, batch, &mut discovered_files);
                let snap = shared.counters.snapshot();
                let status = status_tracker.update_counts(
                    snap.workspace_files_discovered,
                    discovered_files.len() as u64,
                    compiler_workspace_state.session.module_session().sources().len() as u64,
                );
                let _ = event_tx.send(AnalysisEvent::Status(status));
                if scan_complete {
                    scanner = None;
                    shared.scan_in_progress.store(false, Ordering::SeqCst);
                    shared.condvar.notify_all();
                }
            }
            continue;
        }

        let is_first_time = compiler_workspace_state.session.last_snapshot().is_none() || pending.full_workspace_rebuild_requested;

        if !is_first_time {
            // Debounce / edit coalescing: wait for edits to settle
            let timeout = AnalysisService::DEBOUNCE_DURATION;
            let result = shared.condvar.wait_timeout(pending, timeout).expect("worker condvar wait_timeout poisoned");
            pending = result.0;
        }

        // Process pending work batch
        if has_analysis_work(&pending) && !shared.shutdown.load(Ordering::SeqCst) {
            // Take snapshot of work batch under lock
            let batch_epoch = shared.epoch.load(Ordering::SeqCst);

            pending.is_processing = true;
            let mut file_updates = std::mem::take(&mut pending.file_updates);
            let mut removals = std::mem::take(&mut pending.removals);
            let disk_refreshes = std::mem::take(&mut pending.disk_refreshes);
            let _full_rebuild = pending.full_workspace_rebuild_requested;
            pending.full_workspace_rebuild_requested = false;

            if file_updates.len() == 1 {
                status_tracker.set_current_uri(file_updates.keys().next().cloned());
            } else {
                status_tracker.set_current_uri(None);
            }
            let status = status_tracker.transition(AnalysisPhase::Analyzing, Some(AnalysisStep::FlowAnalysis));
            let _ = event_tx.send(AnalysisEvent::Status(status.clone()));
            let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                session: status.session,
                sequence: status.sequence,
                level: AnalysisLogLevel::Info,
                phase: AnalysisPhase::Analyzing,
                event: "semantic.batch.started".to_string(),
                epoch: Some(batch_epoch),
                generation: None,
                uri: status_tracker.snapshot().current_uri,
                revision: None,
                batch_size: Some(file_updates.len() as u32),
                duration_ms: None,
                message: Some("semantic batch started".to_string()),
                counters: Some(shared.counters.snapshot()),
            })));

            // Release lock during heavy semantic execution
            drop(pending);

            #[cfg(test)]
            if let Some(gate) = shared.test_batch_gate.lock().expect("test gate lock poisoned").clone() {
                gate.wait_before();
            }

            shared.counters.semantic_batches_started.fetch_add(1, Ordering::Relaxed);
            let _span = PerfSpan::start_with_context_and_counters(
                "semantic_batch",
                PerfContext {
                    generation: compiler_workspace_state.session.last_snapshot().map(|snapshot| snapshot.generation),
                    epoch: Some(batch_epoch),
                },
                shared.counters.clone(),
            );

            let mut latest_generation = compiler_workspace_state.session.last_snapshot().map_or(0, |snapshot| snapshot.generation);
            let scan_env = ScanEnv {
                source_cache: source_cache.as_ref(),
                shared: &shared,
                event_tx: &event_tx,
            };
            let mut delta = DiskRefreshDelta {
                file_updates: &mut file_updates,
                removals: &mut removals,
            };
            refresh_disk_sources(&scan_env, disk_refreshes, &mut delta);
            let batch = file_updates;
            let cancelled = || shared.shutdown.load(Ordering::SeqCst) || shared.epoch.load(Ordering::Acquire) != batch_epoch;
            let _span = PerfSpan::start_with_context_and_counters(
                "semantic_solve_flow_publish",
                PerfContext {
                    generation: Some(latest_generation),
                    epoch: Some(batch_epoch),
                },
                shared.counters.clone(),
            );
            let mut mutations = Vec::new();
            for uri in removals {
                if let Some(source) = source_location_for_uri(&uri) {
                    mutations.push(phalcom_modules::WorkspaceSourceBatchMutation::RemoveSource { source: source.source_id });
                }
            }
            for (uri, update) in batch {
                let Some(source) = source_location_for_uri(&uri) else { continue };
                mutations.push(phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                    source,
                    text: update.text,
                    revision: update.revision,
                    recovered_program: Some(update.program),
                });
            }
            let publication_result = compiler_workspace_state.session.apply_module_mutations(mutations);
            let mut effects = PublicationEffects::default();
            let mut publication_failed = false;
            match publication_result {
                Ok(publication_result) => {
                    latest_generation = publication_result.snapshot.generation;
                    if !cancelled() {
                        effects = publication_effects_from_compiler(&publication_result.effects);
                        publication.publish(publication_result.snapshot);
                        status_tracker.set_generation(latest_generation);
                        let status = status_tracker.transition(AnalysisPhase::Publishing, None);
                        let _ = event_tx.send(AnalysisEvent::Status(status));
                    }
                }
                Err(err) => {
                    publication_failed = true;
                    let _ = event_tx.send(AnalysisEvent::Error {
                        message: format!("module mutation application failed: {err:?}"),
                    });
                    let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                        session: status_tracker.snapshot().session,
                        sequence: status_tracker.snapshot().sequence,
                        level: AnalysisLogLevel::Error,
                        phase: AnalysisPhase::Analyzing,
                        event: "semantic.batch.error".to_string(),
                        epoch: Some(batch_epoch),
                        generation: None,
                        uri: None,
                        revision: None,
                        batch_size: None,
                        duration_ms: None,
                        message: Some(format!("batch for epoch {batch_epoch} encountered infrastructure error: {err:?}")),
                        counters: Some(shared.counters.snapshot()),
                    })));
                }
            }

            // Epoch staleness check: if newer edits were enqueued during execution, discard intermediate result as stale
            let current_epoch = shared.epoch.load(Ordering::SeqCst);
            if publication_failed || cancelled() || current_epoch > batch_epoch {
                shared.counters.stale_batches_discarded.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::StaleBatchDiscarded { epoch: batch_epoch });
                let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                    session: status_tracker.snapshot().session,
                    sequence: status_tracker.snapshot().sequence,
                    level: AnalysisLogLevel::Info,
                    phase: AnalysisPhase::Analyzing,
                    event: "semantic.batch.cancelled".to_string(),
                    epoch: Some(batch_epoch),
                    generation: None,
                    uri: None,
                    revision: None,
                    batch_size: None,
                    duration_ms: None,
                    message: Some(format!("batch for epoch {} cancelled or superseded by {}", batch_epoch, current_epoch)),
                    counters: Some(shared.counters.snapshot()),
                })));
            } else {
                shared.counters.semantic_batches_published.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::Published {
                    generation: latest_generation,
                    effects,
                });
                let snap = shared.counters.snapshot();
                status_tracker.update_counts(
                    snap.workspace_files_discovered,
                    discovered_files.len() as u64,
                    compiler_workspace_state.session.module_session().sources().len() as u64,
                );
                let _ = event_tx.send(AnalysisEvent::Log(Box::new(AnalysisLogEvent {
                    session: status_tracker.snapshot().session,
                    sequence: status_tracker.snapshot().sequence,
                    level: AnalysisLogLevel::Info,
                    phase: AnalysisPhase::Publishing,
                    event: "snapshot.published".to_string(),
                    epoch: Some(batch_epoch),
                    generation: Some(latest_generation),
                    uri: None,
                    revision: None,
                    batch_size: None,
                    duration_ms: None,
                    message: Some(format!("published snapshot generation {latest_generation}")),
                    counters: Some(snap),
                })));
            }

            // Re-acquire lock and notify any flush callers
            pending = shared.pending.lock().expect("worker pending lock poisoned");
            pending.is_processing = false;
            let scanner_active = shared.scan_in_progress.load(Ordering::SeqCst) || pending.workspace_scan.is_some();
            let pending_newer_work = has_analysis_work(&pending);
            let final_status = finish_status_after_batch(&mut status_tracker, scanner_active, pending_newer_work);
            let _ = event_tx.send(AnalysisEvent::Status(final_status));
            shared.condvar.notify_all();
        }
    }
}

fn finish_status_after_batch(tracker: &mut StatusTracker, scanner_active: bool, pending_newer_work: bool) -> AnalysisStatus {
    if pending_newer_work {
        tracker.transition(AnalysisPhase::Analyzing, Some(AnalysisStep::FlowAnalysis))
    } else if scanner_active {
        tracker.transition(AnalysisPhase::Indexing, Some(AnalysisStep::Discovering))
    } else {
        tracker.transition(AnalysisPhase::Ready, None)
    }
}

fn has_analysis_work(pending: &PendingWork) -> bool {
    !pending.file_updates.is_empty() || !pending.removals.is_empty() || !pending.disk_refreshes.is_empty() || pending.full_workspace_rebuild_requested
}

#[derive(Debug, Default)]
struct CompilerWorkspaceState {
    session: phalcom_semantic::SemanticWorkspaceSession,
}

pub(crate) fn builtin_module_from_uri(uri: &Url) -> Option<phalcom_modules::ModuleId> {
    phalcom_modules::universe_module_from_uri(uri.as_str())
}

fn publication_effects_from_compiler(effects: &phalcom_semantic::SemanticPublicationEffects) -> PublicationEffects {
    PublicationEffects {
        inlay_hints_changed: !effects.formal_changed.is_empty() || !effects.advisory_changed.is_empty(),
        semantic_tokens_changed: !effects.source_index_changed.is_empty(),
    }
}

fn source_epoch(shared: &WorkerShared, uri: &Url) -> u64 {
    shared
        .source_epochs
        .lock()
        .expect("source epoch lock poisoned")
        .get(uri)
        .copied()
        .unwrap_or_default()
}

fn is_open_source(shared: &WorkerShared, uri: &Url) -> bool {
    shared.open_documents.lock().expect("open document lock poisoned").contains(uri)
}

struct ScanEnv<'a> {
    source_cache: Option<&'a SourceCache>,
    shared: &'a WorkerShared,
    event_tx: &'a mpsc::UnboundedSender<AnalysisEvent>,
}

struct DiskRefreshDelta<'a> {
    file_updates: &'a mut BTreeMap<Url, PendingSourceUpdate>,
    removals: &'a mut BTreeSet<Url>,
}

fn refresh_disk_sources(env: &ScanEnv<'_>, refreshes: BTreeSet<Url>, delta: &mut DiskRefreshDelta<'_>) {
    for uri in refreshes {
        let ticket = env.shared.epoch.load(Ordering::Acquire);
        let source_ticket = source_epoch(env.shared, &uri);
        if is_open_source(env.shared, &uri) {
            continue;
        }
        let Ok(path) = uri.to_file_path() else {
            delta.removals.insert(uri);
            continue;
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            if env.shared.epoch.load(Ordering::Acquire) != ticket || source_epoch(env.shared, &uri) != source_ticket || is_open_source(env.shared, &uri) {
                continue;
            }
            if let Some(cache) = env.source_cache {
                cache.write().expect("closed source cache lock poisoned").remove(&canonical_uri(&uri));
            }
            delta.removals.insert(uri.clone());
            let _ = env.event_tx.send(AnalysisEvent::WorkspaceFileRemoved { uri });
            continue;
        };
        let parse = phalcom_ast::parser::parse(&text, 0);
        if env.shared.epoch.load(Ordering::Acquire) != ticket || source_epoch(env.shared, &uri) != source_ticket || is_open_source(env.shared, &uri) {
            continue;
        }
        let source_text: Arc<str> = Arc::from(text.as_str());
        let program = Arc::new(parse.program);
        let revision = SourceRevision(1);
        if let Some(cache) = env.source_cache {
            cache.write().expect("closed source cache lock poisoned").insert(
                canonical_uri(&uri),
                CachedSource {
                    line_index: Arc::new(LineIndex::new(&text)),
                    text: Arc::from(text.clone()),
                    program: program.clone(),
                },
            );
        }
        let _ = env.event_tx.send(AnalysisEvent::WorkspaceFileIndexed {
            uri: uri.clone(),
            text: source_text.clone(),
            revision,
        });
        delta.file_updates.insert(
            uri,
            PendingSourceUpdate {
                revision,
                text: source_text,
                program,
            },
        );
    }
}

fn process_scan_batch(
    env: &ScanEnv<'_>,
    identity: &mut CompilerWorkspaceState,
    _mode: AnalysisMode,
    files: Vec<crate::workspace_scan::DiscoveredFile>,
    discovered_files: &mut BTreeSet<Url>,
) {
    #[cfg(test)]
    if let Some(gate) = env.shared.test_scan_gate.lock().expect("test scan gate lock poisoned").clone() {
        gate.wait();
    }

    let mut semantic_files = Vec::new();
    for discovered in files {
        let ticket = env.shared.epoch.load(Ordering::Acquire);
        let source_ticket = source_epoch(env.shared, &discovered.uri);
        if is_open_source(env.shared, &discovered.uri) {
            env.shared.counters.scan_results_discarded_for_open_document.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&discovered.path) else {
            continue;
        };
        if env.shared.epoch.load(Ordering::Acquire) != ticket
            || source_epoch(env.shared, &discovered.uri) != source_ticket
            || is_open_source(env.shared, &discovered.uri)
        {
            if is_open_source(env.shared, &discovered.uri) {
                env.shared.counters.scan_results_discarded_for_open_document.fetch_add(1, Ordering::Relaxed);
            } else {
                env.shared.counters.scan_results_discarded_as_stale.fetch_add(1, Ordering::Relaxed);
            }
            continue;
        }
        let _span = PerfSpan::start_with_context_and_counters(
            "workspace_source_parse",
            PerfContext {
                generation: identity.session.last_snapshot().map(|snapshot| snapshot.generation),
                epoch: Some(env.shared.epoch.load(Ordering::Acquire)),
            },
            env.shared.counters.clone(),
        );
        let parse = phalcom_ast::parser::parse(&text, 0);
        let source_text: Arc<str> = Arc::from(text.as_str());
        let program = Arc::new(parse.program);
        let revision = SourceRevision(1);
        if let Some(cache) = env.source_cache {
            cache.write().expect("closed source cache lock poisoned").insert(
                canonical_uri(&discovered.uri),
                CachedSource {
                    line_index: Arc::new(LineIndex::new(&text)),
                    text: source_text.clone(),
                    program: program.clone(),
                },
            );
        }
        discovered_files.insert(canonical_uri(&discovered.uri));
        env.shared.counters.workspace_files_discovered.fetch_add(1, Ordering::Relaxed);
        env.shared.counters.workspace_files_parsed.fetch_add(1, Ordering::Relaxed);
        let _ = env.event_tx.send(AnalysisEvent::WorkspaceFileIndexed {
            uri: discovered.uri.clone(),
            text: source_text.clone(),
            revision,
        });
        // Publish discovered disk sources through the canonical session in
        // both modes. Local mode limits scheduling policy; it must not create
        // a second shallow semantic world for imported declarations.
        semantic_files.push((discovered.uri, revision, source_text, (*program).clone()));
    }
    if !semantic_files.is_empty() {
        let _span = PerfSpan::start_with_context_and_counters(
            "scan_semantic_publish",
            PerfContext {
                generation: identity.session.last_snapshot().map(|snapshot| snapshot.generation),
                epoch: Some(env.shared.epoch.load(Ordering::Acquire)),
            },
            env.shared.counters.clone(),
        );
        let mutations = semantic_files.into_iter().filter_map(|(uri, revision, text, program)| {
            let source = source_location_for_uri(&uri)?;
            Some(phalcom_modules::WorkspaceSourceBatchMutation::SetDiskSnapshot {
                source,
                text,
                revision: phalcom_modules::SourceRevision(revision.0),
                recovered_program: Some(Arc::new(program)),
            })
        });
        if let Ok(publication) = identity.session.apply_module_mutations(mutations) {
            let generation = publication.snapshot.generation;
            let effects = publication_effects_from_compiler(&publication.effects);
            env.shared.counters.scan_batches_published.fetch_add(1, Ordering::Relaxed);
            let _ = env.event_tx.send(AnalysisEvent::Published { generation, effects });
        }
    }
}

fn canonical_uri(uri: &Url) -> Url {
    uri.to_file_path()
        .ok()
        .and_then(|path| {
            path.canonicalize().ok().or_else(|| {
                let file_name = path.file_name()?.to_owned();
                let parent = path.parent()?.canonicalize().ok()?;
                Some(parent.join(file_name))
            })
        })
        .and_then(|path| Url::from_file_path(path).ok())
        .unwrap_or_else(|| uri.clone())
}
