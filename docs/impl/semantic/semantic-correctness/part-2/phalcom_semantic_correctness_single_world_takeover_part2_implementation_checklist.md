# Phalcom Semantic Correctness / Single-World Takeover — Part 2 Checklist

> Task tracker for `phalcom_semantic_correctness_single_world_takeover_part2_canonical_identity_projection_advisory_takeover_spec.md`.
> Check an item only after implementation and focused verification pass. Live source overrides stale specification observations.

## Status key

- `[x]` implemented and focused verification passed
- `[~]` partial, deferred, or only narrowly verified
- `[ ]` not implemented

## Scope and ownership

- [x] Part 1 WIP/amendments and attached handoff reviewed; Part 3 lifecycle cutover remains out of scope.
- [x] Existing dirty and untracked work preserved; only Part 2-owned files are changed.
- [~] Part 2 release gate has compiler identity/projection/advisory products and LSP adapters landed; full LSP authority deletion and capability-suite closure remain open.

## Implementation tasks

### Task 1 — Lock identity lifetimes and source-site primitives

- [x] Add snapshot-scoped `SourceSiteLocalId`, `SourceSiteId`, and `SourceSiteRef`.
- [x] Add compiler-owned `SourceOwner`, `SourceSite`, and `SemanticTargetId`.
- [x] Test canonical declaration/callable/field identity across revisions.
- [x] Test stale `SourceSiteRef` rejection across snapshots.
- [x] Run focused source-index test and `cargo check -p phalcom-semantic`.

### Task 2 — Move lexical source identity into compiler ownership

- [x] Add compiler-owned nested scopes and source-order visibility.
- [x] Add imports/classes, method/block/for parameters, destructuring, and mutability.
- [x] Preserve first same-scope binding and record `redeclaration_of`.
- [x] Add focused scope-builder regressions without importing LSP identity types.

### Task 3 — Move exact occurrence indexing into compiler ownership

- [x] Add compiler-owned occurrence records and unresolved hints separate from semantic targets; collect and publish AST-wide occurrences before formal attachment.
- [x] Add bounded interval lookup with sorted starts and prefix max-end.
- [x] Add deterministic nested selection, target reverse index, and large-index coverage.

### Task 4 — Attach formal checker products to source sites

- [x] Attach `(CallableId, BindingId)` and `(CallableId, ExpressionId)` by exact checker identity; publish non-fatal attachment incidents on `SourceSemanticIndex`.
- [x] Publish canonical resolved call targets from existing expression products and project them onto selector occurrences.

### Task 5 — Publish source index and machine-readable formal projection

- [x] Publish compiler source/index products from one immutable `SemanticSnapshot`; indexed formal projection is attached to the same snapshot identity.
- [x] Preserve formal status, bounded causal invalidity, contract relations, callable identity, and dependency ownership through keyed checker products and projection records; machine-readable readiness/invalidity and formal causal/contract payloads are projected.
- [x] Make formal position lookup indexed and analysis-free; source occurrence AST references and canonical reverse-target lookup are published.

### Task 6 — Port advisory domain with canonical identities

- [x] Add compiler-owned `advisory::{shape,fact,provenance}`.
- [x] Keep advisory confidence/origin structurally separate from formal evidence.
- [x] Add bounded deterministic joins/provenance and canonical target IDs.

### Task 7 — Port advisory expression and flow analysis

- [x] Add compiler-owned advisory expression/flow analysis over canonical source scopes, formal resolved-call targets, and injected canonical dispatch adapters; direct, forwarding, cross-module propagation, compiler method-family dispatch, and formal/advisory disagreement behavior for ready, unknown, dynamic, invalid, suppressed, blocked, and cancelled formal states are verified.
- [x] Cover literals, collections, locals, formal call returns, shared binding flow, fields, canonical factory dispatch, and method-family capture with canonical selector/signature products.

### Task 8 — Port contribution-indexed interprocedural advisory solving

- [x] Add canonical parameter slots, contribution replacement/removal, changed-slot deltas, compiler DB advisory keys/dependencies, budget-driven worklist/SCC solving, and direct/forwarding/cross-module caller propagation.
- [x] Add explicit advisory product outcomes, deterministic callable-summary/fact fingerprints, solver budget/cancellation results, and focused incrementality tests.

### Task 9 — Publish advisory workspace in compiler snapshots

- [x] Publish formal, source, occurrence, module, and advisory products coherently under one `SnapshotId`.
- [x] Reuse unchanged source/advisory `Arc` shards; publish explicit advisory non-ready status independently from `ValueShape::Unknown`, and keep advisory publication failures non-fatal to valid formal snapshots.

### Task 10 — Replace LSP identity and snapshot bridges

- [~] Make LSP URI mapping a boundary over canonical module IDs; canonical `by_uri`/`by_module` mapping is active while legacy protocol keys remain for compatibility.
- [~] Remove string selector/owner reconciliation, full callable scans, and duplicate semantic IDs from migrated occurrence/reference/formal/advisory query paths; publication-time canonical callable handles and indexed compiler position/reference/formal reads are primary, while legacy surface/identity adapters remain.

### Task 11 — Delete/demote LSP semantic authority

- [~] Compiler snapshot is primary for occurrence, references, formal lookup, advisory binding/field/parameter/return facts, and canonical targets; LSP scope/dispatch/surface/advisory compatibility ownership remains for protocol parity.
- [~] Keep compiler-backed read-only adapters on migrated paths; expression inference retains a bounded fallback only when compiler expression coverage is missing, while complete old-engine authority removal remains deferred for remaining surface consumers.
- [x] Preserve substantive advisory/source test coverage at compiler ownership.

### Task 12 — Incrementality, performance, and takeover audit

- [~] Verify source/advisory reuse and narrow invalidation; focused Arc reuse and existing product/dependency suites pass.
- [x] Verify migrated position/reference/formal queries use indexed compiler products without request-time analysis or full target scans.
- [~] Workspace check, formatting, graph refresh, semantic and LSP focused gates pass; forbidden-pattern audit and capability-suite closure remain open.

## Part 2 release gate (§62)

- [ ] 01–05 — canonical identities, source binding ownership, snapshot guards, scope/index ownership
- [ ] 06–10 — occurrence/reference indexing, formal attachments, canonical call targets
- [ ] 11–15 — one snapshot, machine-readable formal projection, formal statuses preserved
- [ ] 16–21 — compiler-owned advisory domain, canonical IDs, formal/advisory separation, canonical dispatch
- [ ] 22–27 — compiler DB dependencies, solver convergence/statuses, coherent advisory publication
- [ ] 28–31 — invalid-but-known composition, deterministic fingerprints, reuse/invalidation
- [ ] 32–36 — LSP canonical-module adapters, no scans/string reconciliation/duplicate IDs
- [ ] 37–41 — LSP scope/occurrence/dispatch/module/advisory authority demoted
- [ ] 42–44 — demand-driven core, formal regressions, migrated advisory/source coverage
- [ ] 45–49 — workspace/semantic/LSP tests, forbidden-pattern review, one owner per concept

## Verification log

- `VERIFIED`: Task 1 source-site identity tests passed (3/3); `cargo check -p phalcom-semantic`, `cargo fmt --all -- --check`, and scoped `git diff --check` passed.
- `VERIFIED`: Task 2 scope/source identity tests passed (5/5); compiler check and formatting passed.
- `VERIFIED`: Task 3 interval/occurrence tests passed; AST-wide occurrence collection, selector target projection, compiler snapshot publication, reverse-target index, and large-index coverage passed.
- `VERIFIED`: Formal source attachment and indexed formal projection tests passed; non-fatal attachment incidents and machine-readable formal readiness status are published; workspace, callable-dependency, product-stability, and presentation focused suites passed.
- `VERIFIED`: Advisory domain foundation tests passed (4/4); canonical record/union normalization, collection joins, confidence separation, bounded provenance, and selector identity covered.
- `VERIFIED`: Advisory expression/flow, contribution-summary, snapshot, and incrementality tests passed; canonical literal/collection/local flow, formal call-result reuse, missing-builtin fallback, direct and forwarding-call parameter/return convergence, parameter replacement/removal, recursive convergence, budget/cancel status, explicit status, Arc reuse, and deterministic fingerprints covered.
- `VERIFIED`: Compiler-backed LSP position/reference/scope/dispatch/advisory reads pass the full registered integration suite (52 passed, 2 ignored); missing formal binding attachments remain incidents without suppressing independent expression/call attachments.
- `VERIFIED THIS SLICE`: Formal projection now carries bounded causal invalidity and binding-contract relation metadata; advisory outer convergence is explicitly budget-driven; linked interface exports feed source target attachment; cross-module advisory return propagation and canonical method-family capture pass focused tests; LSP callable reconciliation is publication-time indexed and canonical occurrence/reference paths no longer fall back to workspace scans.
- `BASELINE`: Full semantic suite retains handoff capability baseline 12/40 passed, 28/40 failed; failures are stale `var`/bare-brace fixtures plus documented capability gaps, not introduced Part 2 source-index failures.
- `VERIFIED`: `cargo check --workspace`, `cargo fmt --all -- --check`, full registered LSP integration (52 passed, 2 ignored), and full compiler suite except known capability baseline passed.
- `VERIFIED THIS SLICE`: Compiler-primary LSP binding, field, parameter, and callable-return reads pass the full LSP crate suite (245 unit tests, 52 integration tests, 3 module-navigation tests); inherited field lookup walks compiler hierarchy, and legacy facts are consulted only when no coherent compiler snapshot exists. Expression inference remains compatibility-backed for uncovered compiler expression sites.
- `VERIFIED THIS SLICE`: Advisory comparison is fail-closed for non-ready formal states and non-concrete `TypeKnowledge`; ready disagreement is observational only and leaves formal expression products unchanged. Nested advisory expression observation now publishes compiler-owned source-site facts. Focused advisory suites pass (10 analysis, 5 analyzer, 3 incrementality, 9 callable-dependency, 11 source-index tests).
- `VERIFIED THIS SLICE`: Exact-source LSP signature help now projects canonical compiler callable signatures with formal type terms and compiler advisory fallback; exact-source definition, references, and workspace symbols read compiler target/source indexes first; exact-source inlay formal binding presentation reads compiler formal products first. Focused signature, inlay, binding navigation, cross-file navigation, and module-navigation tests pass. Remaining completion, non-binding inlay lanes, compatibility adapters, and duplicate authority deletion stay open.
- `VERIFIED THIS SLICE`: Compiler-covered occurrence lookup now returns the compiler source-index occurrence even when its target is unresolved and carries only a bounded hint; the legacy LSP occurrence index is consulted only when compiler coverage is absent. Focused local binding/reference and cross-file definition tests pass; `git diff --check` passes for the owned files.
- `VERIFIED THIS SLICE`: Compiler-covered reference lookup now returns the canonical reverse-target index when available and an empty result for unresolved compiler hints; legacy workspace-wide reference scans remain limited to unmapped compatibility requests. Focused local binding/reference and receiver-definition regressions pass.
- `VERIFIED THIS SLICE`: Import/module completion now accepts the pinned compiler semantic snapshot and canonical importer module directly; exact-source requests use `ModuleQueryFacade`, while stale/unmapped requests return no semantic import result. Stage 3 completion and workspace semantic integration pass; `cargo check -p phalcom-lsp -p phalcom-modules` passes.
- `PENDING`: Tasks 10–12 remain partial because legacy LSP semantic compatibility ownership, standalone uncovered compiler expression sites, forbidden-pattern audit, and capability-suite closure are not complete.
