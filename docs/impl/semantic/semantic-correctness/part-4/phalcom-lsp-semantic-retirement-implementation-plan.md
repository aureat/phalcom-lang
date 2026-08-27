# Phalcom Single Semantic World — `phalcom-lsp` Retirement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the entire alternative semantic implementation from `phalcom-lsp`, make one persistent `phalcom_semantic::SemanticWorkspaceSession` the only analyzer used by the editor, publish one canonical `Arc<phalcom_semantic::SemanticSnapshot>`, migrate every semantic LSP feature to canonical IDs/queries, delete legacy semantic fallbacks and `WorkspaceIndex`, and add mechanical gates that prevent a second semantic world from reappearing.

**Architecture:** `phalcom-lsp` remains a protocol/source-buffer/scheduling/presentation crate. `phalcom-semantic` owns all language semantics and immutable semantic products. `phalcom-modules` owns project/module/source identity and source lifecycle. The LSP worker sends source mutations into the persistent canonical session, publishes a canonical immutable snapshot into a tiny LSP-owned publication cell, and request handlers pin that one snapshot for the lifetime of a request. Stale or unmapped editor text degrades to syntax-only assistance; it never invokes an alternative semantic analyzer.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-common`; `phalcom-modules`; `phalcom-semantic`; `tower-lsp 0.20`; Tokio; existing semantic DB/query infrastructure and canonical source indexes.

**Spec:** `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_tech_spec.md`

**Grounded repository:** `aureat/phalcom-lang`  
**Grounded branch:** `main`  
**Grounded HEAD:** `24919cd26019c6b5ffa72b069fa4692255ab0108`  
**Grounded date:** 2026-08-27

## Global Constraints

- [ ] Treat `24919cd26019c6b5ffa72b069fa4692255ab0108` as the implementation baseline for this plan. If `main` advances before implementation begins, re-run the inventory in Task 0 and amend paths/anchors before changing code.
- [ ] Do not implement any Part 4 semantic feature while this plan is incomplete. Part 4 documents may remain in the repository, but implementation is blocked by this Part 3 closure gate.
- [ ] Do not copy or move `phalcom-lsp/src/semantic/**` into `phalcom-semantic`. Missing canonical capability must be implemented against canonical identities/products.
- [ ] Do not add compatibility type aliases such as `type CallableId = phalcom_semantic::CallableId` under an LSP `semantic` module. The legacy ownership namespace must disappear.
- [ ] Do not introduce `tower_lsp` or `lsp_types` dependencies into `phalcom-semantic` or `phalcom-modules`.
- [ ] Do not perform semantic inference in LSP request handlers. AST/lexer use in the LSP is allowed only for syntax recovery, cursor context, source annotation detection, snippets, lexical tokenization, and protocol presentation.
- [ ] Preserve the latest-wins worker model, immutable snapshot publication, open-buffer priority, progressive workspace discovery, and nonblocking request behavior.
- [ ] Every migration task follows TDD: add/adjust a focused test first, prove it fails for the intended reason, make the smallest implementation change, prove it passes, then run the relevant crate suite.
- [ ] Keep changes in reviewable commits. Do not defer all legacy deletion to one final mega-commit; each feature cutover must remove its own fallback immediately once canonical parity is established.
- [ ] Semantic truth on stale source must never come from a fallback analyzer. Staleness may reduce completeness, never change semantic authority.
- [ ] Preserve canonical semantic identity across ordinary edits: persistent `WorkspaceId`, `TypeStoreId`, module identity, and canonical declaration/callable/field identity must not be recreated merely because the editor buffer changed.
- [ ] Do not preserve legacy public APIs solely because tests import them. Rewrite the tests to the target API.
- [ ] `dashmap` remains an LSP dependency after retirement because `DocumentStore` still uses `DashMap`; do not remove it as part of semantic cleanup.
- [ ] Remove the direct `phalcom-native-surface` dependency from `phalcom-lsp` only after the last direct import is removed. Canonical semantic/core crates may continue to depend on it.
- [ ] All final request-path filesystem reads and canonicalization operations must be zero. Filesystem discovery and worker-side source lifecycle are allowed; semantic queries must operate on published snapshot products.

---

# 1. Verified Baseline and Why the Order Matters

At the grounded HEAD, the migration is not hypothetical. Both worlds are actively present.

`phalcom-lsp/src/analysis_service.rs` imports:

```rust
use crate::semantic::{
    CompilerSemanticSnapshot,
    FileRevision,
    SemanticDb,
    SemanticEngine,
    SemanticGeneration,
    SemanticSnapshot,
    SourceAnalysisDepth,
};
```

The worker constructs both:

```rust
let mut engine = SemanticEngine::new_with_counters(...);
let mut compiler_workspace_state = CompilerWorkspaceState::default();
```

and the canonical workspace state contains a persistent:

```rust
phalcom_semantic::SemanticWorkspaceSession
```

The update path still runs the legacy engine first, then refreshes the canonical workspace, attaches the canonical snapshot back to the legacy engine, and publishes the legacy wrapper.

`phalcom-lsp/src/semantic/mod.rs` still defines the LSP-owned `SemanticDb`; `phalcom-lsp/src/semantic/snapshot.rs` still contains both legacy products and an optional compiler snapshot plus `canonical_callables`; `phalcom-lsp/src/request_context.rs` still pins both snapshots; `phalcom-lsp/src/backend.rs` still contains both legacy and canonical receiver/member/navigation paths; `phalcom-lsp/src/inlay_hints.rs` still consumes legacy file facts; and `phalcom-lsp/src/index.rs` remains a text-derived semantic compatibility index.

The implementation order below is designed around one rule:

> Build the canonical capability immediately before the consumer that needs it, switch that consumer completely, then delete the superseded compatibility path.

That prevents a prolonged third transitional architecture.

---

# 2. File Map

## 2.1 Create

- [ ] `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_tech_spec.md`
  - Install the accompanying technical specification in-repository.
- [ ] `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_implementation_plan.md`
  - Install this plan in-repository.
- [ ] `phalcom-semantic/src/editor.rs`
  - Canonical, protocol-independent IDE/query facade over one `SemanticSnapshot`.
- [ ] `phalcom-lsp/src/publication.rs`
  - Tiny immutable snapshot publication cell; no semantic algorithms.
- [ ] `phalcom-lsp/src/core_documents.rs`
  - LSP-only virtual/configured core document transport after semantic core construction is removed from the LSP.
- [ ] `phalcom-semantic/tests/semantic/integration/editor.rs`
  - Canonical editor-query integration tests.
- [ ] `phalcom-lsp/tests/semantic_boundary.rs`
  - Physical/dependency/forbidden-symbol architecture regression gate.

## 2.2 Modify

- [ ] `phalcom-modules/src/session.rs`
  - Add a heterogeneous transactional source-mutation batch API that relinks once.
- [ ] `phalcom-modules/src/query.rs`
  - Add immutable source-to-module provenance queries without filesystem work.
- [ ] `phalcom-semantic/src/session.rs`
  - Add one semantic batch entry point and expose publication directly.
- [ ] `phalcom-semantic/src/snapshot.rs`
  - Publish reverse source provenance and `editor()` facade entry point.
- [ ] `phalcom-semantic/src/source_index/scope.rs`
  - Publish source declaration/member metadata needed by editor presentation.
- [ ] `phalcom-semantic/src/source_index/builder.rs`
  - Populate canonical declaration/callable/field source metadata during the existing AST source-index build.
- [ ] `phalcom-semantic/src/source_index/mod.rs`
  - Expose source metadata queries.
- [ ] `phalcom-semantic/src/presentation.rs`
  - Add canonical advisory `ValueShape` rendering and callable/member presentation helpers that do not contain LSP types.
- [ ] `phalcom-semantic/src/lib.rs`
  - Export editor facade and new presentation/source metadata types.
- [ ] `phalcom-semantic/tests/semantic/integration/mod.rs`
  - Register `mod editor;`.
- [ ] `phalcom-semantic/tests/semantic/integration/source_index.rs`
  - Add source metadata coverage to the existing canonical source-index test suite.
- [ ] `phalcom-semantic/tests/semantic/integration/workspace.rs`
  - Add canonical batch publication/reuse tests where appropriate.
- [ ] `phalcom-lsp/src/documents.rs`
  - Replace legacy `FileRevision` with canonical `phalcom_modules::SourceRevision`.
- [ ] `phalcom-lsp/src/analysis_service.rs`
  - Remove `SemanticEngine`, legacy `SemanticDb`, old snapshot wrapper, old revision/generation types, canonical replay bridge, and legacy semantic counters.
- [ ] `phalcom-lsp/src/request_context.rs`
  - Pin exactly one canonical semantic snapshot.
- [ ] `phalcom-lsp/src/backend.rs`
  - Remove dual-world fields/helpers/fallbacks; route semantic questions to canonical editor queries.
- [ ] `phalcom-lsp/src/completion.rs`
  - Delete legacy receiver model; use canonical editor facade.
- [ ] `phalcom-lsp/src/hover.rs`
  - Delete legacy semantic types and AST semantic-surface reconstruction; consume canonical source/member presentation.
- [ ] `phalcom-lsp/src/signature_help.rs`
  - Delete legacy renderer; consume canonical callable presentation; remove duplicate `ValueShape` renderer.
- [ ] `phalcom-lsp/src/inlay_hints.rs`
  - Replace legacy file/local facts with canonical source/formal/advisory queries.
- [ ] `phalcom-lsp/src/semantic_tokens.rs`
  - Keep lexical pass, replace legacy semantic refinement with canonical occurrences/source sites.
- [ ] `phalcom-lsp/src/diagnostics.rs`
  - Keep protocol conversion; make canonical diagnostics the only semantic input.
- [ ] `phalcom-lsp/src/perf.rs`
  - Remove counters that describe the deleted LSP analyzer; retain scheduler/protocol counters and report canonical semantic update stats separately.
- [ ] `phalcom-lsp/src/lib.rs`
  - Remove `pub mod semantic;` and stale module documentation; add `publication`/`core_documents` as appropriate.
- [ ] `phalcom-lsp/Cargo.toml`
  - Remove direct `phalcom-native-surface`; register new `semantic_boundary` test.
- [ ] `phalcom-lsp/tests/single_world_cutover.rs`
  - Replace legacy DB-mediated publication test with direct publication/session test.
- [ ] `phalcom-lsp/tests/integration.rs`
  - Update navigation/diagnostics/completion assertions to single-world behavior.
- [ ] `phalcom-lsp/tests/core_startup.rs`
  - Prove canonical core semantics and LSP virtual-source transport independently.
- [ ] `phalcom-lsp/tests/module_navigation.rs`
  - Prove canonical source/module provenance drives closed-file navigation.
- [ ] `phalcom-lsp/tests/professional_semantic_presentation.rs`
  - Prove hover/signature/inlay presentation consumes canonical facts.
- [ ] `docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_implementation_checklist.md`
  - Mark only genuinely completed closure items and link this phase.
- [ ] `.agents/skills/semantic-analysis-development/references/current-architecture.md`
  - Remove dual-world descriptions after code cutover.
- [ ] `.agents/skills/phalcom-semantic-model/references/current-implementation-map.md`
  - Update semantic ownership map.
- [ ] `.agents/skills/phalcom-semantic-model/references/modules-and-incrementality.md`
  - Update canonical workspace/session and source lifecycle documentation.

## 2.3 Delete

Delete the entire legacy semantic package after all consumers are cut over:

- [ ] `phalcom-lsp/src/semantic/analyzer.rs`
- [ ] `phalcom-lsp/src/semantic/callable.rs`
- [ ] `phalcom-lsp/src/semantic/core_source.rs`
- [ ] `phalcom-lsp/src/semantic/dispatch.rs`
- [ ] `phalcom-lsp/src/semantic/engine.rs`
- [ ] `phalcom-lsp/src/semantic/facts.rs`
- [ ] `phalcom-lsp/src/semantic/flow.rs`
- [ ] `phalcom-lsp/src/semantic/ids.rs`
- [ ] `phalcom-lsp/src/semantic/infer.rs`
- [ ] `phalcom-lsp/src/semantic/invalidation.rs`
- [ ] `phalcom-lsp/src/semantic/module_graph.rs`
- [ ] `phalcom-lsp/src/semantic/occurrence.rs`
- [ ] `phalcom-lsp/src/semantic/query.rs`
- [ ] `phalcom-lsp/src/semantic/scope.rs`
- [ ] `phalcom-lsp/src/semantic/snapshot.rs`
- [ ] `phalcom-lsp/src/semantic/source.rs`
- [ ] `phalcom-lsp/src/semantic/surface.rs`
- [ ] `phalcom-lsp/src/semantic/mod.rs`
- [ ] `phalcom-lsp/src/index.rs`

---

# 3. Task 0 — Pin the Closure Baseline and Add Characterization Gates

**Purpose:** Make future implementation agents prove what exists before changing it and prevent accidentally using stale assumptions from `e1c8764…`.

**Files:**
- Create the two Part 3 retirement docs.
- Create `phalcom-lsp/tests/semantic_boundary.rs` initially as a characterization test.
- Modify `phalcom-lsp/Cargo.toml` to register the new integration test because `autotests = false`.

### 3.1 Install the docs

- [ ] Copy the technical spec to:
  `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_tech_spec.md`.
- [ ] Copy this implementation plan to:
  `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_implementation_plan.md`.
- [ ] Add a short “Part 3 architectural closure” entry to the Part 3 checklist linking both files.
- [ ] Explicitly record:
  `Grounded HEAD: 24919cd26019c6b5ffa72b069fa4692255ab0108`.

### 3.2 Add an intentionally failing final-boundary test

Register:

```toml
[[test]]
name = "semantic_boundary"
path = "tests/semantic_boundary.rs"
```

Add a test that locates the workspace root from `env!("CARGO_MANIFEST_DIR")` and eventually checks:

```rust
#[test]
fn lsp_has_no_legacy_semantic_package() {
    let lsp = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!lsp.join("src/semantic").exists());
}
```

At Task 0 this test is expected to fail. Keep it ignored until the deletion task, or split it into a helper plus `#[ignore = "enabled at final retirement"]`. Do not make CI permanently red during incremental migration.

Also add dependency-source guards that can pass immediately:

```rust
#[test]
fn semantic_crate_has_no_lsp_dependency() {
    let semantic_manifest = workspace_root().join("phalcom-semantic/Cargo.toml");
    let text = fs::read_to_string(semantic_manifest).unwrap();
    assert!(!text.contains("tower-lsp"));
    assert!(!text.contains("phalcom-lsp"));
}
```

- [ ] Run:
  `cargo test -p phalcom-lsp --test semantic_boundary semantic_crate_has_no_lsp_dependency`
- [ ] Commit:
  `docs(lsp): establish single-world retirement closure gate`

---

# 4. Task 1 — Add One Transactional Module-Source Batch

**Purpose:** The LSP worker must be able to apply a coalesced heterogeneous batch without replaying the entire source catalog and without relinking once per mutation.

**Files:**
- Modify `phalcom-modules/src/session.rs`.
- Add tests in the existing `#[cfg(test)]` section of `session.rs` or the crate’s existing session tests.

## 4.1 Add the canonical batch mutation type

Immediately after `WorkspaceSourceMutation`, add:

```rust
#[derive(Clone, Debug)]
pub enum WorkspaceSourceBatchMutation {
    SetOverlay {
        source: SourceLocation,
        text: Arc<str>,
        revision: SourceRevision,
        recovered_program: Option<Arc<phalcom_ast::ast::Program>>,
    },
    RemoveOverlay {
        source: SourceId,
    },
    RefreshDisk {
        source: SourceLocation,
        revision: SourceRevision,
    },
    RemoveSource {
        source: SourceId,
    },
}
```

Do not put `Url`, LSP version, `DocumentSnapshot`, or tower types here.

## 4.2 Extract one rebuild boundary

The current `WorkspaceModuleSession::apply` and `set_overlays_with_programs` perform overlapping staging/rollback/rebuild work.

Refactor internal mechanics so both single-mutation and batch paths converge on one internal commit function. Recommended internal shape:

```rust
struct StagedWorkspaceMutation {
    set_states: Vec<StagedSourceState>,
    remove_overlays: Vec<ModuleId>,
    remove_sources: Vec<(SourceId, ModuleId)>,
    changed_modules: BTreeSet<ModuleId>,
    removed_modules: BTreeSet<ModuleId>,
    identity_changes: BTreeSet<ModuleId>,
}
```

Add:

```rust
pub fn apply_batch<I>(
    &mut self,
    mutations: I,
) -> Result<WorkspaceModuleUpdate, WorkspaceModuleSessionError>
where
    I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
```

Required behavior:

- [ ] Resolve/read/parse all fallible source inputs before mutating the published source maps whenever possible.
- [ ] Accept a recovered parser `Program` from the LSP boundary for open syntax-error buffers, preserving the existing behavior of `set_overlays_with_programs`.
- [ ] Apply every staged overlay/source map change.
- [ ] Increment `self.generation` exactly once for the batch.
- [ ] Call `self.rebuild(...)` exactly once.
- [ ] On rebuild failure, restore `modules_by_source`, `sources_by_module`, `linked`, `generation`, and the overlay provider to the previous coherent state, following the existing rollback pattern in `set_overlay_batch`.
- [ ] Preserve open overlays when rolling back.
- [ ] Keep `apply(WorkspaceSourceMutation)` as a convenience API by translating one mutation into a one-element batch, unless a specialized single path remains measurably useful.
- [ ] Reimplement `set_overlays_with_programs` in terms of `apply_batch` rather than maintaining a second transaction implementation.

## 4.3 Add tests first

Add tests for:

```rust
#[test]
fn batch_updates_multiple_sources_with_one_generation_increment() { ... }

#[test]
fn batch_update_and_remove_rebuild_once_and_publish_consistent_sources() { ... }

#[test]
fn batch_accepts_recovered_program_for_invalid_live_text() { ... }

#[test]
fn failed_batch_restores_previous_workspace_state() { ... }
```

The first test should capture:

```rust
let before = session.generation();
let update = session.apply_batch([...]).unwrap();
assert_eq!(session.generation(), before + 1);
assert_eq!(update.sources.len(), expected);
```

The rollback test must assert both `session.sources()` and `session.linked()` still describe the previous coherent generation.

- [ ] Run the new test and prove it fails before implementation.
- [ ] Implement the batch.
- [ ] Run:
  `cargo test -p phalcom-modules`
- [ ] Commit:
  `feat(modules): add transactional workspace source batches`

---

# 5. Task 2 — Add One Semantic Batch Entry Point

**Purpose:** A coalesced LSP batch must enter `phalcom-semantic` once and produce one `SemanticWorkspacePublication`.

**Files:**
- Modify `phalcom-semantic/src/session.rs`.
- Modify `phalcom-semantic/src/lib.rs`.
- Modify `phalcom-semantic/tests/semantic/integration/workspace.rs`.

## 5.1 Add the API

Next to `apply_module_mutation`, add:

```rust
pub fn apply_module_mutations<I>(
    &mut self,
    mutations: I,
) -> Result<SemanticWorkspacePublication, WorkspaceModuleSessionError>
where
    I: IntoIterator<Item = phalcom_modules::WorkspaceSourceBatchMutation>,
{
    let update = self.module_session.apply_batch(mutations)?;
    Ok(self.update_module_workspace(update))
}
```

If `update_module_workspace` already has a generation-aware variant, route through the same canonical internal path used by `apply_module_mutation`.

The invariant is:

```text
one LSP coalesced batch
    -> one WorkspaceModuleSession::apply_batch
    -> one linked module update
    -> one SemanticWorkspaceSession update
    -> one SemanticWorkspacePublication
```

Not:

```text
N source mutations -> N link updates -> N semantic snapshots
```

## 5.2 Export the batch mutation type where appropriate

Prefer re-exporting from `phalcom-modules`, not cloning it into semantic:

```rust
pub use phalcom_modules::WorkspaceSourceBatchMutation;
```

only if that is consistent with the current `phalcom-semantic/src/lib.rs` re-export policy.

## 5.3 Tests

Add:

```rust
#[test]
fn semantic_batch_publishes_once_for_multiple_overlay_changes() { ... }

#[test]
fn semantic_batch_preserves_type_store_across_ordinary_edits() { ... }

#[test]
fn semantic_batch_preserves_module_identity_across_overlay_revisions() { ... }
```

Assert:

```rust
assert_eq!(second.snapshot.store.id(), first.snapshot.store.id());
assert!(second.snapshot.generation > first.snapshot.generation);
assert_eq!(second.snapshot.id.workspace(), first.snapshot.id.workspace());
```

Use the actual `SnapshotId` accessors available at implementation time; do not add identity getters purely for a test if equality of constituent IDs is already exposed.

- [ ] Run:
  `cargo test -p phalcom-semantic --test semantic workspace`
- [ ] Run:
  `cargo test -p phalcom-semantic`
- [ ] Commit:
  `feat(semantic): publish one snapshot per workspace batch`

---

# 6. Task 3 — Publish Canonical Reverse Source Provenance

**Purpose:** `RequestContext` and navigation must map LSP source documents to canonical `ModuleId` without legacy `DocumentModuleMap`, URI-as-module identity, request-time canonicalization, or filesystem reads.

**Files:**
- Modify `phalcom-semantic/src/snapshot.rs`.
- Modify `phalcom-modules/src/query.rs`.
- Modify semantic workspace/source-index tests.

## 6.1 Extend `ModuleQueryProducts`

Current `ModuleQueryProducts` stores:

```rust
pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
```

Add canonical reverse maps built once when the snapshot is created:

```rust
pub source_modules: Arc<BTreeMap<phalcom_modules::SourceId, ModuleId>>,
pub display_path_modules: Arc<BTreeMap<PathBuf, ModuleId>>,
```

In `ModuleQueryProducts::new`, derive these from `sources`:

```rust
let source_modules = sources
    .iter()
    .map(|(module, location)| (location.source_id.clone(), module.clone()))
    .collect();

let display_path_modules = sources
    .iter()
    .map(|(module, location)| (location.display_path.clone(), module.clone()))
    .collect();
```

Do not canonicalize paths here. `SourceLocation` is already provenance produced by module infrastructure.

## 6.2 Extend `ModuleQueryFacade`

Add fields for the two reverse maps and change `ModuleQueryFacade::new(...)` accordingly.

Add:

```rust
pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
    self.source_modules.get(source)
}

pub fn module_for_display_path(&self, path: &Path) -> Option<&ModuleId> {
    self.display_path_modules.get(path)
}
```

Update `SemanticSnapshot::module_queries()` to pass the new maps.

## 6.3 Add a snapshot convenience query

In `phalcom-semantic/src/snapshot.rs`, add:

```rust
pub fn module_for_source(&self, source: &phalcom_modules::SourceId) -> Option<&ModuleId> {
    self.module_products.source_modules.get(source)
}
```

Optionally add `module_for_display_path` if request code benefits.

## 6.4 Tests

- [ ] Create two physical source locations with distinct canonical `SourceId`s.
- [ ] Build/update a semantic workspace.
- [ ] Assert source→module lookup is exact.
- [ ] Assert no URI parsing or path string guessing is involved.
- [ ] Assert lookup remains stable across an overlay edit.

- [ ] Run:
  `cargo test -p phalcom-semantic --test semantic workspace`
- [ ] Commit:
  `feat(semantic): publish reverse source provenance`

---

# 7. Task 4 — Publish Canonical Source Declaration/Member Metadata

**Purpose:** Hover, signature help, inlay hints, definition locations, and semantic tokens currently retain legacy AST/member surfaces partly because canonical source products do not expose enough source-member metadata.

**Files:**
- Modify `phalcom-semantic/src/source_index/scope.rs`.
- Modify `phalcom-semantic/src/source_index/builder.rs`.
- Modify `phalcom-semantic/src/source_index/mod.rs`.
- Modify `phalcom-semantic/src/lib.rs`.
- Modify `phalcom-semantic/tests/semantic/integration/source_index.rs`.

## 7.1 Add protocol-independent source metadata types

In `source_index/scope.rs`, add:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceCallableKind {
    Method,
    Getter,
    Setter,
    IndexGet,
    IndexSet,
    Constructor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationSourceInfo {
    pub id: DeclarationId,
    pub name: Box<str>,
    pub declaration_site: SourceSiteId,
    pub name_range: SourceRange,
    pub declaration_range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSourceInfo {
    pub id: CallableId,
    pub kind: SourceCallableKind,
    pub declaration_site: SourceSiteId,
    pub name_range: SourceRange,
    pub declaration_range: SourceRange,
    pub parameter_name_ranges: Arc<[SourceRange]>,
    pub has_explicit_return_annotation: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSourceInfo {
    pub id: FieldId,
    pub declaration_site: SourceSiteId,
    pub name_range: SourceRange,
    pub declaration_range: SourceRange,
    pub has_explicit_annotation: bool,
}
```

Use existing source AST fields to populate exactly what exists. If an AST member has no reliable full declaration range, use the AST node’s current `range`; do not derive one by rescanning text.

Add to `SourceScopeIndex`:

```rust
pub declaration_sources: BTreeMap<DeclarationId, DeclarationSourceInfo>,
pub callable_sources: BTreeMap<CallableId, CallableSourceInfo>,
pub field_sources: BTreeMap<FieldId, FieldSourceInfo>,
```

Keep existing binding lookup map separate. If the current private field named `declarations` means binding declaration ranges, rename it to `binding_declarations` in the same commit to avoid confusing it with canonical class declarations.

## 7.2 Populate metadata in the existing builder

In `visit_class`:

```rust
let site = self.allocate_site(... class.name_range, SourceSiteKind::Declaration(...));
self.index.declaration_sources.insert(
    declaration.clone(),
    DeclarationSourceInfo {
        id: declaration.clone(),
        name: class.name.clone().into(),
        declaration_site: site.clone(),
        name_range: class.name_range,
        declaration_range: class.range,
    },
);
```

In `visit_member`:
- method: `Constructor` if `method.is_constructor`, otherwise `Method`.
- getter: `Getter`.
- setter: `Setter`.
- index get/set: appropriate index kind.
- field: record `FieldSourceInfo`.
- preserve actual parameter name ranges and explicit return/field annotation flags directly from the AST.

Change `visit_callable` to receive a `SourceCallableKind` and annotation flags rather than reconstructing them later.

## 7.3 Add query methods

In `SourceSemanticIndex`:

```rust
pub fn declaration_source(&self, id: &DeclarationId) -> Option<&DeclarationSourceInfo>;
pub fn callable_source(&self, id: &CallableId) -> Option<&CallableSourceInfo>;
pub fn field_source(&self, id: &FieldId) -> Option<&FieldSourceInfo>;
```

Resolve the owning module from the ID, then query the module’s `structure`.

## 7.4 Tests

Extend `source_index.rs` with a fixture containing:
- class
- constructor
- ordinary method
- getter
- setter
- index getter/setter
- annotated and unannotated fields
- annotated and unannotated return types

Assert kind and exact `name_range`/`declaration_range` from the parser.

- [ ] Prove test fails before metadata exists.
- [ ] Implement.
- [ ] Run:
  `cargo test -p phalcom-semantic --test semantic source_index`
- [ ] Commit:
  `feat(semantic): publish canonical source member metadata`

---

# 8. Task 5 — Centralize Canonical Semantic Presentation Primitives

**Purpose:** LSP files should render LSP objects, not reimplement language-semantic shape/type interpretation.

**Files:**
- Modify `phalcom-semantic/src/presentation.rs`.
- Modify `phalcom-semantic/src/lib.rs`.
- Add tests in `phalcom-semantic/tests/semantic/integration/presentation.rs`.

## 8.1 Move `ValueShape` text rendering out of the LSP

`phalcom-lsp/src/signature_help.rs` currently contains `render_compiler_shape`.

Add to `phalcom-semantic/src/presentation.rs`:

```rust
pub struct AdvisoryPresenter;

impl AdvisoryPresenter {
    pub fn present_shape(shape: &crate::ValueShape) -> String {
        // canonical language-level spelling only
    }
}
```

Move the exhaustive shape mapping there. It may emit:
- `?`
- `Never`
- `Unit`
- class instance/class-object spelling
- tuple/list/set/map/range
- selector/method/family forms
- unions

It must not return Markdown, `MarkupContent`, or LSP labels.

## 8.2 Add callable presentation projection

Add a protocol-neutral record:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallablePresentation {
    pub callable: CallableId,
    pub selector: String,
    pub kind: SourceCallableKind,
    pub owner_name: Box<str>,
    pub parameters: Arc<[ParameterPresentation]>,
    pub return_type: FormalPresentation,
    pub documentation: Option<Arc<str>>,
}
```

Only add fields with canonical provenance. Documentation may initially be `None` until Task 9 if source Phaldoc attachment is not yet canonical.

Avoid a second “semantic surface”; this is a read-only rendering projection over canonical IDs/signatures/source metadata.

## 8.3 Tests

- [ ] Move representative shape rendering expectations from LSP tests into semantic presentation tests.
- [ ] Add known/formal/dynamic/unknown callable parameter/return presentation tests.
- [ ] Run:
  `cargo test -p phalcom-semantic --test semantic presentation`
- [ ] Commit:
  `refactor(semantic): centralize IDE-neutral semantic presentation`

---

# 9. Task 6 — Add the Canonical Editor Query Facade

**Purpose:** `backend.rs`, `completion.rs`, and `hover.rs` currently stitch canonical facts together into request-time semantic algorithms. Centralize that reasoning once in `phalcom-semantic`.

**Files:**
- Create `phalcom-semantic/src/editor.rs`.
- Modify `phalcom-semantic/src/lib.rs`.
- Modify `phalcom-semantic/src/snapshot.rs`.
- Create `phalcom-semantic/tests/semantic/integration/editor.rs`.
- Modify `phalcom-semantic/tests/semantic/integration/mod.rs`.

## 9.1 Define facade types

Use canonical IDs only:

```rust
use crate::{
    CallableId, DeclarationId, FieldId, ModuleId, SemanticTargetId,
    SemanticSnapshot, SourceSiteId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiverMode {
    Instance,
    Class,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverAlternative {
    pub declaration: DeclarationId,
    pub mode: ReceiverMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedReceiver {
    pub alternatives: Arc<[ReceiverAlternative]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessContext {
    pub enclosing_declaration: Option<DeclarationId>,
    pub enclosing_callable: Option<CallableId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditorMemberTarget {
    Callable(CallableId),
    Field(FieldId),
}

#[derive(Clone, Debug)]
pub struct EditorMember {
    pub target: EditorMemberTarget,
    pub owner: DeclarationId,
    pub visibility: crate::MemberVisibility,
}
```

Do not add LSP completion kinds, URIs, ranges converted to lines, snippets, or Markdown.

## 9.2 Add facade entry point

In `SemanticSnapshot`:

```rust
pub fn editor(&self) -> EditorSemanticQuery<'_> {
    EditorSemanticQuery::new(self)
}
```

`EditorSemanticQuery` stores only:

```rust
snapshot: &'a SemanticSnapshot
```

It owns no cache/revision/invalidation state.

## 9.3 Add source/target queries

Implement:

```rust
pub fn target_at(&self, module: &ModuleId, offset: usize)
    -> Option<SemanticTargetId>;

pub fn definition_sites(&self, target: &SemanticTargetId)
    -> Vec<SourceSiteId>;

pub fn reference_sites(&self, target: &SemanticTargetId)
    -> Vec<SourceSiteId>;

pub fn access_context_at(&self, module: &ModuleId, offset: usize)
    -> AccessContext;
```

`target_at` should delegate to canonical occurrence/source-index products; do not reinterpret occurrence hints as exact targets.

Definition/reference sites should classify roles using canonical `SemanticOccurrence`/`OccurrenceRole`. If the current reverse target map contains all sites but does not distinguish role, filter through the owning `OccurrenceIndex`.

## 9.4 Move receiver resolution from `Backend`

Port the semantic reasoning currently in `Backend::compiler_receiver_for_range` into:

```rust
pub fn resolve_receiver_at(
    &self,
    module: &ModuleId,
    range: SourceRange,
) -> Option<ResolvedReceiver>;
```

Required evidence order:
1. exact formal expression/source-site facts;
2. exact formal binding facts;
3. canonical advisory fact at the same site;
4. canonical target/declaration denotation;
5. `self`/`super` through the canonical source/callable context;
6. class-object identity for declaration-valued expressions;
7. no AST name guessing.

The query may use the retained canonical parsed program only when source syntax identity is needed to select an existing source site. It must not build a new semantic surface or infer a type directly from a literal in request time. If literal shape is needed by IDE features, that fact must already exist in canonical formal/advisory products.

## 9.5 Add member queries

Implement:

```rust
pub fn members_for_receiver(
    &self,
    receiver: &ResolvedReceiver,
    access: &AccessContext,
) -> Vec<EditorMember>;

pub fn resolve_member(
    &self,
    receiver: &ResolvedReceiver,
    selector: &Selector,
    access: &AccessContext,
) -> Vec<EditorMember>;
```

Use:
- canonical hierarchy;
- `DeclarationSurface`;
- canonical callable/field IDs;
- `MemberVisibility`;
- canonical dispatch.

No LSP-side superclass walks after this task.

Visibility rules must be centralized here:
- public: always;
- private: same defining declaration;
- protected: defining declaration or subclass context;
- internal: only whatever canonical privilege model already defines; if no public IDE privilege model exists, do not invent one in LSP.

## 9.6 Add visible-symbol query

Implement:

```rust
pub fn visible_symbols_at(
    &self,
    module: &ModuleId,
    offset: usize,
) -> Vec<VisibleSymbol>;
```

Back it with `SourceScopeIndex::visible_bindings_at` plus canonical module/class targets. This replaces any need for LSP scope reconstruction.

## 9.7 Tests

Create `tests/semantic/integration/editor.rs`, register `mod editor;`.

Cover at minimum:
- [ ] local binding receiver;
- [ ] parameter receiver;
- [ ] field receiver;
- [ ] result of a canonical call;
- [ ] `self`;
- [ ] `super`;
- [ ] class object;
- [ ] union receiver with multiple alternatives;
- [ ] inherited member;
- [ ] private/protected visibility;
- [ ] exact target definition/reference sites;
- [ ] shadowed visible symbols;
- [ ] unknown receiver returns `None`/empty rather than guessing.

Run:
`cargo test -p phalcom-semantic --test semantic editor`

Then:
`cargo test -p phalcom-semantic`

Commit:
`feat(semantic): add canonical editor query facade`

---

# 10. Task 7 — Add the LSP Publication Cell and Migrate Revisions

**Purpose:** Replace `phalcom_lsp::semantic::SemanticDb` publication duties before deleting its semantic duties.

**Files:**
- Create `phalcom-lsp/src/publication.rs`.
- Modify `phalcom-lsp/src/lib.rs`.
- Modify `phalcom-lsp/src/documents.rs`.
- Begin modifying `phalcom-lsp/src/analysis_service.rs`.
- Modify tests that construct `FileRevision`.

## 10.1 Create publication cell

`phalcom-lsp/src/publication.rs`:

```rust
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct SemanticPublication {
    current: RwLock<Option<Arc<phalcom_semantic::SemanticSnapshot>>>,
}

impl SemanticPublication {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
        self.current.read().expect("semantic publication lock poisoned").clone()
    }

    pub fn publish(
        &self,
        snapshot: Arc<phalcom_semantic::SemanticSnapshot>,
    ) {
        *self.current.write().expect("semantic publication lock poisoned") = Some(snapshot);
    }
}
```

This is intentionally tiny. Do not add:
- file queries;
- class queries;
- compiler snapshot aliases;
- counters;
- mutation methods;
- target translation.

## 10.2 Replace `FileRevision`

In `documents.rs`:

Replace:

```rust
use crate::semantic::FileRevision;
```

with:

```rust
use phalcom_modules::SourceRevision;
```

Change `Document.revision`, `DocumentSnapshot.revision`, `open_or_update*`, and `bump_revision` to `SourceRevision`.

Where the code currently constructs `FileRevision(*entry)`, construct `SourceRevision(*entry)`.

Continue this replacement through:
- `analysis_service.rs`;
- backend cache helpers;
- integration tests.

Do not create an LSP alias named `FileRevision`.

## 10.3 Replace legacy `SemanticGeneration`

For internal worker events, use the canonical snapshot generation directly:

```rust
AnalysisEvent::Published {
    generation: u64,
    effects: PublicationEffects,
}
```

or carry `SnapshotId` if existing event consumers genuinely need identity. Do not keep a legacy semantic generation newtype.

## 10.4 Publication tests

Add to `single_world_cutover.rs`:

```rust
#[test]
fn publication_cell_pins_exact_canonical_arc() {
    // publish Arc A, load A
    // publish Arc B, already-loaded A remains valid
    // new load returns B
}
```

- [ ] Run:
  `cargo test -p phalcom-lsp --test single_world_cutover`
- [ ] Commit:
  `refactor(lsp): publish canonical semantic snapshots directly`

---

# 11. Task 8 — Rewrite `AnalysisService` Around One `SemanticWorkspaceSession`

**Purpose:** Remove the most expensive architectural duplication: two semantic engines per edit.

**Primary file:** `phalcom-lsp/src/analysis_service.rs`

**Related files:** `backend.rs`, `perf.rs`, `single_world_cutover.rs`.

This task is structural and should be split into reviewable subcommits if needed, but do not leave both engines active after the final subcommit.

## 11.1 Replace constructor ownership

Current conceptual constructor:

```rust
AnalysisService::new(db: Arc<SemanticDb>)
```

Target:

```rust
pub fn new(
    publication: Arc<SemanticPublication>,
) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>)
```

If the worker needs injected counters/configuration, pass only those protocol/scheduler dependencies.

`Backend::new` becomes:

```rust
let semantic = Arc::new(SemanticPublication::new());
let (analysis, analysis_events) = AnalysisService::new(semantic.clone());
```

Rename `Backend.semantic` to `semantic_publication` to make ownership obvious.

## 11.2 Replace worker state

Delete:

```rust
let mut engine = SemanticEngine::new_with_counters(...);
let mut compiler_workspace_state = CompilerWorkspaceState::default();
```

Replace with one persistent state:

```rust
struct AnalysisWorkerState {
    semantic: phalcom_semantic::SemanticWorkspaceSession,
}
```

If source discovery/cache state remains in `AnalysisService`, keep it separate and explicitly non-semantic.

## 11.3 Delete the canonical replay bridge

Delete:
- `CompilerWorkspaceState`;
- `refresh_compiler_workspace`;
- `publish_persistent_compiler_workspace`;
- `publish_engine`;
- any `engine.set_compiler_analysis(...)`;
- `DocumentModuleMap` construction/clear/repopulation inside the worker;
- canonical callable index construction;
- compiler snapshot attachment to legacy snapshot.

Instead translate pending LSP work into:

```rust
Vec<phalcom_modules::WorkspaceSourceBatchMutation>
```

and call:

```rust
let publication = worker_state.semantic.apply_module_mutations(mutations)?;
semantic_publication.publish(publication.snapshot.clone());
```

## 11.4 Split source close from source deletion

Current `PendingWork.removals` is too ambiguous for a canonical source lifecycle.

Replace with:

```rust
pub overlay_removals: BTreeSet<Url>,
pub source_removals: BTreeSet<Url>,
pub disk_refreshes: BTreeSet<Url>,
```

Protocol behavior:
- `didClose` on a file-backed open document -> remove overlay or refresh disk, preserving the source/module.
- watched-file delete -> `RemoveSource`.
- non-file synthetic close -> remove the source if it has no persistent backing.
- watched-file change -> `RefreshDisk` unless an open document wins.

Keep `mark_open`/`mark_closed` for race policy; they are scheduling/source-lifecycle state, not semantic inference.

## 11.5 Require source text on live updates

The canonical module session owns source text. Remove production use of any API that enqueues a `Program` without text.

Preferred LSP API:

```rust
pub fn enqueue_file_update(
    &self,
    uri: Url,
    revision: SourceRevision,
    text: Arc<str>,
    program: Arc<Program>,
)
```

If callers currently pass `Program` by value, use `Arc<Program>` to avoid unnecessary clones and align with `ParsedModuleUnit`.

Delete or `#[cfg(test)]` any no-text compatibility overload, then update tests to supply source.

## 11.6 Remove LSP-side semantic import closure walking

Delete worker helpers that manually resolve semantic import identity, including any functions equivalent to:
- `extend_import_closure_with_source`;
- `resolve_source_import`;
- URI/path-based import target guessing.

Workspace scan may discover files syntactically, but import identity/reachability comes from `WorkspaceModuleSession`/linker products.

## 11.7 Publish effects from canonical publication

Replace merged legacy/canonical effects with mapping solely from `phalcom_semantic::SemanticPublicationEffects`.

LSP policy mapping should look conceptually like:

```rust
impl From<&SemanticPublicationEffects> for PublicationEffects {
    fn from(effects: &SemanticPublicationEffects) -> Self {
        Self {
            diagnostics_changed: effects.diagnostics_changed,
            inlay_hints_changed:
                effects.formal_changed
                || effects.advisory_changed
                || effects.source_index_changed,
            semantic_tokens_changed:
                effects.source_index_changed
                || effects.declaration_index_changed,
        }
    }
}
```

Use actual current field semantics. Do not invent semantic invalidation in the LSP.

## 11.8 Rewrite freshness acceptance

Current `AnalysisService::accepts_revision` checks legacy `db.file_snapshot`.

Change acceptance to worker/source-epoch state plus canonical source revisions already owned by `WorkspaceModuleSession`.

If a public check is needed, expose a source revision query from `SemanticWorkspaceSession`/module session while on the worker thread; do not put mutable source revisions into the publication cell.

## 11.9 Tests first / updated tests

Rewrite `worker_reuses_compiler_snapshot_store_across_edits` in `single_world_cutover.rs`:

Before:
```rust
let db = Arc::new(phalcom_lsp::semantic::SemanticDb::new());
let (service, _events) = AnalysisService::new(db.clone());
...
let first = db.compiler_snapshot().expect(...);
```

After:
```rust
let publication = Arc::new(SemanticPublication::new());
let (service, _events) = AnalysisService::new(publication.clone());

service.enqueue_file_update(
    uri.clone(),
    SourceRevision(1),
    Arc::from(source1),
    Arc::new(parse(source1, 0).program),
);
service.flush();

let first = publication.load().expect("semantic publication");
```

Assert:
- same TypeStore across ordinary edits;
- same canonical module ID;
- different immutable snapshot IDs;
- one publication per processed coalesced batch.

Add a counter/test hook proving the old flow/solver never runs—once `SemanticEngine` is gone this becomes structurally true.

- [ ] Run:
  `cargo test -p phalcom-lsp --test single_world_cutover`
- [ ] Run analysis service/status/logging tests.
- [ ] Commit:
  `refactor(lsp): route analysis worker through one semantic session`

---

# 12. Task 9 — Separate Core Document Transport From Core Semantics

**Purpose:** Delete `phalcom-lsp/src/semantic/core_source.rs` without losing virtual core-document UX.

**Files:**
- Create `phalcom-lsp/src/core_documents.rs`.
- Modify `backend.rs`.
- Modify `core_startup.rs`.
- Later delete `semantic/core_source.rs`.
- Later remove `phalcom-native-surface` from LSP Cargo.

## 12.1 Preserve only transport concerns

Move/rewrite only:
- configured physical core path discovery;
- workspace physical canonical universe source discovery if still useful for opening source;
- bundled virtual text generation;
- physical URI bookkeeping;
- `phalcom://` virtual document rendering/serving.

Do **not** move:
- `build_core_surface`;
- legacy `ClassSurface` construction;
- native member merging;
- legacy `ClassId`/`CallableId`;
- `NativeReturnShape` semantic use;
- native parameter reconstruction.

## 12.2 Canonical semantic core rule

The authoritative core semantic universe is whatever `phalcom-semantic`/canonical module/core-surface infrastructure already publishes.

A configured LSP core path must not silently replace canonical semantics by rebuilding an LSP-only core surface.

If configuration is intended only to choose source/provenance for editor navigation, keep it transport-only.

If the project still requires a configured core path to alter compiler semantics, implement that as an explicit input to canonical module/semantic infrastructure **before** deleting legacy behavior. The input must be protocol-independent (`PathBuf`/source provider config), and the resulting core declarations must enter the same canonical snapshot as every other semantic product.

Do not preserve semantic override behavior solely inside `core_documents.rs`.

## 12.3 Use canonical core presentation

Where the LSP needs generated virtual core content, use canonical `phalcom_semantic::core_surface` presentation products (e.g. `ClassPresentation::render_virtual_source`) or canonical builtin source provider output. Do not read `phalcom_native_surface` directly.

## 12.4 Tests

Update `core_startup.rs` to prove separately:
- [ ] a fresh canonical semantic workspace knows core declarations;
- [ ] completion/hover over core members uses canonical IDs;
- [ ] virtual core URI rendering works without an LSP semantic DB;
- [ ] configured physical source URI selection affects navigation/provenance only unless canonical semantic configuration explicitly says otherwise;
- [ ] no direct `phalcom_native_surface` import exists in LSP source.

Commit:
`refactor(lsp): separate core document transport from semantics`

---

# 13. Task 10 — Collapse `RequestContext` to One Semantic Snapshot

**Purpose:** Establish the request-level invariant before migrating individual handlers.

**File:** `phalcom-lsp/src/request_context.rs`

## 13.1 Replace the struct

Current dual-world shape must become:

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}
```

Delete:
- legacy `semantic: Arc<crate::semantic::SemanticSnapshot>`;
- `compiler: Option<Arc<...>>`;
- old LSP `ModuleId`;
- `compiler_module()` adapter;
- any canonical↔legacy mapping.

## 13.2 Resolve module from canonical source provenance

Backend request creation:

```rust
let semantic = self.semantic_publication.load()?;
let module = source_id_for_uri(&uri)
    .as_ref()
    .and_then(|source_id| semantic.module_for_source(source_id))
    .cloned();
```

`source_id_for_uri` is a pure boundary conversion from a file URI to the exact source identity form used when the worker registered it. Do not call `canonicalize()` on every request. If URI normalization is needed, normalize once during ingestion and retain the normalized `SourceId` in a non-semantic LSP source catalog keyed by URI.

## 13.3 Keep `SourceMatch`, change its comparison source

`SourceMatch` remains:

```rust
pub enum SourceMatch {
    Exact,
    Stale,
    Unmapped,
}
```

Exact:
- canonical module exists;
- `semantic.sources[module].text == document.text`.

Stale:
- canonical module exists;
- text differs.

Unmapped:
- no canonical source/module mapping.

Do not compare a legacy file revision/snapshot.

## 13.4 Add request-consistency tests

Tests:
- [ ] request pins snapshot A; publication changes to B; all request queries remain on A;
- [ ] exact live text;
- [ ] stale live text;
- [ ] unmapped synthetic text;
- [ ] no request path reads disk.

Commit:
`refactor(lsp): pin one canonical snapshot per request`

---

# 14. Task 11 — Cut Diagnostics Over Completely

**Purpose:** Remove the simplest semantic consumer first and establish the stale policy.

**Files:**
- `phalcom-lsp/src/diagnostics.rs`
- `phalcom-lsp/src/backend.rs`
- diagnostics tests.

## 14.1 Simplify combined diagnostics

Replace any signature that accepts:
- optional compiler snapshot;
- legacy document/module map;
- legacy semantic diagnostics

with:

```rust
fn combined_diagnostics_for(
    documents: &DocumentStore,
    semantic: Option<&phalcom_semantic::SemanticSnapshot>,
    uri: &Url,
) -> Option<DiagnosticPublication>
```

Policy:
- always include live parser syntax diagnostics;
- include semantic diagnostics only when `SourceMatch::Exact`;
- never include stale semantic ranges;
- no legacy semantic diagnostics.

Prefer constructing a `RequestContext` and passing it into the helper if that avoids duplicate source-match logic.

## 14.2 Publication event

In `Backend::initialized` event loop, replace:

```rust
let semantic_snapshot = semantic.snapshot();
combined_diagnostics_for(
    &documents,
    semantic_snapshot.compiler_snapshot.as_deref(),
    &semantic_snapshot.documents,
    &uri,
)
```

with a direct canonical publication load and request/source match.

## 14.3 Tests

- [ ] exact semantic diagnostic appears;
- [ ] stale semantic diagnostic suppressed, syntax diagnostic remains;
- [ ] unmapped document receives syntax diagnostics only;
- [ ] no duplicate semantic diagnostics from two worlds.

Commit:
`refactor(lsp): make diagnostics canonical-only`

---

# 15. Task 12 — Cut Navigation, References, Rename, and Workspace Symbols Over

**Purpose:** Remove identity translation and make canonical source identity the only navigation substrate.

**Files:**
- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/index.rs` consumers
- `module_navigation.rs`
- integration tests.

## 15.1 Replace target lookup

Delete `compiler_target_at_request` naming once there is no competitor. Use:

```rust
let target = request
    .module
    .as_ref()
    .and_then(|module| request.semantic.editor().target_at(module, offset));
```

For stale/unmapped source: return no semantic target.

## 15.2 Replace definition locations

Create one LSP conversion helper:

```rust
fn source_site_location(
    snapshot: &phalcom_semantic::SemanticSnapshot,
    site: &SourceSiteId,
) -> Option<Location>
```

Algorithm:
1. `snapshot.source_site(site)` -> module/range;
2. module -> `snapshot.module_queries().definition_source(module)`/canonical source provenance;
3. source location -> URI;
4. line index from open document or closed-source cache;
5. range -> LSP `Range`.

No legacy `ClassSurface`, `MemberSurface`, `DocumentModuleMap`, or URI-bearing semantic ID.

## 15.3 Definition

Delete all branches over legacy:

```rust
SemanticTarget::Binding
SemanticTarget::Class
SemanticTarget::Callable
SemanticTarget::Field
SemanticTarget::Module
```

Replace with canonical `SemanticTargetId` + canonical definition site query.

Module targets use `ModuleQueryFacade::definition_source`.

## 15.4 References

Use canonical `occurrences_for_target`/`reference_sites`.

Do not fallback to `WorkspaceIndex.references`.

## 15.5 Rename

Resolve one exact canonical target, collect canonical write/read occurrences according to current rename semantics, group by source module, convert ranges. If the target is unresolved/stale, reject/return no edit rather than selector-text renaming.

## 15.6 Workspace symbols

Before deleting `WorkspaceIndex`, replace selector-map-backed workspace symbols with canonical source/declaration indexes. Add a canonical editor query such as:

```rust
pub fn workspace_symbols(&self, query: &str) -> Vec<WorkspaceSymbolView>;
```

if the existing source index cannot efficiently enumerate declarations/callables/fields.

This query belongs in `phalcom-semantic::editor`, not in LSP.

## 15.7 Tests

`module_navigation.rs`:
- [ ] cross-file declaration;
- [ ] import alias;
- [ ] selective import;
- [ ] callable;
- [ ] field;
- [ ] closed file;
- [ ] same spelling in two modules resolves by canonical target identity;
- [ ] stale source refuses semantic navigation rather than using text selector fallback.

Commit:
`refactor(lsp): route navigation through canonical source identity`

---

# 16. Task 13 — Cut Completion Over and Delete LSP Receiver Semantics

**Files:**
- `phalcom-lsp/src/completion.rs`
- `phalcom-lsp/src/backend.rs`
- semantic editor tests and LSP completion tests.

## 16.1 Delete dual receiver types

Delete:

```rust
SemanticResolvedReceiver
CompilerResolvedReceiver
```

Keep one LSP-side thin adapter only if needed for rendering, preferably use canonical:

```rust
phalcom_semantic::ResolvedReceiver
```

directly.

Rename any function named `compiler_*completion*` to its neutral name.

## 16.2 Delete `Backend::compiler_receiver_for_range`

All semantic receiver resolution becomes:

```rust
let receiver = request
    .semantic
    .editor()
    .resolve_receiver_at(module, receiver_range)?;
```

Delete any LSP code that manually combines:
- scope resolution;
- formal binding facts;
- advisory facts;
- expression source sites;
- initializer AST recognition;
- hierarchy;
- class object inference.

## 16.3 Delete LSP hierarchy/member filtering

Completion should ask:

```rust
let access = editor.access_context_at(module, offset);
let members = editor.members_for_receiver(&receiver, &access);
```

Then `completion.rs` converts each canonical member into:
- label;
- completion kind;
- snippet;
- detail;
- sort/filter text.

Selector/snippet construction remains LSP presentation.

## 16.4 Preserve syntax-only stale completion

Keep `syntax_visible_completions` or equivalent for stale/unmapped documents, but enforce:
- keywords;
- locally visible syntax names only if recovered syntactically;
- snippets;
- import path syntax completion if based on canonical module products that are position-independent;
- no inferred receiver surface.

Do not fallback to legacy receiver/member semantics.

## 16.5 Tests

- [ ] exact member completion from canonical receiver;
- [ ] inherited canonical member;
- [ ] visibility filtering;
- [ ] union receiver behavior;
- [ ] class-side vs instance-side;
- [ ] stale `foo.` returns syntax-only/no inferred members;
- [ ] incomplete syntax still gives safe snippets.

Commit:
`refactor(lsp): make completion canonical-only`

---

# 17. Task 14 — Cut Hover Over and Canonicalize Phaldoc Attachment

**Files:**
- `phalcom-lsp/src/hover.rs`
- `phalcom-lsp/src/backend.rs`
- `phalcom-semantic` source/presentation as needed
- `professional_semantic_presentation.rs`.

## 17.1 Remove legacy semantic imports

Delete imports of:

```rust
crate::semantic::{
    BindingInfo,
    ClassId,
    Confidence,
    InferredValue,
    SemanticBindingKind,
    ValueShape,
    ...
}
```

Use canonical semantic types/presentation only.

## 17.2 Keep lexical keyword hover in LSP

`keyword_at_offset` and static keyword blurbs are syntax/prose behavior and can remain.

## 17.3 Remove semantic AST reconstruction

Delete any hover path that:
- calls `build_module_surface` from a request;
- builds a member/class surface from `DocumentSnapshot.parse.program`;
- consults `WorkspaceIndex::definition_info`;
- uses `legacy_hover_at`;
- infers owner/kind by selector text.

## 17.4 Canonicalize member kind/source metadata

Use Task 4 metadata for:
- callable kind;
- name/declaration range;
- owner;
- field/callable distinction.

## 17.5 Canonicalize Phaldoc source attachment

Current `hover.rs` harvests `///` text by scanning raw source after a declaration is known. The scanning itself may remain an LSP presentation concern **only if** it receives an exact canonical declaration/member source range first.

Preferred stronger closure:
- add a `DocCommentSource`/`SourceDocumentation` field to canonical source metadata during source-index build;
- attach raw doc text/range without interpreting Markdown;
- let LSP render it.

If changing the parser/source index for raw comments is disproportionate, keep the lexical harvest helper in LSP but make its input:

```rust
(source_text, canonical_declaration_range)
```

and remove all text-based declaration resolution. This is acceptable because it does not infer semantics; it decorates an already-resolved canonical semantic target.

## 17.6 Core hover

Use canonical core presentation metadata; no direct native catalog lookup from LSP.

## 17.7 Tests

- [ ] class hover from canonical declaration ID;
- [ ] method/getter/setter/constructor kind;
- [ ] canonical signature/formal state;
- [ ] field hover;
- [ ] local binding hover;
- [ ] Phaldoc attaches to canonical declaration;
- [ ] core/native docs come from canonical core presentation;
- [ ] stale source returns keyword/syntax hover only, no legacy semantic answer.

Commit:
`refactor(lsp): make hover canonical-only`

---

# 18. Task 15 — Cut Signature Help Over

**File:** `phalcom-lsp/src/signature_help.rs` and backend signature-help handler.

## 18.1 Keep call-site syntax recovery

`CallSite` and `call_site_at` remain LSP syntax recovery.

## 18.2 Delete legacy renderer

Delete:

```rust
pub fn render_signature_help(
    member: &MemberSurface,
    formal: Option<&FormalCallablePresentation>,
    advisory: Option<&CallableSignature>,
    ...
)
```

Rename:

```rust
render_compiler_signature_help
```

to:

```rust
render_signature_help
```

but change it to consume one canonical protocol-neutral callable presentation result rather than separately stitching signature/store/advisory in LSP where possible.

## 18.3 Delete local `render_compiler_shape`

Use:

```rust
phalcom_semantic::AdvisoryPresenter::present_shape(...)
```

## 18.4 Resolve callable canonically

For a dotted call:
- syntax recovers receiver range + selector candidate;
- canonical editor facade resolves receiver/member.

For an unqualified call:
- canonical target/source scope query resolves it.

Stale/unmapped: no semantic signature help.

## 18.5 Tests

- [ ] normal method;
- [ ] labeled params;
- [ ] rest params;
- [ ] inferred/advisory fallback display;
- [ ] class-side constructor;
- [ ] inherited method;
- [ ] stale document no semantic signature.

Commit:
`refactor(lsp): make signature help canonical-only`

---

# 19. Task 16 — Rewrite Inlay Hints Against Canonical Source/Formal/Advisory Products

**Purpose:** This is the largest feature-specific cutover because `inlay_hints.rs` still depends heavily on legacy file snapshots/local facts.

**Files:**
- `phalcom-lsp/src/inlay_hints.rs`
- semantic source metadata/editor/presentation tests
- professional presentation tests.

## 19.1 Preserve policy and protocol construction

Keep:
- `HintPolicy`;
- visible range filtering;
- “suppress obvious” user policy;
- LSP `InlayHint` construction;
- explicit source annotation suppression if it is purely syntactic.

## 19.2 Delete legacy input types

Remove:
- `SemanticDb`;
- legacy `SemanticSnapshot`;
- `FileSemanticSnapshot`;
- legacy `InferredValue`;
- legacy `Confidence`;
- legacy `SemanticBindingKind`;
- legacy `ValueShape`.

Change the main entry point to:

```rust
pub fn hints_for_request(
    request: &RequestContext,
    visible: Range,
    policy: HintPolicy,
    suppress_obvious: bool,
) -> Vec<InlayHint>
```

with canonical `request.semantic` only.

Delete standalone DB-based compatibility functions if only tests use them; rewrite tests around `RequestContext`/canonical fixtures.

## 19.3 Replace binding hints

Use canonical source bindings:

```rust
module_index.structure.bindings
```

and canonical source-site/formal/advisory facts:
- source binding site;
- formal fact at site;
- advisory fact at site.

Explicit annotations can still be suppressed by `ExplicitAnnotationIndex`, but the type/value being hinted must come from canonical products.

## 19.4 Replace parameter hints

Task 4 `CallableSourceInfo.parameter_name_ranges` + canonical callable formal/advisory summary provide:
- exact parameter source position;
- formal parameter type;
- advisory runtime shape when formal is unknown.

Do not find legacy binding facts by `(name, range)`.

## 19.5 Replace field hints

Use `FieldSourceInfo` and canonical declaration/member surface/advisory data.

## 19.6 Replace return hints

Use `CallableSourceInfo`, its explicit-return-annotation flag, and canonical callable semantic signature/formal analysis.

Do not rediscover member kind through legacy `MemberKind`.

`find_return_hint_offset` may remain a syntax-only source placement helper if it needs AST punctuation position. Change its `member_kind` argument to `SourceCallableKind` or, preferably, use the canonical declaration source range plus cached parse to locate punctuation. It must not decide semantic kind itself.

## 19.7 Replace closure hints

The current recursive AST walk looks up legacy local facts. Replace fact lookup with canonical source binding sites and `semantic_site_at`/formal/advisory facts.

The AST recursion may remain solely to select visible closure parameter declarations and annotation syntax. It may not infer their types.

## 19.8 Tests

Add cases:
- [ ] local `let`;
- [ ] method parameter;
- [ ] closure parameter;
- [ ] field;
- [ ] return;
- [ ] explicit annotation suppression;
- [ ] formal fact preferred over advisory;
- [ ] advisory shown only under policy;
- [ ] invalid/blocked formal state not laundered into a stable hint;
- [ ] stale source returns no semantic inlays.

Commit:
`refactor(lsp): source inlay hints from canonical semantic facts`

---

# 20. Task 17 — Cut Semantic Tokens Over

**File:** `phalcom-lsp/src/semantic_tokens.rs`

## 20.1 Keep lexer pass

Keep:
- token legend;
- lexical classification;
- string interpolation handling;
- syntax attributes;
- AST-assisted declaration-name fallback when canonical source is stale/unmapped.

This is syntax presentation, not semantic inference.

## 20.2 Delete legacy semantic refinement

Remove imports:

```rust
crate::semantic::{SemanticDb, SemanticOccurrenceKind}
```

For exact source, refine tokens using:
- `request.semantic.source_index`;
- canonical `OccurrenceKind`/`SemanticTargetId`;
- Task 4 source metadata.

Map canonical categories to LSP token kinds:
- declaration -> class;
- callable -> method;
- field -> property;
- binding parameter -> parameter;
- local binding -> variable.

## 20.3 Stale policy

On stale/unmapped:
- lexical tokenization and safe AST syntax classification only;
- no old semantic occurrence fallback.

## 20.4 Tests

- [ ] exact semantic field/callable/parameter classifications;
- [ ] stale text still lexically colored;
- [ ] no stale semantic-range application;
- [ ] canonical source changes trigger token refresh effect.

Commit:
`refactor(lsp): refine semantic tokens from canonical source index`

---

# 21. Task 18 — Delete `WorkspaceIndex`

**Purpose:** Once navigation, hover, completion, workspace symbols, references, rename, and semantic tokens are canonical, the text-derived compatibility index has no legitimate semantic role.

**Files:**
- Delete `phalcom-lsp/src/index.rs`.
- Modify `backend.rs`.
- Modify `analysis_service.rs`.
- Modify `lib.rs`.
- Modify tests.

## 21.1 Remove backend ownership

Delete:

```rust
index: Arc<WorkspaceIndex>,
```

and all:
- `index.update_file`;
- `index.remove_file`;
- `definitions`;
- `references`;
- `definition_info`;
- class-member lookup;
- workspace symbol lookup.

## 21.2 Remove worker scan indexing

Workspace scan should feed discovered source into the canonical workspace session and source cache only.

`indexed_files` may remain if it is strictly UI/source-discovery bookkeeping. Rename it to `discovered_files` if that better describes it.

## 21.3 Closed source behavior

Canonical semantic snapshot must contain closed discovered files. Navigation to them uses snapshot source provenance plus LSP `closed_sources` only for line-index/text presentation.

`closed_sources` must not store semantic identity or semantic facts.

## 21.4 Delete file and update docs

Delete `src/index.rs`.
Remove `pub mod index;` from `lib.rs`.

Run:
`rg "WorkspaceIndex|crate::index|DefinitionInfo|ClassMemberInfo" phalcom-lsp`

Expected: no production semantic references.

Commit:
`refactor(lsp): retire text-derived workspace semantic index`

---

# 22. Task 19 — Remove Legacy Semantic Performance Counters

**File:** `phalcom-lsp/src/perf.rs`

## 22.1 Keep LSP-owned counters

Retain counters for:
- source updates enqueued/coalesced/discarded;
- batches started/published/stale-discarded;
- scan/discovery work;
- refresh requests;
- query filesystem reads/canonicalizations (expected zero);
- scheduler/source race behavior.

## 22.2 Delete analyzer-owned counters

Remove fields that only described the deleted LSP engine:
- `flow_passes`;
- `solver_rounds`;
- `callables_analyzed`;
- `dirty_callables_seeded`;
- `solver_callables_visited`;
- `solver_callables_changed`;
- `semantic_candidate_state_clones`;
- `published_file_products_reused`;
- `published_class_products_reused`;
- `published_summary_products_reused`;
- `parameter_sources_replaced`;
- `parameter_slots_touched`;
- `parameter_slots_changed`.

Update reset/snapshot/serialization/tests accordingly.

## 22.3 Report canonical update stats

When useful for logs, take metrics from:

```rust
phalcom_semantic::SemanticUpdateStats
```

attached to each `SemanticWorkspacePublication`.

Do not mirror those numbers into a second mutable LSP semantic counter system unless they are explicitly telemetry totals; prefer logging the canonical per-update stats directly.

Commit:
`refactor(lsp): remove retired semantic-engine counters`

---

# 23. Task 20 — Physically Delete `phalcom-lsp/src/semantic/**`

**Purpose:** The architecture is not complete until the alternate implementation is physically gone.

## 23.1 Run pre-delete reference inventory

Before deletion:

```bash
rg "crate::semantic|phalcom_lsp::semantic|SemanticDb|SemanticEngine|FileSemanticSnapshot|SemanticResolvedReceiver|canonical_callables|canonical_target_to_lsp" phalcom-lsp
```

Every production reference must already be gone. Test references must have been migrated.

Also run:

```bash
rg "ClassId|CallableId|FieldId|ScopeGraph|ValueShape|InferredValue|DispatchResolver|ModuleGraph" phalcom-lsp/src
```

Interpret carefully:
- canonical imports such as `phalcom_semantic::CallableId` are allowed;
- locally defined semantic identity/engine types are not.

## 23.2 Delete all legacy files

Delete the full directory listed in §2.3.

## 23.3 Remove crate export

In `phalcom-lsp/src/lib.rs`, delete:

```rust
pub mod semantic;
```

Rewrite crate docs:
- remove “live semantic database and bounded local runtime-value inference”;
- describe `analysis_service` as scheduler;
- describe canonical semantic consumption;
- describe stale syntax-only policy.

## 23.4 Remove direct native dependency

Run:

```bash
rg "phalcom_native_surface|phalcom-native-surface" phalcom-lsp
```

If only `Cargo.toml` remains, delete:

```toml
phalcom-native-surface = { path = "../phalcom-native-surface" }
```

Do not remove semantic/native dependencies from canonical crates.

## 23.5 Keep `dashmap`

Do not remove `dashmap = "6.2.1"` because `documents.rs` still uses `DashMap`.

## 23.6 Build immediately

Run:

```bash
cargo check -p phalcom-lsp
cargo test -p phalcom-lsp --test single_world_cutover
```

Any compilation failure referring to deleted semantic types is a real migration gap. Fix by using canonical APIs, not by restoring aliases.

Commit:
`refactor(lsp)!: delete legacy semantic engine`

---

# 24. Task 21 — Turn the Architecture Gate On

**Files:**
- `phalcom-lsp/tests/semantic_boundary.rs`
- `phalcom-lsp/Cargo.toml`

Remove the `#[ignore]` from the final boundary tests.

## 24.1 Physical directory gate

```rust
#[test]
fn legacy_semantic_directory_does_not_exist() {
    assert!(!lsp_root().join("src/semantic").exists());
}
```

## 24.2 Forbidden local semantic definitions

Read every `.rs` file under `phalcom-lsp/src` and reject definitions matching a narrow list:

```text
struct SemanticDb
struct SemanticEngine
struct ScopeGraph
struct ModuleGraph
struct DispatchResolver
struct InferredValue
enum ValueShape
struct ClassId
struct CallableId
struct FieldId
enum SemanticTarget
```

The gate should inspect definition syntax strings (`"struct SemanticEngine"`, etc.), not merely names, so canonical imports do not create false positives.

## 24.3 Dependency gate

Parse/read:
- `phalcom-lsp/Cargo.toml`: must depend on `phalcom-semantic`.
- `phalcom-semantic/Cargo.toml`: must not depend on `tower-lsp` or `phalcom-lsp`.
- `phalcom-modules/Cargo.toml`: must not depend on `tower-lsp` or `phalcom-lsp`.

## 24.4 Direct native dependency gate

Assert `phalcom-lsp/Cargo.toml` no longer includes `phalcom-native-surface`.

## 24.5 No legacy bridge names

Reject production source occurrences of:
- `compiler_snapshot` when used as a nested/optional secondary snapshot;
- `canonical_callables`;
- `canonical_target_to_lsp`;
- `CompilerResolvedReceiver`;
- `SemanticResolvedReceiver`;
- `legacy_hover_at`;
- `build_module_surface` inside LSP.

Be careful not to ban the term “compiler” from logs/tests generically.

## 24.6 Query-I/O gate

Keep existing counters/tests and assert semantic requests do not increment:
- `query_filesystem_canonicalizations`;
- `query_disk_reads`.

Run:
`cargo test -p phalcom-lsp --test semantic_boundary`

Commit:
`test(lsp): enforce single semantic world`

---

# 25. Task 22 — Rewrite the Single-World/Protocol Test Ownership

**Purpose:** Move language-semantic correctness tests to the semantic crate and leave LSP tests focused on protocol projection and consistency.

## 25.1 Semantic crate owns

Verify or add canonical tests for:
- inference;
- branch joins;
- generics;
- dispatch;
- subtype/assignability;
- field lifecycle;
- parameter evidence;
- callable summaries;
- import identity;
- module invalidation;
- source target identity;
- reference identity;
- scope resolution;
- constructor semantics;
- incremental reuse.

Do not duplicate these as LSP tests.

## 25.2 LSP owns

Keep/add tests for:
- byte↔UTF-16 conversion;
- open document snapshots;
- request source match;
- snapshot pinning;
- latest-wins/coalescing;
- syntax-only stale degradation;
- semantic result→LSP `Location`;
- semantic result→hover Markdown;
- semantic result→completion item/snippet;
- semantic result→signature help;
- semantic result→inlay hint;
- semantic result→semantic token;
- diagnostic conversion;
- workspace scanning events;
- virtual core documents;
- refresh notifications.

## 25.3 Rewrite `single_world_cutover.rs`

Final assertions must include:

```rust
#[test]
fn worker_publication_is_canonical_snapshot() { ... }

#[test]
fn request_pins_one_snapshot_even_if_worker_publishes_newer_snapshot() { ... }

#[test]
fn one_edit_batch_advances_one_semantic_publication() { ... }

#[test]
fn ordinary_edits_reuse_type_store_and_module_identity() { ... }

#[test]
fn stale_source_never_invokes_semantic_fallback() { ... }
```

There must be no construction of `phalcom_lsp::semantic::SemanticDb`.

Commit:
`test(lsp): align tests with canonical semantic ownership`

---

# 26. Task 23 — Close Part 3 Documentation and Block/Unblock Part 4 Correctly

**Files:**
- Part 3 checklist.
- Current architecture agent references.
- Any LSP ADR/current architecture docs that claim the LSP owns semantics.

## 26.1 Update the Part 3 checklist

Mark completed only after final tests pass:
- persistent canonical workspace session is the sole analyzer;
- canonical source/formal/advisory products drive IDE features;
- no duplicate IDs/graphs/engines;
- no request-time semantic AST reconstruction;
- one immutable semantic snapshot per request;
- workspace source/module identity canonicalized;
- stale source syntax-only;
- LSP old semantic package deleted.

Add an explicit completion record:

```text
Single Semantic World closure:
- completion commit: <sha>
- semantic directory absent
- semantic_boundary test green
- full workspace test green
```

## 26.2 Update agent architecture references

Remove references describing:
- LSP `SemanticDb`;
- LSP engine;
- LSP local flow/inference;
- compiler snapshot nested in LSP snapshot.

Replace with:

```text
AnalysisService = scheduling/source ingestion
SemanticWorkspaceSession = semantic owner
SemanticPublication = immutable Arc handoff
RequestContext = one canonical snapshot + source match
```

## 26.3 Part 4 gate

Only after all Definition of Done checks pass:
- change Part 4 docs from “blocked pending Part 3 architectural closure” to ready for implementation;
- re-ground Part 4 implementation plans to the new HEAD because paths/APIs will have changed substantially.

Do **not** blindly execute existing Part 4 line-level plans written against the dual-world architecture.

Commit:
`docs(semantic): close part 3 single-world takeover`

---

# 27. Detailed Code Replacement Map

This section gives direct “replace X with Y” anchors for the highest-risk files.

## 27.1 `phalcom-lsp/src/analysis_service.rs`

### Remove imports

Remove:

```rust
use crate::semantic::{
    CompilerSemanticSnapshot,
    FileRevision,
    SemanticDb,
    SemanticEngine,
    SemanticGeneration,
    SemanticSnapshot,
    SourceAnalysisDepth,
};
```

Add:

```rust
use phalcom_modules::{
    SourceId,
    SourceLocation,
    SourceRevision,
    WorkspaceSourceBatchMutation,
};
use phalcom_semantic::{
    SemanticPublicationEffects,
    SemanticUpdateStats,
    SemanticWorkspaceSession,
};
use crate::publication::SemanticPublication;
```

Keep `phalcom_ast::ast::Program` only for recovered parser programs supplied with live source mutations.

### Replace `CachedSource`

Before:

```rust
pub(crate) struct CachedSource {
    pub(crate) revision: FileRevision,
    pub(crate) text: Arc<str>,
    pub(crate) program: Arc<Program>,
    pub(crate) line_index: Arc<LineIndex>,
}
```

After:

```rust
pub(crate) struct CachedSource {
    pub(crate) revision: SourceRevision,
    pub(crate) text: Arc<str>,
    pub(crate) program: Arc<Program>,
    pub(crate) line_index: Arc<LineIndex>,
}
```

This cache is source/presentation only. Do not add semantic facts.

### Replace `PendingWork`

Before:

```rust
pub file_updates: BTreeMap<Url, (FileRevision, Program)>,
source_texts: BTreeMap<Url, Arc<str>>,
pub core_update: Option<(FileRevision, Program)>,
core_text: Option<Arc<str>>,
pub removals: BTreeSet<Url>,
pub disk_refreshes: BTreeSet<Url>,
...
```

After conceptually:

```rust
pub file_updates: BTreeMap<Url, PendingSourceUpdate>,
pub overlay_removals: BTreeSet<Url>,
pub source_removals: BTreeSet<Url>,
pub disk_refreshes: BTreeSet<Url>,
...
```

with:

```rust
pub struct PendingSourceUpdate {
    pub revision: SourceRevision,
    pub text: Arc<str>,
    pub program: Arc<Program>,
}
```

Core transport/config lives separately; do not special-case a legacy semantic “core update” in the semantic batch.

### Replace worker initialization

Before:

```rust
let mut engine = SemanticEngine::new_with_counters(...);
let mut compiler_workspace_state = CompilerWorkspaceState::default();
```

After:

```rust
let mut semantic = SemanticWorkspaceSession::new();
```

If canonical semantic workspace roots must be configured, set them through canonical module/session APIs before processing source mutations.

### Replace publication

Before:

```rust
let legacy = engine.apply_mutations_with_source_cancel_and_core_depth(...);
let compiler = refresh_compiler_workspace(...);
engine.set_compiler_analysis(...);
publish_engine(&db, &engine);
```

After:

```rust
let mutations = build_workspace_mutations(...);
let result = semantic.apply_module_mutations(mutations)?;
publication.publish(result.snapshot.clone());

let effects = PublicationEffects::from(&result.effects);
emit(AnalysisEvent::Published {
    generation: result.snapshot.generation,
    effects,
});
```

No nested snapshot. No semantic copy.

## 27.2 `phalcom-lsp/src/request_context.rs`

Delete every reference to `crate::semantic`.

Target imports:

```rust
use std::sync::Arc;
use phalcom_modules::ModuleId;
use phalcom_semantic::SemanticSnapshot;
use tower_lsp::lsp_types::Url;
```

Target fields:

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Arc<SemanticSnapshot>,
    pub module: Option<ModuleId>,
    pub source_match: SourceMatch,
}
```

No `compiler` field.

## 27.3 `phalcom-lsp/src/backend.rs`

### Backend fields

Before conceptually:

```rust
index: Arc<WorkspaceIndex>,
semantic: Arc<SemanticDb>,
analysis: AnalysisService,
```

After:

```rust
semantic_publication: Arc<SemanticPublication>,
analysis: AnalysisService,
```

Remove `index`.

### `request_context`

Before:
- load legacy snapshot;
- extract optional compiler snapshot;
- resolve old module;
- map to canonical module.

After:
- load canonical snapshot once;
- resolve canonical module from source provenance;
- compare canonical source text to live document;
- return one context.

### Receiver helper

Delete:
- `semantic_receiver...`;
- `compiler_receiver_for_range...`.

Replace all calls with:

```rust
request
    .module
    .as_ref()
    .and_then(|module| {
        request.semantic.editor().resolve_receiver_at(module, range)
    })
```

### Navigation

Delete legacy target branches and fallback index lookup. Use `SemanticTargetId`.

### Hover

Delete `legacy_hover_at` and old class/member conversion.

### Signature

Delete old member-surface path.

## 27.4 `phalcom-lsp/src/completion.rs`

Delete:

```rust
pub struct SemanticResolvedReceiver { ... }
pub struct CompilerResolvedReceiver { ... }
```

Do not replace them with an LSP-local copy. Import canonical `ResolvedReceiver`.

Completion helpers should accept canonical `EditorMember` records and produce `CompletionItem`.

## 27.5 `phalcom-lsp/src/signature_help.rs`

Delete imports:

```rust
use crate::semantic::{
    CallableSignature,
    FormalCallablePresentation,
    MemberSurface,
};
```

Delete the legacy renderer and local `render_compiler_shape`.

Use canonical presentation primitives.

## 27.6 `phalcom-lsp/src/inlay_hints.rs`

Delete:

```rust
use crate::semantic::{
    CompilerSemanticSnapshot,
    Confidence,
    FileSemanticSnapshot,
    InferredValue,
    SemanticBindingKind,
    SemanticDb,
    SemanticSnapshot,
    ValueShape,
};
```

Use:
- canonical `SemanticSnapshot`;
- `FormalFactSite`;
- `AdvisoryFact`;
- `SourceBindingInfo`;
- Task 4 source member info;
- `AdvisoryPresenter`.

## 27.7 `phalcom-lsp/src/semantic_tokens.rs`

Delete:

```rust
use crate::semantic::{SemanticDb, SemanticOccurrenceKind};
```

Use canonical occurrence kinds.

## 27.8 `phalcom-lsp/src/lib.rs`

Delete:

```rust
pub mod semantic;
pub mod index;
```

Add:

```rust
pub mod core_documents;
pub mod publication;
```

Rewrite docs so no statement says the LSP owns runtime-value inference or semantic database state.

---

# 28. Stale/Unmapped Behavior Matrix

Implement this matrix as tests, not only prose.

| Feature | Exact | Stale | Unmapped |
|---|---|---|---|
| Syntax diagnostics | yes | yes | yes |
| Semantic diagnostics | yes | no | no |
| Definition/references/rename | canonical | no semantic answer | no semantic answer |
| Receiver member completion | canonical | no inferred members | no inferred members |
| Keyword/snippet completion | yes | yes | yes |
| Import-path completion | canonical module data if position-independent | allowed if not range-dependent | limited |
| Hover keyword docs | yes | yes | yes |
| Semantic hover | canonical | no | no |
| Signature help | canonical | no semantic signature | no |
| Inlay hints | canonical | no | no |
| Semantic token lexical pass | yes | yes | yes |
| Semantic token refinement | canonical | no stale refinement | no |
| Workspace symbols | canonical workspace snapshot | canonical workspace snapshot | canonical workspace snapshot |

Critical rule:

```text
stale != use older semantic range optimistically
stale != use legacy semantic engine
stale == syntax assistance + non-position-dependent canonical workspace facts only
```

---

# 29. Performance Acceptance Plan

The migration is partly a correctness cleanup and partly removal of duplicate work. Prove both.

## 29.1 Before deleting legacy engine

Capture baseline on the grounded or immediately pre-cutover HEAD with existing perf tooling:
- semantic batch elapsed time;
- allocations/RSS if harness exists;
- flow/solver counters from old engine;
- canonical `SemanticUpdateStats`;
- completion/hover median request time;
- large-workspace scan/publish behavior.

## 29.2 After cutover

Expected structural result:
- zero legacy flow passes because code is deleted;
- zero legacy solver rounds because code is deleted;
- one canonical semantic update per processed source batch;
- same TypeStore across ordinary edits;
- no whole-table request materialization;
- zero query filesystem reads;
- zero query filesystem canonicalizations;
- lower CPU and lower retained semantic memory than dual-world baseline.

## 29.3 Required tests/bench assertions

At minimum:
- [ ] coalesced 10 edits result in fewer semantic publications than enqueued edits and exactly one publication for the final processed batch;
- [ ] one processed batch invokes `SemanticWorkspaceSession` once;
- [ ] request snapshot pinning does not clone the semantic snapshot;
- [ ] closed-file navigation does no disk read on request path;
- [ ] semantic completion does no AST semantic reconstruction.

Do not hard-code an arbitrary percent speedup as a correctness gate unless stable benchmark infrastructure supports it. The structural “one analyzer” invariant is mandatory; measured speedup is expected evidence.

---

# 30. Failure and Rollback Strategy

This work should be landed as a sequence of commits, but the repository must never intentionally support two authoritative semantic answers after a feature is cut over.

For each feature:
1. add canonical query/product if missing;
2. add parity test;
3. switch feature;
4. delete fallback in same commit or immediately following paired commit.

If a canonical gap is found:
- stop that feature migration;
- add the missing canonical product/query;
- do **not** restore or enhance legacy semantic inference.

If an LSP UX regression exists only on stale text:
- improve syntax recovery;
- shorten publication latency;
- add a non-semantic cached syntax product;
- do **not** revive semantic fallback.

If canonical source provenance cannot map a closed file:
- fix `WorkspaceModuleSession` ingestion or snapshot provenance;
- do **not** reintroduce URI-as-module semantic IDs.

If core presentation lacks metadata:
- extend canonical core presentation;
- do **not** import the native catalog directly into LSP.

---

# 31. Final Verification Commands

Run from workspace root in this order.

## 31.1 Formatting and compile

```bash
cargo fmt --all -- --check
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-lsp
```

## 31.2 Focused canonical tests

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-semantic --test semantic source_index
cargo test -p phalcom-semantic --test semantic editor
cargo test -p phalcom-semantic --test semantic presentation
cargo test -p phalcom-semantic --test semantic workspace
```

## 31.3 Focused LSP tests

```bash
cargo test -p phalcom-lsp --test single_world_cutover
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test professional_semantic_presentation
cargo test -p phalcom-lsp --test core_startup
cargo test -p phalcom-lsp --test integration
cargo test -p phalcom-lsp --test analysis_status
cargo test -p phalcom-lsp --test analysis_logging
cargo test -p phalcom-lsp --test semantic_boundary
```

## 31.4 Full crate/workspace verification

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-lsp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If the workspace intentionally has pre-existing unrelated clippy failures, record them explicitly; do not weaken the retirement-specific checks.

## 31.5 Physical architecture searches

All of these should return no forbidden production hits:

```bash
test ! -d phalcom-lsp/src/semantic

rg "pub mod semantic;" phalcom-lsp/src
rg "struct SemanticDb|struct SemanticEngine|enum ValueShape|struct ScopeGraph|struct ModuleGraph" phalcom-lsp/src
rg "canonical_callables|canonical_target_to_lsp|CompilerResolvedReceiver|SemanticResolvedReceiver|legacy_hover_at" phalcom-lsp/src
rg "phalcom_native_surface" phalcom-lsp/src
rg "crate::semantic" phalcom-lsp/src
```

Canonical type imports are allowed:

```bash
rg "phalcom_semantic::(DeclarationId|CallableId|FieldId|SemanticTargetId|SourceSiteId)" phalcom-lsp/src
```

Those are evidence of the desired one-way dependency.

---

# 32. Definition of Done

Do not declare this plan complete until every item below is true.

- [ ] `phalcom-lsp/src/semantic/` does not exist.
- [ ] `phalcom-lsp/src/index.rs` does not exist.
- [ ] `phalcom-lsp` defines no language-semantic identity universe.
- [ ] `phalcom-lsp` defines no semantic DB, semantic engine, flow engine, semantic dispatch resolver, semantic module graph, semantic fixed-point solver, or inferred-value lattice.
- [ ] One persistent `phalcom_semantic::SemanticWorkspaceSession` is the only analyzer driven by `AnalysisService`.
- [ ] One coalesced source batch causes one canonical module rebuild/link boundary and one semantic publication.
- [ ] `AnalysisService` publishes `Arc<phalcom_semantic::SemanticSnapshot>` directly.
- [ ] The publication cell contains no query or semantic mutation API.
- [ ] `RequestContext` pins exactly one canonical semantic snapshot.
- [ ] Every semantic result within one request comes from that pinned snapshot ID.
- [ ] Source/module mapping comes from canonical source provenance, not legacy URI/module maps.
- [ ] `FileRevision` is gone from LSP; source lifecycle uses `phalcom_modules::SourceRevision`.
- [ ] Canonical source metadata exposes declaration/callable/field identity and source ranges needed by IDE presentation.
- [ ] Receiver resolution is implemented in `phalcom-semantic`, not `backend.rs`.
- [ ] Hierarchy member lookup/visibility used by semantic completion is implemented in canonical editor queries.
- [ ] Diagnostics use canonical semantic diagnostics only.
- [ ] Definition/references/rename use canonical `SemanticTargetId`/source sites only.
- [ ] Completion uses canonical receiver/member queries only.
- [ ] Hover uses canonical semantic/source/presentation products only.
- [ ] Signature help uses canonical callable products only.
- [ ] Inlay hints use canonical formal/advisory/source facts only.
- [ ] Semantic-token semantic refinement uses canonical source occurrences only.
- [ ] Stale/unmapped documents have explicit syntax-only degradation.
- [ ] No stale semantic request falls back to text-derived semantic identity.
- [ ] Closed-file semantic navigation is served from the canonical workspace snapshot.
- [ ] Core semantics are canonical; LSP core handling is transport/presentation only.
- [ ] `phalcom-lsp` has no direct `phalcom-native-surface` dependency.
- [ ] `phalcom-semantic` and `phalcom-modules` have no `tower-lsp` dependency.
- [ ] Semantic correctness tests live in `phalcom-semantic`; LSP tests test protocol projection/scheduling/freshness.
- [ ] `semantic_boundary` is enabled and green.
- [ ] Query-path disk reads and filesystem canonicalization remain zero.
- [ ] TypeStore identity is reused across ordinary edits.
- [ ] Module identity is reused across ordinary overlays.
- [ ] Part 3 checklist records the single-world closure as complete.
- [ ] Part 4 plans are re-grounded to the post-retirement HEAD before implementation starts.

---

# 33. Recommended Commit Sequence

Keep the migration bisectable with approximately this sequence:

1. `docs(lsp): establish single-world retirement closure gate`
2. `feat(modules): add transactional workspace source batches`
3. `feat(semantic): publish one snapshot per workspace batch`
4. `feat(semantic): publish reverse source provenance`
5. `feat(semantic): publish canonical source member metadata`
6. `refactor(semantic): centralize IDE-neutral semantic presentation`
7. `feat(semantic): add canonical editor query facade`
8. `refactor(lsp): publish canonical semantic snapshots directly`
9. `refactor(lsp): route analysis worker through one semantic session`
10. `refactor(lsp): separate core document transport from semantics`
11. `refactor(lsp): pin one canonical snapshot per request`
12. `refactor(lsp): make diagnostics canonical-only`
13. `refactor(lsp): route navigation through canonical source identity`
14. `refactor(lsp): make completion canonical-only`
15. `refactor(lsp): make hover canonical-only`
16. `refactor(lsp): make signature help canonical-only`
17. `refactor(lsp): source inlay hints from canonical semantic facts`
18. `refactor(lsp): refine semantic tokens from canonical source index`
19. `refactor(lsp): retire text-derived workspace semantic index`
20. `refactor(lsp): remove retired semantic-engine counters`
21. `refactor(lsp)!: delete legacy semantic engine`
22. `test(lsp): enforce single semantic world`
23. `test(lsp): align tests with canonical semantic ownership`
24. `docs(semantic): close part 3 single-world takeover`

A feature cutover commit is not complete if it merely prefers the canonical path but leaves the legacy semantic fallback live.

---

# 34. Handoff to Part 4

After this plan is complete, Part 4 starts from a materially simpler semantic architecture:

```text
source/editor mutation
        │
        ▼
phalcom-modules
  source/module lifecycle
  project identity
  linking
        │
        ▼
phalcom-semantic
  SemanticWorkspaceSession
  canonical semantic DB
  one source index
  one identity world
  one formal checker
  one advisory world
  one invalidation model
        │
        ▼
Arc<SemanticSnapshot>
        │
        ▼
phalcom-lsp
  publication/pinning
  syntax recovery
  protocol presentation
```

At that point, new Part 4 work—abrupt exits, field lifecycle, pattern decomposition, deeper invalidation, import identity hardening, and other semantic-correctness changes—has exactly one implementation target.

That is the architectural payoff of this retirement: Part 4 no longer needs to negotiate with a legacy IDE semantic engine at all.
