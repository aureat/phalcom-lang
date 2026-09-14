# Phalcom LSP Asynchronous Performance Architecture — Implementation Spec

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert `phalcom-lsp` from a synchronous batch-style semantic analyzer into a responsive asynchronous language server with immutable published semantic generations, coalesced background analysis, progressive workspace discovery, substantially lower semantic CPU cost, and precise incremental invalidation.

**Architecture:** LSP handlers own only fast live-source work and never run interprocedural semantic solving. A dedicated analysis worker owns all mutable semantic state, coalesces source revisions, performs workspace and semantic work cooperatively, and atomically publishes immutable `Arc<SemanticSnapshot>` generations. Hover, completion, navigation, and inlay-hint requests read the latest compatible published snapshot without waiting for in-flight analysis.

**Tech Stack:** Rust 2024, `tower-lsp 0.20`, Tokio 1, `std::thread`, `std::sync::{Arc, RwLock, Mutex, Condvar}`, existing `dashmap`, Phalcom AST/common/native-surface crates, VS Code `vscode-languageclient 8.1.x`.

## Global Constraints

- Preserve all language semantics established by LSP Specs 1–4.
- Preserve `ValueShape` / `InferredValue` as advisory runtime/editor knowledge; do not turn them into the future Phalcom formal type system.
- Preserve module-qualified `ClassId`, `CallableId`, `FieldId`, dispatch side, exact semantic occurrences, lexical `BindingId`, and `ScopeId`.
- `BindingId` remains stable only within one file/source snapshot. Do not invent cross-reparse persistent binding identity in this performance phase.
- Preserve solver-bottom vs semantic `Unknown`.
- Do not execute the Phalcom VM in the LSP.
- Do not add `phalcom-core` as an LSP dependency.
- Do not add a new dependency for atomic snapshot publication in the first implementation. Use `RwLock<Arc<SemanticSnapshot>>`; the lock must never surround analysis.
- Do not use one `spawn_blocking` job per edit. There must be one owned semantic worker with latest-wins/coalescing semantics.
- Do not block `initialize`, hover, completion, inlay hints, shutdown, watched-file notifications, or workspace-folder handlers on deep semantic analysis.
- No LSP request may wait for “fresh” semantics. It must use current live syntax plus the latest safe published semantic generation.
- Syntax diagnostics remain immediate and independent of semantic convergence.
- Workspace bulk work must yield to open-document work.
- File-system watcher batches must be semantically batched.
- A physical `core.ph` must never coexist as both an ordinary file-qualified semantic module and the logical `phalcom://core` module.
- Source structures that are immutable for a file revision (`Program`, `ModuleSurface`, `ScopeGraph`, `OccurrenceIndex`) must not be rebuilt inside fixed-point rounds.
- Dispatch resolution must not deep-clone method bodies.
- The unified flow engine must not be re-run independently just to extract parameter, summary, local, and field products.
- Prefer named functions/types as the authoritative source anchors. Line ranges below are pinned to the inspected `main` baseline and will move as this work lands.

---

# 0. Baseline and source material

**Repository:** `aureat/phalcom-lang`  
**Inspected branch:** `main`  
**Inspected current commit:** `c384d18cea776f90b72f8111a1e14e7166435539`  
**Date:** 2026-08-13

This specification is a performance/incrementality successor to:

- `PHALCOM_LSP_ANALYSIS_DIAGNOSIS_AND_PLAN.md`
- `PHALCOM_LSP_SPEC_01_SOURCE_TARGETS_AND_SCOPES.md`
- `PHALCOM_LSP_SPEC_02_UNIFIED_INFERENCE_AND_DISPATCH.md`
- `PHALCOM_LSP_SPEC_03_FLOW_SUMMARIES_FIELDS_PARAMETERS.md`
- `PHALCOM_LSP_SPEC_04_LSP_INTEGRATION_TESTS_TYPING_BRIDGE.md`

The previous specs correctly established a coherent semantic model. In particular:

- Spec 1 introduced exact semantic targets, lexical scopes, `BindingId`, and `OccurrenceIndex`.
- Spec 2 consolidated expression analysis and canonical side-aware dispatch.
- Spec 3 required one structured flow traversal and retained fixed-point/invalidation machinery.
- Spec 4 required all editor features to consume `SemanticDb` consistently and included basic performance gates.

This specification changes **execution architecture**, not Phalcom language semantics.

## 0.1 Reported measurements

The following measurements were supplied during the performance diagnosis and were **not independently re-benchmarked as part of writing this document**:

| Scenario | Reported debug build time |
|---|---:|
| no workspace | ~14.3 s |
| `examples/` — 209 `.ph` files | ~30.7 s |
| repository root — 1,096 `.ph` files | ~103 s |
| simple leaf edit after full scan | ~70 ms |

These numbers are useful as regression baselines, but the acceptance criteria in this document are primarily architectural and deterministic rather than fragile wall-clock-only tests.

---

# 1. Current performance diagnosis

## 1.1 Eager semantic core work happens before `initialize`

Current anchors:

- `phalcom-lsp/src/main.rs`
- `phalcom-lsp/src/backend.rs:~160-180` — `Backend::new`
- `phalcom-lsp/src/semantic/mod.rs:~160-200` — `SemanticDb::new`
- `phalcom-lsp/src/semantic/core_source.rs`

Current chain:

```text
LspService::new(Backend::new)
  -> Backend::new
     -> SemanticDb::new
        -> bundled_parse()
        -> update_core()
        -> update_file()
        -> update_files_batch()
        -> semantic fixed-point rebuild
```

This means the server can spend substantial CPU time before the LSP `initialize` request is even handled.

**Required change:** `Backend::new` and the published semantic database must be cheap. Core source selection and core analysis move to the analysis worker after initialization configuration/workspace roots are known.

---

## 1.2 `initialize` performs the whole workspace scan synchronously

Current anchors:

- `phalcom-lsp/src/backend.rs:~250-310` — `scan_workspace`
- `phalcom-lsp/src/backend.rs:~805-900` — `collect_ph_files*`, `initialize`

Current path:

```text
initialize
  -> discover every .ph
  -> read every file
  -> parse every file
  -> update legacy WorkspaceIndex
  -> SemanticDb::update_files_batch(all files)
  -> scan_core_source
  -> only then return InitializeResult
```

The scanner excludes hidden directories, `target`, and `node_modules`, but it does not distinguish deep-semantic source from tests, fixtures, examples, or unrelated Phalcom projects nested under a large repository root.

**Required change:** `initialize` records configuration and roots and returns capabilities immediately. Workspace discovery begins after initialization as background worker work.

---

## 1.3 `didChange` performs full semantic work before returning

Current anchors:

- `phalcom-lsp/src/documents.rs:~1-130`
- `phalcom-lsp/src/backend.rs:~180-205` — `publish_diagnostics_for`
- `phalcom-lsp/src/backend.rs:~990-1030` — `did_open`, `did_change`

Current path:

```text
didChange
  -> DocumentStore::open_or_update
     -> full parse
     -> LineIndex rebuild
  -> publish_diagnostics_for
     -> WorkspaceIndex::update_file
     -> optional second recovery parse
     -> SemanticDb::update_file
     -> fixed-point rebuild under write lock
     -> syntax diagnostics conversion
  -> publish diagnostics
```

Syntax diagnostics therefore wait behind semantic inference even though they only require the parser result.

**Required change:** separate live-source update/syntax diagnostics from deep semantic scheduling.

---

## 1.4 A DashMap document guard currently surrounds semantic solving

`DocumentStore::with_document` intentionally keeps the DashMap guard for the duration of its callback. `publish_diagnostics_for` invokes semantic work inside that callback.

`DocumentStore` already provides:

```rust
pub fn snapshot(&self, uri: &Url) -> Option<DocumentSnapshot>
```

which cheaply clones `Arc`s without retaining a map guard.

**Required change:** no semantic analysis may run inside `with_document`. Use `DocumentSnapshot` / immutable source snapshots.

---

## 1.5 Semantic updates hold one global write lock for the entire rebuild

Current anchor:

- `phalcom-lsp/src/semantic/mod.rs:~160-350`

Current state:

```rust
pub struct SemanticDb {
    state: RwLock<SemanticState>,
}
```

`update_files_batch` acquires `state.write()` before source reconstruction, graph updates, invalidation, fixed-point solving, local facts, field facts, parameter facts, and generation publication.

Every query uses the same state lock for reading.

Therefore:

```text
semantic writer ------------------------------+
                                              |
hover -> state.read()      waits -------------+
inlay -> state.read()      waits -------------+
completion -> state.read() waits -------------+
```

**Required change:** mutable working state and immutable published state must be separate objects. The publication lock may only protect an `Arc` pointer swap/load.

---

## 1.6 The unified flow engine is still executed multiple times

Current anchors:

- `phalcom-lsp/src/semantic/infer.rs:~1-340`
- `phalcom-lsp/src/semantic/flow.rs:~1-220`
- `phalcom-lsp/src/semantic/mod.rs:~680-860`

The implementation has a correct unified `flow::analyze_surface`, but wrappers still call it independently:

```text
parameter_facts_for_program -> analyze_surface
field_facts_for_surface     -> analyze_surface
collect_local_facts...      -> analyze_surface
summaries_for_surface...    -> analyze_surface
```

Inside a fixed-point round the affected solver runs at least:

```text
flow pass for parameters
flow pass for summaries
```

Then `rebuild_affected_state` runs further passes for parameter contributions, locals, and fields.

**Required change:** each semantic analysis unit returns all relevant products from one traversal. Never re-run the same AST flow merely to extract a different field from `SurfaceFlowAnalysis`.

---

## 1.7 Scope graphs are rebuilt inside flow analysis

Current `FileSemanticSnapshot` already stores a `ScopeGraph`, but `flow::analyze_surface` currently rebuilds the scope graph again.

**Required change:** source structures are constructed once per source revision and passed by reference/`Arc` into semantic analysis.

---

## 1.8 Global maps are cloned repeatedly

Current anchor:

- `phalcom-lsp/src/semantic/mod.rs:~680-860`
- `phalcom-lsp/src/semantic/infer.rs:~80-330`

Examples:

```rust
let previous_summaries = state.summaries.clone();
let previous_parameters = state.parameter_facts.clone();
let previous_dependents = state.callable_dependents.clone();
```

and every solver round clones summary/parameter state again.

The current solver also carries unaffected summaries in `seed_summaries`, so some “affected” rounds still clone large workspace-level maps.

**Required change:** first eliminate unnecessary full-pass cloning by partitioning immutable base state and affected overlays. Later move to callable-level worklist propagation so only changed slots are recomputed.

---

## 1.9 `MemberSurface` duplicates AST bodies and dispatch clones them

Current anchors:

- `phalcom-lsp/src/semantic/surface.rs:~1-160`
- `phalcom-lsp/src/semantic/dispatch.rs:~1-100`

Current:

```rust
pub struct MemberSurface {
    ...
    pub body: Vec<Statement>,
}
```

`build_module_surface` copies method bodies from the already-retained `Program`, and `ClassSurface` stores member surfaces in both `members` and `members_by_side`.

`DispatchResolver::resolve` then clones the resolved `MemberSurface`.

Because `MemberSurface` owns `Vec<Statement>`, a method dispatch can deep-clone an entire method body.

**Required change:** declaration surfaces become lightweight. AST bodies stay in the single canonical parsed program. Dispatch returns stable IDs/lightweight metadata, never owned AST bodies.

---

## 1.10 Global class state is reconstructed on ordinary edits

`rebuild_affected_state` currently rebuilds `state.classes` from every file surface during the affected rebuild loop.

A method-body-only edit should not reconstruct the global class universe.

**Required change:** update the class index incrementally when a module's declaration surface changes. Body-only edits leave it untouched.

---

## 1.11 Module reverse dependencies are not indexed

Current anchor:

- `phalcom-lsp/src/semantic/module_graph.rs:~1-120`

Current `dependents_of` scans all module edges. `dependent_closure` repeatedly invokes it.

`refresh_resolutions` walks every import edge in the workspace and is called from normal semantic updates even when available files did not change.

**Required change:** maintain explicit forward and reverse import maps. Re-resolve imports only for file-set/import-surface changes, not ordinary method-body edits.

---

## 1.12 `ModuleId::from_uri` performs file-system canonicalization in hot paths

Current anchor:

- `phalcom-lsp/src/semantic/ids.rs:~1-45`

Many query methods call `ModuleId::from_uri`, which may call `std::fs::canonicalize`.

**Required change:** resolve/canonicalize module identity once when a source enters the source catalog. Hot hover/completion/inlay queries use cached `ModuleId`.

---

## 1.13 Closed-file hover/navigation rereads and reparses files

Current anchor:

- `phalcom-lsp/src/backend.rs:~350-550` — `occurrence_to_location`, `with_source_snapshot`, `member_phaldoc`

Closed defining files are read, reparsed, and given a new `LineIndex` on request.

**Required change:** background source indexing retains compact closed-file source metadata sufficient for line mapping, declarations, and Phaldoc. Interactive queries perform no synchronous disk reads.

---

## 1.14 Configuration changes can rebuild core twice

Current anchor:

- `phalcom-lsp/src/backend.rs:~915-940` — `did_change_configuration`

Current code restores bundled core and then scans/rebuilds live core regardless of whether only presentation configuration changed.

**Required change:** diff configuration. Inlay presentation changes cause zero semantic analysis. Core is reselected only if core-source-relevant configuration changed.

---

## 1.15 `phalcom.analysis.mode` exists in package metadata but is not wired

Current anchors:

- `tools/vsphalcom/package.json:~90-110`
- `tools/vsphalcom/src/extension.ts:~30-60`
- `phalcom-lsp/src/backend.rs:~45-115` — `ServerConfig`

The setting is declared but not sent in initialization options and not represented by `ServerConfig`.

**Required change:** implement it end to end.

---

## 1.16 Watcher/workspace batches cause repeated semantic rebuilds

Current anchors:

- `phalcom-lsp/src/backend.rs:~940-1000`

`did_change_watched_files` loops each file and refreshes it individually. Workspace removal loops files and calls `remove_file` individually.

**Required change:** every incoming batch becomes one coalesced analysis transaction.

---

# 2. Required end-state invariants

The implementation is complete only if these invariants are true.

## 2.1 Request-path invariant

No LSP handler directly invokes:

```text
solve_affected_callables
rebuild_affected_state
analyze_surface over workspace/dependent modules
SemanticEngine::apply_*
workspace recursive scan
```

Handlers may perform bounded current-document work:

- apply full-text update;
- parse current file;
- build/update `LineIndex`;
- build current-file shallow source identity structures if required;
- publish syntax diagnostics;
- enqueue semantic work;
- read an immutable semantic snapshot.

## 2.2 Published-generation invariant

A semantic generation is immutable after publication.

```text
worker owns mutable state
    -> converges
    -> builds/pins SemanticSnapshot
    -> publication pointer swap
readers see old or new complete generation
never a partially-written generation
```

## 2.3 No-freshness-wait invariant

Hover/completion/inlay/navigation never await analysis completion.

A request chooses:

1. current live source facts;
2. newest compatible semantic snapshot;
3. conservative partial result when semantics are stale.

It does not wait.

## 2.4 One-writer invariant

Only the dedicated analysis worker mutates deep semantic state.

## 2.5 Latest-wins invariant

For repeated edits to one file, queued work keeps the newest revision. Intermediate queued revisions are discarded before deep analysis.

At most one older semantic computation may already be running.

## 2.6 Workspace-priority invariant

Open-document semantic work has priority over bulk workspace scanning.

## 2.7 Source-product invariant

`Program`, `ModuleSurface`, `ScopeGraph`, and `OccurrenceIndex` are built at most once for the exact source/recovery snapshot that uses them. Fixed-point rounds reuse them.

## 2.8 Core identity invariant

Exactly one logical `CORE_MODULE_URI` semantic source exists. The selected physical path is metadata, not a second semantic module.

## 2.9 Dispatch-cost invariant

Resolving a member never clones its AST body.

## 2.10 Flow-pass invariant

One traversal returns all flow products needed for that analysis unit.

---

# 3. Target architecture

```text
                   ┌─────────────────────────────────────────┐
                   │               LSP / Tokio               │
                   │                                         │
didOpen/change ───►│ DocumentStore                           │
                   │ current text / Parse / LineIndex        │
                   │ current shallow source identity         │
                   │                                         │
                   │ publish syntax diagnostics immediately  │
                   └──────────────┬──────────────────────────┘
                                  │ SourceUpdate(revision)
                                  ▼
                   ┌─────────────────────────────────────────┐
                   │            AnalysisService              │
                   │                                         │
                   │ PendingWork                             │
                   │ - latest edit per URI                   │
                   │ - batch disk changes                    │
                   │ - workspace/root/config changes         │
                   │ - shutdown flag                         │
                   │ - source epoch                          │
                   └──────────────┬──────────────────────────┘
                                  │ Mutex + Condvar
                                  ▼
                   ┌─────────────────────────────────────────┐
                   │       dedicated std::thread worker      │
                   │                                         │
                   │ SemanticEngine mutable state            │
                   │ progressive workspace scan state        │
                   │ reverse dependency indices              │
                   │ fixed point / callable worklist         │
                   │ caches                                  │
                   │                                         │
                   │ checks pending work between units       │
                   └──────────────┬──────────────────────────┘
                                  │ coherent generation
                                  ▼
                   ┌─────────────────────────────────────────┐
                   │  RwLock<Arc<SemanticSnapshot>>          │
                   │                                         │
                   │ lock only to clone/swap Arc             │
                   └──────────────┬──────────────────────────┘
                                  │
             ┌────────────────────┼─────────────────────┐
             ▼                    ▼                     ▼
           hover              completion             inlay
         no blocking          no blocking           no blocking
```

A second worker-to-Tokio event channel requests editor refreshes after publication:

```text
worker publication
    -> AnalysisEvent::Published
    -> tokio::sync::mpsc::UnboundedSender
    -> async task created in initialized()
    -> client.inlay_hint_refresh().await
    -> client.semantic_tokens_refresh().await when needed
```

`tower-lsp 0.20` provides `Client::inlay_hint_refresh`.

---

# 4. New file/module structure

Create these focused files.

## 4.1 `phalcom-lsp/src/analysis_service.rs`

Responsibility:

- owns published snapshot pointer;
- owns pending work / worker wakeup;
- spawns dedicated worker;
- exposes scheduling APIs to `Backend`;
- sends publication events to Tokio;
- exposes `snapshot()` to LSP consumers;
- handles shutdown signal.

It must not contain semantic flow logic.

## 4.2 `phalcom-lsp/src/semantic/engine.rs`

Responsibility:

- mutable worker-only semantic state;
- apply source batches;
- apply removals;
- core replacement;
- invalidation;
- call solver/worklist;
- generate immutable `SemanticSnapshot`.

Move mutation APIs out of the query-facing `SemanticDb`.

## 4.3 `phalcom-lsp/src/semantic/snapshot.rs`

Responsibility:

- immutable published state;
- query methods used by hover/completion/inlay/navigation;
- generation/file revision stamps;
- no mutation APIs.

## 4.4 `phalcom-lsp/src/semantic/source.rs`

Responsibility:

- immutable source semantic products for one file revision;
- cached `ModuleId`;
- canonical parse/program ownership;
- `ModuleSurface`;
- `ScopeGraph`;
- `OccurrenceIndex`;
- import surface;
- lightweight AST lookup helpers such as `member_ast`.

## 4.5 `phalcom-lsp/src/workspace_scan.rs`

Responsibility:

- progressive source discovery;
- include/exclude policy;
- scanner state that can process a bounded chunk and yield;
- no semantic solver logic.

Do not keep recursive workspace traversal embedded in `backend.rs`.

## 4.6 `phalcom-lsp/src/perf.rs`

Responsibility:

- lightweight `Instant` spans/counters;
- test-visible counters;
- optional stderr/log output controlled by one environment/config flag;
- no new tracing dependency required in this phase.

## 4.7 Existing files to modify

- `phalcom-lsp/src/lib.rs`
- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/documents.rs`
- `phalcom-lsp/src/inlay_hints.rs`
- `phalcom-lsp/src/completion.rs`
- `phalcom-lsp/src/semantic/mod.rs`
- `phalcom-lsp/src/semantic/infer.rs`
- `phalcom-lsp/src/semantic/flow.rs`
- `phalcom-lsp/src/semantic/surface.rs`
- `phalcom-lsp/src/semantic/dispatch.rs`
- `phalcom-lsp/src/semantic/module_graph.rs`
- `phalcom-lsp/src/semantic/ids.rs`
- `phalcom-lsp/src/semantic/occurrence.rs`
- `phalcom-lsp/src/semantic/scope.rs`
- `phalcom-lsp/src/semantic/core_source.rs`
- `phalcom-lsp/tests/integration.rs` plus focused new test modules if preferred
- `tools/vsphalcom/src/extension.ts`
- `tools/vsphalcom/package.json`
- VS Code extension tests under `tools/vsphalcom/src/test/`

No `Cargo.toml` dependency addition is required for the core architecture. The current `tokio` features already include `sync`.

---

# 5. Core data model

## 5.1 Published database wrapper

Replace the current query+mutation `SemanticDb` role with a publication/query wrapper.

Recommended:

```rust
#[derive(Default)]
pub struct SemanticDb {
    current: RwLock<Arc<SemanticSnapshot>>,
}

impl SemanticDb {
    pub fn empty() -> Self {
        Self {
            current: RwLock::new(Arc::new(SemanticSnapshot::default())),
        }
    }

    pub fn snapshot(&self) -> Arc<SemanticSnapshot> {
        self.current
            .read()
            .expect("semantic publication lock poisoned")
            .clone()
    }

    pub(crate) fn publish(&self, snapshot: Arc<SemanticSnapshot>) {
        *self
            .current
            .write()
            .expect("semantic publication lock poisoned") = snapshot;
    }
}
```

Rules:

- `SemanticDb::empty()` performs no parsing and no semantic solving.
- Never call a solver while holding `current.write()`.
- Query adapters should progressively migrate to `let semantic = db.snapshot();` and make all query calls against that same object.
- Transitional convenience query methods on `SemanticDb` may load one snapshot and forward once, but multi-query request paths must pin one snapshot explicitly.

## 5.2 Immutable published snapshot

Recommended initial shape:

```rust
#[derive(Clone, Debug, Default)]
pub struct SemanticSnapshot {
    pub generation: SemanticGeneration,
    pub files: BTreeMap<ModuleId, Arc<FileSemanticSnapshot>>,
    pub classes: BTreeMap<ClassId, Arc<ClassSurface>>,
    pub summaries: BTreeMap<CallableId, Arc<CallableSummary>>,
    pub field_facts: BTreeMap<FieldId, InferredValue>,
    pub parameter_facts: BTreeMap<(CallableId, String), InferredValue>,
    pub graph: ModuleGraphSnapshot,
}
```

Use `Arc` for large values. Cloning a top-level map still has O(number of keys) cost, but it copies cheap `Arc`s instead of AST-bearing values. The later callable-worklist task reduces how often whole maps need to be reconstructed.

Do not prematurely introduce persistent-map crates.

## 5.3 Source snapshot

Recommended:

```rust
#[derive(Clone)]
pub struct FileSourceSnapshot {
    pub revision: FileRevision,
    pub uri: Url,
    pub module: ModuleId,
    pub parse: Arc<phalcom_ast::parser::Parse>,
    pub surface: Arc<ModuleSurface>,
    pub scopes: Arc<ScopeGraph>,
    pub occurrences: Arc<OccurrenceIndex>,
    pub imports: Arc<[ImportSpec]>,
}
```

The `Parse` owns the canonical `Program`; do not clone `Program` into every semantic update.

For live editor source, `DocumentSnapshot` should carry or reference this shallow semantic source snapshot when available.

## 5.4 File semantic snapshot

Recommended:

```rust
#[derive(Clone, Debug)]
pub struct FileSemanticSnapshot {
    pub source: Arc<FileSourceSnapshot>,
    pub local_facts: Arc<LocalFacts>,
    pub field_facts: Arc<FieldFacts>,
    pub parameter_facts: Arc<ParameterFacts>,
    pub dependencies: Arc<DependencySet>,
}
```

The semantic snapshot owns analysis products; the source snapshot owns immutable source structures.

---

# 6. Analysis worker and scheduler

## 6.1 `AnalysisService`

Recommended interface:

```rust
pub struct AnalysisService {
    semantic: Arc<SemanticDb>,
    shared: Arc<WorkerShared>,
    events: tokio::sync::mpsc::UnboundedSender<AnalysisEvent>,
}

impl AnalysisService {
    pub fn new(
        semantic: Arc<SemanticDb>,
        events: tokio::sync::mpsc::UnboundedSender<AnalysisEvent>,
    ) -> Self;

    pub fn schedule_source(&self, update: SourceUpdate);

    pub fn schedule_disk_changes(&self, changes: Vec<DiskSourceChange>);

    pub fn schedule_workspace_roots(&self, roots: Vec<Url>);

    pub fn schedule_config(&self, config: AnalysisConfig);

    pub fn shutdown(&self);

    pub fn snapshot(&self) -> Arc<SemanticSnapshot>;
}
```

`Backend` owns `Arc<SemanticDb>` and `AnalysisService`.

## 6.2 Pending-work representation

Use latest-wins state, not FIFO edit jobs.

```rust
struct WorkerShared {
    pending: Mutex<PendingWork>,
    wake: Condvar,
    epoch: AtomicU64,
}

#[derive(Default)]
struct PendingWork {
    shutdown: bool,
    source_updates: BTreeMap<Url, SourceUpdate>,
    disk_changes: BTreeMap<Url, DiskSourceChange>,
    workspace_roots: Option<Vec<Url>>,
    config: Option<AnalysisConfig>,
    core_reselect: bool,
}
```

Scheduling an open-file source update:

1. increment epoch;
2. replace `source_updates[uri]` only if revision is newer;
3. wake worker.

Do not enqueue one object per keystroke.

## 6.3 Debounce policy

Use a fixed internal constant initially:

```rust
const EDIT_DEBOUNCE: Duration = Duration::from_millis(150);
```

Rules:

- debounce only deep analysis of repeated interactive edits;
- syntax parse/diagnostics remain immediate;
- workspace/disk events can be batch-coalesced for a short interval;
- shutdown bypasses debounce;
- an opened document that currently has no semantic state may be analyzed immediately or with a shorter first-edit delay.

Do not expose a user tuning setting until measurements justify it.

## 6.4 Worker loop

The worker is one long-lived `std::thread`.

Pseudo-code:

```rust
fn worker_loop(shared: Arc<WorkerShared>, semantic: Arc<SemanticDb>, events: EventSender) {
    let mut engine = SemanticEngine::new();

    loop {
        if shared.is_shutdown() {
            break;
        }

        if let Some(batch) = take_ready_interactive_batch(&shared, EDIT_DEBOUNCE) {
            let source_epoch = batch.epoch;
            let outcome = engine.apply_batch(batch);

            if !shared.is_shutdown() && source_epoch == shared.epoch.load(Ordering::Acquire) {
                let snapshot = Arc::new(engine.snapshot());
                semantic.publish(snapshot.clone());
                let _ = events.send(AnalysisEvent::Published(outcome.publication(snapshot.generation)));
            }

            continue;
        }

        if engine.has_workspace_scan_work() {
            let outcome = engine.scan_step(SCAN_BUDGET);

            if outcome.publishable {
                semantic.publish(Arc::new(engine.snapshot()));
            }

            // Loop immediately so newly queued interactive work wins.
            continue;
        }

        wait_for_work(&shared);
    }
}
```

Important:

- stale running analysis may finish, but it must not publish if a newer epoch superseded it;
- this bounds wasted work to at most one currently-running stale solve;
- after callable-level worklist refactoring, add cooperative epoch checks between callable units so stale work can stop earlier.

## 6.5 Do not synchronously join the worker from `shutdown`

`LanguageServer::shutdown` should:

```rust
self.analysis.shutdown();
Ok(())
```

Do not wait for semantic convergence or `JoinHandle::join()`.

The process is exiting and a detached worker will be terminated with it. Worker code must observe the flag between bounded work units for normal cleanup.

For tests, expose a `#[cfg(test)] wait_for_idle` / `shutdown_and_join_for_test` helper.

---

# 7. Live document update path

## 7.1 Split `publish_diagnostics_for`

Delete the semantic side effect from `publish_diagnostics_for`.

Target path:

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    let version = params.text_document.version;
    let Some(change) = params.content_changes.into_iter().next_back() else {
        return;
    };

    self.documents.open_or_update(uri.clone(), change.text);

    let Some(doc) = self.documents.snapshot(&uri) else {
        return;
    };

    self.index.update_file(uri.clone(), &doc.parse.program);

    let diagnostics = syntax_errors_to_diagnostics(doc.parse.errors.as_slice(), &doc.line_index);
    self.client.publish_diagnostics(uri.clone(), diagnostics, Some(version)).await;

    self.analysis.schedule_source(SourceUpdate::open(uri, doc));
}
```

The exact accessor for parse errors may use the existing `Document::errors()` equivalent; preserve existing diagnostics behavior.

## 7.2 Move semantic recovery parsing off the LSP handler

Current `semantic_recovery_parse` can perform a second full parse.

Move it to source-preparation work consumed by the analysis worker.

The user-facing parse and syntax diagnostics remain based on the unmodified live source.

Completion's dedicated incomplete-dot recovery may remain request-local if it is already bounded and necessary for editor recovery, but it must not trigger workspace semantic rebuild.

## 7.3 Use document snapshots, not document guards

No callback passed to `DocumentStore::with_document` may call analysis or disk I/O.

Prefer:

```rust
let Some(doc) = self.documents.snapshot(&uri) else { ... };
```

for any operation that survives beyond a trivial source lookup.

---

# 8. Shallow current-source semantics

Deep semantics may lag a document revision. Exact cursor identity should not.

For open documents, build shallow semantic source structures from the current parse without interprocedural inference:

```text
ModuleSurface
ScopeGraph
OccurrenceIndex
```

This can be done as part of `DocumentStore::open_or_update` or immediately after it in a focused `build_live_source_snapshot` function.

Instrument it. If it is unexpectedly large for giant files, move it to a separate high-priority source worker later; do not conflate that possible optimization with deep semantic solving.

The immediate rule is:

```text
current exact source identity = current revision
deep inferred facts           = latest completed compatible revision
```

## 8.1 Stale binding facts

Because `BindingId` is only stable within a file snapshot:

- if published semantic file revision != live source revision, do not apply old local binding facts to the new `BindingId`;
- render a binding hover with `?` / unknown value rather than blocking;
- do not show stale-position inlay hints.

## 8.2 Stable callable/class facts

`ClassId` and `CallableId` are module-qualified and more stable across body edits.

If the current shallow source target still resolves to the same `CallableId` or `ClassId`, the latest published return/declaration information can be used while the new local revision analyzes.

If declaration/surface identity changed, be conservative.

---

# 9. Inlay-hint freshness policy

`inlay_hints.rs` currently asks `SemanticDb::file_snapshot` and scans binding facts.

Change the request path to:

```rust
let semantic = self.analysis.snapshot();
let live = self.documents.snapshot(&uri);

if semantic.file_revision(&module) != Some(live.revision) {
    return Ok(Some(Vec::new()));
}
```

Then render hints from the matching semantic snapshot.

After a new generation is published for open documents, the async publication event task calls:

```rust
let _ = client.inlay_hint_refresh().await;
```

This avoids stale source ranges and avoids `"Loading..."`.

If semantic-token classification later depends on newly published semantic facts, call `semantic_tokens_refresh` only when the token output can actually change.

---

# 10. Core source selection

## 10.1 Remove eager core analysis from `SemanticDb::new`

`SemanticDb::empty` returns generation zero with no source analysis.

Native declarations that are cheap compile-time constants may be available immediately if they can be installed without parsing/flow solving; otherwise core setup belongs to the worker.

## 10.2 Select core exactly once

Worker source-selection precedence:

```text
explicit configured sysroot/core path
    else workspace conventional phalcom-core/core/core.ph
    else workspace conventional core/core.ph
    else bundled core source
```

Create:

```rust
enum CoreSource {
    Configured { physical_uri: Url, text: Arc<str> },
    Workspace { physical_uri: Url, text: Arc<str> },
    Bundled { text: &'static str },
}
```

All three publish semantic identity:

```text
ModuleId::new(CORE_MODULE_URI)
```

The physical URI is retained only for:

- open-buffer precedence;
- file-watcher invalidation;
- source-definition/doc navigation metadata.

## 10.3 Exclude selected physical core from ordinary workspace semantic registration

When workspace scanner encounters the selected core path, do not add a second ordinary `file:///.../core.ph` semantic module.

If the selected core file is open, its live document text replaces its worker core source while retaining `CORE_MODULE_URI`.

## 10.4 Configuration diffing

Define:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisConfig {
    pub sysroot_path: Option<PathBuf>,
    pub mode: AnalysisMode,
    pub excludes: Vec<String>,
}
```

Presentation config remains outside this struct.

`did_change_configuration`:

1. parse new full server config;
2. compare old/new;
3. apply presentation-only settings immediately;
4. call `schedule_config` only if analysis config changed;
5. mark `core_reselect` only if `sysroot_path` changed.

Changing inlay-hint policy must never rebuild core.

---

# 11. Implement `phalcom.analysis.mode`

## 11.1 Extension wiring

Current `package.json` declares:

```json
"phalcom.analysis.mode": {
  "type": "string",
  "enum": ["local", "workspace"],
  "default": "workspace"
}
```

Change the default to:

```json
"default": "local"
```

and send it in `readInitializationOptions()`:

```ts
analysis: {
    mode: config.get<string>("analysis.mode", "local"),
    exclude: config.get<string[]>("analysis.exclude", [])
}
```

Add:

```json
"phalcom.analysis.exclude": {
  "type": "array",
  "items": { "type": "string" },
  "default": [],
  "description": "Glob-style workspace paths excluded from Phalcom source indexing/analysis."
}
```

The scanner's built-in exclusions for hidden directories, `target`, and `node_modules` remain unconditional.

## 11.2 Server config

Add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnalysisMode {
    #[default]
    Local,
    Workspace,
}
```

`ServerConfig` includes:

```rust
pub analysis_mode: AnalysisMode,
pub analysis_exclude: Vec<String>,
```

## 11.3 Semantics of modes

`local` means:

- shallow workspace discovery/indexing may still progress for navigation;
- deep flow/interprocedural analysis is prioritized/limited to:
  - logical core;
  - open documents;
  - transitive imports required by open documents;
  - semantic dependencies necessary to resolve those files.

`workspace` means:

- same priority rules;
- after interactive/local closure converges, deep-analyze the remaining discovered workspace modules in background.

This separation preserves workspace navigation without forcing deep flow inference for every test/example merely because it exists under the root.

---

# 12. Progressive workspace scan

Move the recursive scanner into `workspace_scan.rs`.

Do not implement one recursive function that traverses the whole tree before yielding.

Recommended state:

```rust
pub struct WorkspaceScanState {
    pending_dirs: Vec<PathBuf>,
    pending_files: VecDeque<PathBuf>,
    roots: Vec<PathBuf>,
    excluded: ExcludeMatcher,
}
```

One worker step has a fixed work budget:

```rust
pub struct ScanBudget {
    pub max_dirs: usize,
    pub max_files: usize,
}

pub const SCAN_BUDGET: ScanBudget = ScanBudget {
    max_dirs: 16,
    max_files: 32,
};
```

After each step, return to the main worker loop and re-check interactive pending work.

For each discovered file:

1. canonicalize/derive `ModuleId` once;
2. skip selected physical core ordinary registration;
3. read;
4. parse;
5. build shallow source products;
6. update shallow workspace index/cache;
7. enqueue/deep-analyze according to analysis mode/priority.

Do not wait until every workspace file is parsed before publishing useful source/index state.

---

# 13. Closed-file source cache

Introduce worker/source-catalog entries sufficient for cross-file requests:

```rust
pub struct IndexedSource {
    pub uri: Url,
    pub module: ModuleId,
    pub text: Arc<str>,
    pub parse: Arc<Parse>,
    pub line_index: Arc<LineIndex>,
    pub source: Arc<FileSourceSnapshot>,
    pub phaldoc: Arc<PhaldocIndex>,
}
```

If memory profiling later shows full text/AST retention is too large, add an LRU policy for cold AST/text while retaining:

- `LineIndex`;
- declarations;
- Phaldoc;
- module/import data;
- source hash/mtime.

Do not prematurely implement eviction before measuring.

Modify:

- `Backend::with_source_snapshot`
- `Backend::occurrence_to_location`
- `Backend::member_phaldoc`

to read this cache/published source state instead of synchronous disk I/O.

No hover/definition/reference request should call `std::fs::read_to_string`.

---

# 14. Remove AST-body duplication from semantic surfaces

## 14.1 Add AST references

Current class members are direct entries inside a top-level `Statement::Class`, so use compact indices.

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemberAstRef {
    pub statement_index: u32,
    pub member_index: u32,
}
```

and equivalent field access can reuse `MemberAstRef`.

Change `MemberSurface`:

```rust
pub struct MemberSurface {
    pub callable: CallableId,
    pub kind: MemberKind,
    pub visibility: MemberVisibility,
    pub side: DispatchSide,
    pub is_constructor: bool,
    pub native_return: Option<NativeReturnShape>,
    pub source_range: SourceRange,
    pub name_range: SourceRange,
    pub params: Vec<ParamSurface>,
    pub ast_ref: Option<MemberAstRef>,
}
```

Remove:

```rust
pub body: Vec<Statement>
```

For native members:

```rust
ast_ref: None
```

Change `FieldSurface` to hold `ast_ref` rather than clone `initializer: Option<Expr>`.

## 14.2 Source lookup helpers

On `FileSourceSnapshot`:

```rust
pub fn member_ast(&self, reference: MemberAstRef) -> Option<&ClassMember>;

pub fn member_body(&self, reference: MemberAstRef) -> Option<&[Statement]>;

pub fn field_initializer(&self, reference: MemberAstRef) -> Option<&Expr>;
```

These borrow directly from `self.parse.program`.

## 14.3 Remove duplicate side-blind member storage

Spec 2 already established `members_by_side` as authoritative dispatch state.

Migrate remaining compatibility consumers and remove:

```rust
ClassSurface.members
```

when no active consumer requires it.

If a side-blind display/navigation lookup is still required, provide an explicit helper rather than duplicate full `MemberSurface` values.

---

# 15. Make dispatch results lightweight

Current `ResolvedDispatch` owns a cloned `MemberSurface`.

Change to:

```rust
#[derive(Clone, Debug)]
pub struct ResolvedDispatch {
    pub callable: CallableId,
    pub receiver_class: ClassId,
    pub side: DispatchSide,
}
```

`DispatchResolver` provides:

```rust
pub fn resolve(
    &self,
    receiver: &DispatchReceiver,
    selector: &str,
) -> Option<ResolvedDispatch>;

pub fn member(&self, callable: &CallableId) -> Option<&MemberSurface>;
```

During hierarchy lookup avoid allocating `(selector.to_string(), side)` on every class step.

Preferred surface indexing shape:

```rust
pub struct ClassSurface {
    pub members_by_selector: BTreeMap<String, SideMembers>,
    ...
}

pub struct SideMembers {
    pub instance: Option<MemberSurface>,
    pub class: Option<MemberSurface>,
}
```

Then lookup can use borrowed `&str` directly:

```rust
surface.members_by_selector.get(selector)
```

If changing the map shape in the same patch is too invasive, retain `members_by_side` temporarily but add a no-allocation helper that iterates/ranges by borrowed selector; do not leave per-step `selector.to_string()` as the final implementation.

---

# 16. Reuse source structures in flow

Change `flow::analyze_surface` to accept existing source structures.

Instead of:

```rust
fn analyze_surface(
    program: &Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    ...
)
```

prefer:

```rust
pub fn analyze_surface(
    source: &FileSourceSnapshot,
    context: &SolverContext<'_>,
    revision: SemanticGeneration,
) -> SurfaceFlowAnalysis
```

Inside:

```rust
let scopes = &source.scopes;
let surface = &source.surface;
let program = &source.parse.program;
```

Delete internal `build_scope_graph`.

Do not reconstruct `DispatchResolver` repeatedly inside nested helper wrappers when the same immutable class snapshot can be passed in one solver context.

---

# 17. Consume one flow analysis result

Current `SurfaceFlowAnalysis` already exposes:

```rust
pub struct SurfaceFlowAnalysis {
    pub local_facts: LocalFacts,
    pub field_facts: FieldFacts,
    pub parameter_facts: ParameterFacts,
    pub summaries: Vec<(CallableSummary, bool)>,
}
```

Use this object directly.

Delete execution patterns where:

```text
parameter_facts_for_program
field_facts_for_surface
collect_local_facts_with_returns
summaries_for_surface
```

each independently invoke `analyze_surface`.

Transitional wrappers may remain only if they accept an already-computed `&SurfaceFlowAnalysis` and select a field. They must not call flow again.

## 17.1 Solver round

One round per affected module should be conceptually:

```rust
let analysis = analyze_surface(source, &context, generation);

next_parameters.merge_from(&analysis.parameter_facts);

for (summary, has_evidence) in analysis.summaries {
    ...
}

round_local_results.insert(module.clone(), analysis.local_facts);
round_field_results.insert(module.clone(), analysis.field_facts);
```

Some local/field products depend on stabilized parameter facts. If final local/field facts require one post-convergence pass, permit exactly one final pass per affected source after fixed-point convergence. Do not run separate local and field passes.

Target:

```text
N solver rounds => N unified passes per affected source
+ at most 1 final presentation/facts pass if required
```

not `2N + 3` or more.

---

# 18. Incremental class index

Remove this pattern from `rebuild_affected_state`:

```rust
let mut classes = BTreeMap::new();
for file in state.files.values() {
    classes.extend(...);
}
state.classes = classes.clone();
```

When replacing a module source:

1. capture old module class IDs;
2. build new surface;
3. remove old class entries for that module;
4. insert new class surfaces;
5. compare a declaration-surface fingerprint/change classification.

A **body-only edit** must not reconstruct class mappings.

## 18.1 Change classification

At minimum classify source updates as:

```rust
pub enum SourceChangeKind {
    BodyOnly,
    ImportSurface,
    DeclarationSurface,
    FileAddedRemoved,
    CoreSurface,
}
```

This classification may initially compare old/new normalized declaration structures rather than hash them.

Do not include source ranges/body text in declaration equality; moving a declaration without changing its semantic surface is not a dispatch-surface change.

Rare `DeclarationSurface` changes may conservatively invalidate a broader class/module frontier in the first implementation. Ordinary body edits must use the narrow path.

---

# 19. Reverse module graph

Refactor `ModuleGraph`:

```rust
#[derive(Clone, Debug, Default)]
pub struct ModuleGraph {
    forward: BTreeMap<ModuleId, Vec<ImportEdge>>,
    reverse: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
}
```

On module import update:

1. remove old reverse edges contributed by that module;
2. compute/resolve new imports;
3. add new reverse edges.

Then:

```rust
pub fn dependents_of(&self, target: &ModuleId) -> impl Iterator<Item = &ModuleId> {
    self.reverse.get(target).into_iter().flatten()
}
```

`dependent_closure` becomes a normal adjacency traversal.

Do not call `refresh_resolutions(&available)` for ordinary body-only edits.

Only global/targeted import resolution is triggered by:

- source file added;
- source file removed;
- source URI/path changed;
- import declaration changed;
- workspace roots/excludes changed.

---

# 20. Batch source mutation APIs

Mutable worker engine APIs should be batch-oriented:

```rust
pub fn apply_source_batch(&mut self, changes: SourceBatch) -> AnalysisOutcome;

pub fn remove_modules_batch(&mut self, modules: &[ModuleId]) -> AnalysisOutcome;
```

Do not expose a backend-facing one-file mutating `SemanticDb::update_file`.

`did_change_watched_files` must collect one `Vec<DiskSourceChange>` and schedule once.

Workspace-root removal must schedule one root mutation/batch rather than `remove_file` in a loop.

---

# 21. Phase-1 fixed-point optimization: base + affected overlay

Before the callable-level solver lands, remove avoidable global clones.

Represent unaffected summaries/parameters as immutable base references and affected results as overlay maps.

Conceptually:

```rust
struct SolverFacts<'a> {
    base_summaries: &'a BTreeMap<CallableId, Arc<CallableSummary>>,
    summary_overlay: BTreeMap<CallableId, CallableSummary>,
    base_parameters: &'a ParameterFacts,
    parameter_overlay: ParameterFacts,
}
```

Lookup:

```rust
fn summary(&self, id: &CallableId) -> Option<&CallableSummary> {
    self.summary_overlay
        .get(id)
        .or_else(|| self.base_summaries.get(id).map(Arc::as_ref))
}
```

Do not clone unaffected workspace summaries on every round merely to produce an owned map.

Publish/merge after convergence.

This is an intermediate optimization; Task 10 replaces module-wide repeated solving with a callable worklist.

---

# 22. Callable-level incremental solver

This is the final deep-analysis model.

## 22.1 Dirty unit

The primary semantic invalidation unit is `CallableId`.

A body edit inside one method initially marks only that callable dirty, plus any source-local top-level analysis unit required for top-level statements.

## 22.2 Maintain reverse callable dependencies incrementally

State:

```rust
callable_dependencies: BTreeMap<CallableId, BTreeSet<CallableId>>,
callable_dependents: BTreeMap<CallableId, BTreeSet<CallableId>>,
```

When reanalyzing one callable:

1. produce its new dependency set from resolved call events;
2. diff old/new dependencies;
3. remove/add reverse edges;
4. if its published summary changed, enqueue dependents.

Do not rebuild the entire reverse dependency map from all summaries after every edit.

## 22.3 Worklist

Conceptual algorithm:

```rust
let mut queue = VecDeque::from(initial_dirty);
let mut queued = initial_dirty.clone();

while let Some(callable) = queue.pop_front() {
    queued.remove(&callable);

    check_epoch_or_shutdown();

    let before = state.summary(&callable).cloned();
    let result = analyze_callable(callable, &state);

    state.apply_callable_analysis(&callable, result);

    if state.summary(&callable) != before.as_ref() {
        for dependent in state.callable_dependents(&callable) {
            if queued.insert(dependent.clone()) {
                queue.push_back(dependent.clone());
            }
        }
    }
}
```

## 22.4 Parameter evidence must be contribution-based

Current module-level aggregate rebuilding is too broad.

Maintain contributions:

```rust
ParameterSlot -> BTreeMap<ContributionSource, InferredValue>
```

where:

```rust
pub struct ParameterSlot {
    pub callable: CallableId,
    pub name: String,
}

pub enum ContributionSource {
    Callable(CallableId),
    TopLevel(ModuleId),
}
```

When a caller is reanalyzed:

1. remove its old contributions;
2. add new call-site contributions;
3. rejoin only affected parameter slots;
4. if a callee parameter value changes, enqueue that callee;
5. if joined value stays equal, stop propagation there.

## 22.5 Recursive SCC optimization

Do not require SCC construction for the first callable-worklist patch if monotone worklist convergence is correct and bounded.

If profiling shows repeated recursion cycles are material, add SCC condensation over the callable dependency graph:

- acyclic singleton SCC => analyze once per upstream change;
- recursive SCC => iterate only members of the SCC until stable/widened.

Preserve `MAX_SHAPE_UNION` and solver-bottom semantics.

---

# 23. Cooperative cancellation after callable worklist lands

Initial async shell may allow one stale solve to finish and suppress publication.

After analysis is callable-granular, add:

```rust
fn cancelled(&self, started_epoch: u64) -> bool {
    self.shared.shutdown.load(...) ||
    self.shared.epoch.load(Ordering::Acquire) != started_epoch
}
```

Check:

- between workspace scan chunks;
- between modules;
- between callable worklist items;
- between recursive SCC rounds.

When cancelled:

- do not publish partial state;
- either discard a temporary transaction or merge only if the worker's mutable state is explicitly designed for restartable dirty propagation.

Preferred safe first design: mutate a worker transaction/overlay and commit to working state only when the batch reaches a coherent boundary.

Do not abort halfway through arbitrary in-place mutation without rollback semantics.

---

# 24. Query-side indexing improvements

These are P2/P3 after architectural stalls are gone.

## 24.1 `OccurrenceIndex::occurrence_at`

Current sorted vector still performs a full linear filter.

Build an index enabling bounded candidate search.

A simple implementation:

- sort by start;
- binary-search the last occurrence whose `start <= offset`;
- scan backward only while ranges can still contain the offset;
- choose shortest/preferred target.

Add a pathological test with thousands of occurrences and a test-only inspected-candidate counter.

## 24.2 `ScopeGraph::scope_at`

Precompute scopes sorted by start/range nesting, or maintain a compact interval index.

Do not scan every scope for every query.

## 24.3 `binding_for_declaration`

Add:

```rust
declarations: BTreeMap<SourceRange, BindingId>
```

or an equivalent key type if `SourceRange` ordering is unsuitable.

## 24.4 Completion member cache

Published snapshot may cache:

```rust
(ClassId, DispatchSide) -> Arc<[CompletionMember]>
```

Build lazily per immutable generation or precompute for open-document receiver classes.

Invalidate naturally by generation publication.

## 24.5 Dispatch cache

Within one immutable semantic generation:

```rust
(DispatchReceiverKey, selector) -> Option<ResolvedDispatch>
```

may be cached if profiling shows hierarchy walks are material.

Do not add it before eliminating AST cloning and global rebuilds.

---

# 25. Extension restart resilience

Current:

```ts
await lspClient?.stop()
lspClient = startLspClient(context)
```

means a rejected stop prevents replacement startup.

Change to a helper:

```ts
async function restartLspClient(context: ExtensionContext): Promise<void> {
    const old = lspClient
    lspClient = undefined

    if (old) {
        try {
            await old.stop()
        } catch (error) {
            lspOutput?.appendLine(`Graceful language-server stop failed: ${String(error)}`)
            await old.dispose()
        }
    }

    if (workspace.getConfiguration("phalcom").get<boolean>("lsp.enabled", true)) {
        lspClient = startLspClient(context)
    }
}
```

`vscode-languageclient` 8.x provides `LanguageClient#dispose`; use it to fully dispose the failed client.

Use this helper for:

- `Phalcom: Restart Language Server`;
- enabled/server-path configuration restarts.

This is defensive. The server-side async architecture should make normal shutdown fast enough that the fallback is rarely reached.

---

# 26. Analysis publication event loop

Add to `Backend`:

```rust
analysis_events: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<AnalysisEvent>>>,
```

In `Backend::new`:

1. create channel;
2. create cheap empty `SemanticDb`;
3. create `AnalysisService`;
4. store receiver for `initialized`.

In `initialized`:

```rust
if let Some(mut events) = self.analysis_events.lock().unwrap().take() {
    let client = self.client.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            match event {
                AnalysisEvent::Published {
                    refresh_inlay_hints,
                    refresh_semantic_tokens,
                    ..
                } => {
                    if refresh_inlay_hints {
                        let _ = client.inlay_hint_refresh().await;
                    }
                    if refresh_semantic_tokens {
                        let _ = client.semantic_tokens_refresh().await;
                    }
                }
            }
        }
    });
}
```

Then schedule initial roots/config/core selection.

Do not make the worker call async `Client` methods directly.

---

# 27. Instrumentation

Create lightweight phase timing without adding a dependency.

Recommended counters:

```rust
pub struct PerfCounters {
    pub source_updates_enqueued: AtomicU64,
    pub source_updates_coalesced: AtomicU64,
    pub semantic_batches_started: AtomicU64,
    pub semantic_batches_published: AtomicU64,
    pub stale_batches_discarded: AtomicU64,
    pub workspace_files_discovered: AtomicU64,
    pub workspace_files_parsed: AtomicU64,
    pub flow_passes: AtomicU64,
    pub solver_rounds: AtomicU64,
    pub callables_analyzed: AtomicU64,
}
```

Record durations for:

```text
backend.construct
initialize
source.parse
source.shallow
workspace.discovery
workspace.parse
core.select
core.analyze
semantic.batch
semantic.solve
semantic.flow
semantic.publish
lsp.hover
lsp.completion
lsp.inlay
```

When `PHALCOM_LSP_PERF=1`, emit compact stderr lines such as:

```text
phalcom-lsp perf semantic.batch generation=17 elapsed_ms=42
  changed_files=1 affected_modules=1 affected_callables=3
  flow_passes=2 solver_rounds=2 stale=false
```

Do not emit per-node logs.

Tests may inspect counters directly under `cfg(test)`.

---

# 28. Performance/concurrency test harness

Wall-clock tests alone are too flaky. Add deterministic hooks.

Under `cfg(test)`:

```rust
pub struct AnalysisTestHooks {
    pub before_batch: Option<Arc<Barrier>>,
    pub after_batch: Option<Arc<Barrier>>,
    pub counters: Arc<PerfCounters>,
}
```

or an equivalent gate/channel mechanism.

Required deterministic tests:

## 28.1 Initialize does not wait for worker

1. block worker before core/workspace analysis;
2. send LSP `initialize`;
3. assert initialize response arrives;
4. release worker.

## 28.2 Hover reads old snapshot while worker is busy

1. publish generation N;
2. block worker starting generation N+1;
3. send edit;
4. issue hover;
5. assert hover returns without releasing worker;
6. release worker;
7. assert later snapshot reaches N+1.

## 28.3 Inlay hints do not block

Same setup; with revision mismatch the request returns empty/current-safe result immediately.

## 28.4 Revision coalescing

1. schedule revisions 1 through 100 for one URI while worker is gated;
2. release worker;
3. assert pending deep analysis contains revision 100 only;
4. assert published file revision becomes 100;
5. assert counter shows many coalesced updates.

## 28.5 Stale generation is not published

1. start analysis for revision 10;
2. enqueue revision 11 before 10 finishes;
3. finish 10;
4. assert generation derived from revision 10 is not published as newest;
5. finish 11;
6. assert revision 11 publishes.

## 28.6 Shutdown is nonblocking

1. block worker;
2. call LSP shutdown;
3. assert shutdown response returns before worker release;
4. release worker for clean test teardown.

## 28.7 Watched-file batch is one semantic transaction

Send N watched changes and assert one batch/generation transaction, not N synchronous solves.

## 28.8 Presentation config does not rebuild core

Change only inlay-hint config and assert core-analysis counter unchanged.

## 28.9 Core has one logical semantic module

Repository workspace contains physical core source. Assert snapshot contains logical `phalcom://core` and does not contain a duplicate ordinary physical-core semantic class namespace.

## 28.10 Flow pass bound

For a fixture converging in R solver rounds, assert each affected module has approximately R unified flow passes plus at most one final stabilized pass, not multiple independent extraction passes.

---

# 29. Wall-clock benchmark harness

Add an ignored/manual integration benchmark command rather than unstable ordinary tests.

Suggested executable/test helper:

```bash
cargo test -p phalcom-lsp --test integration perf_ -- --ignored --nocapture
```

Measure:

- backend construction;
- initialize response;
- first open-document shallow semantics;
- initial core analysis;
- initial shallow workspace indexing;
- full local-mode deep convergence;
- full workspace-mode convergence;
- leaf body edit;
- edit changing return shape with one dependent;
- class-surface edit;
- 20 rapid edits coalescing;
- hover while worker busy.

Record debug and release builds.

Do not encode the supplied 14.3/30.7/103-second values as strict CI thresholds.

---

# 30. Task-by-task implementation plan

## Task 1: Add performance counters and deterministic worker test hooks

**Files:**
- Create: `phalcom-lsp/src/perf.rs`
- Modify: `phalcom-lsp/src/lib.rs`
- Test: `phalcom-lsp/tests/integration.rs` or new `phalcom-lsp/tests/performance.rs`

**Interfaces:**
- Produces: `PerfCounters`, `PerfSpan`, test-visible counter snapshot.
- No semantic behavior changes.

- [ ] Add counters listed in Section 27.
- [ ] Add `PerfSpan::start(name)` / drop-or-finish duration recording gated by `PHALCOM_LSP_PERF`.
- [ ] Instrument current backend construction, initialize, `update_files_batch`, solver rounds, `analyze_surface`, hover, completion, inlay hints.
- [ ] Add one test proving counters are deterministic.
- [ ] Run `cargo test -p phalcom-lsp`.
- [ ] Commit: `perf(lsp): add semantic performance instrumentation`.

## Task 2: Split immutable publication state from mutable semantic engine

**Files:**
- Create: `phalcom-lsp/src/semantic/snapshot.rs`
- Create: `phalcom-lsp/src/semantic/engine.rs`
- Modify: `phalcom-lsp/src/semantic/mod.rs`
- Modify: semantic query consumers incrementally.
- Test: semantic unit/integration tests.

**Interfaces:**
- Produces: `SemanticDb::empty`, `SemanticDb::snapshot`, `SemanticDb::publish`, `SemanticEngine`.
- Existing semantic correctness tests must continue to pass through test helper engine APIs.

- [ ] Write a failing test that constructs `SemanticDb::empty()` and asserts generation zero without core/file state.
- [ ] Move mutable `SemanticState`, `update_files_batch`, `remove_file`, `rebuild_affected_state` behind `SemanticEngine`.
- [ ] Create immutable `SemanticSnapshot`.
- [ ] Change query APIs to operate on/pin `Arc<SemanticSnapshot>`.
- [ ] Keep test helpers that synchronously drive `SemanticEngine` for existing semantic unit tests; do not force every unit test through the background worker.
- [ ] Prove no solver executes while publication lock is held by a test hook/assertion.
- [ ] Run all LSP tests.
- [ ] Commit: `refactor(lsp): separate semantic engine from published snapshots`.

## Task 3: Add `AnalysisService` worker, latest-wins scheduling, and nonblocking shutdown

**Files:**
- Create: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/lib.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Test: new async worker tests.

**Interfaces:**
- Produces: scheduling APIs and `AnalysisEvent`.

- [ ] Write worker-blocked initialize/hover/shutdown tests first.
- [ ] Implement `WorkerShared`, `PendingWork`, epoch, `Condvar`.
- [ ] Spawn one `std::thread`.
- [ ] Implement 150 ms edit debounce/coalescing.
- [ ] Suppress stale publication if epoch changed.
- [ ] Add `AnalysisEvent` channel.
- [ ] Make shutdown signal-only.
- [ ] Run concurrency tests repeatedly, e.g. `for i in {1..20}; do cargo test -p phalcom-lsp worker_; done`.
- [ ] Commit: `feat(lsp): add asynchronous semantic worker`.

## Task 4: Make backend lifecycle and edit handlers nonblocking

**Files:**
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/documents.rs`
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Test: JSON-RPC integration tests.

**Interfaces:**
- Consumes: `AnalysisService`.
- Produces: source update + immediate diagnostics pipeline.

- [ ] Replace analysis inside `publish_diagnostics_for`.
- [ ] Use `DocumentStore::snapshot`, never long-running `with_document`.
- [ ] Move semantic recovery parse to worker/source preparation.
- [ ] Make `initialize` return before scan/core analysis.
- [ ] In `initialized`, spawn publication event task and schedule initial analysis.
- [ ] Make `did_open`, `did_change`, `did_close`, workspace-folder, watched-file handlers schedule work.
- [ ] Make inlay hints return empty if semantic file revision mismatches live revision.
- [ ] Request `client.inlay_hint_refresh()` after relevant publication.
- [ ] Run LSP protocol and VS Code E2E tests.
- [ ] Commit: `perf(lsp): remove semantic solving from request handlers`.

## Task 5: Implement analysis mode and progressive workspace scanning

**Files:**
- Create: `phalcom-lsp/src/workspace_scan.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `tools/vsphalcom/src/extension.ts`
- Modify: `tools/vsphalcom/package.json`
- Test: Rust workspace scan tests + VS Code config tests.

**Interfaces:**
- Produces: `AnalysisMode`, `AnalysisConfig`, `WorkspaceScanState`.

- [ ] Add `local`/`workspace` parsing and make `local` default.
- [ ] Add `analysis.exclude`.
- [ ] Send analysis config from extension initialization options.
- [ ] Replace recursive full scan with chunked `WorkspaceScanState`.
- [ ] Prioritize open-document work before every scan chunk.
- [ ] Keep shallow indexing available in local mode; limit deep semantics to open/import closure.
- [ ] Test that a gated bulk scan does not delay an open-file analysis job.
- [ ] Commit: `feat(lsp): add prioritized progressive workspace analysis`.

## Task 6: Fix core selection and configuration invalidation

**Files:**
- Modify: `phalcom-lsp/src/semantic/core_source.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/workspace_scan.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Test: core/config tests.

**Interfaces:**
- Produces: `CoreSource`, one logical core identity.

- [ ] Remove core parse/analysis from backend/database construction.
- [ ] Implement configured/workspace/bundled precedence.
- [ ] Exclude selected physical core from ordinary semantic module registration.
- [ ] Diff config; rebuild/reselect core only on `sysroot_path`/relevant source change.
- [ ] Add tests for inlay-only config update causing zero core analysis.
- [ ] Add duplicate-core regression.
- [ ] Commit: `perf(lsp): analyze one canonical core source`.

## Task 7: Remove AST-body duplication and cheapen dispatch

**Files:**
- Modify: `phalcom-lsp/src/semantic/surface.rs`
- Modify: `phalcom-lsp/src/semantic/source.rs`
- Modify: `phalcom-lsp/src/semantic/dispatch.rs`
- Modify: `phalcom-lsp/src/semantic/analyzer.rs`
- Modify: `phalcom-lsp/src/semantic/flow.rs`
- Modify: completion/hover consumers as required.
- Test: existing dispatch/inference suite plus no-body-clone unit tests.

**Interfaces:**
- Produces: `MemberAstRef`, lightweight `MemberSurface`, lightweight `ResolvedDispatch`.

- [ ] Add AST index refs during surface construction.
- [ ] Remove `MemberSurface.body`.
- [ ] Remove field initializer AST clone.
- [ ] Add source lookup helpers.
- [ ] Change dispatch result to IDs/light metadata.
- [ ] Migrate all semantic consumers.
- [ ] Remove side-blind duplicate member map once unused.
- [ ] Add a test ensuring dispatch resolution does not clone/copy method body data.
- [ ] Commit: `perf(lsp): make semantic surfaces and dispatch lightweight`.

## Task 8: Reuse source structures and run one unified flow pass

**Files:**
- Modify: `phalcom-lsp/src/semantic/flow.rs`
- Modify: `phalcom-lsp/src/semantic/infer.rs`
- Modify: `phalcom-lsp/src/semantic/engine.rs`
- Test: flow/solver tests.

**Interfaces:**
- Produces: source-backed `analyze_surface` and one-pass solver use.

- [ ] Change `analyze_surface` to accept `FileSourceSnapshot`.
- [ ] Delete internal scope rebuild.
- [ ] Remove wrappers that re-enter `analyze_surface`.
- [ ] Consume `SurfaceFlowAnalysis` directly.
- [ ] Permit at most one final post-convergence unified pass if stabilized local/field facts require it.
- [ ] Add flow-pass counter assertions.
- [ ] Run all Spec 1–4 semantic regression tests.
- [ ] Commit: `perf(lsp): reuse source graphs and unify flow passes`.

## Task 9: Make module/class invalidation incremental and batch mutations

**Files:**
- Modify: `phalcom-lsp/src/semantic/module_graph.rs`
- Modify: `phalcom-lsp/src/semantic/engine.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Test: invalidation/watcher tests.

**Interfaces:**
- Produces: reverse module index, source change classification, batch APIs.

- [ ] Add forward/reverse import maps.
- [ ] Stop full `refresh_resolutions` on body-only edits.
- [ ] Incrementally remove/insert module class surfaces.
- [ ] Add `SourceChangeKind`.
- [ ] Add batch source/remove APIs.
- [ ] Make watched-file and root-removal notifications one scheduled transaction.
- [ ] Test body-only edit does not rebuild global class/import state.
- [ ] Commit: `perf(lsp): narrow module graph and class invalidation`.

## Task 10: Replace module-wide solver with callable worklist and incremental parameter contributions

**Files:**
- Modify: `phalcom-lsp/src/semantic/infer.rs`
- Modify: `phalcom-lsp/src/semantic/engine.rs`
- Modify: `phalcom-lsp/src/semantic/callable.rs`
- Modify: `phalcom-lsp/src/semantic/facts.rs`
- Test: dependency/invalidation tests.

**Interfaces:**
- Produces: callable dirty worklist, reverse callable edges, contribution-based parameters.

- [ ] Write tests where a leaf callable edit recomputes only that callable when summary is unchanged.
- [ ] Write tests where changed return summary recomputes exactly true dependents.
- [ ] Add incremental dependency edge diff.
- [ ] Add parameter contribution source tracking.
- [ ] Add callable work queue and bounded convergence.
- [ ] Preserve bottom vs Unknown and widening behavior.
- [ ] Add cooperative epoch/shutdown checks between callable units.
- [ ] Add recursion fixture; confirm convergence. Add SCC optimization only if measured necessary after the worklist is correct.
- [ ] Commit: `perf(lsp): add callable-grained incremental semantic solving`.

## Task 11: Cache closed source/Phaldoc and optimize hot source queries

**Files:**
- Modify/create source catalog/cache file as chosen in Task 5.
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/semantic/occurrence.rs`
- Modify: `phalcom-lsp/src/semantic/scope.rs`
- Modify: `phalcom-lsp/src/completion.rs`
- Test: hover/navigation/query tests.

**Interfaces:**
- Produces: no-disk interactive navigation and bounded lookup indexes.

- [ ] Cache line indexes/Phaldoc/shallow source for indexed closed files.
- [ ] Remove closed-file disk parsing from hover/definition path.
- [ ] Add binary/bounded occurrence lookup.
- [ ] Add direct binding declaration lookup.
- [ ] Optimize `scope_at`.
- [ ] Add completion-member cache if profiling still shows meaningful time there.
- [ ] Add tests that fail if hover path invokes disk-read test hook.
- [ ] Commit: `perf(lsp): cache source metadata and accelerate semantic queries`.

## Task 12: Make VS Code restart resilient

**Files:**
- Modify: `tools/vsphalcom/src/extension.ts`
- Test: `tools/vsphalcom/src/test/...`

**Interfaces:**
- Produces: `restartLspClient`.

- [ ] Add stop-failure test/mocked client path if extension test harness permits.
- [ ] Replace duplicated stop/start logic with `restartLspClient`.
- [ ] On stop error, log and `dispose` old client.
- [ ] Start replacement regardless of graceful-stop failure when LSP remains enabled.
- [ ] Run `npm test` and `npm run test:lsp:e2e`.
- [ ] Commit: `fix(vsphalcom): make language server restart resilient`.

## Task 13: Verification and performance acceptance

**Files:**
- Add/modify ignored perf tests and documentation if appropriate.
- No production semantic changes unless a gate exposes a defect.

- [ ] Run formatting:
  ```bash
  cargo fmt --all -- --check
  ```
- [ ] Run focused crates:
  ```bash
  cargo test -p phalcom-ast
  cargo test -p phalcom-native-surface
  cargo test -p phalcom-lsp
  ```
- [ ] Run workspace:
  ```bash
  cargo test --workspace
  ```
- [ ] Run VS Code:
  ```bash
  cd tools/vsphalcom
  npm ci
  npm run lint
  npm run compile
  npm test
  npm run test:lsp:e2e
  ```
- [ ] Run ignored performance harness in debug and release.
- [ ] Manually verify rapid typing/hover/inlay behavior in a repository-root workspace.
- [ ] Verify restart while semantic worker is deliberately busy.
- [ ] Record before/after timings and worker counters in the final PR description.
- [ ] Commit any benchmark harness/docs: `test(lsp): lock asynchronous performance regressions`.

---

# 31. Detailed semantic compatibility requirements

All existing semantic behavior from Specs 1–4 must survive.

The async refactor must not regress:

- exact hover ranges;
- no keyword/literal hover;
- lexical binding identity;
- shadowing;
- closure parameters/captures;
- `for` bindings;
- getter/setter/operator/subscript inference;
- side-aware class/instance dispatch;
- `super`;
- constructor result semantics;
- string interpolation inference;
- field facts;
- parameter call-site evidence;
- block non-local return handling;
- callable dependencies;
- definition/references;
- semantic tokens;
- inlay policies;
- future formal typing separation.

When semantics are temporarily stale, degrade the **amount of inferred information**, not semantic identity correctness.

Example:

```text
current binding declaration known, inference stale
=> "parameter owner — inferred value: ?"
```

not:

```text
wait for solver
```

and not:

```text
misidentify owner as a class/global because value is unavailable
```

---

# 32. Startup behavior after implementation

Expected sequence:

```text
process starts
  -> cheap Backend::new
  -> empty semantic generation 0

initialize arrives
  -> parse config
  -> record roots
  -> return capabilities immediately

initialized
  -> start publication event listener
  -> schedule core selection
  -> schedule progressive workspace scan

user opens file
  -> parse + shallow current source
  -> syntax diagnostics
  -> high-priority semantic update
  -> editor is usable while scan continues
```

No workspace size may directly determine the `initialize` handshake duration.

---

# 33. Edit behavior after implementation

Expected:

```text
keystroke
  -> full-text DocumentStore update (for now)
  -> parse + line index
  -> current shallow source identity
  -> syntax diagnostics
  -> replace pending SourceUpdate for URI
  -> return

150 ms quiet period
  -> worker analyzes newest revision only

while worker is busy
  -> hover uses current source + latest safe snapshot
  -> completion uses current source + latest safe snapshot
  -> inlay returns matching generation or empty
  -> no request waits on worker

worker converges
  -> atomically publish generation
  -> request inlay refresh
```

---

# 34. Shutdown/restart behavior after implementation

Expected:

```text
VS Code stop
  -> LSP shutdown
     -> set worker shutdown flag
     -> return immediately
  -> client exits transport/process

if graceful client stop fails:
  -> extension logs error
  -> dispose old LanguageClient
  -> start replacement anyway
```

A fixed-point solve cannot hold the LSP request executor hostage because it no longer runs there.

---

# 35. Explicit non-goals

Do not expand this project into unrelated work.

Not required for this performance phase:

- incremental parser/text edit application;
- new Phalcom formal type system;
- SSA/CFG compiler IR;
- VM execution;
- distributed analysis;
- persistent on-disk semantic cache;
- third-party persistent immutable collection dependency;
- full package/project model redesign;
- speculative parallel semantic solving across many CPU cores.

Full-document parsing may remain until instrumentation shows it is the next material bottleneck.

---

# 36. Acceptance gate

This implementation is complete when all of the following are true.

1. `Backend::new` performs no full core semantic analysis.
2. `initialize` performs no recursive workspace scan or deep semantic solve.
3. deep semantic work runs only on the dedicated analysis worker.
4. `SemanticDb` publication lock is never held during analysis.
5. LSP feature requests pin immutable semantic snapshots.
6. hover/completion/inlay requests return while the worker is deliberately blocked.
7. repeated edits coalesce to the newest pending revision.
8. stale analysis cannot overwrite a newer source epoch.
9. syntax diagnostics publish without waiting for deep inference.
10. inlay hints never use stale binding ranges and refresh after compatible semantics publish.
11. `analysis.mode` works end to end; `local` is the default deep-analysis policy.
12. workspace scan is progressive and yields to open-document work.
13. the physical selected core file is not simultaneously indexed as a second semantic core namespace.
14. inlay/presentation configuration changes do not rebuild core.
15. watched-file and workspace-folder batches cause batched semantic transactions.
16. `ModuleGraph` has explicit reverse dependencies.
17. body-only edits do not globally re-resolve imports or rebuild all class surfaces.
18. source `ScopeGraph`/surface/occurrence structures are reused across solver rounds.
19. dispatch resolution does not deep-clone `MemberSurface` bodies; bodies are not owned by surfaces.
20. unified flow analysis is not independently rerun for parameter/summary/local/field extraction.
21. callable dependencies are maintained incrementally rather than rebuilt globally.
22. ordinary body edits eventually recompute at callable granularity and stop propagation when summaries/parameter facts do not change.
23. closed-file hover/navigation performs no synchronous reread/reparse on the request path.
24. all existing semantic correctness tests from Specs 1–4 remain green.
25. restart starts a replacement client even if graceful stop fails.
26. instrumentation reports source updates coalesced, flow passes, solver/callable work, and publication timing.
27. manual repository-root usage remains responsive during background indexing/deep analysis.

The central success criterion is not merely reducing total batch-analysis seconds. It is this invariant:

> **Editor latency is decoupled from semantic convergence latency.**

A large workspace may still require meaningful CPU to reach complete workspace semantics, but that work must be progressive, interruptible/coalescible at safe boundaries, and invisible to interactive request latency.
