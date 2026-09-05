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
