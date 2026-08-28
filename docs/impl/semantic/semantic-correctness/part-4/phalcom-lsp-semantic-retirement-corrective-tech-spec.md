# Phalcom Single Semantic World — Corrective `phalcom-lsp` Retirement Technical Specification

**Status:** Corrective architectural closure specification  
**Repository:** `aureat/phalcom-lang`  
**Grounded branch:** `main`  
**Grounded HEAD:** `9b30ec324d4361128f285154fe236e25746df750`  
**Grounded date:** 2026-08-27  
**Supersedes as implementation guidance:** the earlier retirement tech spec/plan grounded at `24919cd26019c6b5ffa72b069fa4692255ab0108`  
**Scope:** Finish Part 3 single-world closure only. Part 4 semantic features remain out of scope.

---

## 1. Executive decision

The original retirement direction is correct:

> `phalcom-semantic` is the only implementation and owner of Phalcom semantics; `phalcom-lsp` is an editor transport, scheduling, syntax-recovery, freshness-policy, and protocol-presentation layer.

The current migration failed because it changed that architecture **across the middle of the ownership spine**. Canonical primitives were added successfully, then `AnalysisService` and `RequestContext` were partially rewritten while `Backend`, feature modules, integration tests, and the old `SemanticDb` still used the previous object graph.

The repair is **not** to restore the old APIs until the crate compiles. It is also **not** to continue changing individual consumers opportunistically.

The repair is to complete one coherent ownership cut:

```text
Backend
  ├── DocumentStore
  ├── AnalysisService
  │     ├── scheduler/latest-wins state
  │     ├── ONE private SemanticPublication
  │     └── worker thread
  │           └── ONE persistent SemanticWorkspaceSession
  ├── closed-source presentation cache
  └── LSP configuration/presentation state

request
  └── RequestContext
        ├── live DocumentSnapshot
        ├── Option<Arc<phalcom_semantic::SemanticSnapshot>>
        ├── Option<phalcom_modules::ModuleId>
        └── SourceMatch
```

There is no `phalcom_lsp::semantic::SemanticDb`, no outer semantic snapshot, no nested compiler snapshot, no LSP-local semantic identity conversion, and no request-time text-derived semantic reconstruction.

A second corrective decision is equally important:

> Moving text-driven receiver inference from `phalcom-lsp` into `phalcom-semantic::editor` does not make it canonical. Query-time semantic reconstruction must disappear, not merely move crates.

The editor facade may **select and compose already-published compiler facts**. If a fact needed by completion/hover/signature help does not exist, the semantic update path must publish it. The editor query must fail closed rather than parse a dotted source string and simulate dispatch.

---

## 2. Verification result at current `main`

The previous worker reported 43 `phalcom-lsp` compile errors after the migration. The current source tree independently exposes multiple hard API contradictions, so the failure is structurally credible even without reproducing the exact diagnostic list.

The current `main` contains four pushed retirement commits above the old grounded baseline:

```text
10c875ee  feat(semantic): add canonical editor query primitives
e2715695  refactor(lsp): migrate semantic publication spine
bbcbc90f  docs(semantic): add retirement closure plans
9b30ec32  docs: add type system foundation notes
```

The useful work in those commits should be retained, but the migration is incomplete.

### 2.1 Hard compile contradiction: `Backend` still calls a deleted constructor

Current `phalcom-lsp/src/analysis_service.rs` exposes:

```rust
pub(crate) fn new_with_publication(
    publication: Arc<SemanticPublication>,
    source_cache: Option<SourceCache>,
) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>)
```

Current `phalcom-lsp/src/backend.rs` still calls:

```rust
AnalysisService::new_with_index_and_cache(
    db.clone(),
    Some(index.clone()),
    Some(closed_sources.clone()),
)
```

This is not a semantic bug. It is an ownership cut left halfway through compilation.

### 2.2 Hard compile contradiction: `RequestContext` was changed before its callers

Current `RequestContext::new_with_compiler` accepts:

```rust
(document, compiler, uri)
```

Current `Backend::request_context` still passes:

```rust
(document, legacy_semantic_snapshot, compiler_snapshot, uri)
```

Current consumers also refer to removed/transitional fields such as `request.semantic` and, in some cases, `request.module`.

### 2.3 Publication topology is currently split

This is the most dangerous latent bug because a naive compile fix can produce a server that builds but never publishes semantic updates to requests.

The old LSP `SemanticDb` still contains its own:

```rust
publication: Arc<crate::publication::SemanticPublication>
```

The rewritten `AnalysisService` also owns:

```rust
publication: Arc<SemanticPublication>
```

The old `Backend` still constructs and reads `SemanticDb`.

Therefore there are currently two possible publication-cell owners. If a repair simply creates a fresh `SemanticPublication` for `AnalysisService` while leaving `Backend` reading `SemanticDb::compiler_snapshot()`, the worker and requests observe different cells.

**Normative correction:** `SemanticDb` must not remain between the worker and requests. `AnalysisService` owns the one publication cell and exposes a read-only `snapshot()` method. `Backend` reads that method.

### 2.4 The legacy semantic world still exists physically and conceptually

`phalcom-lsp/src/semantic/` still exports the old engine, IDs, snapshots, scopes, flow, invalidation, dispatch, module graph, advisory shapes, and query API. `phalcom-lsp/src/lib.rs` still exports:

```rust
pub mod semantic;
pub mod index;
```

`phalcom-lsp/src/backend.rs`, `hover.rs`, `inlay_hints.rs`, and `semantic_tokens.rs` still import those types.

### 2.5 Canonical receiver resolution contains query-time text semantics

Current `phalcom-semantic/src/editor.rs::resolve_receiver_at` correctly consults formal/advisory products, but it also:

- slices raw source text by the request range;
- recognizes `"self"` and `"super"` by string comparison;
- parses dotted expression components;
- decodes selector spelling from those strings;
- walks declaration surfaces;
- simulates chained call returns;
- synthesizes constructor result shapes.

This violates the intended boundary. A request-time read-only query has become a small semantic evaluator.

### 2.6 Canonical editor coverage is insufficient

The editor facade currently has two focused integration tests. That is not sufficient coverage for a query surface responsible for:

- bindings;
- parameters;
- fields;
- call results;
- class objects;
- `self`;
- `super`;
- unions;
- inheritance;
- visibility;
- exact definition/reference identity;
- shadowing;
- unknown/fail-closed behavior.

The migration started consuming the facade before the facade was characterized to parity.

### 2.7 Workspace scan sources are currently inserted as overlays

`process_scan_batch` converts discovered closed files to:

```rust
WorkspaceSourceBatchMutation::SetOverlay { ... }
```

That marks scanner-discovered disk sources as `open_overlay = true` in `WorkspaceModuleSession`.

This confuses two distinct lifecycle states:

```text
editor overlay        !=        discovered disk snapshot
```

The batch API therefore needs one missing canonical mutation: an already-read/parsed **disk snapshot** that remains disk-backed.

### 2.8 The worker still mirrors canonical source lifecycle

`AnalysisService` still owns:

```text
source_catalog: BTreeMap<Url, (SourceRevision, Arc<str>, Program)>
```

and manually computes relative import closure through:

```text
extend_import_closure_with_source
resolve_source_import
```

The canonical session is persistent, but LSP still mirrors source/program state and contains import-path semantics.

The LSP may schedule discovery. It may not define module import meaning.

### 2.9 Semantic generation is still externally coordinated by the LSP

The worker calls:

```rust
SemanticWorkspaceSession::apply_module_mutations_at_generation(...)
```

with a generation derived in LSP.

Semantic publication generation is canonical workspace state. The LSP may carry a worker epoch for latest-wins scheduling, but it must not assign semantic snapshot generations.

### 2.10 The core update path is dead transitional state

`AnalysisService` still accepts and queues `core_update`, but the main worker path binds it as:

```rust
let _core_update = core_update;
```

The canonical semantic session already bootstraps the core/native universe. The LSP-selected core source is now a transport/provenance concern, so the semantic `core_update` queue must be removed rather than completed.

---

## 3. What from the previous implementation should be kept

The failed migration still produced useful canonical foundations.

Keep and finish these pieces unless focused tests expose a correctness defect:

1. `phalcom-modules::WorkspaceSourceBatchMutation`.
2. Transactional `WorkspaceModuleSession::apply_batch`.
3. `SemanticWorkspaceSession::apply_module_mutations`.
4. Reverse canonical source provenance (`module_for_source`, `module_for_display_path`).
5. Canonical declaration/callable/field source metadata.
6. `AdvisoryPresenter` and protocol-neutral callable presentation.
7. `phalcom-semantic::EditorSemanticQuery` as a concept.
8. `phalcom-lsp::SemanticPublication` as a tiny immutable handoff.
9. `SourceRevision` in `DocumentStore`.
10. `core_documents.rs` as the transport-only replacement for old core semantic construction.
11. Canonical paths already present in completion, signature help, inlay hints, and semantic tokens.

The repair should be surgical: preserve these foundations, correct their boundaries, and finish consumer migration.

---

## 4. Normative ownership model

### 4.1 `phalcom-modules`

Owns:

- project identity;
- module identity;
- source identity;
- source revision vocabulary;
- source overlay/disk lifecycle;
- module resolution;
- import resolution;
- linking;
- source-to-module provenance.

It may perform filesystem work on the worker/update path.

It must not depend on LSP/protocol types.

### 4.2 `phalcom-semantic`

Owns:

- semantic database/cache;
- `TypeStore`;
- semantic IDs;
- formal type/proof state;
- advisory state;
- declaration surfaces;
- dispatch;
- hierarchy;
- source semantic index;
- semantic occurrences;
- semantic diagnostics;
- semantic invalidation;
- semantic publication effects/stats;
- read-only editor query composition;
- protocol-neutral semantic presentation.

It must not depend on `tower-lsp`, LSP URLs, Markdown, or client state.

### 4.3 `phalcom-lsp`

Owns:

- `tower-lsp`;
- open buffers;
- recovered live parse;
- line/UTF-16 indexes;
- worker scheduling and latest-wins epochs;
- workspace filesystem discovery policy;
- source transport URI adaptation;
- closed-source text/line-index cache;
- exact/stale/unmapped policy;
- syntax-only recovery;
- snippets;
- Markdown;
- LSP object conversion;
- refresh notifications;
- virtual core source transport.

It must not own semantic identities or semantic reasoning.

---

## 5. Publication architecture

### 5.1 Single publication owner

`SemanticPublication` remains private to `phalcom-lsp`.

Recommended final shape:

```rust
pub struct AnalysisService {
    publication: Arc<SemanticPublication>,
    counters: PerfCountersHandle,
    shared: Arc<WorkerShared>,
    worker_thread: Option<JoinHandle<()>>,
}
```

The worker receives a clone of this exact `Arc`.

`AnalysisService` exposes:

```rust
pub fn snapshot(&self) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
    self.publication.load()
}
```

No external component gets a `publish` capability.

### 5.2 Why publication belongs behind `AnalysisService`

This makes an invalid topology difficult to construct:

```text
worker publication ─────┐
                        ├── same cell
request snapshot read ──┘
```

There is no second `SemanticDb`, no second publication field on `Backend`, and no public constructor for an unrelated publication cell.

`AnalysisService` remains conceptually a scheduler/source-ingestion service. `snapshot()` is a publication accessor, not a semantic query API.

### 5.3 Publication rules

- A request clones one `Arc`, never a semantic snapshot body.
- Publishing snapshot B never mutates snapshot A.
- Worker failure preserves the previous publication.
- Superseded worker candidates are not published.
- There is no empty legacy snapshot used as startup compatibility.

---

## 6. Request context

Use one neutral structure:

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}
```

Constructor:

```rust
pub fn new(
    document: DocumentSnapshot,
    semantic: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
    uri: &Url,
) -> Self
```

The snapshot is optional only because startup may precede the first semantic publication. This is not dual authority.

Delete transitional names:

```text
compiler
canonical_module
compiler_module()
new_with_compiler()
CompilerSemanticSnapshot
```

### 6.1 Source matching

`Exact` means:

- a canonical module is mapped;
- the pinned canonical source text equals the live document text.

`Stale` means:

- canonical source exists;
- text differs.

`Unmapped` means:

- no publication or no canonical source/module mapping.

No revision comparison is needed for semantic truth; text identity is the final range-safety check.

### 6.2 URI/source conversion

Create one protocol-only helper module, e.g.:

```text
phalcom-lsp/src/source_transport.rs
```

with pure, non-I/O conversion:

```rust
pub(crate) fn source_location_for_uri(uri: &Url) -> Option<SourceLocation>;
pub(crate) fn source_id_for_uri(uri: &Url) -> Option<SourceId>;
pub(crate) fn uri_for_source(source: &SourceLocation) -> Option<Url>;
```

Use the same helper at ingestion and request lookup.

Never call `Path::canonicalize()` in a semantic request.

---

## 7. Correct canonical source lifecycle

The existing heterogeneous batch is almost correct but lacks a way to represent a discovered disk file that was already read and parsed.

### 7.1 Required mutation vocabulary

Add:

```rust
WorkspaceSourceBatchMutation::SetDiskSnapshot {
    source: SourceLocation,
    text: Arc<str>,
    revision: SourceRevision,
    recovered_program: Option<Arc<Program>>,
}
```

Semantics:

```text
SetOverlay
  source text is an editor-owned live overlay
  provider overlay is installed
  WorkspaceSourceState.open_overlay = true

SetDiskSnapshot
  source text came from worker/scanner disk discovery
  no provider overlay is installed
  WorkspaceSourceState.open_overlay = false

RemoveOverlay
  close an editor overlay and fall back to current disk source

RefreshDisk
  reread a known disk source on worker path

RemoveSource
  delete source/module from workspace lifecycle
```

### 7.2 Why this matters

Using `SetOverlay` for scanner results makes a closed workspace file behave as if an editor buffer owns it. That can block proper disk refresh semantics and corrupt the meaning of “close”.

Source origin is lifecycle state, not a presentation detail.

### 7.3 Batch transaction rules

`apply_batch` must:

- stage all changes;
- rebuild exactly once;
- increment module generation once for a non-empty accepted batch;
- preserve module identity across ordinary source revisions;
- preserve existing overlays when another mutation fails;
- commit only after successful rebuild;
- report the actual error rather than translating all failure into cancellation.

---

## 8. Worker source model

### 8.1 Replace split pending maps

Current:

```text
file_updates
source_texts
core_update
core_text
removals
disk_refreshes
```

Target:

```rust
pub struct PendingSourceUpdate {
    pub revision: SourceRevision,
    pub text: Arc<str>,
    pub program: Arc<Program>,
}

pub struct PendingWork {
    pub file_updates: BTreeMap<Url, PendingSourceUpdate>,
    pub overlay_removals: BTreeSet<Url>,
    pub source_removals: BTreeSet<Url>,
    pub disk_refreshes: BTreeSet<Url>,
    // scan/scheduler fields...
}
```

No program-only production update may synthesize `Arc::from("")`.

### 8.2 Remove `source_catalog` as semantic mirror

The worker may retain a lightweight discovery registry, but it must not retain a second authoritative `(text, Program)` universe for semantics.

Allowed registry state:

```text
Url
SourceLocation
last source revision
open/closed discovery state
presentation cache pointer
```

Not allowed:

```text
semantic Program graph
manual import resolution
parallel module identity
```

The `SemanticWorkspaceSession`/`WorkspaceModuleSession` owns parsed canonical source state.

### 8.3 Import closure

Delete:

```text
extend_import_closure_with_source
resolve_source_import
```

If `AnalysisMode::Local` requires dependency-directed discovery, add a protocol-independent resolver query to `phalcom-modules` that uses `ProjectUniverse`/`ModuleResolver` and returns canonical source locations. The LSP may ask modules infrastructure **what source to discover**; it must not reproduce dot-count/path semantics.

---

## 9. Semantic generation and worker epochs are different concepts

Keep LSP worker epoch:

```text
epoch = scheduler freshness / cancellation ordering
```

Keep canonical semantic generation:

```text
SemanticSnapshot.generation = canonical workspace publication generation
```

Delete production LSP use of:

```rust
apply_module_mutations_at_generation(...)
```

Use:

```rust
session.apply_module_mutations(mutations)
```

If `apply_module_mutations_at_generation` has no non-transitional callers after search, remove it from `phalcom-semantic`.

A skipped/superseded publication may create gaps in semantic generation numbers. That is valid. Generation is identity/order, not an LSP-visible contiguous counter contract.

---

## 10. Error handling

Current `.ok()` conversion in the worker loses the reason a canonical update failed.

Target:

```rust
match state.semantic.apply_module_mutations(mutations) {
    Ok(result) => {
        if !cancelled() {
            state.commit_transport_state(...);
            publication.publish(result.snapshot.clone());
            emit_published(...);
        }
    }
    Err(error) => {
        emit(AnalysisEvent::Error {
            message: error.to_string(),
        });
        // transaction left canonical session coherent;
        // retain prior published snapshot;
        // do not commit matching transport/discovery state.
    }
}
```

Do not call a module/link/parse error “solve cancelled”.

Latest-wins cancellation remains a scheduler decision before/after an expensive update. Canonical semantic query cancellation remains compiler-owned where the semantic engine supports it.

---

## 11. Editor query facade: corrected contract

`EditorSemanticQuery` is retained, but its contract becomes strict:

> It may compose immutable canonical products. It may not perform source-string semantic interpretation.

### 11.1 Allowed operations

- exact occurrence/target lookup;
- formal fact lookup;
- advisory fact lookup;
- source-site lookup;
- canonical lexical-scope lookup;
- hierarchy/surface lookup;
- canonical visibility filtering;
- mapping a known `ValueShape`/formal type to receiver alternatives;
- collecting definition/reference sites;
- enumerating workspace symbols.

### 11.2 Forbidden operations

Delete from request-time editor queries:

- raw dotted-expression parsing;
- selector reconstruction from arbitrary source strings;
- call-chain simulation;
- constructor result synthesis from text;
- source-name guessing;
- literal inference;
- AST semantic surface building.

### 11.3 `self` and `super`

Do not recognize them by:

```rust
source_text.trim() == "self"
```

Publish source syntax identity during source-index build.

One acceptable canonical addition is:

```rust
pub enum SourceReceiverKind {
    SelfValue,
    SuperValue,
}

pub receiver_kinds: BTreeMap<SourceSiteId, SourceReceiverKind>;
```

The exact representation may instead be a richer `SourceSiteKind`; the requirement is that the parser/source-index build records the identity once and requests merely read it.

### 11.4 Chained expressions

For:

```phalcom
factory.make().member
```

the compiler source index/checker should already own an expression/source site for `factory.make()`.

`resolve_receiver_at(receiver_range)` must locate the most specific canonical expression site covering that exact receiver range and read its formal/advisory result.

If that site/fact is absent, return `None`.

Do **not** split `"factory.make()"` and re-execute dispatch in the editor query.

### 11.5 Canonical editor test matrix

Before LSP cutover depends on this facade, tests must cover at least:

1. local binding receiver;
2. parameter receiver;
3. field receiver;
4. call-result receiver;
5. `self`;
6. `super`;
7. class-object receiver;
8. module/import receiver where supported;
9. union receiver;
10. inherited members;
11. private visibility;
12. protected visibility;
13. definition sites;
14. reference sites;
15. shadowed visible symbols;
16. unknown receiver fails closed;
17. missing intermediate chain fact fails closed rather than parsing text.

---

## 12. Feature contracts after cutover

### 12.1 Diagnostics

Exact:

```text
live syntax diagnostics
+
canonical semantic diagnostics
```

Stale/unmapped:

```text
live syntax diagnostics only
```

### 12.2 Definition/references/rename

Exact only:

```text
offset
→ editor.target_at
→ canonical sites
→ source provenance
→ LSP locations
```

No `WorkspaceIndex` or selector-text fallback.

### 12.3 Workspace symbols

Use canonical declaration/callable/field source metadata across the snapshot.

Add:

```rust
EditorSemanticQuery::workspace_symbols(query)
```

if a convenient canonical enumeration is absent.

### 12.4 Completion

LSP keeps:

- incomplete syntax recovery;
- receiver range extraction;
- snippets;
- item kinds/details.

Canonical editor query owns:

- receiver identity;
- visible symbols;
- member enumeration;
- hierarchy;
- visibility.

Stale member completion returns no inferred members.

### 12.5 Hover

LSP keeps:

- keyword docs;
- Phaldoc lexical harvesting;
- Markdown layout.

Hover inputs become canonical:

- `DeclarationId`;
- `CallableId`;
- `FieldId`;
- `SourceBindingInfo`;
- `DeclarationSourceInfo`;
- `CallableSourceInfo`;
- `FieldSourceInfo`;
- `FormalPresentation`;
- canonical `AdvisoryFact`.

Delete canonical→legacy `ClassId`/`CallableId` conversion.

Phaldoc harvesting may scan raw text only **after** a canonical declaration range is known.

### 12.6 Signature help

Current canonical renderer is broadly the desired shape.

Keep syntax-only call-site recovery.

Resolve the callable canonically; render `CallableSemanticSignature` plus canonical advisory fallback.

### 12.7 Inlay hints

Keep:

- annotation-suppression AST walk;
- visible-range placement;
- LSP hint construction.

Delete DB/file-snapshot compatibility APIs and legacy fact types.

Displayed semantic content must come from canonical source/formal/advisory products.

### 12.8 Semantic tokens

Keep lexer and syntax declaration fallback.

Exact source receives canonical occurrence refinement.

Delete legacy `tokens_for(SemanticDb, ...)` and legacy occurrence types.

### 12.9 Stale/unmapped matrix

| Feature | Exact | Stale | Unmapped |
|---|---|---|---|
| syntax diagnostics | yes | yes | yes |
| semantic diagnostics | canonical | no | no |
| definition/references/rename | canonical | no | no |
| member completion | canonical | no inferred members | no inferred members |
| syntax/keyword completion | yes | yes | yes |
| keyword hover | yes | yes | yes |
| semantic hover | canonical | no | no |
| signature help | canonical | no | no |
| inlay hints | canonical | no | no |
| lexical semantic tokens | yes | yes | yes |
| semantic token refinement | canonical | no | no |
| workspace symbols | canonical snapshot | canonical snapshot | canonical snapshot |

---

## 13. `WorkspaceIndex`

`WorkspaceIndex` must be deleted.

It is not rescued as a “shallow” semantic fallback.

Workspace scanning may retain a transport/discovery cache. That cache may answer:

```text
which files have been discovered?
what source text/line index is cached for protocol presentation?
```

It may not answer:

```text
what does this selector refer to?
what members does this class have?
what are the semantic references?
```

---

## 14. Core documents

`core_documents.rs` is transport-only.

The current semantic core update queue should be removed because the canonical semantic session bootstraps core semantics independently.

### 14.1 Decision for current `sysroot_path`

For this retirement phase, configured/workspace core source selection is **presentation/provenance only**. It chooses which source text/URI the editor opens.

It does not replace or mutate the canonical semantic universe.

If configurable semantic sysroots are desired later, that must become an explicit protocol-independent compiler/session input in a separate design.

### 14.2 Remove

- `AnalysisService::enqueue_core_update*`;
- `PendingWork.core_update/core_text`;
- old `semantic/core_source.rs`;
- LSP direct `phalcom-native-surface` imports/dependency.

---

## 15. Compile-recovery strategy

The earlier plan assumed the repository could migrate with many tiny compile-safe provider-first commits. Current `main` is already broken, so repeating that sequence prolongs the invalid state.

The repair uses two classes of tasks:

### 15.1 Canonical tasks

These remain independently green:

```text
phalcom-modules
phalcom-semantic
```

Finish editor/query and source-lifecycle correctness first.

### 15.2 One bounded LSP ownership integration wave

Once canonical APIs are ready, migrate together:

```text
AnalysisService snapshot access
RequestContext
Backend construction
diagnostics
navigation
completion call sites
hover
signature-help call sites
inlay hints
semantic tokens
tests that construct SemanticDb/FileRevision
```

The wave is larger than an ideal feature commit because these files form one ownership graph. Splitting them at arbitrary API seams is what broke `main`.

The exit condition for this wave is non-negotiable:

```bash
cargo check -p phalcom-lsp --lib
```

green without compatibility aliases and without a running LSP semantic engine.

After that, delete unused `WorkspaceIndex` and `src/semantic/**` in focused cleanup commits.

---

## 16. Build discipline

No further broken retirement commit should be pushed to `main`.

Execution should occur on a dedicated branch/worktree.

Every commit after the recovery baseline must satisfy at least:

```bash
cargo fmt --all -- --check
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-lsp --lib
```

A task may intentionally create a failing focused test before implementation, but the commit itself must be green.

The current `main` exception is historical debt to repair, not a precedent.

---

## 17. Architecture gates

`semantic_boundary` must become mechanical proof, not only a directory-existence test.

Final gates:

1. `phalcom-lsp/src/semantic` absent.
2. `phalcom-lsp/src/index.rs` absent.
3. no `pub mod semantic;`.
4. no LSP-local definitions of:
   - `SemanticDb`;
   - `SemanticEngine`;
   - semantic `ClassId`/`CallableId`/`FieldId`;
   - `ScopeGraph`;
   - semantic `ModuleGraph`;
   - `DispatchResolver`;
   - `InferredValue`;
   - semantic `ValueShape`.
5. no production `crate::semantic`.
6. no canonical→legacy bridge names:
   - `canonical_callables`;
   - `canonical_target_to_lsp`;
   - `class_for_canonical`;
   - `member_surface_for_canonical`;
   - `CompilerResolvedReceiver`;
   - `SemanticResolvedReceiver`.
7. no LSP `phalcom-native-surface` dependency.
8. semantic/modules crates have no LSP dependency.
9. request feature files contain no filesystem reads/canonicalization.
10. worker contains no manual `resolve_source_import`.
11. worker contains no `apply_module_mutations_at_generation`.
12. query filesystem counters remain zero in integration tests.

---

## 18. Performance acceptance

Required structural results:

- one persistent `SemanticWorkspaceSession`;
- one module rebuild per accepted canonical mutation batch;
- one semantic publication per accepted canonical batch;
- zero legacy LSP flow/solver passes;
- zero canonical→legacy identity maps;
- no request-time semantic text-chain evaluation;
- `TypeStoreId` stable across ordinary edits;
- `ModuleId` stable across ordinary overlay revisions;
- closed-file navigation served from snapshot products without request disk I/O.

Do not require an arbitrary percentage speedup. The duplicate work must be structurally absent.

---

## 19. Definition of done

Retirement is complete only when:

- `cargo check -p phalcom-lsp` is green;
- `phalcom-lsp/src/semantic/` does not exist;
- `phalcom-lsp/src/index.rs` does not exist;
- `Backend` contains no `SemanticDb` and no `WorkspaceIndex`;
- `AnalysisService` drives one persistent `SemanticWorkspaceSession`;
- `AnalysisService::snapshot()` is the only LSP semantic publication read path;
- one private `SemanticPublication` exists per server;
- `RequestContext` pins at most one canonical snapshot;
- canonical module/source provenance maps requests;
- `SourceRevision` is the only LSP-to-module source revision type;
- scan-discovered files are disk snapshots, not overlays;
- worker no longer mirrors semantic source programs in `source_catalog`;
- worker no longer resolves relative imports itself;
- semantic generation is canonical, worker epoch is LSP-owned;
- editor receiver queries use published facts only;
- editor query matrix is fully covered;
- diagnostics/navigation/completion/hover/signature/inlay/tokens are canonical-only;
- stale/unmapped semantic behavior fails closed;
- core semantic meaning is not rebuilt in LSP;
- `phalcom-native-surface` is not a direct LSP dependency;
- old semantic counters/bridges are gone;
- `semantic_boundary` is enabled and green;
- full semantic/LSP/workspace tests pass;
- Part 3 closure docs are updated;
- Part 4 remains blocked until those gates are satisfied.

---

## 20. Final architecture

```text
editor mutation / workspace discovery
              │
              ▼
      phalcom-lsp transport
      - DocumentStore
      - source URI adaptation
      - latest-wins scheduler
              │
              ▼
       AnalysisService worker
              │
              ▼
      phalcom-modules
      - source lifecycle
      - project/module identity
      - resolver/linker
              │
              ▼
      phalcom-semantic
      - SemanticWorkspaceSession
      - one semantic DB
      - one identity world
      - formal/advisory facts
      - source index
      - editor query composition
              │
              ▼
       Arc<SemanticSnapshot>
              │
              ▼
   private SemanticPublication
              │
              ▼
         RequestContext
              │
              ▼
 syntax recovery + LSP presentation
```

The key closure criterion is not that LSP “uses compiler data most of the time.”

It is:

> There is no second place left where a Phalcom semantic answer can be produced.
