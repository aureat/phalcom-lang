# Phalcom Single Semantic World — Corrective `phalcom-lsp` Retirement Implementation Plan

> **For agentic workers:** Execute this plan task-by-task on an isolated branch/worktree. The original user constraint for this effort is inline execution with at most one active worker; do not dispatch concurrent subagents. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Repair the broken partial migration on current `main`, make one persistent `phalcom_semantic::SemanticWorkspaceSession` the only semantic analyzer used by the LSP, migrate every semantic request to one canonical snapshot, and physically delete the old LSP semantic world and `WorkspaceIndex`.

**Architecture:** `AnalysisService` owns scheduling and one private canonical snapshot publication; its worker owns one persistent `SemanticWorkspaceSession`. `RequestContext` pins one optional canonical snapshot and canonical `ModuleId`. `phalcom-lsp` performs syntax recovery and protocol rendering only. Canonical editor queries compose precomputed compiler facts and never interpret source strings semantically.

**Tech Stack:** Rust 2024 workspace; `phalcom-ast`; `phalcom-common`; `phalcom-modules`; `phalcom-semantic`; `tower-lsp 0.20`; Tokio.

**Spec:** `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_corrective_tech_spec.md`

**Grounded repository:** `aureat/phalcom-lang`  
**Grounded branch:** `main`  
**Grounded HEAD:** `9b30ec324d4361128f285154fe236e25746df750`  
**Grounded date:** 2026-08-27

---

## Global constraints

- [ ] Do not implement Part 4 semantic features.
- [ ] Do not reset or rewrite unrelated user work.
- [ ] Do not restore `SemanticDb`, `SemanticEngine`, old semantic fields, or compatibility aliases merely to make compilation pass.
- [ ] Do not copy `phalcom-lsp/src/semantic/**` into `phalcom-semantic`.
- [ ] Do not add `tower-lsp`/`lsp-types` dependencies to `phalcom-semantic` or `phalcom-modules`.
- [ ] Do not perform semantic inference in LSP request handlers.
- [ ] Do not move request-time string/AST semantic reconstruction into `phalcom-semantic::editor`.
- [ ] Preserve latest-wins scheduling, immutable snapshot pinning, progressive discovery, and open-buffer priority.
- [ ] Keep `dashmap`; `DocumentStore` still uses it.
- [ ] Keep request-path filesystem reads/canonicalizations at zero.
- [ ] Every committed change after the recovery baseline must be compile-green. The LSP ownership repair in Tasks 4A–4D is one bounded **uncommitted recovery work unit** and is committed only after Task 4D restores `cargo check -p phalcom-lsp --lib`.
- [ ] Treat worker epoch and semantic generation as separate identities.
- [ ] Scan-discovered closed files must not become editor overlays.
- [ ] Stale/unmapped source may lose semantic completeness but must never gain a fallback semantic authority.

---

# 1. Current broken baseline

The current tree contains a known incomplete ownership migration.

Hard contradictions to verify before editing:

```text
analysis_service.rs:
  has AnalysisService::new_with_publication(...)

backend.rs:
  calls AnalysisService::new_with_index_and_cache(...)

request_context.rs:
  RequestContext::new_with_compiler(document, compiler, uri)

backend.rs:
  calls RequestContext::new_with_compiler(
      document,
      legacy_semantic,
      compiler_snapshot,
      uri
  )

request_context.rs:
  has compiler / canonical_module

backend.rs + inlay_hints.rs:
  still consume request.semantic / request.module in old paths

single_world_cutover.rs:
  constructs phalcom_lsp::semantic::SemanticDb
  calls AnalysisService::new(...)
  constructs legacy FileRevision
```

Do not “fix” these one at a time with aliases. They prove the owner/consumer graph must be migrated coherently.

---

# 2. File map

## 2.1 Create

- [ ] `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_corrective_tech_spec.md`
- [ ] `docs/impl/semantic/semantic-correctness/part-3/phalcom_lsp_semantic_retirement_corrective_implementation_plan.md`
- [ ] `phalcom-lsp/src/source_transport.rs`

## 2.2 Modify — canonical infrastructure

- [ ] `phalcom-modules/src/session.rs`
- [ ] `phalcom-modules/src/lib.rs`
- [ ] `phalcom-modules/tests/workspace_session.rs`
- [ ] `phalcom-semantic/src/session.rs`
- [ ] `phalcom-semantic/src/editor.rs`
- [ ] `phalcom-semantic/src/source_index/scope.rs`
- [ ] `phalcom-semantic/src/source_index/builder.rs`
- [ ] `phalcom-semantic/src/source_index/mod.rs`
- [ ] `phalcom-semantic/src/lib.rs`
- [ ] `phalcom-semantic/tests/semantic/integration/editor.rs`
- [ ] `phalcom-semantic/tests/semantic/integration/source_index.rs`
- [ ] `phalcom-semantic/tests/semantic/integration/workspace.rs`

## 2.3 Modify — LSP ownership spine

- [ ] `phalcom-lsp/src/analysis_service.rs`
- [ ] `phalcom-lsp/src/publication.rs`
- [ ] `phalcom-lsp/src/request_context.rs`
- [ ] `phalcom-lsp/src/backend.rs`
- [ ] `phalcom-lsp/src/documents.rs`
- [ ] `phalcom-lsp/src/core_documents.rs`
- [ ] `phalcom-lsp/src/completion.rs`
- [ ] `phalcom-lsp/src/hover.rs`
- [ ] `phalcom-lsp/src/signature_help.rs`
- [ ] `phalcom-lsp/src/inlay_hints.rs`
- [ ] `phalcom-lsp/src/semantic_tokens.rs`
- [ ] `phalcom-lsp/src/diagnostics.rs`
- [ ] `phalcom-lsp/src/perf.rs`
- [ ] `phalcom-lsp/src/lib.rs`
- [ ] `phalcom-lsp/Cargo.toml`

## 2.4 Modify — tests

- [ ] `phalcom-lsp/tests/single_world_cutover.rs`
- [ ] `phalcom-lsp/tests/integration.rs`
- [ ] `phalcom-lsp/tests/module_navigation.rs`
- [ ] `phalcom-lsp/tests/professional_semantic_presentation.rs`
- [ ] `phalcom-lsp/tests/core_startup.rs`
- [ ] `phalcom-lsp/tests/analysis_status.rs`
- [ ] `phalcom-lsp/tests/analysis_logging.rs`
- [ ] `phalcom-lsp/tests/semantic_boundary.rs`

## 2.5 Delete

After all external references are gone:

```text
phalcom-lsp/src/index.rs
phalcom-lsp/src/semantic/analyzer.rs
phalcom-lsp/src/semantic/callable.rs
phalcom-lsp/src/semantic/core_source.rs
phalcom-lsp/src/semantic/dispatch.rs
phalcom-lsp/src/semantic/engine.rs
phalcom-lsp/src/semantic/facts.rs
phalcom-lsp/src/semantic/flow.rs
phalcom-lsp/src/semantic/ids.rs
phalcom-lsp/src/semantic/infer.rs
phalcom-lsp/src/semantic/invalidation.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/occurrence.rs
phalcom-lsp/src/semantic/query.rs
phalcom-lsp/src/semantic/scope.rs
phalcom-lsp/src/semantic/snapshot.rs
phalcom-lsp/src/semantic/source.rs
phalcom-lsp/src/semantic/surface.rs
phalcom-lsp/src/semantic/mod.rs
```

---

# 3. Task 0 — Establish a protected recovery branch and exact failure inventory

**Purpose:** Current `main` is already red. Freeze the exact baseline and prevent another partial migration from being pushed as if it were complete.

**Files:** no production changes.

**Produces:** one written failure inventory for the executor and a branch/worktree at `9b30ec32`.

- [ ] **Step 1: Create an isolated branch/worktree from exact HEAD**

```bash
git fetch origin
git switch -c fix/lsp-single-world-retirement 9b30ec324d4361128f285154fe236e25746df750
```

If using a worktree, create it from that same SHA instead of switching the current checkout.

- [ ] **Step 2: Verify no unrelated local changes are being overwritten**

```bash
git status --short
git rev-parse HEAD
```

Expected HEAD:

```text
9b30ec324d4361128f285154fe236e25746df750
```

- [ ] **Step 3: Capture the actual compile failure**

```bash
cargo check -p phalcom-lsp --lib --message-format=short \
  2>&1 | tee /tmp/phalcom-lsp-retirement-baseline.txt
```

Do not edit before this command.

- [ ] **Step 4: Inventory transition symbols**

```bash
rg -n \
  "new_with_index_and_cache|new_with_compiler|request\.semantic|request\.module|SemanticDb|SemanticEngine|FileRevision|WorkspaceIndex|CompilerSemanticSnapshot|canonical_callables|class_for_canonical|member_surface_for_canonical|resolve_source_import|extend_import_closure_with_source|apply_module_mutations_at_generation|enqueue_core_update" \
  phalcom-lsp phalcom-semantic
```

Save the output with the task notes.

- [ ] **Step 5: Verify canonical foundations still compile independently**

```bash
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
```

- [ ] **Step 6: Do not commit baseline-only output**

The next commit starts after Task 1 is green.

---

# 4. Task 1 — Correct `EditorSemanticQuery` before more LSP code depends on it

**Purpose:** Remove the text-driven mini-analyzer currently embedded in `phalcom-semantic::editor` and characterize the canonical query surface.

**Files:**
- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/src/source_index/scope.rs`
- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Test: `phalcom-semantic/tests/semantic/integration/editor.rs`
- Test: `phalcom-semantic/tests/semantic/integration/source_index.rs`

**Consumes:**
- `SemanticSnapshot::formal_*`
- `SemanticSnapshot::advisory_*`
- canonical source index
- canonical hierarchy/surfaces

**Produces:**
- `EditorSemanticQuery::resolve_receiver_at` that reads published facts only
- source-index identity for `self`/`super`

### 4.1 Add source receiver identity

- [ ] **Step 1: Write failing source-index tests for `self` and `super`**

Add fixtures with a base/subclass and assert source sites for `self` and `super` carry explicit compiler-owned receiver syntax identity.

Recommended type:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceReceiverKind {
    SelfValue,
    SuperValue,
}
```

Add to `SourceScopeIndex`:

```rust
pub receiver_kinds: BTreeMap<SourceSiteId, SourceReceiverKind>,
```

Expose:

```rust
pub fn receiver_kind(&self, site: &SourceSiteId) -> Option<SourceReceiverKind>;
```

- [ ] **Step 2: Run the focused test and prove it fails**

```bash
cargo test -p phalcom-semantic --test semantic source_index -- --nocapture
```

Expected failure: receiver-kind metadata does not yet exist.

- [ ] **Step 3: Populate metadata in `source_index/builder.rs`**

When visiting `Expr::SelfVar` and `Expr::SuperVar`, register the existing source site and attach:

```rust
SourceReceiverKind::SelfValue
SourceReceiverKind::SuperValue
```

Do not add rendered text or editor protocol types.

### 4.2 Remove string-driven chain interpretation

- [ ] **Step 4: Add failing editor tests**

Extend `editor.rs` with all of:

```text
editor_resolves_local_binding_receiver
editor_resolves_parameter_receiver
editor_resolves_field_receiver
editor_resolves_call_result_receiver_from_expression_fact
editor_resolves_self_from_source_site_kind
editor_resolves_super_from_source_site_kind
editor_resolves_class_object_target
editor_resolves_union_receiver
editor_members_include_inherited_members
editor_visibility_filters_private_member
editor_visibility_allows_protected_for_subclass
editor_definition_and_reference_sites_preserve_identity
editor_visible_symbols_respect_shadowing
editor_unknown_receiver_fails_closed
editor_missing_chain_fact_does_not_parse_source_text
```

The last test is critical: construct a source range containing a dotted/chained spelling for which no formal/advisory result is published and assert `None`.

- [ ] **Step 5: Delete query-time raw-source semantics**

From `phalcom-semantic/src/editor.rs`, delete the source-text semantic branches equivalent to:

```rust
source.text.get(range.start..range.end)
text.trim() == "self"
text.trim() == "super"
resolve_chained_receiver(...)
dotted_expression_parts(...)
Selector::try_decode_exact(...) // when used to interpret raw receiver text
```

`resolve_receiver_at` should use this evidence order:

```text
1. exact/matched canonical expression site for receiver range
2. exact formal expression fact
3. exact formal binding fact
4. advisory fact for the same canonical site
5. canonical target denotation (Declaration => class object, Module => module)
6. SourceReceiverKind::SelfValue / SuperValue
7. fail closed
```

When a formal/advisory shape is a union, map each canonical alternative to `ReceiverAlternative`.

- [ ] **Step 6: Run editor/source-index tests**

```bash
cargo test -p phalcom-semantic --test semantic editor -- --nocapture
cargo test -p phalcom-semantic --test semantic source_index -- --nocapture
```

- [ ] **Step 7: Run the whole semantic integration target**

```bash
cargo test -p phalcom-semantic --test semantic
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/editor.rs \
        phalcom-semantic/src/source_index \
        phalcom-semantic/tests/semantic/integration/editor.rs \
        phalcom-semantic/tests/semantic/integration/source_index.rs
git commit -m "fix(semantic): make editor queries fact-driven"
```

---

# 5. Task 2 — Distinguish discovered disk snapshots from live overlays

**Purpose:** Scanner-discovered closed files currently enter the canonical module session as overlays. Fix the lifecycle vocabulary before removing LSP source mirroring.

**Files:**
- Modify: `phalcom-modules/src/session.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Test: `phalcom-modules/tests/workspace_session.rs`

**Produces:**

```rust
WorkspaceSourceBatchMutation::SetDiskSnapshot {
    source: SourceLocation,
    text: Arc<str>,
    revision: SourceRevision,
    recovered_program: Option<Arc<Program>>,
}
```

- [ ] **Step 1: Write failing tests**

Add:

```rust
#[test]
fn disk_snapshot_is_not_an_open_overlay() { ... }

#[test]
fn scanner_style_disk_snapshot_refreshes_from_disk() { ... }

#[test]
fn mixed_overlay_and_disk_batch_rebuilds_once() { ... }
```

The first assertion must inspect:

```rust
assert!(!session.source(&module).unwrap().open_overlay);
```

- [ ] **Step 2: Run the tests and prove the new variant is absent**

```bash
cargo test -p phalcom-modules --test workspace_session -- --nocapture
```

- [ ] **Step 3: Add `SetDiskSnapshot`**

Implementation inside `apply_batch`:

```rust
WorkspaceSourceBatchMutation::SetDiskSnapshot {
    source,
    text,
    revision,
    recovered_program,
} => {
    let module = staged.module_for_location(&source)?;
    let kind = staged.kind_for_source(&module, &source);
    let parsed = recovered_program.map_or_else(
        || parse_source(module.clone(), kind, source.clone(), text.clone()),
        |program| {
            Ok(Arc::new(ParsedModuleUnit::new(
                module.clone(),
                kind,
                Some(source.clone()),
                text.clone(),
                program,
            )))
        },
    )?;
    staged.provider.remove_overlay(&module);
    staged.insert_state(
        module.clone(),
        kind,
        source,
        revision,
        parsed,
        false,
    );
    changed.insert(module);
}
```

Do not install an overlay.

- [ ] **Step 4: Preserve one rebuild/one generation behavior**

The tests must assert one batch advances generation exactly once.

- [ ] **Step 5: Run module tests**

```bash
cargo test -p phalcom-modules
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-modules/src/session.rs \
        phalcom-modules/src/lib.rs \
        phalcom-modules/tests/workspace_session.rs
git commit -m "fix(modules): distinguish disk snapshots from overlays"
```

---

# 6. Task 3 — Return semantic generation ownership to `SemanticWorkspaceSession`

**Purpose:** LSP worker epoch is not semantic snapshot generation.

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic/integration/workspace.rs`

- [ ] **Step 1: Add/strengthen generation tests**

Test:

```text
two accepted module batches advance canonical module/semantic generation
without any externally supplied generation
```

Also assert `TypeStoreId` and ordinary module identity remain stable.

- [ ] **Step 2: Search callers of the transitional API**

```bash
rg -n "apply_module_mutations_at_generation|update_module_workspace_at_generation" .
```

Classify every caller.

- [ ] **Step 3: Remove public external-generation entry point if only transitional callers exist**

Delete:

```rust
pub fn apply_module_mutations_at_generation(...)
```

Keep an internal helper only if semantic tests genuinely require custom generation construction. Production worker code must not call it.

- [ ] **Step 4: Run semantic workspace tests**

```bash
cargo test -p phalcom-semantic --test semantic workspace -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration/workspace.rs
git commit -m "refactor(semantic): own workspace publication generation"
```

---

# 7. Task 4A — Normalize LSP source transport and pending updates (uncommitted recovery subtask)

**Purpose:** Remove split source text/program queues and give request/worker code one identical URI→source conversion. **Do not commit this subtask by itself:** current `main` is already broken at the Backend/RequestContext ownership seam. Keep these edits in the same recovery worktree until Task 4D restores the LSP compile gate.

**Files:**
- Create: `phalcom-lsp/src/source_transport.rs`
- Modify: `phalcom-lsp/src/lib.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/request_context.rs`
- Test: unit tests in `source_transport.rs` and `analysis_service.rs`

**Produces:**

```rust
pub(crate) struct PendingSourceUpdate {
    revision: SourceRevision,
    text: Arc<str>,
    program: Arc<Program>,
}
```

and pure source conversion functions.

### 7.1 Source transport helpers

- [ ] **Step 1: Create tests for pure conversion**

Verify file URI mapping round-trips without `canonicalize()` or file reads.

- [ ] **Step 2: Implement**

```rust
pub(crate) fn source_location_for_uri(uri: &Url) -> Option<SourceLocation> {
    let path = uri.to_file_path().ok()?;
    Some(SourceLocation {
        source_id: SourceId(path.to_string_lossy().into()),
        display_path: path,
    })
}

pub(crate) fn source_id_for_uri(uri: &Url) -> Option<SourceId> {
    source_location_for_uri(uri).map(|source| source.source_id)
}
```

Move equivalent logic out of `analysis_service.rs`.

### 7.2 Pending update model

- [ ] **Step 3: Replace split maps**

Replace:

```rust
file_updates: BTreeMap<Url, (SourceRevision, Program)>
source_texts: BTreeMap<Url, Arc<str>>
```

with:

```rust
file_updates: BTreeMap<Url, PendingSourceUpdate>
```

- [ ] **Step 4: Remove production program-only update APIs**

Target:

```rust
pub fn enqueue_file_update(
    &self,
    uri: Url,
    revision: SourceRevision,
    text: Arc<str>,
    program: Arc<Program>,
)
```

No overload may inject empty text.

Update `enqueue_file_updates` to receive complete updates.

- [ ] **Step 5: Remove core semantic update queue**

Delete:

```text
PendingWork.core_update
PendingWork.core_text
enqueue_core_update
enqueue_core_update_with_source
```

`core_documents` source selection remains.

- [ ] **Step 6: Compare the remaining compile errors to the baseline**

```bash
cargo check -p phalcom-lsp --lib --message-format=short \
  2>&1 | tee /tmp/phalcom-lsp-retirement-after-4a.txt
```

The crate is expected to remain red until Task 4D because `Backend` still uses the old ownership graph. Confirm that any newly reported errors are direct consequences of the planned API cut, not unrelated regressions.

- [ ] **Step 7: Do not commit**

Keep Task 4A changes uncommitted. Continue immediately to Task 4B in the same worktree. Canonical crates must remain green:

```bash
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo fmt --all -- --check
```

---

# 8. Task 4B — Remove worker-side import semantics and source-program mirroring (uncommitted recovery subtask)

**Purpose:** `SemanticWorkspaceSession` is persistent, but the worker still mirrors source programs and resolves relative import closure itself. **Do not commit this subtask separately.**

**Files:**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-modules/src/session.rs` or `phalcom-modules/src/query.rs` only if Local-mode dependency discovery needs a new canonical resolver query
- Test: LSP scan tests and module resolver tests

### 8.1 Delete semantic source catalog

- [ ] **Step 1: Add a worker test proving a scan result becomes a disk snapshot**

After processing a discovered file:

```rust
assert!(!session.module_session().source(&module).unwrap().open_overlay);
```

If worker internals are not externally observable, add a narrow `#[cfg(test)]` trace hook reporting mutation kinds; do not expose session mutation publicly.

- [ ] **Step 2: Replace scan `SetOverlay` with `SetDiskSnapshot`**

In `process_scan_batch`, build:

```rust
WorkspaceSourceBatchMutation::SetDiskSnapshot {
    source,
    text,
    revision,
    recovered_program: Some(program),
}
```

- [ ] **Step 3: Delete `extend_import_closure_with_source` and `resolve_source_import`**

Run:

```bash
rg -n "extend_import_closure_with_source|resolve_source_import" phalcom-lsp
```

Expected after migration: no production hit.

### 8.2 Preserve Local-mode discovery without duplicating resolver meaning

- [ ] **Step 4: If Local mode loses dependency-directed discovery, add one canonical modules query**

The query must accept canonical module/import identities, not URI dot-count strings.

One acceptable API:

```rust
impl WorkspaceModuleSession {
    pub fn resolve_import_source(
        &self,
        importer: &ModuleId,
        path: &phalcom_ast::ast::ImportPath,
    ) -> Result<Option<SourceLocation>, WorkspaceModuleSessionError>;
}
```

Its implementation must delegate to the same `ModuleResolver`/`ProjectUniverse` path used by linking.

The LSP may use the returned `SourceLocation` to schedule discovery. It must not calculate module paths itself.

- [ ] **Step 5: Remove `source_catalog` `(text, Program)` storage**

Keep only transport/discovery bookkeeping actually required by scan UI/status. Rename it to `discovered_sources` or `source_registry` and restrict values to source identity/revision.

- [ ] **Step 6: Handle canonical update errors explicitly**

Replace:

```rust
let publication = session.apply_module_mutations(...).ok();
let solve_cancelled = publication.is_none();
```

with an explicit `match`.

On `Err(error)`:
- emit `AnalysisEvent::Error`;
- retain previous publication;
- do not commit corresponding transport state.

- [ ] **Step 7: Use canonical generation**

Call:

```rust
session.apply_module_mutations(mutations)
```

not `apply_module_mutations_at_generation`.

- [ ] **Step 8: Run canonical module tests and capture LSP compile delta**

```bash
cargo test -p phalcom-modules
cargo check -p phalcom-lsp --lib --message-format=short \
  2>&1 | tee /tmp/phalcom-lsp-retirement-after-4b.txt
```

Do not require the LSP check to be green yet; Backend is intentionally still on the old ownership spine until Task 4D.

- [ ] **Step 9: Do not commit**

Continue to Task 4C with these changes in the same worktree.

---

# 9. Task 4C — Prepare canonical-only presentation entry points (uncommitted recovery subtask)

**Purpose:** Make hover/inlay/token renderers ready before the ownership-spine cut, so `Backend` does not need canonical→legacy adapters. **Do not commit this subtask separately.**

**Files:**
- Modify: `phalcom-lsp/src/hover.rs`
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Modify: `phalcom-lsp/src/semantic_tokens.rs`
- Modify: `phalcom-lsp/src/signature_help.rs`
- Test: module unit tests + `professional_semantic_presentation.rs`

This task may add canonical functions beside legacy wrappers temporarily. It must not add semantic inference.

### 9.1 Hover

- [ ] **Step 1: Add canonical renderer tests**

Use:

```text
phalcom_semantic::SourceBindingInfo
DeclarationId
CallableId
FieldId
SourceCallableKind
FormalPresentation
AdvisoryFact
```

Do not construct legacy `ClassId`, `BindingInfo`, `InferredValue`, or `ValueShape`.

- [ ] **Step 2: Introduce canonical renderer inputs**

Example:

```rust
pub(crate) fn render_binding_hover(
    binding: &phalcom_semantic::SourceBindingInfo,
    formal: Option<&phalcom_semantic::FormalPresentation>,
    advisory: Option<&phalcom_semantic::AdvisoryFact>,
    phaldoc: Option<&PhaldocDoc>,
) -> String
```

Render advisory text through:

```rust
phalcom_semantic::AdvisoryPresenter::present_shape(&fact.shape)
```

For callable/field hover, use canonical IDs/source info directly.

- [ ] **Step 3: Keep Phaldoc harvesting lexical but anchor it canonically**

The harvest helper receives a canonical `declaration_range`; it does not search for semantic identity.

### 9.2 Inlay hints

- [ ] **Step 4: Make `hints_for_request` the only production API**

Its semantic types must be canonical.

Remove internal dependence on:

```text
FileSemanticSnapshot
InferredValue
SemanticBindingKind
legacy ValueShape
legacy Confidence
```

Do not yet delete legacy public wrappers if current tests/backend still compile against them; mark them for Task 4D deletion and ensure no new code calls them.

### 9.3 Semantic tokens

- [ ] **Step 5: Ensure `tokens_for_request` is fully canonical**

It already uses canonical occurrences for `Exact`; retain syntax-only fallback for stale/unmapped.

Prepare to delete:

```rust
tokens_for(&SemanticDb, ...)
apply_semantic_overrides(...)
```

in Task 4D.

### 9.4 Signature help

- [ ] **Step 6: Keep current canonical renderer**

No legacy semantic renderer should be added.

- [ ] **Step 7: Add/update presentation tests but defer execution until Task 4D**

Because integration tests compile the full library, the pre-existing Backend mismatch can still prevent this target from running. Ensure the tests themselves use canonical fixtures and contain no `SemanticDb`/legacy IDs.

- [ ] **Step 8: Do not commit**

Continue directly to Task 4D. Task 4D is the first point where the LSP recovery work unit may be committed.

---

# 10. Task 4D — Execute the coherent LSP ownership-spine cut, restore compilation, and commit the recovery wave

**Purpose:** This is the critical recovery integration point. It completes Tasks 4A–4C, migrates owner + context + all directly coupled semantic consumers together, and is the **only commit boundary** for the LSP recovery work unit. Do not split provider changes from their consumers again.

**Files:**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/request_context.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Modify: `phalcom-lsp/src/completion.rs`
- Modify: `phalcom-lsp/src/hover.rs`
- Modify: `phalcom-lsp/src/signature_help.rs`
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Modify: `phalcom-lsp/src/semantic_tokens.rs`
- Modify: tests listed below

**Exit condition:**

```bash
cargo check -p phalcom-lsp --lib
```

must pass before this task is committed.

### 10.1 Make `AnalysisService` the single publication read handle

- [ ] **Step 1: Add a public constructor for tests**

```rust
pub fn new() -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
    Self::new_with_source_cache(None)
}
```

Use an internal constructor:

```rust
pub(crate) fn new_with_source_cache(
    source_cache: Option<SourceCache>,
) -> (Self, mpsc::UnboundedReceiver<AnalysisEvent>) {
    let publication = Arc::new(SemanticPublication::new());
    // spawn worker with publication.clone()
}
```

- [ ] **Step 2: Add snapshot accessor**

```rust
pub fn snapshot(&self) -> Option<Arc<phalcom_semantic::SemanticSnapshot>> {
    self.publication.load()
}
```

Do not expose `publish`.

### 10.2 Rewrite `RequestContext`

- [ ] **Step 3: Replace fields**

```rust
pub struct RequestContext {
    pub uri: Url,
    pub document: DocumentSnapshot,
    pub semantic: Option<Arc<phalcom_semantic::SemanticSnapshot>>,
    pub module: Option<phalcom_modules::ModuleId>,
    pub source_match: SourceMatch,
}
```

- [ ] **Step 4: Replace constructor**

```rust
pub fn new(
    document: DocumentSnapshot,
    semantic: Option<Arc<SemanticSnapshot>>,
    uri: &Url,
) -> Self
```

Delete:

```text
compiler
canonical_module
compiler_module()
new_with_compiler()
```

Use `source_transport` and canonical reverse provenance only.

### 10.3 Rewrite `Backend` ownership

- [ ] **Step 5: Delete `SemanticDb` from constructor/fields**

Remove:

```rust
let db = Arc::new(SemanticDb::with_counters(...));
semantic: Arc<SemanticDb>,
```

Construct:

```rust
let (analysis, event_rx) =
    AnalysisService::new_with_source_cache(Some(closed_sources.clone()));
```

`Backend` reads semantic publication only through:

```rust
self.analysis.snapshot()
```

- [ ] **Step 6: Rewrite `request_context`**

```rust
fn request_context(&self, uri: &Url) -> Option<RequestContext> {
    let document = self.documents.snapshot(uri)?;
    Some(RequestContext::new(
        document,
        self.analysis.snapshot(),
        uri,
    ))
}
```

### 10.4 Migrate diagnostics

- [ ] **Step 7: Change `combined_diagnostics_for` to canonical snapshot**

Signature:

```rust
fn combined_diagnostics_for(
    documents: &DocumentStore,
    semantic: Option<&phalcom_semantic::SemanticSnapshot>,
    uri: &Url,
) -> Option<DiagnosticPublication>
```

Only attach semantic diagnostics when source text is exact.

Delete `CompilerSemanticSnapshot` alias use.

### 10.5 Migrate navigation/references/rename/workspace symbols

- [ ] **Step 8: Target lookup**

Use:

```rust
let semantic = request.semantic.as_deref()?;
let module = request.module.as_ref()?;
let target = semantic.editor().target_at(module, offset)?;
```

Only when `SourceMatch::Exact`.

- [ ] **Step 9: Definition/reference location conversion**

Create one helper:

```rust
fn source_site_location(
    snapshot: &phalcom_semantic::SemanticSnapshot,
    site: &phalcom_semantic::SourceSiteId,
    open_documents: &DocumentStore,
    closed_sources: &SourceCache,
) -> Option<Location>
```

The helper:
1. gets canonical site/module/range;
2. gets canonical source provenance;
3. gets line index from open document or closed-source presentation cache;
4. converts to LSP range.

No `WorkspaceIndex`.

- [ ] **Step 10: Workspace symbols**

Add/use:

```rust
request_or_snapshot.editor().workspace_symbols(query)
```

If `EditorSemanticQuery::workspace_symbols` is not yet present, implement it in `phalcom-semantic/src/editor.rs` in this step with a semantic integration test. It enumerates canonical source metadata only.

### 10.6 Migrate completion

- [ ] **Step 11: Neutralize compiler naming**

Rename:

```text
CompilerCompletionContext -> SemanticCompletionContext
compiler_contextual_completions -> contextual_completions
compiler_class_completions -> class_completions
compiler_union_completions -> union_completions
compiler_visible_completions -> visible_completions
```

- [ ] **Step 12: Resolve receiver canonically**

```rust
let receiver = match (
    request.semantic.as_deref(),
    request.module.as_ref(),
    request.source_match,
) {
    (Some(snapshot), Some(module), SourceMatch::Exact) => {
        snapshot.editor().resolve_receiver_at(module, target.receiver_range)
    }
    _ => None,
};
```

No LSP semantic inference fallback.

### 10.7 Migrate hover

- [ ] **Step 13: Delete canonical→legacy conversion**

Delete backend/helper uses of:

```text
class_for_canonical
member_surface_for_canonical
crate::semantic::CallableId
crate::semantic::ClassId
legacy InferredValue
```

Use canonical source metadata and canonical renderer inputs prepared in Task 6.

For docs:
- locate canonical declaration/member source range;
- load exact source text from open/closed source cache;
- harvest Phaldoc from that range.

### 10.8 Migrate signature help

- [ ] **Step 14: Resolve exact canonical callable**

For dotted call:
- recover receiver range syntactically;
- canonical `resolve_receiver_at`;
- decode the already-recovered `CallSite.selector`;
- canonical `resolve_member`.

For unqualified call:
- use canonical target/source scope.

Render current canonical signature helper.

Stale/unmapped returns `None`.

### 10.9 Migrate inlay hints

- [ ] **Step 15: Use only `hints_for_request` canonical path**

Delete production calls to legacy DB APIs.

Fix all transitional field names:

```text
request.compiler -> request.semantic
request.compiler_module() -> request.module.as_ref()
request.module legacy field -> canonical request.module
```

Delete shallow semantic hint fallbacks that depend on old semantic facts. Syntax-only annotation suppression may remain, but stale semantic hints return empty.

### 10.10 Migrate semantic tokens

- [ ] **Step 16: Use only canonical request API**

Delete:

```rust
tokens_for(&SemanticDb, ...)
apply_semantic_overrides(...)
```

`tokens_for_request`:
- lexical base always;
- canonical occurrence refinement only for Exact;
- AST syntax declaration refinement for stale/unmapped.

### 10.11 Rewrite single-world tests before compile gate

- [ ] **Step 17: Replace `single_world_cutover.rs` legacy construction**

Target test:

```rust
#[test]
fn worker_reuses_type_store_and_module_identity_across_edits() {
    let (service, _events) = AnalysisService::new();

    let first_text: Arc<str> = Arc::from("class Main { run() {} }\n");
    service.enqueue_file_update(
        uri.clone(),
        SourceRevision(1),
        first_text.clone(),
        Arc::new(parse(&first_text, 0).program),
    );
    service.flush();

    let first = service.snapshot().expect("canonical publication");
    let store = first.store.id();
    let module = first.sources.keys().next().cloned().unwrap();

    let second_text: Arc<str> =
        Arc::from("class Main { run() {} edit() {} }\n");
    service.enqueue_file_update(
        uri,
        SourceRevision(2),
        second_text.clone(),
        Arc::new(parse(&second_text, 0).program),
    );
    service.flush();

    let second = service.snapshot().expect("second publication");
    assert_eq!(second.store.id(), store);
    assert!(second.sources.contains_key(&module));
    assert_ne!(first.id, second.id);
}
```

No `SemanticDb`; no `FileRevision`.

Add:
- request pins A while service publishes B;
- stale context suppresses semantic result;
- one accepted coalesced batch causes one publication.

### 10.12 Compile recovery gate

- [ ] **Step 18: Run formatting**

```bash
cargo fmt --all
cargo fmt --all -- --check
```

- [ ] **Step 19: Run the decisive check**

```bash
cargo check -p phalcom-lsp --lib
```

Expected: PASS.

If errors mention old semantic types, migrate the caller to canonical APIs. Do not add aliases.

- [ ] **Step 20: Run focused tests**

```bash
cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture
cargo test -p phalcom-lsp --test module_navigation -- --nocapture
cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture
```

- [ ] **Step 21: Commit the coherent ownership cut**

```bash
git add phalcom-lsp phalcom-semantic/src/editor.rs \
        phalcom-semantic/tests/semantic/integration/editor.rs
git commit -m "refactor(lsp): complete canonical request ownership cut"
```

Do not commit this task until `cargo check -p phalcom-lsp --lib` is green.

---

# 11. Task 5 — Delete all `WorkspaceIndex` semantic use and then the file

**Purpose:** Ensure no text-derived workspace semantic fallback remains.

**Files:**
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/lib.rs`
- Modify: scan/discovery code if it used index only for bookkeeping
- Delete: `phalcom-lsp/src/index.rs`
- Test: `module_navigation.rs`, integration workspace-symbol tests

- [ ] **Step 1: Inventory index consumers**

```bash
rg -n "WorkspaceIndex|crate::index|DefinitionInfo|ClassMemberInfo|Occurrence" phalcom-lsp/src phalcom-lsp/tests
```

- [ ] **Step 2: Prove canonical workspace-symbol/navigation coverage**

Tests must cover:
- two modules with same spelling but distinct canonical targets;
- closed-file definition;
- references;
- workspace symbols;
- stale document refuses text-derived semantic navigation.

- [ ] **Step 3: Delete backend field/construction/update calls**

Delete:

```text
index: Arc<WorkspaceIndex>
WorkspaceIndex::new()
index.update_file(...)
index.remove_file(...)
index.definitions(...)
index.references(...)
index.definition_info(...)
index.symbols_matching(...)
```

- [ ] **Step 4: Retain only discovery bookkeeping**

If `indexed_files` means “files discovered”, rename it to `discovered_files`.

Do not store semantic member/definition structures in it.

- [ ] **Step 5: Delete file and export**

```bash
rm phalcom-lsp/src/index.rs
```

Delete from `lib.rs`:

```rust
pub mod index;
```

- [ ] **Step 6: Verify no production references**

```bash
rg -n "WorkspaceIndex|crate::index|DefinitionInfo|ClassMemberInfo" phalcom-lsp/src
```

Expected: no hits.

- [ ] **Step 7: Run tests/check**

```bash
cargo check -p phalcom-lsp
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test integration
```

- [ ] **Step 8: Commit**

```bash
git add -A phalcom-lsp
git commit -m "refactor(lsp): delete text-derived workspace semantic index"
```

---

# 12. Task 6 — Make core source transport truly transport-only

**Purpose:** Remove dead semantic core update plumbing and old core semantic construction.

**Files:**
- Modify: `phalcom-lsp/src/core_documents.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Test: `phalcom-lsp/tests/core_startup.rs`

- [ ] **Step 1: Add tests that separate semantics from source transport**

Prove:
1. fresh `SemanticWorkspaceSession` exposes canonical core semantics without an LSP core update;
2. selecting configured/workspace core source changes source URI/text presentation only;
3. worker has no semantic core mutation queue.

- [ ] **Step 2: Remove unused parse/update plumbing**

If `CoreSource::parse()` exists only for `enqueue_core_update`, delete it.

Delete all remaining:

```text
enqueue_core_update
core_update
core_text
```

- [ ] **Step 3: Keep only source selection/virtual text**

`CoreSource::select`, `text`, and `physical_uri` may remain.

- [ ] **Step 4: Run core tests**

```bash
cargo test -p phalcom-lsp --test core_startup -- --nocapture
```

- [ ] **Step 5: Commit**

```bash
git add phalcom-lsp/src/core_documents.rs \
        phalcom-lsp/src/analysis_service.rs \
        phalcom-lsp/src/backend.rs \
        phalcom-lsp/tests/core_startup.rs
git commit -m "refactor(lsp): make core handling transport-only"
```

---

# 13. Task 7 — Remove retired semantic performance vocabulary

**Purpose:** Do not leave fake ownership in telemetry after the engine disappears.

**File:**
- Modify: `phalcom-lsp/src/perf.rs`
- Modify tests/logging snapshots as required

- [ ] **Step 1: Search old analyzer counters**

```bash
rg -n \
  "flow_passes|solver_rounds|callables_analyzed|dirty_callables_seeded|solver_callables_visited|solver_callables_changed|semantic_candidate_state_clones|published_file_products_reused|published_class_products_reused|published_summary_products_reused|parameter_sources_replaced|parameter_slots_touched|parameter_slots_changed" \
  phalcom-lsp
```

- [ ] **Step 2: Remove those fields/reset/snapshot serialization**

Keep scheduler/protocol counters such as:
- updates enqueued/coalesced/discarded;
- batches started/published;
- scan counters;
- refresh counters;
- query filesystem reads/canonicalizations.

- [ ] **Step 3: Log canonical `SemanticUpdateStats` per publication where useful**

Do not mirror them as another mutable semantic engine.

- [ ] **Step 4: Run logging/status tests**

```bash
cargo test -p phalcom-lsp --test analysis_status
cargo test -p phalcom-lsp --test analysis_logging
```

- [ ] **Step 5: Commit**

```bash
git add phalcom-lsp/src/perf.rs \
        phalcom-lsp/src/analysis_service.rs \
        phalcom-lsp/tests/analysis_status.rs \
        phalcom-lsp/tests/analysis_logging.rs
git commit -m "refactor(lsp): remove retired semantic engine telemetry"
```

---

# 14. Task 8 — Prove the legacy semantic package has no external caller

**Purpose:** Physical deletion is safe only when no production or test code outside the package imports it.

- [ ] **Step 1: Run external-reference search**

```bash
rg -n \
  "crate::semantic|phalcom_lsp::semantic|SemanticDb|SemanticEngine|FileSemanticSnapshot|FileRevision|InferredValue|SemanticBindingKind|canonical_callables|class_for_canonical|member_surface_for_canonical|CompilerSemanticSnapshot" \
  phalcom-lsp/src phalcom-lsp/tests \
  -g '!phalcom-lsp/src/semantic/**'
```

Expected: zero old-semantic hits. Canonical `phalcom_semantic::...` is allowed.

- [ ] **Step 2: Rewrite any remaining tests instead of preserving APIs**

Typical repair:
- build `SemanticWorkspaceSession` fixture;
- publish through `AnalysisService`;
- use canonical IDs/facts;
- test LSP projection, not old engine behavior.

- [ ] **Step 3: Run LSP compile/test subset**

```bash
cargo check -p phalcom-lsp
cargo test -p phalcom-lsp --test single_world_cutover
cargo test -p phalcom-lsp --test professional_semantic_presentation
```

No commit is necessary if this task is search/test-only; fold rewrites into Task 9.

---

# 15. Task 9 — Physically delete `phalcom-lsp/src/semantic/**`

**Purpose:** Finish the architectural retirement.

**Files:**
- Delete entire `phalcom-lsp/src/semantic/`
- Modify: `phalcom-lsp/src/lib.rs`
- Modify: `phalcom-lsp/Cargo.toml`
- Modify any tests exposed by compilation

- [ ] **Step 1: Delete export first**

Remove:

```rust
pub mod semantic;
```

- [ ] **Step 2: Delete directory**

```bash
rm -rf phalcom-lsp/src/semantic
```

- [ ] **Step 3: Remove direct native dependency**

Run:

```bash
rg -n "phalcom_native_surface|phalcom-native-surface" phalcom-lsp
```

When no source import remains, remove from `phalcom-lsp/Cargo.toml`:

```toml
phalcom-native-surface = { path = "../phalcom-native-surface" }
```

- [ ] **Step 4: Rewrite crate docs**

`phalcom-lsp/src/lib.rs` should describe:
- `AnalysisService` scheduling/publication;
- canonical semantic consumption;
- syntax-only stale behavior;
- no LSP semantic database.

- [ ] **Step 5: Compile immediately**

```bash
cargo check -p phalcom-lsp
```

Any missing old type is a real migration gap. Repair with canonical APIs only.

- [ ] **Step 6: Run focused LSP tests**

```bash
cargo test -p phalcom-lsp --test single_world_cutover
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test professional_semantic_presentation
cargo test -p phalcom-lsp --test core_startup
cargo test -p phalcom-lsp --test integration
```

- [ ] **Step 7: Commit**

```bash
git add -A phalcom-lsp
git commit -m "refactor(lsp)!: delete legacy semantic implementation"
```

---

# 16. Task 10 — Turn `semantic_boundary` into a complete architecture gate

**Files:**
- Modify: `phalcom-lsp/tests/semantic_boundary.rs`
- `phalcom-lsp/Cargo.toml` already registers the test

- [ ] **Step 1: Remove the ignore**

The directory-deletion test becomes active.

- [ ] **Step 2: Add physical gates**

Assert absent:

```text
src/semantic
src/index.rs
```

- [ ] **Step 3: Add forbidden-definition scan**

Walk `.rs` files under `phalcom-lsp/src` and reject exact definition patterns:

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

Do not reject canonical imports.

- [ ] **Step 4: Add bridge-name gates**

Reject:

```text
canonical_callables
canonical_target_to_lsp
class_for_canonical
member_surface_for_canonical
CompilerResolvedReceiver
SemanticResolvedReceiver
legacy_hover_at
apply_module_mutations_at_generation
resolve_source_import
extend_import_closure_with_source
```

- [ ] **Step 5: Add dependency gates**

Check:
- LSP depends on `phalcom-semantic`;
- LSP has no `phalcom-native-surface`;
- semantic/modules have no `tower-lsp`, `lsp-types`, or `phalcom-lsp`.

- [ ] **Step 6: Add request-path I/O source scan**

Inspect semantic request modules:

```text
backend.rs
completion.rs
hover.rs
signature_help.rs
inlay_hints.rs
semantic_tokens.rs
request_context.rs
```

Reject:

```text
std::fs::read
std::fs::read_to_string
.canonicalize()
```

Phaldoc may read source only through the existing open/closed source cache, not direct request disk I/O.

- [ ] **Step 7: Run gate**

```bash
cargo test -p phalcom-lsp --test semantic_boundary -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-lsp/tests/semantic_boundary.rs phalcom-lsp/Cargo.toml
git commit -m "test(lsp): enforce one semantic world"
```

---

# 17. Task 11 — Complete exact/stale/unmapped behavioral coverage

**Purpose:** The one-world architecture deliberately returns less semantic information while source is stale. Lock that contract.

**Files:**
- Modify: `phalcom-lsp/tests/integration.rs`
- Modify: `module_navigation.rs`
- Modify: `professional_semantic_presentation.rs`
- Modify: `single_world_cutover.rs`

- [ ] **Step 1: Add diagnostics matrix**

Assert:
- Exact: syntax + canonical semantic diagnostics.
- Stale: syntax only.
- Unmapped: syntax only.

- [ ] **Step 2: Add navigation matrix**

Assert stale/unmapped returns no semantic:
- definition;
- references;
- rename.

- [ ] **Step 3: Add completion matrix**

Assert:
- exact receiver gets canonical members;
- stale `value.|` gets no inferred members;
- syntax-visible non-member names/snippets remain available.

- [ ] **Step 4: Add hover/signature/inlay matrix**

Assert:
- keyword hover works stale;
- semantic hover does not;
- signature help does not;
- inlay hints do not.

- [ ] **Step 5: Add semantic-token matrix**

Assert:
- lexical coloring always;
- semantic occurrence refinement exact only.

- [ ] **Step 6: Add publication topology test**

Test:
1. service publishes snapshot A;
2. request pins A;
3. service publishes B;
4. request still reads A;
5. a new request reads B.

- [ ] **Step 7: Run all focused LSP tests**

```bash
cargo test -p phalcom-lsp --test single_world_cutover
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test professional_semantic_presentation
cargo test -p phalcom-lsp --test integration
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-lsp/tests
git commit -m "test(lsp): lock single-world freshness behavior"
```

---

# 18. Task 12 — Final performance and lifecycle proofs

**Purpose:** Prove the retirement removed duplicate work and preserved source identity/lifecycle.

**Files:**
- tests only unless a missing observable stat must be exposed

- [ ] **Step 1: Add coalescing/publication test**

Enqueue multiple revisions before worker release; assert only the latest accepted batch publishes semantic state.

Do not require one publication per raw keystroke.

- [ ] **Step 2: Add source lifecycle test**

Sequence:

```text
disk snapshot
→ open overlay
→ edit overlay
→ close
→ disk fallback
→ watched disk refresh
→ delete source
```

Assert:
- module identity stable until delete;
- `open_overlay` true only while open;
- close does not remove module;
- delete does.

- [ ] **Step 3: Add TypeStore reuse test**

Ordinary body edit:

```rust
assert_eq!(first.store.id(), second.store.id());
```

- [ ] **Step 4: Add request I/O counter test**

Semantic completion/navigation/hover on closed canonical source must leave:

```text
query_filesystem_canonicalizations == 0
query_disk_reads == 0
```

- [ ] **Step 5: Run focused suites**

```bash
cargo test -p phalcom-modules --test workspace_session
cargo test -p phalcom-semantic --test semantic workspace
cargo test -p phalcom-lsp --test single_world_cutover
cargo test -p phalcom-lsp --test semantic_boundary
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-modules/tests \
        phalcom-semantic/tests \
        phalcom-lsp/tests
git commit -m "test(semantic): prove single-world lifecycle and reuse"
```

---

# 19. Task 13 — Full verification

Run in this order.

- [ ] **Formatting**

```bash
cargo fmt --all -- --check
git diff --check
```

- [ ] **Compile**

```bash
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-lsp
```

- [ ] **Canonical focused tests**

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-semantic --test semantic source_index
cargo test -p phalcom-semantic --test semantic editor
cargo test -p phalcom-semantic --test semantic presentation
cargo test -p phalcom-semantic --test semantic workspace
```

- [ ] **LSP focused tests**

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

- [ ] **Crate/workspace suites**

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-lsp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If unrelated pre-existing clippy failures exist, record exact diagnostics and prove retirement-touched targets are clean. Do not weaken `semantic_boundary`.

- [ ] **Physical searches**

```bash
test ! -d phalcom-lsp/src/semantic
test ! -f phalcom-lsp/src/index.rs

rg "pub mod semantic;" phalcom-lsp/src
rg "crate::semantic|phalcom_lsp::semantic" phalcom-lsp
rg "struct SemanticDb|struct SemanticEngine|enum ValueShape|struct ScopeGraph|struct ModuleGraph" phalcom-lsp/src
rg "canonical_callables|canonical_target_to_lsp|class_for_canonical|member_surface_for_canonical" phalcom-lsp/src
rg "resolve_source_import|extend_import_closure_with_source|apply_module_mutations_at_generation" phalcom-lsp/src
rg "phalcom_native_surface|phalcom-native-surface" phalcom-lsp
```

Expected forbidden searches: no production hits.

Canonical imports are expected:

```bash
rg "phalcom_semantic::(DeclarationId|CallableId|FieldId|SemanticTargetId|SourceSiteId)" phalcom-lsp/src
```

---

# 20. Task 14 — Close Part 3 documentation

**Files:**
- Modify Part 3 implementation checklist
- Modify architecture reference docs that still describe dual-world LSP semantics
- Install this corrective spec/plan in the repository

- [ ] **Step 1: Install documents**

```text
docs/impl/semantic/semantic-correctness/part-3/
  phalcom_lsp_semantic_retirement_corrective_tech_spec.md
  phalcom_lsp_semantic_retirement_corrective_implementation_plan.md
```

- [ ] **Step 2: Record completion evidence**

Include:

```text
grounded starting SHA: 9b30ec324d4361128f285154fe236e25746df750
completion SHA: run `git rev-parse HEAD` after Task 13 verification and record that exact SHA here before the documentation commit
semantic directory absent: yes
WorkspaceIndex absent: yes
semantic_boundary: green
cargo check -p phalcom-lsp: green
cargo test --workspace: green
```

Run `git rev-parse HEAD` after Task 13 verification and record that exact SHA before committing documentation.

- [ ] **Step 3: Rewrite architecture references**

Final ownership text:

```text
AnalysisService = scheduling + immutable publication access
WorkspaceModuleSession = project/module/source lifecycle
SemanticWorkspaceSession = sole semantic analyzer
SemanticSnapshot = sole semantic publication
RequestContext = live document + one optional pinned canonical snapshot
LSP feature modules = syntax recovery + protocol presentation
```

- [ ] **Step 4: Keep Part 4 blocked until final verification is complete**

Only after Task 13 is fully green may Part 4 be re-grounded to the new HEAD.

- [ ] **Step 5: Commit**

```bash
git add docs .agents
git commit -m "docs(semantic): close single-world LSP retirement"
```

---

# 21. Recommended commit sequence

Use this sequence unless a focused test reveals a necessary dependency change:

```text
1. fix(semantic): make editor queries fact-driven
2. fix(modules): distinguish disk snapshots from overlays
3. refactor(semantic): own workspace publication generation
4. refactor(lsp): complete canonical request ownership cut
   - includes Tasks 4A–4D: source ingestion, worker cleanup, canonical presentation, Backend/RequestContext cut
5. refactor(lsp): delete text-derived workspace semantic index
6. refactor(lsp): make core handling transport-only
7. refactor(lsp): remove retired semantic engine telemetry
8. refactor(lsp)!: delete legacy semantic implementation
9. test(lsp): enforce one semantic world
10. test(lsp): lock single-world freshness behavior
11. test(semantic): prove single-world lifecycle and reuse
12. docs(semantic): close single-world LSP retirement
```

Commit 4 is intentionally larger than the others. It is the coherent ownership-spine cut containing Tasks 4A–4D. Do not split it into provider-only commits that make `Backend` and `RequestContext` disagree again.

---

# 22. Stop conditions during implementation

If a canonical feature gap appears:

```text
stop that consumer
add the missing canonical product/query
test it in phalcom-semantic
resume the consumer
```

Do not:
- revive LSP inference;
- add a legacy alias;
- guess from selector text;
- parse a source chain in `EditorSemanticQuery`;
- read disk on a request;
- use `WorkspaceIndex` as a temporary semantic answer.

If a stale-source UX regression appears:

```text
improve syntax recovery
or reduce publication latency
```

Do not restore stale semantic fallback.

If Local-mode import discovery regresses:

```text
add a canonical module-resolution discovery query
```

Do not restore `resolve_source_import`.

---

# 23. Final definition of done

- [ ] Current compile break is eliminated without restoring old APIs.
- [ ] `AnalysisService` owns exactly one private publication cell.
- [ ] The worker owns exactly one persistent `SemanticWorkspaceSession`.
- [ ] `Backend` does not own/read a `SemanticDb`.
- [ ] `RequestContext` contains no legacy semantic snapshot.
- [ ] `RequestContext` pins one optional canonical snapshot for the full request.
- [ ] Worker epoch does not control semantic generation.
- [ ] Scanner disk files are not overlays.
- [ ] Worker has no semantic `source_catalog` mirror.
- [ ] Worker has no manual import resolver.
- [ ] Worker core update queue is gone.
- [ ] `EditorSemanticQuery` contains no raw-text semantic chain evaluation.
- [ ] Canonical editor facade has the full receiver/identity/visibility test matrix.
- [ ] Diagnostics are canonical-only on exact source.
- [ ] Navigation/references/rename are canonical-only.
- [ ] Workspace symbols are canonical-only.
- [ ] Completion is canonical-only for semantic members.
- [ ] Hover uses canonical IDs/source/facts and no canonical→legacy bridge.
- [ ] Signature help resolves canonical callables.
- [ ] Inlay hints use canonical source/formal/advisory facts only.
- [ ] Semantic token refinement uses canonical occurrences only.
- [ ] Stale/unmapped requests are syntax-only for position-dependent semantics.
- [ ] `WorkspaceIndex` is physically deleted.
- [ ] `phalcom-lsp/src/semantic/**` is physically deleted.
- [ ] `pub mod semantic` is absent.
- [ ] direct `phalcom-native-surface` dependency is absent.
- [ ] retired semantic counters are absent.
- [ ] query filesystem I/O/canonicalization remains zero.
- [ ] `semantic_boundary` is enabled and green.
- [ ] focused canonical tests are green.
- [ ] focused LSP tests are green.
- [ ] full workspace tests are green.
- [ ] clippy is green or unrelated pre-existing failures are explicitly isolated.
- [ ] Part 3 closure documentation is updated.
- [ ] Part 4 is re-grounded only after this closure.
