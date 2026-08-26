# Phalcom Part 3 Persistent Workspace and LSP Cutover Checklist

Spec: docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_persistent_workspace_lsp_cutover_professional_ide_spec.md

Plan: docs/impl/semantic/semantic-correctness/part-3/phalcom_semantic_correctness_single_world_takeover_part3_implementation_plan.md

## How to use

- [ ] Check an item only after live source evidence and its focused test gate pass.
- [ ] Status labels are planning evidence, not completion marks:
  - partial: related code exists, but required behavior or proof is incomplete.
  - candidate: current source appears aligned, but focused verification is still required.
  - pending: implementation or verification remains.
  - blocked: predecessor gate must land first.
- [ ] Keep Part 1 and Part 2 release gates visible. A green Part 3 slice is not release completion.
- [ ] Record exact test commands, commit IDs, and audit output in the evidence section after each slice.
- [ ] Preserve unrelated dirty, staged, deleted, and untracked files.

## Initial live-state classification

- [ ] Part 1 release gate and corrections/amendments are verified from live source and tests. Status: blocked/unverified.
- [ ] Part 2 release gate and canonical identity/projection/advisory takeover are verified from live source and tests. Status: blocked/unverified.
- [ ] WorkspaceModuleSession exists and owns part of lifecycle. Status: partial.
- [ ] SemanticWorkspaceSession retains module session, SemanticDb, TypeStore, and last-known-good snapshot. Status: partial.
- [ ] SemanticSnapshot already carries source, formal, advisory, module, diagnostic, and source-index products. Status: partial.
- [x] System.print runtime, metadata, and current docs show Unit. Status: verified by focused native-contract and semantic tests; end-to-end IDE coverage remains open.
- [ ] Initial professional presentation tests exist. Status: partial; forbidden labels are covered, but all consumer cutovers remain open.
- [ ] LSP compiler publication bridge remains under transitional protocol adapters. Status: partial; obsolete static reconstruction names and fallback path are removed, duplicate semantic consumers remain.
- [ ] LSP WorkspaceIndex and duplicate phalcom-lsp/src/semantic implementation remain. Status: pending deletion.

## Implementation task checklist

### Task 0: Re-grounding

- [ ] Read Part 1 spec and corrections.
- [ ] Read Part 2 spec and checklist.
- [ ] Read current Part 3 spec.
- [ ] Run graphify query for affected owners and consumers.
- [ ] Record worktree ownership boundaries.
- [x] Run focused baseline tests: modules workspace session, semantic suite, core native contracts, LSP presentation, and LSP single-world tests.

### Task 1: Presentation and native red regressions

- [x] Add plain-label and forbidden-pattern tests.
- [x] Add contextual evidence wording tests.
- [ ] Add formal/advisory disagreement tests.
- [x] Add System.print and System.gc metadata/runtime coherence tests.

### Task 2: Trusted fixed returns

- [x] Prove native metadata reaches canonical callable signatures.
- [x] Prove System.print call result is Established Unit.
- [x] Prove caller normal-tail return is Established Unit.
- [ ] Add table-driven fixed-return coverage.
- [ ] Prove advisory products cannot replace formal fixed returns.

### Task 3: Module lifecycle

- [x] Stable project identity across ordinary edit.
- [x] Stable ModuleId across ordinary edit.
- [x] Stable standalone synthetic identity across edit and reopen.
- [x] Overlay precedence over disk.
- [x] Close restores disk source.
- [x] Delete clears source/module mappings.
- [ ] Rename/move follows canonical identity transition.
- [ ] Project/config changes use explicit rebuild path.
- [ ] Batch overlays link once.

### Task 4: Semantic publication

- [x] Add publication effects.
- [x] Publish one coherent snapshot atom.
- [x] Retain TypeStore across ordinary revisions.
- [ ] Invalidate removed-module reverse closure.
- [ ] Publish semantic errors with current products.
- [ ] Discard cancelled, budget-exceeded, and stale candidates.
- [ ] Retain last-known-good on infrastructure failure.
- [ ] Add structural recomputation counters.

### Task 5: Worker cutover

- [ ] Convert protocol events to canonical compiler mutations at boundary.
- [ ] Route worker updates through one SemanticWorkspaceSession.
- [x] Delete obsolete static workspace reconstruction production path.
- [x] Delete StaticWorkspaceIdentity and StaticWorkspacePublication.
- [x] Delete obsolete static-analysis refresh bridge.
- [x] Delete engine.set_static_analysis bridge.
- [ ] Retain debounce/latest-wins/cancellation/status/log behavior.

### Task 6: Request pinning and diagnostics

- [x] RequestContext pins one compiler snapshot.
- [x] Add Exact, Stale, and Unmapped source match.
- [x] Suppress stale semantic ranges on current open text.
- [x] Render current syntax diagnostics independently.
- [x] Render compiler semantic diagnostics only when source-coherent.
- [ ] Test concurrent old-snapshot request immutability.

### Task 7: Presentation renderer

- [ ] Add compiler-side formal/advisory presentation lanes.
- [ ] Add bounded evidence summaries.
- [ ] Formal Known wins visible primary type.
- [ ] Advisory fallback is visible only when formal has no usable concrete type.
- [ ] Dynamic and Unknown preserve their formal meaning.
- [ ] Evidence wording maps to causes, not internal enum names or scores.
- [ ] Renderer performs no semantic computation.

### Task 8: Hover/inlay/signature

- [ ] Hover reads compiler source-site views.
- [ ] Hover primary fact precedes docs and contextual evidence.
- [ ] Hover preserves declared/current distinction.
- [ ] Hover explains narrowing, specialization, native return, advisory-only, and actionable Unknown only when useful.
- [ ] Explicit annotations suppress duplicate inlays.
- [ ] Inlays use colon/arrow language syntax.
- [x] Signature help uses canonical callable signatures on exact-source requests; compatibility fallback remains for missing compiler products.
- [ ] No advisory glyph or status decoration remains.

### Task 9: Completion

- [ ] Remove shallow receiver semantic reconstruction.
- [ ] Resolve receiver through compiler source/advisory products.
- [ ] Preserve self, super, class object, union, native/core, and incomplete-dot behavior.
- [ ] Use bounded lexical/global fallback only for stale/unmapped source.
- [ ] Completion items contain no advisory decoration.

### Task 10: Navigation and WorkspaceIndex deletion

- [x] Definition uses compiler exact target first on exact-source requests.
- [ ] Advisory target is fallback only when exact target is absent.
- [x] References use compiler reverse target index on exact-source requests.
- [x] Workspace symbols use compiler declaration/source index first, with text-index fallback.
- [ ] Snapshot-local identities cannot alias across revisions.
- [ ] All semantic WorkspaceIndex consumers are removed.
- [ ] WorkspaceIndex is deleted or reduced to a documented non-semantic text-only role.

### Task 11: Modules and core navigation

- [ ] Import completion uses ModuleQueryFacade/module query products.
- [ ] Request path performs no filesystem semantic resolution.
- [ ] Core/native locations use canonical source provenance.
- [ ] Virtual source content remains protocol adaptation only.

### Task 12: Semantic tokens

- [x] Lexer fallback remains safe.
- [x] Exact compiler occurrences refine semantic roles through `CompilerSemanticSnapshot::source_index`.
- [x] Stale/unmapped requests omit stale semantic refinement and retain lexical/AST syntax coloring.
- [ ] Refresh notifications follow product effects/fingerprints.

### Task 13: Duplicate semantic deletion

- [ ] Consumer parity passes before deletion.
- [ ] Duplicate LSP analyzer, facts, flow, dispatch, scope, occurrence, IDs, module graph, invalidation, query, and snapshot files are deleted.
- [ ] LSP imports compiler/module identities directly.
- [ ] No compatibility shim retains mutable semantic authority.
- [ ] graphify update is run and graphify-out status is inspected.

### Task 14: Acceptance suites

- [ ] Cold/incremental final-world parity.
- [ ] Open/change/close/reopen lifecycle.
- [ ] Delete/rename lifecycle.
- [ ] Project configuration lifecycle.
- [ ] Cancellation/latest-wins.
- [ ] Semantic errors still publish.
- [ ] Concurrent old-snapshot request.
- [ ] Body-only structural reuse counters.
- [ ] System.print end-to-end golden.
- [ ] Similar fixed-return table.
- [ ] Practical IDE golden matrix.

### Task 15: Documentation and audits

- [ ] Port valuable bridge behavioral tests before deleting obsolete bridge tests.
- [ ] Update ownership comments and architecture documentation.
- [ ] Run UX forbidden-pattern audit.
- [ ] Run single-world forbidden-symbol audit.
- [ ] Run native-contract audit.
- [ ] Run formatting, workspace check, crate tests, and clippy.

## Part 3 section 88 completion gates

Every item remains unchecked until independently evidenced. Initial status describes current planning knowledge only.

### Predecessors and lifecycle

- [ ] 1. Part 1 release gate passes. Status: blocked/unverified.
- [ ] 2. Part 1 corrections/amendments are implemented. Status: blocked/unverified.
- [ ] 3. Part 2 release gate passes. Status: blocked/unverified.
- [ ] 4. WorkspaceModuleSession owns persistent project/source/module lifecycle. Status: partial.
- [ ] 5. Ordinary source edits retain project identity. Status: partial.
- [ ] 6. Ordinary source edits retain canonical ModuleId when logical identity is unchanged. Status: partial.
- [ ] 7. Standalone synthetic identity is stable across edits. Status: partial.
- [ ] 8. Open source overlays have precedence over disk. Status: partial.
- [ ] 9. Closing an overlay restores disk source without inventing a new module identity. Status: partial.
- [ ] 10. Project/root/dependency changes use an explicit project lifecycle invalidation path. Status: partial.

### Session, snapshots, and epistemic boundaries

- [ ] 11. SemanticWorkspaceSession is the sole semantic session owner. Status: partial.
- [ ] 12. One TypeStore/TypeStoreId is retained across ordinary workspace revisions. Status: partial.
- [x] 13. Obsolete static workspace reconstruction is deleted from production. Status: verified by symbol audit.
- [x] 14. StaticWorkspaceIdentity is deleted. Status: verified by symbol audit.
- [x] 15. Nested LSP static_snapshot publication is deleted. Status: verified by symbol audit; transitional compiler adapter remains.
- [x] 16. The LSP worker publishes one compiler Arc<SemanticSnapshot>. Status: focused worker test passes; all consumers remain open.
- [x] 17. A request pins one compiler snapshot handle. Status: source implementation landed; concurrent immutability test remains open.
- [x] 18. Source-position semantic queries require source-revision coherence. Status: Exact/Stale/Unmapped classification landed; full consumer audit remains open.
- [ ] 19. Stale semantic diagnostics are not rendered against current open-buffer ranges. Status: pending.
- [ ] 20. Semantic errors still publish current semantic products. Status: partial.
- [ ] 21. Cancelled/stale candidate updates never publish. Status: pending.
- [ ] 22. Last-known-good publication survives infrastructure failure. Status: partial.
- [ ] 23. Formal and advisory facts remain distinct internally. Status: partial.
- [ ] 24. Advisory facts cannot emit hard type diagnostics. Status: partial.
- [ ] 25. Advisory facts cannot replace Established formal knowledge. Status: partial.
- [ ] 26. Advisory facts cannot upgrade formal assumptions/proofs. Status: partial.

### Professional IDE presentation

- [x] 27. Ordinary advisory type labels contain no advisory glyph. Status: UX audit passed.
- [x] 28. Inlay hints contain no advisory glyph. Status: UX audit passed.
- [x] 29. Signature help contains no advisory glyph. Status: UX audit passed.
- [ ] 30. Completion items contain no advisory glyph/status decoration. Status: pending.
- [x] 31. Production hover contains no Confidence taxonomy. Status: UX audit passed.
- [x] 32. Production hover contains no Observed type/Observed return boilerplate. Status: UX audit passed.
- [ ] 33. Hover primary line uses canonical ordinary Phalcom spelling. Status: partial.
- [ ] 34. Hover shows declared/current distinction when materially useful. Status: pending.
- [ ] 35. Hover explains Assumed evidence when materially useful. Status: pending.
- [ ] 36. Hover explains flow narrowing when materially useful. Status: pending.
- [ ] 37. Hover explains generic specialization when materially useful. Status: pending.
- [ ] 38. Hover can explain advisory-only inference without mathematical notation. Status: partial.
- [ ] 39. Phaldoc remains prominent. Status: partial.

### Native contracts and formal return proving

- [x] 40. System.print native metadata returns Unit. Status: focused native-contract test passes.
- [x] 41. System.print Rust implementation returns vm.unit_value(). Status: focused native-contract test passes.
- [x] 42. docs/spec/current/system.md documents Unit return for print. Status: live docs verified.
- [x] 43. Compiler native import establishes Unit for System.print. Status: focused semantic test passes.
- [x] 44. A System.print call expression is Established Unit. Status: focused semantic test passes.
- [x] 45. A method with System.print as its normal tail is Established Unit. Status: focused semantic test passes.
- [ ] 46. Such a method's inlay/hover never reports Option. Status: pending end-to-end regression.
- [ ] 47. Trusted fixed-return formal contracts take precedence over advisory summaries generally. Status: pending generic regression.
- [ ] 48. Native metadata self-consistency audit passes. Status: pending.
- [x] 49. System.gc native metadata is reconciled to canonical documented None without changing it to Unit. Status: focused native-contract test passes.

### Compiler snapshot consumers

- [ ] 50. Diagnostics consume the compiler snapshot directly. Status: pending.
- [ ] 51. Hover consumes compiler source-site/presentation views directly. Status: pending.
- [~] 52. Inlay hints consume compiler source sites directly. Status: formal binding lane is compiler-first; field, parameter, return, and closure lanes remain compatibility-backed.
- [x] 53. Signature help resolves canonical compiler callable signatures. Status: exact-source path projects canonical compiler signatures with formal terms and advisory fallback; focused integration passes.
- [ ] 54. Completion consumes compiler receiver/surface/advisory products. Status: partial.
- [x] 55. Definition consumes compiler target/location indexes. Status: exact-source compiler target/location path is primary; focused navigation tests pass.
- [x] 56. References consume compiler reverse target index. Status: exact-source reverse target path is primary, including binding declaration roots; focused binding navigation passes.
- [x] 57. Workspace symbols consume compiler declaration index. Status: compiler source declaration/callable/field index is primary with text-index fallback; focused cross-file symbol test passes.
- [ ] 58. Import/module completion consumes ModuleQueryFacade. Status: pending.
- [ ] 59. Core/native navigation consumes canonical source provenance. Status: partial.
- [x] 60. Semantic token semantic refinement consumes compiler occurrences. Status: direct source-index path plus focused LSP build/tests verified.
- [ ] 61. No semantic handler performs filesystem resolution on request path. Status: pending audit.
- [ ] 62. No semantic handler runs formal analysis on request path. Status: pending audit.
- [ ] 63. No semantic handler runs advisory solving on request path. Status: pending audit.
- [ ] 64. No semantic handler rebuilds declaration/module surfaces from AST. Status: pending audit.

### Physical deletion and release behavior

- [ ] 65. Duplicate LSP semantic engine/database is deleted. Status: pending.
- [ ] 66. Duplicate LSP semantic IDs are deleted. Status: pending.
- [ ] 67. Duplicate LSP scope/occurrence/dispatch/module graph/advisory solver is deleted. Status: pending.
- [ ] 68. WorkspaceIndex semantic authority is deleted. Status: pending.
- [ ] 69. Cold and incremental final semantic products pass parity tests. Status: pending.
- [ ] 70. Open/change/close lifecycle tests pass. Status: pending.
- [ ] 71. Delete/rename lifecycle tests pass. Status: pending.
- [ ] 72. Project configuration lifecycle tests pass. Status: pending.
- [ ] 73. Cancellation/latest-wins tests pass. Status: pending.
- [ ] 74. Concurrent old-snapshot request immutability test passes. Status: pending.
- [ ] 75. Body-only edit structural counters show no project-universe rebuild. Status: pending.
- [ ] 76. Unrelated callables remain reused after isolated body edit. Status: pending.
- [ ] 77. Presentation-only/semantic-token refreshes are fingerprint-driven. Status: pending.

### Broad verification and final ownership proof

- [x] 78. cargo check --workspace passes. Status: verified.
- [x] 79. cargo test -p phalcom-modules passes. Status: verified.
- [x] 80. cargo test -p phalcom-semantic passes. Status: verified.
- [ ] 81. cargo test -p phalcom-core passes. Status: unverified.
- [x] 82. cargo test -p phalcom-lsp passes. Status: verified.
- [ ] 83. IDE golden acceptance tests pass. Status: pending.
- [x] 84. UX forbidden-pattern audit is manually reviewed. Status: verified; no forbidden production strings remain.
- [x] 85. Single-world forbidden-symbol audit is manually reviewed. Status: verified for obsolete static names; duplicate authority remains open and is recorded.
- [x] 86. Native-contract audit is manually reviewed. Status: verified by the focused native-contract target.
- [ ] 87. A reviewer can point to exactly one owner for project/module identity. Status: pending architecture review.
- [ ] 88. A reviewer can point to exactly one owner for formal semantics. Status: pending architecture review.
- [ ] 89. A reviewer can point to exactly one owner for advisory semantics. Status: pending architecture review.
- [ ] 90. A reviewer can point to exactly one immutable semantic snapshot consumed by all semantic LSP requests. Status: pending architecture review.

## Verification record

### Focused gates

- [x] Continuation slice: compiler-owned advisory module-member projection and cross-module parameter propagation.
  Result: `cargo test -p phalcom-lsp --test integration workspace_semantics -- --nocapture` — 9 passed; `cargo test -p phalcom-semantic --test semantic advisory_parameter_transfer -- --nocapture` — 2 passed.
  Evidence: imported module aliases use canonical source targets; linked module exports and compiler declaration surfaces resolve `Provider.Service` as a verified class object; join and forwarding regressions pass without restoring LSP semantic authority.
- [x] Focused consumer/lifecycle rerun after continuation slice.
  Result: `cargo check -p phalcom-lsp -p phalcom-modules` passed; `cargo test -p phalcom-modules --test workspace_session -- --nocapture` — 5 passed; `cargo test -p phalcom-lsp --test integration -- --nocapture` — 53 passed, 2 ignored; module navigation — 3 passed; professional presentation — 2 passed; single-world cutover — 2 passed.
  Evidence: stage 1–7 integration, workspace semantic propagation, module navigation, presentation, persistent identity, and snapshot-store reuse remain green after compiler advisory changes.
- [x] Continuation graph refresh: `graphify update . --no-cluster`.
  Result: graph refreshed with 85,324 nodes and 130,021 edges; immediate `git status --short --branch` showed no graphify output changes.
- [x] `cargo test -p phalcom-modules --test workspace_session -- --nocapture`
  Result: 5 passed.
  Evidence: persistent project/standalone identity, overlays, disk fallback, removal, relative imports, and root reset.
- [x] `cargo test -p phalcom-core --test native_contracts -- --nocapture`
  Result: passed.
  Evidence: System.print Unit metadata/runtime and System.gc None metadata/runtime.
- [x] `cargo test -p phalcom-semantic --test semantic -- --nocapture`
  Result: 382 passed, 10 ignored.
  Evidence: trusted native Unit call/tail publication and compiler semantic baseline.
- [x] `cargo test -p phalcom-lsp --test professional_semantic_presentation -- --nocapture`
  Result: 2 passed.
  Evidence: ordinary type labels and contextual advisory wording.
- [x] `cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture`
  Result: 2 passed.
  Evidence: persistent compiler session identity and retained TypeStore across worker edits.
- [x] Compiler publication-effect routing regression gate
  Result: `cargo check -p phalcom-lsp` passed; `cargo test -p phalcom-lsp --test single_world_cutover -- --nocapture` passed with 2 tests.
  Evidence: inlay and semantic-token refresh effects now include compiler formal/advisory/source-index product changes while preserving protocol-adapter compatibility.
- [x] `cargo test -p phalcom-semantic --test semantic publication_effects_distinguish_initial_graph_build_from_body_edit -- --nocapture`
  Result: 1 passed.
  Evidence: initial publication reports graph/declaration effects; body-only edit retains TypeStore and suppresses graph/declaration rebuild effects.
- [x] Recovered syntax publication regression gate
  Result: stage 7 diagnostic integration test passed after the module session accepted the boundary's recovered AST while retaining raw invalid text.
  Evidence: `cargo test -p phalcom-lsp --test integration stage7_static_diagnostics::test_static_mismatch_publishes_typecheck_diagnostics -- --nocapture`.
- [x] `cargo test -p phalcom-lsp --lib`
  Result: 246 passed.
  Evidence: worker, request, presentation, semantic-adapter, lifecycle, and protocol unit suite.
- [x] `cargo test -p phalcom-lsp --test integration`
  Result: 52 passed, 2 ignored.
  Evidence: stage 1–7 integration, workspace lifecycle, cross-file navigation/hover, syntax recovery, and compiler-consumer regressions.
- [x] `cargo test -p phalcom-lsp`
  Result: 246 unit tests passed; 52 integration tests passed, 2 ignored; module navigation 3 passed; professional presentation 2 passed; single-world cutover 2 passed; doc tests passed.
  Evidence: full LSP package target set, including relative import navigation and publication-gap fallbacks.
- [x] `cargo check --workspace`
  Result: passed.
  Evidence: all workspace crates type-check after recovered-program publication support.
- [x] `cargo test -p phalcom-modules`
  Result: passed.
  Evidence: full modules suite, including 5 workspace-session lifecycle tests.
- [x] `cargo test -p phalcom-semantic`
  Result: 382 passed, 10 ignored.
  Evidence: full semantic unit/integration suite, including publication effects and trusted fixed-return coverage.
- [ ] `cargo test -p phalcom-core --test lang -- --nocapture`
  Result: 53 passed, 1 failed, 4 ignored.
  Evidence: all `System.print` Unit expectations now pass; remaining failure is the existing `compile_error_destructure_no_initializer` diagnostic wording mismatch (`binding.const_requires_initializer` versus the compiler fixture's `requires an initializer to unpack`).
- [x] `cargo test -p phalcom-core --test native_contracts -- --nocapture`
  Result: 3 passed.
  Evidence: System.print Unit metadata/runtime and System.gc None metadata/runtime.
- [x] `graphify update .`
  Result: graph rebuilt; status inspected immediately.
  Evidence: 85,299 nodes and 121,893 edges; no unrelated graph files appeared in `git status`.

### Broad gates

- [ ] cargo fmt --check
  Result: fails only on unrelated formatting in `phalcom-semantic/tests/semantic/capabilities/deep_regressions.rs`, `phalcom-semantic/tests/semantic/capabilities/iteration.rs`, and `phalcom-semantic/tests/semantic/support/regressions.rs`; modified files were rustfmt-checked explicitly.
- [x] cargo check --workspace
- [x] cargo test -p phalcom-modules
- [x] cargo test -p phalcom-semantic
- [ ] cargo test -p phalcom-core
  Result: core unit/integration/invariant targets pass; language aggregate is 53 passed, 1 failed, 4 ignored for the same diagnostic wording mismatch.
- [x] cargo test -p phalcom-lsp
  Result: full package target set passed; see focused record above.
- [ ] cargo clippy --workspace

### Audit evidence

- [x] UX forbidden-pattern audit reviewed: no production `≈`, `Observed type:`, `Observed return:`, or `Confidence:` strings under `phalcom-lsp/src`.
- [x] Single-world forbidden-symbol audit reviewed: obsolete static workspace names and `static_snapshot` are absent; duplicate semantic engine/WorkspaceIndex remain explicitly open.
- [x] Compiler-consumer audit reviewed: hover/definition/references gate compiler occurrence reads on `SourceMatch::Exact`; semantic-token refinement reads compiler source occurrences directly.
- [x] Native-contract audit reviewed: native-contract target passed 3 tests; System.print is Unit and System.gc is None in runtime and metadata.
- [x] graphify update completed and graphify-out status inspected.
  Result: 85,301 nodes and 121,904 edges; no graphify files appeared in the worktree status.
- [ ] Final ownership review names exactly one owner per semantic authority.

## Final completion statement

Do not write a completion statement until gates 1 through 90 are checked and each has focused evidence. A focused green slice, partial implementation, or passing broad compile alone is not Part 3 completion.
