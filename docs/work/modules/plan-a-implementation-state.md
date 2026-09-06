# Phalcom LSP Module Architecture — Plan A Implementation State

## Checkpoint Baseline
- Plan-A entry baseline: `d60e4589352ac5f4167ba295e7e2a5f6c870ef4b`
- Commit message: `docs: record final module architecture gate`
- Start date: 2026-09-05
- Local dirty state at entry:
  - `docs/.obsidian/workspace.json`
  - `docs/type tests.md`
  - untracked `docs/impl/lsp/architecture/`

## Test Baseline at A0 Entry
- `cargo test -p phalcom-modules`: PASS (85 passed, 0 failed)
- `cargo test -p phalcom-lsp`: PASS (145 passed, 0 failed, 2 ignored across unit, integration, navigation, boundary, and cutover suites)
- `cargo test -p phalcom-semantic`: 1089 passed, 1 failed (`semantic::integration::resolver::qualified_type_resolution_preserves_single_member_lookup`), 42 ignored
- `cargo test -p phalcom-core`: Pre-existing failures preserved (historical recorded 483 passed, 24 failed, 33 ignored); evaluation stopped per user instruction to focus on module architecture plan.

---

## Checkpoint A0 — Transactional Current-State Correctness

### Status
COMPLETED for atomic/current-state correctness; PARTIAL for the requested
delta/COW cost model.

### Tasks
- [x] Task 1 — Re-ground the current failure and test baseline, add late-failure test seams.
- [x] Task 2 — Introduce a true private workspace-module transaction and one commit barrier.
- [x] Task 3 — Make current-generation interface/product validity explicit.
- [x] Task 4 — Make transitive source/interface/import discovery fully tolerant.
- [x] Task 5 — Make forward/reverse dependency replacement and tolerant runtime graph publication exact.

### Checkpoint A0 Verification Evidence
1. Late-failure atomicity: `tests/workspace_session.rs::failed_transaction_does_not_mutate_committed_state` passes.
2. Interface-invalid current edit publishes partial state without stale interface: `tests/workspace_session.rs::interface_invalid_current_edit_publishes_partial_state_without_stale_interface` passes.
3. Invalid transitive dependency publishes partial state: `tests/workspace_session.rs::invalid_transitive_dependency_publishes_partial_state` passes.
4. Exact reverse edge replacement leaves no stale dependency: `tests/workspace_session.rs::exact_reverse_edge_replacement_leaves_no_stale_dependency` passes.
5. Tolerant runtime survivor order: `tests/linker.rs::tolerant_runtime_cycle_preserves_independent_survivor_order` passes.
6. Full module test suite: `RUSTFLAGS='' cargo test -p phalcom-modules` passes with zero failures.
7. Downstream checks: `cargo test -p phalcom-semantic --test semantic module_query_provenance` passes; `cargo check -p phalcom-lsp` passes with zero errors.

### Corrective amendment after audit
1. `phalcom-lsp/src/source_transport.rs` remains a pure URI-to-protocol-source
   conversion. Canonical source identity is established by the module
   workspace, which retains protocol/display-to-canonical aliases for later
   updates and deletion events.
2. Staged overlay derivation no longer rediscovers a removed overlay through
   the base provider, and removed source identities are recorded before the
   private rebuild. Deleted provider facts therefore cannot survive the commit
   barrier.
3. The provider edit and rename/delete lifecycle tests pass, and the full LSP
   suite is now green. The first bad Plan-A checkpoint was A0 (`2365d39`), so
   the earlier `9f7ded35` reproduction was not a valid inherited-baseline
   classification.
4. Ordinary `apply_batch` still clones the two committed source/module maps.
   Atomicity is complete; true O(delta) staging remains open.

---

## Checkpoint A1 — Indexed Topology Is a Production Product

### Status
COMPLETED

### Baseline Commit
- `2365d39d0c86ec9d75a1cd7ccdf4361c54fa1dae` (`feat(modules): implement checkpoint A0 transactional module session`)

### Tasks
- [x] Task 6 — Retain current `ModuleTopology` in `WorkspaceModuleSession`, aligned with session generation and published in `WorkspaceModuleUpdate`.
- [x] Task 7 — Publish topology and direct reverse-import index into `SemanticSnapshot::ModuleQueryProducts` and configure `ModuleQueryFacade`.
- [x] Task 8 — Remove normal scan fallbacks, instrument query fallback work count, and verify zero fallback scans on indexed facades.

### Checkpoint A1 Verification Evidence
1. Topology lifecycle in session: `tests/workspace_session.rs::topology_is_retained_in_session_aligned_with_generation_and_published_in_update` passes.
2. Fallback scan instrumentation & zero-scan indexed queries: `tests/query.rs::unindexed_facade_records_fallback_scans_while_indexed_records_zero` passes.
3. Large-scale synthetic topology scaling test: `tests/query.rs::synthetic_large_scale_topology_query_work_count` passes (1,111 nodes, 1,110 edges, 0 fallback scans, immediate indexed response).
4. Semantic snapshot integration & generation alignment: `tests/module_query_provenance.rs::semantic_snapshot_publishes_relative_import_alias_path_and_provenance` passes (`queries.is_fully_indexed()`, generation match, 0 fallback scans).
5. Module test suites:
   - `RUSTFLAGS='' cargo test -p phalcom-modules --test query` (5 passed, 0 failed)
   - `RUSTFLAGS='' cargo test -p phalcom-modules --test topology` (6 passed, 0 failed)
   - `cargo test -p phalcom-modules` (91 passed, 0 failed)
6. Downstream checks:
   - `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic module_query_provenance` (1 passed, 0 failed)
   - `cargo check -p phalcom-lsp` (clean compilation, zero errors)

---

## Checkpoint A2 — Import Resolution is Bound to Identity and Validated Before Re-Resolution

### Status
COMPLETED

### Baseline Commit
- `39b410415ad9b5cfe3e4a1f41ad19a8fb50fb33c` (`feat(modules): implement checkpoint A1 indexed topology product`)

### Tasks
- [x] Task 9 — Define stable `ImportSiteId` (`ImportSiteLocalId` + `ModuleId`), assign during interface extraction.
- [x] Task 10 — Migrate retained resolution products to site identity, retain prefix targets (`ResolvedImportPrefix`).
- [x] Task 11 — Record positive and negative topology dependencies (`AbsentCandidateFact`, `ResolutionTopologyDependencies`).
- [x] Task 12 — Introduce `TopologyDelta` and validate-before-resolve algorithm with deterministic work statistics.
- [x] Task 13 — Maintain exact forward/reverse import-site indexes (`sites_by_importer`, `reverse_site_importers`).
- [x] Task 14 — Add `DirectorySnapshot` cache in `FilesystemSourceProvider`.

### Checkpoint A2 Verification Evidence
1. Evidence 1: Module with 20 imports, editing 1 import path resolves exactly 1 import and reuses 19 (`imports_resolved == 1`, `import_resolutions_reused == 19`): `tests/checkpoint_a2.rs::checkpoint_a2_evidence_1_twenty_imports_edit_one_resolves_one_reuses_nineteen` passes.
2. Evidence 2: Body-only edit across multi-module session produces `imports_resolved == 0` (zero import re-resolutions): `tests/checkpoint_a2.rs::checkpoint_a2_evidence_2_body_only_edit_zero_import_resolutions` passes.
3. Evidence 3: Export-only edit across multi-module session produces `imports_resolved == 0` for all dependent modules: `tests/checkpoint_a2.rs::checkpoint_a2_evidence_3_export_only_edit_zero_import_resolutions` passes.
4. Evidence 4: Negative resolution survives unrelated source addition: adding `b.ph` when import was for missing `c.ph` produces `imports_resolved == 0`, negative resolution reused (`negative_resolutions_reused == 1`): `tests/checkpoint_a2.rs::checkpoint_a2_evidence_4_and_5_negative_resolution_survives_unrelated_and_invalidates_on_candidate` passes.
5. Evidence 5: Negative resolution correctly invalidates when candidate appears: adding `c.ph` re-resolves only the site that was waiting for `c` (`imports_resolved == 1`), diagnostic clears: `tests/checkpoint_a2.rs::checkpoint_a2_evidence_4_and_5_negative_resolution_survives_unrelated_and_invalidates_on_candidate` passes.
6. Evidence 6: Prefix provenance: compound import retains canonical `ModuleId`s for prefixes and invalidates when an intermediate prefix module is removed: `tests/checkpoint_a2.rs::checkpoint_a2_evidence_6_prefix_provenance_compound_imports` passes.
7. Module test suites:
   - `RUSTFLAGS='' cargo test -p phalcom-modules --test checkpoint_a2` (5 passed, 0 failed)
   - `RUSTFLAGS='' cargo test -p phalcom-modules` (96 passed, 0 failed)
8. Downstream checks:
   - `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic module_query_provenance` (1 passed, 0 failed)
   - `cargo check -p phalcom-lsp` (clean compilation, zero errors)

---

## Checkpoint A3 — Affected-Component Incremental Linking

### Status
COMPLETED

### Baseline Commit
- `0983cf8794c48974a9eb4fb9534ee9713c7a36cb` (`feat(modules): implement checkpoint A2 import site identity and topology resolution caching`)

### Tasks
- [x] Task 15 — Define `ComponentId` and `ComponentLinkedProduct` for retained component products.
- [x] Task 16 — Seed affected connected components from interface changes, module additions/removals/reidentifications, recomputed import sites, and membership deltas.
- [x] Task 17 — Split public interface fingerprinting (`LinkedInterfaceFingerprint`) from private linkage dependency fingerprinting (`LinkedDependencyFingerprint`).
- [x] Task 18 — Implement component-scoped incremental linking in `WorkspaceModuleSession` and retain unaffected component products.
- [x] Task 19 — Implement tolerant component-scoped linking in `ModuleLinker` and compute topological initialization order over surviving runtime graph.

### Checkpoint A3 Verification Evidence
1. Evidence 1: Body-only edit recomputes 0 components and reuses all 3 retained components (`linked_components_recomputed == 0`, `linked_components_reused == 3`): `tests/checkpoint_a3.rs::checkpoint_a3_evidence_1_body_only_edit_reuses_all_components` passes.
2. Evidence 2: Public interface edit recomputes only affected component and retains all other components (`linked_components_recomputed == 1`, `linked_components_reused == 2`, `Arc::ptr_eq` structural reuse verified): `tests/checkpoint_a3.rs::checkpoint_a3_evidence_2_public_interface_edit_recomputes_only_affected_component` passes.
3. Evidence 3: Import target change recomputes affected merged component only while unaffected components are retained (`linked_components_recomputed == 1`, `linked_components_reused >= 1`): `tests/checkpoint_a3.rs::checkpoint_a3_evidence_3_import_target_change_affects_only_target_component` passes.
4. Evidence 4: Private dependency fingerprint split: changing internal linkage/private local declarations alters `LinkedDependencyFingerprint` while `LinkedInterfaceFingerprint` remains stable: `tests/checkpoint_a3.rs::checkpoint_a3_evidence_4_private_dependency_fingerprint_split` passes.
5. Evidence 5: Strict vs tolerant linking parity: valid closed components produce identical linked modules and initialization orders under strict and tolerant linking with 0 diagnostics/blocked modules: `tests/checkpoint_a3.rs::checkpoint_a3_evidence_5_strict_and_tolerant_linking_parity` passes.
6. Evidence 6: Tolerant runtime cycle isolation: cyclic modules are isolated/blocked while surviving unblocked modules retain valid topological initialization order (dependency W initializes before Z): `tests/checkpoint_a3.rs::checkpoint_a3_evidence_6_cycle_survivors_retain_initialization_order` passes.
7. Module test suites:
   - `cargo test -p phalcom-modules --test checkpoint_a3` (6 passed, 0 failed)
   - `cargo test -p phalcom-modules` (102 passed, 0 failed)
8. Downstream checks:
   - `cargo test -p phalcom-semantic --test semantic module_query_provenance` (1 passed, 0 failed)
   - `cargo check -p phalcom-lsp` (clean compilation, zero errors)

---

## Checkpoint A4 — Exact Module Facts Join the Existing SemanticDb Dependency Graph

### Status
COMPLETED

### Baseline Commit
- `fa1f0094b5db875d6c880fed870bd47b87376803` (`feat(modules): implement checkpoint A3 affected-component incremental linking`)

### Tasks
- [x] Task 20 — Define minimal exact module-semantic query keys/products (`QueryKey`, `SemanticDependency`, `SemanticProduct` variants for `ResolvedImport`, `LinkedName`, `PublicExport`).
- [x] Task 21 — Define fingerprint functions and query module keys for exact module facts in `db/fingerprint.rs` and `db/mod.rs`.
- [x] Task 22 — Update `TrackingTypeResolver` in `checker/context.rs` and query execution in `db/query.rs` to record exact `LinkedName` and `PublicExport` dependencies.
- [x] Task 23 — Prove absence and re-export retargeting behavior in `tests/checkpoint_a4.rs`.
- [x] Task 24 — Audit aggregate `LinkedInterface` dependencies and verify no parallel dependency engine exists.

### Corrective Completion Evidence
1. Declaration surfaces, callable signatures, and field signatures no longer retain coarse consumer edges to their owner `LinkedInterface`; exact `LinkedName`/`PublicExport` edges remain query-owned, while aggregate edges remain only for canonical projections or genuine aggregate prerequisites.
2. Callable-body A4 inputs use retained canonical unlinked interfaces plus the broad linked semantic product. Workspace-level fingerprints are computed once per update; A5 precision narrowing remains deferred.
3. `ResolvedImport` input identity retains canonical resolution/topology evidence, while its product identity contains only observable target/prefix mapping or stable failure category.
4. Callable-signature source-span input hashing includes both range endpoints; product hashing remains source-movement stable.
5. Focused evidence: incremental (131 passed, 4 ignored), imported-resolution (9 passed), checkpoint A4 (2 passed), and semantic crate (1,095 passed, 42 ignored).
6. `cargo check -p phalcom-lsp` passes. The historical A4 run had two
   provider-lifecycle publication timeouts; later bisection showed the first
   bad checkpoint was A0, not an inherited baseline. Both are covered by the
   corrective A0 evidence above.
7. Nightly workspace format and strict clippy gates remain blocked by pre-existing unrelated workspace drift/warnings; no unrelated formatting or lint cleanup was applied.

---

## Checkpoint A5 — Coarse Semantic Safety Fingerprints

### Status
PARTIAL — Tasks 25/26 are implemented; Task 27 remains incomplete pending its
cross-module unused-export, high-fanout, stable-intermediate, and cold/parity
proofs.

### Tasks
- [x] Task 25 — Remove all-source resolution hashing from callable-body direct
  input.
- [x] Task 26 — Remove whole-`LinkedProgram` hashing from callable-body direct
  input.
- [~] Task 27 — Absent-name recovery, exact dependency recording, and
  product-stability barriers are proven; required cross-module unused-export
  and high-fanout evidence remains open.

### Verification evidence
- `phalcom-semantic/tests/checkpoint_a4.rs::test_previously_missing_public_name_appears_invalidates_consumer`
  proves absent-to-present invalidation.
- Incremental product-stability tests prove validated revisions can advance
  without changing computation revisions for reused products.
- `phalcom-semantic/src/db/fingerprint.rs` and `db/query.rs` no longer use
  whole-workspace source/link products as ordinary callable-body direct input.
- A true cross-module unused-export/high-fanout proof at the PA-5/PA-6 target
  scale is not yet present; do not mark A5 release evidence complete from the
  focused tests alone.

---

## Checkpoint A6 — Incremental Semantic Workspace Aggregation

### Status
PARTIAL. Explicit module deltas and retained shards are implemented; global
aggregate/worklist closure is not yet proven.

### Tasks
- [x] Task 28 — Retain per-module structural semantic shards and reuse
  unchanged shards across explicit module deltas.
- [~] Task 29 — Delta-maintain declaration namespace/table composition; module
  delta plumbing is present, but global declaration composition remains.
- [~] Task 30 — Delta-maintain hierarchy/supertype direct-edge products;
  structural worklists are narrowed, but aggregate closure remains open.
- [~] Task 31 — Module-shard type aliases and generic headers; retained alias
  contributions exist, but aggregate composition still has broad passes.
- [~] Task 32 — Publish declaration surfaces/callable/field signatures through
  exact worklists; ordinary edits still rebuild some aggregate state.
- [~] Task 33 — Compose immutable snapshots from retained shards with cold/
  incremental parity; required parity and work-count proof remains open.

### Verification evidence
- `SemanticModuleDelta` now crosses `WorkspaceModuleSession` into the semantic
  update path instead of forcing semantic to rediscover unchanged modules by
  hashing every source.
- `ModuleSemanticStructureShard::with_source` retains declaration/alias
  contributions when source text changes without an interface change.
- Changed/removed modules seed semantic and structural worklists; unchanged
  shards and linked dependency fingerprints are retained.
- `apply_module_mutations` now exposes module-layer work counts through the
  semantic publication and has a production-path body-edit test in
  `incremental::a7_performance`.
- Full semantic suite: 1,097 passed, 0 failed, 42 ignored. Module suite and
  provider lifecycle LSP tests are green.

### Remaining A6 gap
The semantic session still reconstructs some declaration, alias, hierarchy,
signature, and field-lifecycle aggregates by traversing retained workspace
state. The delta is no longer discarded at the module boundary, but this is
not yet the plan's strict “no ordinary-edit total semantic traversal” gate.

---

## Checkpoint A7 — Performance and Release Closure

### Status

PARTIAL: instrumentation and focused evidence are green. Plan A is not
release-complete because A0 COW staging, A6 aggregate delta maintenance, the
full PA-1..PA-10 evidence matrix, and exact core baseline comparison remain
open.

### Task 34 — Deterministic work metrics

- [x] `WorkspaceModuleStats::linked_components_considered` now distinguishes
  component scan work from recomputed and retained component products.
- [x] `SemanticUpdateStats` now publishes query products recomputed,
  revalidated, exact-name products recomputed/reused, reverse candidates
  considered, and dependency-bearing semantic products recomputed/reused.
- [x] Query metric accounting uses computation revision separately from current
  validation revision.

### Task 35 — Synthetic fixtures

- [x] `phalcom-modules/tests/checkpoint_a7.rs` provides deterministic builders
  for linear, star, disconnected, import-heavy, negative-import, re-export,
  hierarchy-fanout, and alias-SCC graphs.
- [x] The 1,000-module linear fixture is exercised in the ordinary module test
  lane and proves a leaf body edit performs zero import resolution and zero
  component recomputation.

### Task 36 — Acceptance matrix evidence

- [~] PA-1 body-only edit: production module-delta evidence now proves zero
  import resolution, zero component recomputation, and structural-shard reuse;
  required cross-module semantic-consumer work assertion remains open.
- [x] PA-2 one-of-20 import path, PA-3 unrelated negative import, PA-4
  missing-target recovery, and PA-7 disconnected retention pass in existing
  A2/A3/A7 fixtures.
- [ ] PA-5 unused public export: no load-bearing cross-module assertion that
  the consumer body computation revision remains unchanged.
- [ ] PA-6 high fanout: no 5,000 reverse-connected/~100 exact-consumer test
  asserting expensive recomputation or `reverse_candidates_considered` bounds.
- [~] PA-8 stable intermediate product: metric instrumentation exists, but no
  explicit A→B recompute/B-stable→C-reuse assertion is recorded here.
- [ ] PA-9 declaration/hierarchy isolation at required scale.
- [ ] PA-10 cold/incremental parity over the complete presentation set.

### Task 37 — Compiler/LSP parity and regression gate

- [x] `RUSTFLAGS='' cargo test -p phalcom-modules` passes.
- [x] `RUST_MIN_STACK=8388608 RUSTFLAGS='' cargo test -p phalcom-semantic`
  passes: 1,097 passed, 42 ignored.
- [x] `RUSTFLAGS='' cargo check -p phalcom-lsp` passes.
- [x] Full LSP release gate: provider lifecycle, semantic boundary, navigation,
  integration, unit, and doc-test lanes pass; aggregate integration lane is
  58 passed, 0 failed, 2 ignored.

### Task 38 — Workspace comparison and Plan-B handoff

- [ ] Compare exact current `phalcom-core` failure names with the A0 baseline
  set before classifying the workspace comparison. Counts alone do not prove
  “no new failures.” A clean `d60e458` library run passed; its 540-test
  integration lane did not finish within the verification window, so no exact
  workspace comparison is claimed.
- [ ] Plan-B handoff surface is recorded below, but Plan A remains blocked by
  the open A0/A6/matrix gates above.

### Plan-B handoff

- Final pushed HEAD: `93ded260feb13fc58e028db3e76983845e60ad73`
  (`fix(lsp): preserve module identity across protocol aliases`).
  The unrelated `docs/.obsidian/workspace.json` edit remains unstaged.
- Retained module products: `ModuleTopology`, `ImportSiteId` keyed resolution
  products with prefix provenance, `ComponentId`/`ComponentLinkedProduct`,
  `WorkspaceModuleStats`, and published reverse import/site indexes.
- Exact semantic products: `ResolvedImport`, `LinkedName`, `PublicExport`,
  `LinkedInterface`, declaration/surface/signature/field/body query products,
  and explicit `SemanticDb::purge_module` lifecycle.
- Snapshot topology/reverse-index APIs remain published through
  `SemanticWorkspaceInput` and semantic module query products.
- Remaining limitations: A0 map-copy staging, A6 aggregate traversal, missing
  PA-5/PA-6/PA-8/PA-9/PA-10 evidence, and exact core failure-set comparison.
  Source/reference index performance and editor overlay transaction work stay
  Plan B scope.
- Current corrective results: modules green; semantic green; full LSP green;
  the release verdict remains PARTIAL, not COMPLETE.
