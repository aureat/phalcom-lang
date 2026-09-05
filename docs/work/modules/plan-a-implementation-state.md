# Phalcom LSP Module Architecture — Plan A Implementation State

## Checkpoint Baseline
- Remote baseline / Local HEAD: `d60e4589352ac5f4167ba295e7e2a5f6c870ef4b`
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
COMPLETED

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
6. Module test suite: `cargo test -p phalcom-modules` passes 89 passed (0 failed).
7. Downstream checks: `cargo test -p phalcom-semantic --test semantic module_query_provenance` passes; `cargo check -p phalcom-lsp` passes with zero errors.

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
IN PROGRESS (HANDOFF PREPARED)

### Baseline Commit
- `fa1f0094b5db875d6c880fed870bd47b87376803` (`feat(modules): implement checkpoint A3 affected-component incremental linking`)

### Tasks
- [x] Task 20 — Define minimal exact module-semantic query keys/products (`QueryKey`, `SemanticDependency`, `SemanticProduct` variants for `ResolvedImport`, `LinkedName`, `PublicExport`).
- [x] Task 21 — Define fingerprint functions and query module keys for exact module facts in `db/fingerprint.rs` and `db/mod.rs`.
- [ ] Task 22 — Update `TrackingTypeResolver` in `checker/context.rs` and query execution in `db/query.rs` to record exact `LinkedName` and `PublicExport` dependencies.
- [ ] Task 23 — Prove absence and re-export retargeting behavior in `tests/checkpoint_a4.rs`.
- [ ] Task 24 — Audit aggregate `LinkedInterface` dependencies and verify no parallel dependency engine exists.


