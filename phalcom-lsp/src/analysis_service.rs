//! Asynchronous semantic analysis service with latest-wins edit coalescing and background worker thread.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use phalcom_ast::ast::{Program, Statement};
use tokio::sync::mpsc;
use tower_lsp::lsp_types::Url;

use crate::index::WorkspaceIndex;
use crate::line_index::LineIndex;
use crate::perf::COUNTERS;
use crate::semantic::{FileRevision, SemanticDb, SemanticGeneration};
use crate::workspace_scan::{AnalysisMode, ExcludeMatcher, ScanBudget, WorkspaceScanState};

/// Closed-file source metadata populated by the worker before it publishes
/// semantic state. Query handlers can read this cache without waiting for the
/// asynchronous event notification task.
#[derive(Clone)]
pub(crate) struct CachedSource {
    pub(crate) text: Arc<str>,
    pub(crate) program: Arc<Program>,
    pub(crate) line_index: Arc<LineIndex>,
}

pub(crate) type SourceCache = Arc<RwLock<BTreeMap<Url, CachedSource>>>;

/// Latest workspace discovery configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceScanRequest {
    /// Filesystem roots to discover.
    pub roots: Vec<PathBuf>,
    /// Deep-analysis policy for discovered files.
    pub mode: AnalysisMode,
    /// User-configured path exclusions.
    pub excludes: Vec<String>,
    /// Selected physical core source, kept out of ordinary indexing.
    pub core_source_path: Option<PathBuf>,
}

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
    /// Latest workspace scan request. New roots/config replace older requests.
    pub workspace_scan: Option<WorkspaceScanRequest>,
}

impl PendingWork {
    fn is_idle(&self) -> bool {
        self.file_updates.is_empty()
            && self.core_update.is_none()
            && self.removals.is_empty()
            && !self.full_workspace_rebuild_requested
            && !self.is_processing
            && self.workspace_scan.is_none()
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.file_updates.clear();
        self.core_update = None;
        self.removals.clear();
        self.full_workspace_rebuild_requested = false;
        self.is_processing = false;
        self.workspace_scan = None;
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
    /// Open documents whose live buffers have priority over disk discovery.
    pub open_documents: Mutex<BTreeSet<Url>>,
    /// True while a configured workspace scan still has undiscovered work.
    pub scan_in_progress: AtomicBool,
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
    /// A source file was shallow-indexed by progressive workspace discovery.
    WorkspaceFileIndexed {
        /// URL of the discovered file.
        uri: Url,
        /// Source text retained for closed-file LSP metadata queries.
        text: Arc<str>,
        /// Revision assigned to the cached source snapshot.
        revision: FileRevision,
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
        Self::new_with_index(db, None)
    }

    /// Creates an analysis service that also maintains a concurrent shallow
    /// workspace index while scanning.
    pub fn new_with_index(db: Arc<SemanticDb>, workspace_index: Option<Arc<WorkspaceIndex>>) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
        Self::new_with_index_and_cache(db, workspace_index, None)
    }

    /// Creates an analysis service with a worker-owned closed-source cache.
    pub(crate) fn new_with_index_and_cache(
        db: Arc<SemanticDb>,
        workspace_index: Option<Arc<WorkspaceIndex>>,
        source_cache: Option<SourceCache>,
    ) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let shared = Arc::new(WorkerShared {
            epoch: AtomicU64::new(0),
            pending: Mutex::new(PendingWork::default()),
            condvar: Condvar::new(),
            shutdown: AtomicBool::new(false),
            open_documents: Mutex::new(BTreeSet::new()),
            scan_in_progress: AtomicBool::new(false),
        });

        let db_clone = db.clone();
        let index_clone = workspace_index.clone();
        let source_cache_clone = source_cache.clone();
        let shared_clone = shared.clone();
        let event_tx_clone = event_tx.clone();

        let worker_thread = thread::Builder::new()
            .name("phalcom-lsp-analyzer".to_string())
            .spawn(move || {
                worker_loop(db_clone, index_clone, source_cache_clone, shared_clone, event_tx_clone);
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
        if !self.accepts_revision(&pending, &uri, revision) {
            return;
        }
        pending.removals.remove(&uri);
        if pending.file_updates.insert(uri, (revision, program)).is_some() {
            COUNTERS.source_updates_coalesced.fetch_add(1, Ordering::Relaxed);
        }
        COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues one coalesced batch of source replacements.
    pub fn enqueue_file_updates(&self, updates: Vec<(Url, FileRevision, Program)>) {
        if updates.is_empty() {
            return;
        }
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        for (uri, revision, program) in updates {
            if !self.accepts_revision(&pending, &uri, revision) {
                continue;
            }
            pending.removals.remove(&uri);
            if pending.file_updates.insert(uri, (revision, program)).is_some() {
                COUNTERS.source_updates_coalesced.fetch_add(1, Ordering::Relaxed);
            }
            COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        }
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    /// Enqueues active core module replacement.
    pub fn enqueue_core_update(&self, revision: FileRevision, program: Program) {
        let uri = Url::parse(crate::semantic::CORE_MODULE_URI).expect("core module URI must parse");
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        if !self.accepts_revision(&pending, &uri, revision) {
            return;
        }
        pending.core_update = Some((revision, program));
        COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);
        self.shared.condvar.notify_all();
    }

    fn accepts_revision(&self, pending: &PendingWork, uri: &Url, revision: FileRevision) -> bool {
        if pending.file_updates.get(uri).is_some_and(|(queued, _)| *queued >= revision) {
            return false;
        }
        if self.db.file_snapshot(uri).is_some_and(|file| file.revision >= revision) {
            return false;
        }
        if uri.as_str() == crate::semantic::CORE_MODULE_URI && pending.core_update.as_ref().is_some_and(|(queued, _)| *queued >= revision) {
            return false;
        }
        true
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

    /// Enqueues one coalesced batch of source removals.
    pub fn enqueue_file_removals(&self, uris: Vec<Url>) {
        if uris.is_empty() {
            return;
        }
        let mut pending = self.shared.pending.lock().expect("worker pending lock poisoned");
        for uri in uris {
            pending.file_updates.remove(&uri);
            pending.removals.insert(uri);
            COUNTERS.source_updates_enqueued.fetch_add(1, Ordering::Relaxed);
        }
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
        self.shared.open_documents.lock().expect("open document lock poisoned").insert(uri);
    }

    /// Allows subsequent disk discovery for a closed document.
    pub fn mark_closed(&self, uri: &Url) {
        self.shared.open_documents.lock().expect("open document lock poisoned").remove(uri);
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
    workspace_index: Option<Arc<WorkspaceIndex>>,
    source_cache: Option<SourceCache>,
    shared: Arc<WorkerShared>,
    event_tx: mpsc::UnboundedSender<AnalysisEvent>,
) {
    let mut scanner = None;
    let mut selected_core_uri = None;
    let mut core_initialized = false;
    let mut analysis_mode = AnalysisMode::Local;
    let mut source_catalog = BTreeMap::new();
    let mut workspace_roots = Vec::new();
    let mut configured_sysroot = None;

    loop {
        let mut pending = shared.pending.lock().expect("worker pending lock poisoned");

        while !has_analysis_work(&pending) && pending.workspace_scan.is_none() && scanner.is_none() && !shared.shutdown.load(Ordering::SeqCst) {
            pending = shared.condvar.wait(pending).expect("worker condvar wait poisoned");
        }

        if shared.shutdown.load(Ordering::SeqCst) {
            break;
        }

        let mut core_reselect = false;
        if let Some(request) = pending.workspace_scan.take() {
            let mut next = WorkspaceScanState::new(request.mode, ExcludeMatcher::new(&request.excludes));
            next.set_roots(request.roots.clone(), request.core_source_path.clone());
            analysis_mode = request.mode;
            scanner = Some(next);
            workspace_roots = request.roots;
            if configured_sysroot != request.core_source_path {
                configured_sysroot = request.core_source_path;
                core_reselect = true;
            }
        }

        // Interactive semantic work always wins over one background scan chunk.
        if !has_analysis_work(&pending) {
            drop(pending);
            if core_reselect || !core_initialized {
                let core_source = crate::semantic::core_source::CoreSource::select(configured_sysroot.as_deref(), &workspace_roots);
                selected_core_uri = core_source.physical_uri().cloned();
                let program = phalcom_ast::parser::parse(core_source.text(), 0).program;
                let generation = db.update_core(FileRevision(1), &program);
                core_initialized = true;
                let _ = event_tx.send(AnalysisEvent::Published { generation });
                continue;
            }
            if let Some(scan) = scanner.as_mut() {
                let batch = scan.step(ScanBudget::default());
                process_scan_batch(
                    &db,
                    workspace_index.as_ref(),
                    source_cache.as_ref(),
                    &shared,
                    &event_tx,
                    scan.mode,
                    batch,
                    &mut source_catalog,
                    selected_core_uri.as_ref(),
                );
                if !scan.has_work() {
                    scanner = None;
                    shared.scan_in_progress.store(false, Ordering::SeqCst);
                    shared.condvar.notify_all();
                }
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
                let batch = scan.step(ScanBudget::default());
                process_scan_batch(
                    &db,
                    workspace_index.as_ref(),
                    source_cache.as_ref(),
                    &shared,
                    &event_tx,
                    scan.mode,
                    batch,
                    &mut source_catalog,
                    selected_core_uri.as_ref(),
                );
                if !scan.has_work() {
                    scanner = None;
                    shared.scan_in_progress.store(false, Ordering::SeqCst);
                    shared.condvar.notify_all();
                }
            }
            continue;
        }

        let is_first_time =
            pending.file_updates.keys().any(|uri| db.file_snapshot(uri).is_none()) || pending.core_update.is_some() || pending.full_workspace_rebuild_requested;

        if !is_first_time {
            // Debounce / edit coalescing: wait for edits to settle
            let timeout = AnalysisService::DEBOUNCE_DURATION;
            let result = shared.condvar.wait_timeout(pending, timeout).expect("worker condvar wait_timeout poisoned");
            pending = result.0;
        }

        // Process pending work batch
        while has_analysis_work(&pending) && !shared.shutdown.load(Ordering::SeqCst) {
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
let _span = crate::perf::PerfSpan::start("semantic_batch");

            let mut latest_generation = db.generation();
            let mut next_source_catalog = source_catalog.clone();
            for uri in &removals {
                next_source_catalog.remove(&canonical_uri(uri));
            }
            let mut batch = Vec::new();
            let mut seen = BTreeSet::new();
            for (uri, (revision, program)) in file_updates {
                next_source_catalog.insert(canonical_uri(&uri), (revision, program.clone()));
                seen.insert(uri.clone());
                batch.push((uri.clone(), revision, program.clone()));
                if analysis_mode == AnalysisMode::Local {
                    extend_import_closure(&uri, &program, &next_source_catalog, &mut seen, &mut batch);
                }
            }
            let cancelled = || shared.shutdown.load(Ordering::SeqCst) || shared.epoch.load(Ordering::Acquire) != batch_epoch;
            let generation = db.apply_mutations_with_cancel(removals.into_iter().collect(), batch, core_update, &cancelled);
            let solve_cancelled = generation.is_none();
            if let Some(generation) = generation {
                latest_generation = generation;
                source_catalog = next_source_catalog;
            }

            // Epoch staleness check: if newer edits were enqueued during execution, discard intermediate result as stale
            let current_epoch = shared.epoch.load(Ordering::SeqCst);
            if solve_cancelled || current_epoch > batch_epoch {
                COUNTERS.stale_batches_discarded.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::StaleBatchDiscarded { epoch: batch_epoch });
            } else {
                COUNTERS.semantic_batches_published.fetch_add(1, Ordering::Relaxed);
                let _ = event_tx.send(AnalysisEvent::Published { generation: latest_generation });
            }

            // Re-acquire lock and notify any flush callers
            pending = shared.pending.lock().expect("worker pending lock poisoned");
            pending.is_processing = false;
            shared.condvar.notify_all();
            break;
        }
    }
}

fn has_analysis_work(pending: &PendingWork) -> bool {
    !pending.file_updates.is_empty() || pending.core_update.is_some() || !pending.removals.is_empty() || pending.full_workspace_rebuild_requested
}

fn process_scan_batch(
    db: &SemanticDb,
    workspace_index: Option<&Arc<WorkspaceIndex>>,
    source_cache: Option<&SourceCache>,
    shared: &WorkerShared,
    event_tx: &mpsc::UnboundedSender<AnalysisEvent>,
    mode: AnalysisMode,
    files: Vec<crate::workspace_scan::DiscoveredFile>,
    source_catalog: &mut BTreeMap<Url, (FileRevision, Program)>,
    selected_core_uri: Option<&Url>,
) {
    let mut semantic_files = Vec::new();
    for discovered in files {
        if Some(&discovered.uri) == selected_core_uri {
            continue;
        }
        if shared.open_documents.lock().expect("open document lock poisoned").contains(&discovered.uri) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&discovered.path) else {
            continue;
        };
        let parse = phalcom_ast::parser::parse(&text, 0);
        let program = Arc::new(parse.program);
        let revision = db
            .file_snapshot(&discovered.uri)
            .map_or(FileRevision(1), |file| FileRevision(file.revision.0.saturating_add(1)));
        if let Some(index) = workspace_index {
            index.update_file(discovered.uri.clone(), &program);
        }
        if let Some(cache) = source_cache {
            cache.write().expect("closed source cache lock poisoned").insert(
                canonical_uri(&discovered.uri),
                CachedSource {
                    line_index: Arc::new(LineIndex::new(&text)),
                    text: Arc::from(text.clone()),
                    program: program.clone(),
                },
            );
        }
        source_catalog.insert(canonical_uri(&discovered.uri), (revision, (*program).clone()));
        COUNTERS.workspace_files_discovered.fetch_add(1, Ordering::Relaxed);
        COUNTERS.workspace_files_parsed.fetch_add(1, Ordering::Relaxed);
        let _ = event_tx.send(AnalysisEvent::WorkspaceFileIndexed {
            uri: discovered.uri.clone(),
            text: Arc::from(text),
            revision,
        });
        if mode == AnalysisMode::Workspace {
            semantic_files.push((discovered.uri, revision, (*program).clone()));
        }
    }
    if !semantic_files.is_empty() {
        let generation = db.update_files_batch(semantic_files);
        let _ = event_tx.send(AnalysisEvent::Published { generation });
    }
    if mode == AnalysisMode::Local {
        let open_documents = shared.open_documents.lock().expect("open document lock poisoned").clone();
        let mut batch = Vec::new();
        let mut seen = BTreeSet::new();
        for uri in open_documents {
            let catalog_uri = canonical_uri(&uri);
            let Some((revision, program)) = source_catalog.get(&catalog_uri) else {
                continue;
            };
            if seen.insert(uri.clone()) {
                batch.push((uri.clone(), *revision, program.clone()));
                extend_import_closure(&uri, program, source_catalog, &mut seen, &mut batch);
            }
        }
        if !batch.is_empty() {
            let generation = db.update_files_batch(batch);
            let _ = event_tx.send(AnalysisEvent::Published { generation });
        }
    }
}

fn extend_import_closure(
    uri: &Url,
    program: &Program,
    source_catalog: &BTreeMap<Url, (FileRevision, Program)>,
    seen: &mut BTreeSet<Url>,
    batch: &mut Vec<(Url, FileRevision, Program)>,
) {
    for statement in &program.statements {
        let Statement::Import(import) = statement else {
            continue;
        };
        let Some(import_uri) = resolve_source_import(uri, &import.path) else {
            continue;
        };
        let Some((revision, imported_program)) = source_catalog.get(&import_uri) else {
            continue;
        };
        if seen.insert(import_uri.clone()) {
            batch.push((import_uri.clone(), *revision, imported_program.clone()));
            extend_import_closure(&import_uri, imported_program, source_catalog, seen, batch);
        }
    }
}

fn resolve_source_import(uri: &Url, import: &str) -> Option<Url> {
    let source = uri.to_file_path().ok()?;
    let mut candidate = source.parent()?.join(import);
    if candidate.extension().is_none() {
        candidate.set_extension("ph");
    }
    Url::from_file_path(candidate.canonicalize().ok()?).ok()
}

fn canonical_uri(uri: &Url) -> Url {
    uri.to_file_path()
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .and_then(|path| Url::from_file_path(path).ok())
        .unwrap_or_else(|| uri.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;
    use std::fs;
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

    #[test]
    fn local_workspace_scan_indexes_closed_files_without_deep_analysis() {
        let root = std::env::temp_dir().join(format!("phalcom_lsp_scan_service_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("closed.ph");
        fs::write(&path, "class Closed { marker() {} }\n").unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let db = Arc::new(SemanticDb::new());
        let index = Arc::new(WorkspaceIndex::new());
        let (service, _rx) = AnalysisService::new_with_index(db.clone(), Some(index.clone()));

        service.configure_workspace(WorkspaceScanRequest {
            roots: vec![root.clone()],
            mode: AnalysisMode::Local,
            excludes: Vec::new(),
            core_source_path: None,
        });
        service.flush();

        assert!(!index.symbols_matching("marker").is_empty());
        assert!(db.file_snapshot(&uri).is_none());
        service.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}
