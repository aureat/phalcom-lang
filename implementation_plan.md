# Phalcom Compiler, LSP, and IDE Integration — Incremental Formal Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete compiler, LSP, and VS Code integration around one compiler-owned formal semantic authority, one canonical module authority, immutable LSP snapshots, navigable core sources, truthful diagnostics, and structural incremental-performance guarantees.

**Architecture:** `phalcom-modules` owns module identity, resolution, linking, visibility, exposure, and dependency rules. `phalcom-semantic::db::SemanticDb` owns formal semantic products, revisions, dependencies, invalidation, stable type storage, and publication. `phalcom-lsp` schedules source updates and presents immutable compiler products; its `ValueShape` and advisory flow remain explicitly non-authoritative. The compiler may retain a cold-analysis wrapper, but production LSP refreshes must not create a second formal database, module lifecycle, or whole-workspace checker.

**Tech Stack:** Rust workspace, Cargo integration tests, `phalcom-semantic` formal checker and `SemanticDb`, `phalcom-modules` project/module model, `phalcom-lsp` LSP adapter, VS Code TypeScript extension, TOML IDE-golden expectations.

**Spec:** `docs/work/analyses/typing/2026-08-23-phalcom-compiler-lsp-ide-integration-incremental-semantics.md`

## Global Constraints

- Preserve existing user-owned dirty changes. Do not reset, discard, or reformat unrelated work.
- Keep `phalcom-modules` as the sole authority for module meaning. LSP URI handling may map editor documents to canonical source identity, but may not define import semantics.
- Keep `phalcom-semantic::db::SemanticDb` as the sole formal semantic owner. Do not add a second LSP formal cache, type store, revision counter, or invalidation graph.
- Treat `ValueShape`, LSP advisory flow, and runtime observations as evidence only. They cannot upgrade formal `Unknown`, `Dynamic`, `Invalid`, `Blocked`, cancellation, budget, or internal-failure outcomes.
- Separate formal type knowledge from analysis status. A formal type may be `Known`, `Unknown`, or `Dynamic`; expression/callable status may independently be `Ready`, `Invalid`, `Blocked`, `Cancelled`, `BudgetExceeded`, `InternalFailure`, or another explicit non-ready state.
- `ProgramAnalyzer` may return an analyzed snapshot containing semantic errors. Code generation remains gated by semantic validity in `ProgramCompiler`.
- Startup core analysis is demand-driven. It must not eagerly request every universe callable body. Deep analysis may run for an opened/edited body, a required semantic query, or an explicit background deep-analysis mode.
- Wall-clock targets are reference-machine SLOs. CI correctness gates use structural counters, fingerprints, dependency closures, and forbidden-operation assertions.
- Every implementation step must add or update focused tests before broad validation.

## Landed Baseline

The following behavior is present or verified and must not be reimplemented as duplicate infrastructure:

- LSP status lifecycle repairs in `phalcom-lsp/src/analysis_service.rs` and `phalcom-lsp/src/analysis_status.rs`; focused status tests pass.
- Structured analysis logging in `phalcom-lsp/src/analysis_log.rs`, backend forwarding, serializable performance counters, and logging integration tests; focused logging tests pass.
- Compiler analysis/validity separation in `phalcom-core/src/modules/compile.rs`, with registered tests in `phalcom-core/Cargo.toml` and `phalcom-core/tests/semantic_analysis.rs`; focused semantic-analysis tests pass.
- Formal flow substrate exists in `phalcom-semantic/src/checker/flow/` and protocol-only iteration exists in `phalcom-semantic/src/checker/statement.rs`.
- `CallableAnalysis` exists and is directly published into `SemanticSnapshot::callable_analyses` by the current workspace analysis path.
- Compiler-owned `SemanticDb` substrate contains typed query keys, query state, dependency tracking, reverse closure, cancellation/budget outcomes, stale publication rejection, metrics, and a stable `Arc<TypeStore>` field.
- `examples/ide-golden` contains source fixtures and expectation files, but it is not yet a complete automated acceptance runner for landed Waves 0–7.

These facts do not establish full production ownership. In particular, current DB callable queries still execute analysis instead of returning a stored product, product encoding still publishes empty bytes with a zero fingerprint, `analyze_workspace` creates a fresh `TypeStore`, and the LSP production path still contains `run_static_workspace_analysis` with fresh module/formal-analysis construction.

## Progress Log

- 2026-08-23: Task 4A completed. All production and test `SemanticDiagnostic` construction now uses explicit source-owned `ModuleId`; relation-policy helpers propagate the checker module; implicit `error`/`warning` constructors were removed.
- 2026-08-23: Task 4B completed for current CLI/LSP consumers. CLI text output now uses canonical rich rendering; JSON preserves semantic fields; LSP secondary labels resolve through canonical module-source mappings.
- 2026-08-23 verification: `cargo check -p phalcom-semantic`, `cargo check -p phalcom-core`, `cargo check -p phalcom-lsp`, semantic diagnostic tests, LSP diagnostic unit tests, core semantic-analysis tests, and LSP integration (48 passed, 2 ignored) pass.
- 2026-08-23 baseline note: `phalcom-semantic --test type_annotations lowers_union_and_rejects_unsaturated_or_invalid_applications` fails before and after this slice because `resolve_type_annotation` applies `Object` to an unsaturated constructor; the same behavior and expectation exist at `HEAD`. Deferred to type-annotation semantics, outside Task 4.
- 2026-08-23: Task 5 completed. Inlay-hint collection now derives explicit annotation ranges from the pinned AST and suppresses advisory hints for annotated bindings, fields, parameters, and callable returns while retaining hints for unannotated declarations.
- 2026-08-23 verification: inlay-hint unit tests (10 passed) and registered `stage6_inlay_hints` integration tests (2 passed) pass.
- 2026-08-23: Task 6 startup slice implemented. `SemanticEngine::update_core_surface_only` now publishes core declarations/native surfaces without entering callable flow solving; explicit queued core replacements use the deep path for open/edit demand.
- 2026-08-23 verification: core startup integration tests pass with non-empty core classes, zero eager callable summaries, zero callable-body analysis, and deep analysis after an explicit core replacement; status and structured-log suites still pass.
- 2026-08-23: Task 7 typed-query slice implemented. `phalcom-semantic::db::SemanticDb` now retains typed `SemanticProduct`s beside query states; callable-body cache hits return the stored `Arc<CallableAnalysis>`, non-ready terminal states are recorded, and callable fingerprints are populated instead of zero.
- 2026-08-23 verification: `cargo test -p phalcom-semantic --test db -- --nocapture` passes (5 tests), including typed callable cache-hit identity and invalidation.
- 2026-08-23: Task 7 cache-input slice implemented. Callable-body reuse now compares a deterministic callable/body/store input fingerprint; changed bodies invalidate the old product and reverse dependents before recomputation, while unchanged inputs retain typed `Arc` identity.
- 2026-08-23 verification: DB regression coverage passes for nonzero input fingerprints, unchanged typed cache hits, changed-body recomputation, and isolated invalidation.
- 2026-08-23: Task 8 presentation slice implemented. `TypePresenter` delegates canonical type spelling to `TypeStore`; `SemanticPresentationIndex` projects callable and expression products without owning inference, identities, or invalidation.
- 2026-08-23 verification: presentation tests pass (4 tests), including canonical generic specialization formatting; DB tests pass (6 tests), including stable `TypeStoreId` across revisions.
- 2026-08-23: Task 9 formal-state adapter slice implemented. LSP hover/inlay paths now preserve compiler-owned formal `Known`/`Unknown`/`Dynamic`/invalid and non-ready states; advisory labels use `≈` and cannot replace an available formal result.
- 2026-08-23 verification: semantic presentation (3), LSP hover (21), inlay (10), stage6 integration (2), and callable-publication/formal-type worker tests pass.
- 2026-08-23: Task 9b provider slice implemented. LSP signature help now recovers incomplete call sites, receiver/unqualified calls, labels, active parameters, and compiler formal parameter/return presentations with advisory `≈` fallback.
- 2026-08-23 verification: signature-help recovery tests (2), backend capability initialization, and end-to-end incomplete receiver-call integration pass.
- 2026-08-23: Task 10 facade slice implemented. `phalcom-modules::ModuleQueryFacade` now exposes roots, exposed children, linked exports, precomputed import targets, reverse importers, and source provenance as a read-only view over supplied canonical products; no module session or invalidation owner was added.
- 2026-08-23 verification: module query facade tests pass (2); SemanticDb-backed product/session wiring remains pending.

## Architecture Decision Register

| ID | Decision |
| --- | --- |
| DEC-INTEG-001 | `phalcom-modules` is the sole module-resolution authority. |
| DEC-INTEG-002 | `phalcom-semantic` is the sole formal type, flow, callable-analysis, and semantic-diagnostic authority. |
| DEC-INTEG-003 | Existing `phalcom-semantic::db::SemanticDb` becomes the active incremental query database. |
| DEC-INTEG-004 | One semantic workspace epoch retains stable `TypeStoreId` across revisions. |
| DEC-INTEG-005 | LSP query handlers consume immutable published snapshots and do not mutate formal state. |
| DEC-INTEG-006 | Analysis products remain publishable with semantic errors; code generation rejects invalid products. |
| DEC-INTEG-007 | Source-owned semantic diagnostics use explicit `ModuleId`; implicit core ownership is removed from production construction. |
| DEC-INTEG-008 | Formal and advisory presentation are separate axes. Explicit annotations suppress inferred formal hints. Formal `Known(T)` renders `T`; formal `Unknown`/`Dynamic` and non-ready statuses are never strengthened by advisory inference. Advisory evidence may be shown separately as `Observed shape: ≈ T`. |
| DEC-INTEG-009 | Universe body analysis is demand-driven, not eagerly performed for every core callable. |
| DEC-INTEG-010 | `project.toml` is the startup project authority; source roots and dependency roots are derived once through canonical project products. |
| DEC-INTEG-011 | Import and module completion candidates are produced from resolvable canonical module products. |
| DEC-INTEG-012 | Core/native definitions retain canonical physical or virtual source provenance. |
| DEC-INTEG-013 | Failed formal updates preserve the last-known-good published formal snapshot. |
| DEC-INTEG-014 | Completed analysis batches end in `Ready` or `Error`, never indefinitely in `Publishing`. |
| DEC-INTEG-015 | Structured analysis logs use `phalcom/analysisLog` and `phalcom.analysis.logLevel`. |
| DEC-INTEG-016 | Structural performance counters and fingerprints enforce incremental boundaries in CI. |
| DEC-INTEG-017 | Names distinguish `phalcom_semantic::db::SemanticDb` from any LSP published-snapshot holder. |

## Implementation Order

1. Keep Tasks 1–3 as landed verification gates; do not rebuild them.
2. Finish diagnostic source ownership and compiler/LSP information-preserving conversion.
3. Finish typed DB products, stable workspace lifecycle, callable-body reuse, and exact invalidation.
4. Connect the LSP worker to the compiler semantic session and delete the legacy static formal path.
5. Add pure formal presentation, formal-first hover/inlays, annotation suppression, and signature help.
6. Expose canonical module queries, completion, module occurrences, and source navigation.
7. Finish project-aware scheduling, diagnostics presentation, and VS Code status/log/source integration.
8. Add parity, golden-runner, and structural performance gates; then run broad acceptance validation.

---

## Track A — Safe Immediate Integration Work

### Task 1 — Verify LSP Analysis Status Lifecycle

**Status:** Complete; verification-only.

**Evidence:** `finish_status_after_batch` exists in `phalcom-lsp/src/analysis_service.rs`; `set_error` exists in `phalcom-lsp/src/analysis_status.rs`; edit-only publication returns to `Ready` in `phalcom-lsp/tests/analysis_status.rs`.

- [x] Retain monotonic session and sequence behavior.
- [x] Retain stale-batch discard behavior.
- [x] Retain cancellation/error terminal transitions.
- [ ] Re-run `cargo test -p phalcom-lsp --test analysis_status -- --nocapture` after each later worker change.

### Task 2 — Verify Structured Analysis Logs and Failure Observability

**Status:** Complete for Rust notification/event plumbing; extension consumption remains in Task 22.

**Evidence:** `phalcom-lsp/src/analysis_log.rs`, backend forwarding, serializable `CounterSnapshot`, and `phalcom-lsp/tests/analysis_logging.rs` exist. The focused test passes.

- [x] Retain `AnalysisLogLevel`, `AnalysisLogEvent`, and `AnalysisLogNotification`.
- [x] Retain event emission for workspace start, core surface load, semantic batch start, and snapshot publication.
- [x] Retain real session/sequence context for failure status notifications.
- [ ] Re-run `cargo test -p phalcom-lsp --test analysis_logging -- --nocapture` after later analysis-service changes.
- [ ] Complete client configuration/output routing in Task 22.

### Task 3 — Separate Compiler Analysis from Program Validity

**Status:** Complete; verification-only.

**Files:** `phalcom-core/src/modules/compile.rs`, `phalcom-core/Cargo.toml`, `phalcom-core/tests/semantic_analysis.rs`.

**Landed behavior:** `ProgramAnalyzer` returns `AnalyzedProgram` even when semantic diagnostics contain errors. `ProgramCompiler::compile_analyzed` converts snapshot diagnostics through `ProgramSemanticDiagnostics::from_snapshot` and rejects invalid analysis before code generation.

- [x] Register `semantic_analysis` explicitly because `phalcom-core` uses `autotests = false`.
- [x] Test invalid analysis preservation, source preservation, valid compilation, and invalid compilation rejection.
- [ ] Re-run `cargo test -p phalcom-core --test semantic_analysis -- --nocapture`.
- [ ] Re-run `cargo test -p phalcom-core --test modules_compile -- --nocapture`.

### Task 4A — Complete Source-Owned Diagnostic Infrastructure

**Status:** Complete.

**Files:** `phalcom-semantic/src/diagnostic.rs`, `phalcom-semantic/src/checker/`, `phalcom-semantic/tests/diagnostic_ownership.rs`, related compiler/LSP semantic construction sites.

- [x] Audit every production `SemanticDiagnostic` constructor under `phalcom-semantic`, `phalcom-core`, and `phalcom-lsp`.
- [x] Replace implicit `ModuleId::core()` ownership with `error_in(module, ...)`, `warning_in(module, ...)`, and equivalent explicit-source constructors.
- [x] Thread the owning `ModuleId` through checker context and relation-policy helpers.
- [x] Verify primary and secondary source ownership across two non-core modules in `identity_diagnostic_foundation`.
- [x] Confirm `rg -n "SemanticDiagnostic::(error|warning)\\(" --glob '*.rs' .` returns no implicit production/test constructor.
- [x] Run `cargo test -p phalcom-semantic --test identity_diagnostic_foundation -- --nocapture` and `cargo test -p phalcom-semantic --test spec04_5_causal_suppression -- --nocapture`.

### Task 4B — Preserve Diagnostic Information in CLI and LSP Consumers

**Status:** Complete for current compiler/LSP consumers.

**Files:** `phalcom-semantic/src/diagnostic.rs`, `phalcom-core/bin/phalcom/cli.rs`, `phalcom-core/src/diagnostics/`, `phalcom-lsp/src/diagnostics.rs`, diagnostic integration tests.

- [x] Make `phalcom check` text output use the canonical semantic diagnostic renderer for primary and same-source secondary labels, with foreign labels, notes, and helps retained as related text.
- [x] Make JSON output a serde-backed representation containing code, severity, source, primary span, every secondary label with its own source module/path and span, notes, helps, fixes, explanations, and root cause.
- [x] Resolve each LSP label to its own canonical module URI and published source line index; do not map known foreign labels to the primary URI.
- [x] Preserve canonical code, severity, source, message, related information, and labels in semantic-to-LSP conversion.
- [x] Add and pass an LSP conversion test with primary and secondary labels in different modules; manually validate CLI text and JSON output.
- [x] Run `cargo check -p phalcom-core`, `cargo check -p phalcom-lsp`, and `cargo test -p phalcom-lsp --lib diagnostics::tests -- --nocapture`.
- [x] Run the manual checks:
  - `cargo run -p phalcom-core --bin phalcom -- check --source 'const count: String = 1'`
  - `cargo run -p phalcom-core --bin phalcom -- check --format json --source 'const count: String = 1'`

### Task 5 — Suppress Inlay Hints at Explicit Annotations

**Status:** Complete.

**Files:** `phalcom-lsp/src/inlay_hints.rs`, AST/source helpers, `phalcom-lsp/tests/stage6_inlay_hints.rs`.

- [x] Detect explicit annotations on bindings, fields, parameters, and method returns from the parsed AST/source model.
- [x] Suppress inferred formal and advisory type hints when the source already states the type.
- [x] Preserve hints for unannotated declarations.
- [x] Add binding and member-level cases covering annotated and unannotated equivalents.
- [x] Keep suppression source-owned; advisory shape is never used to decide whether an explicit annotation exists.
- [x] Run the registered LSP integration target with the `stage6_inlay_hints` filter.

### Task 6 — Demand-Driven Core Analysis Depth

**Status:** In progress; startup `SurfaceOnly` policy is implemented, while explicit deep-demand scheduling remains.

**Files:** `phalcom-lsp/src/semantic/engine.rs`, `phalcom-lsp/src/semantic/core_source.rs`, `phalcom-lsp/src/analysis_service.rs`, `phalcom-lsp/src/workspace_scan.rs`, `phalcom-lsp/tests/core_startup.rs`.

- [x] Introduce an explicit `SourceAnalysisDepth` policy with `SurfaceOnly` and `Deep` states at the compiler/LSP scheduling seam.
- [x] Load universe declarations, classes, members, native surfaces, and source provenance at startup without eagerly querying every universe `CallableBody`.
- [x] Permit deep analysis for an explicitly queued open/edit core replacement; formal-query dependency and explicit background deep mode remain to be wired.
- [x] Record callable-body analysis counters and assert startup performs zero eager universe-body solves outside explicitly required dependencies.
- [x] Test startup readiness and explicit open/edit deep analysis; required-query demand and explicit background deep mode remain pending.
- [x] Run `cargo test -p phalcom-lsp --test core_startup -- --nocapture`.

### Task 10 — Canonical Module Query Facade

**Status:** In progress; read-only product facade exists, while compiler `SemanticDb` session wiring remains pending. Do not create an independent `ModuleWorkspaceSession`.

**Files:** `phalcom-modules/src/query.rs`, `phalcom-semantic/src/session.rs` or the existing semantic workspace session seam, `phalcom-lsp/src/import_completion.rs`, module/query tests.

**Ownership rule:** `phalcom-modules` provides canonical algorithms and query types. `SemanticDb` owns when parsed modules, interfaces, project resolution, and linked products are computed, cached, revised, and invalidated. The LSP consumes those products.

- [x] Expose query APIs for `import_roots(importer)`, `import_children(importer, prefix)`, `public_exports(module)`, `resolved_import_target(importer, path)`, `definition_source(target)`, and `reverse_importers(module)`.
- [ ] Back every query through the compiler-owned `SemanticDb` session's `ParsedModule`, `UnlinkedInterface`, `LinkedInterface`, project-resolution, or provenance products.
- [ ] Reuse the workspace epoch revision and source fingerprints already owned by `SemanticDb`.
- [ ] Do not add a second revision counter, source-fingerprint map, module invalidation graph, or project lifecycle owner.
- [x] Test canonical roots, exposure filtering, linked exports, precomputed import targets, reverse importers, and source provenance.
- [ ] Add root/relative/absolute resolver integration and run full module/session query suites.

### Task 11 — Module and Import Completion

**Status:** Pending.

**Files:** `phalcom-lsp/src/import_completion.rs`, `phalcom-lsp/src/completion.rs`, canonical module query facade, completion tests.

- [ ] Complete import-root candidates from canonical project products.
- [ ] Complete child modules for absolute and relative paths.
- [ ] Complete selective exports only when exposure and visibility rules permit them.
- [ ] Guarantee every emitted candidate resolves through the canonical resolver/query facade.
- [ ] Preserve completion behavior for incomplete import paths and partially typed selective imports.
- [ ] Add tests for project roots, relative imports, absolute imports, selective exports, rejected private exports, and missing prefixes.
- [ ] Run the registered LSP integration completion tests with `--test-threads=2`.

### Task 12 — Module Completion Diagnostics and Query Reuse

**Status:** Pending.

**Files:** canonical module query facade, `phalcom-lsp/src/completion.rs`, `phalcom-lsp/src/diagnostics.rs`, module/LSP integration tests.

- [ ] Ensure completion and diagnostics consume the same linked module/export products.
- [ ] Ensure completion requests are read-only: no semantic mutation, filesystem scan, project rebuild, or independent resolver construction.
- [ ] Add a regression test that performs completion after an edit and asserts no unrelated module product rebuild.
- [ ] Add a regression test that fixes a broken import and clears its module diagnostic incrementally.
- [ ] Run focused module completion and diagnostic tests.

### Task 14 — Physical and Virtual Core Source Navigation

**Status:** Pending; partial core URI identity support exists, but source-text serving and extension registration are absent.

**Files:** `phalcom-lsp/src/virtual_source.rs`, `phalcom-lsp/src/backend.rs`, `phalcom-lsp/src/semantic/ids.rs`, `phalcom-lsp/src/semantic/core_source.rs`, `tools/vsphalcom/src/extension.ts`, source-navigation tests.

- [ ] Preserve canonical provenance from `DeclarationId`, `CallableId`, `ModuleId`, or native-surface identity to a physical `file://` or virtual `phalcom://` source.
- [ ] Implement `phalcom/sourceText` for universe and standard-library virtual documents.
- [ ] Register `vscode.workspace.registerTextDocumentContentProvider("phalcom", ...)` in the extension.
- [ ] Ensure definition locations point to declaration spans, not merely owner-module start positions.
- [ ] Keep native provenance available for a future Go to Implementation path without confusing it with Phalcom source definition.
- [ ] Test core class, method, inherited member, and native-surface navigation.
- [ ] Run LSP source-navigation tests and extension compilation/package checks.

---

## Track B — Formal Semantics and Persistent DB

### Task 0 — Verify Completed Semantic Platform

**Status:** Verification gate; flow substrate verified, production ownership not yet proven.

**Files:** `phalcom-semantic/src/checker/flow/`, `phalcom-semantic/src/checker/statement.rs`, `phalcom-semantic/src/checker/analysis.rs`, `phalcom-semantic/src/db/`, focused semantic tests.

- [x] Confirm formal checker flow remains in `phalcom-semantic/src/checker/flow/`.
- [x] Confirm protocol-only iteration through `iteratorValue(_)`/`iterate(_)` remains the formal iteration rule.
- [x] Confirm `CallableAnalysis` fields include callable identity, expression/binding products, flow graph, entry/exits, diagnostics, explanations, dependencies, dependency fingerprint, and status.
- [x] Confirm DB substrate contains query keys, state, dependency index, reverse closure, stale publication rejection, cancellation, budget outcomes, and metrics.
- [ ] Record the exact DB/session/query APIs consumed by downstream tasks in tests and module documentation.
- [ ] Keep LSP advisory flow explicitly separate from formal checker flow in ownership documentation and parity assertions.
- [x] `cargo test -p phalcom-semantic --test db -- --nocapture` passes 5/5.
- [x] `cargo test -p phalcom-semantic --test spec04_5_flow_graph -- --nocapture` passes 7/7.

### Task 7 — Complete CallableAnalysis DB Publication and Reuse

**Status:** In progress; typed DB-owned callable publication and cache hits are implemented, while workspace integration and full dependency-fingerprint invalidation remain.

**Files:** `phalcom-semantic/src/checker/analysis.rs`, `phalcom-semantic/src/db/query.rs`, `phalcom-semantic/src/db/product.rs`, `phalcom-semantic/src/db/`, `phalcom-semantic/src/workspace.rs`, `phalcom-semantic/src/snapshot.rs`, callable-analysis tests.

- [x] Preserve the existing single `CallableAnalysis` representation; do not introduce a second body-analysis structure.
- [x] Make the callable-body query return the cached typed `Arc<CallableAnalysis>` product when its dependency fingerprint is reusable in the current DB revision.
- [x] Replace placeholder empty-byte `QueryValue` publication and zero fingerprints with typed product storage plus a product discriminator envelope.
- [x] Store expression analyses, bindings, flow graph, entry/exit facts, diagnostics, explanations, dependencies, fingerprint, and non-ready status in the typed callable product.
- [x] Ensure cancellation, budget exhaustion, and blocked outcomes remain explicit DB states and do not publish as `Ready`.
- [x] Add tests for first computation, cache hit, changed-body recomputation, unchanged-body reuse, and isolated product invalidation.
- [ ] Add dependency-edge invalidation coverage.
- [x] Retain last-known-good typed callable products when a later refresh is cancelled, budget-exhausted, or blocked.
- [ ] Run `cargo test -p phalcom-semantic --test callable_analysis -- --nocapture` and the DB tests.

### Task 8 — Formal Semantic Presentation Projection

**Status:** In progress; pure presenter and callable/expression projection exist, while broader product coverage and revision-stability tests remain.

**Files:** `phalcom-semantic/src/presentation.rs`, `phalcom-semantic/src/types/`, `phalcom-semantic/src/snapshot.rs`, `phalcom-semantic/tests/presentation.rs`.

- [x] Define a pure `TypePresenter` over formal type products covering canonical type spelling plus dynamic and unknown knowledge states.
- [x] Define immutable type-site/projection records keyed by canonical source identity and semantic callable/expression identity.
- [x] Keep projection input limited to compiler-owned products; no inference or second cache/invalidation owner exists.
- [x] Preserve invalid, blocked, cancelled, budget, partial, dynamic, and unknown formal states instead of strengthening them with advisory text.
- [x] Add tests for canonical formatting, formal status formatting, source ranges, and projection site identity.
- [x] Add generic-substitution and stable output across unchanged revisions coverage.
- [x] Run `cargo test -p phalcom-semantic --test presentation -- --nocapture`.

### Task 9 — Formal-First Hover and Inlays

**Status:** In progress; formal-state consumption and advisory separation are implemented, while full status fixtures, last-known-good coverage, and formal signature help remain.

**Files:** `phalcom-lsp/src/semantic/snapshot.rs`, `phalcom-lsp/src/hover.rs`, `phalcom-lsp/src/inlay_hints.rs`, presentation adapter code, formal presentation tests.

- [x] Consume formal type knowledge and analysis status from immutable compiler products.
- [x] Render formal `Known(T)` as formal `T`.
- [x] Render formal `Unknown`, `Dynamic`, `Invalid`, `Blocked`, cancelled, budget-exceeded, and internal-failure results without advisory strengthening in the presentation adapter.
- [x] Permit advisory evidence only in a distinct `Observed ...: ≈ T` section or `≈ T` inlay label.
- [x] Keep explicit source annotations authoritative and suppress duplicate inferred hints.
- [x] Keep hover and inlay handlers read-only; no new semantic mutation, filesystem access, or workspace rebuild was added.
- [ ] Add full LSP fixtures for each formal status, advisory-only behavior, and last-known-good snapshot retention.
- [x] Run focused formal presentation, hover, inlay, and callable-publication tests.

### Task 9b — Formal Signature Help

**Status:** In progress; read-only LSP provider, syntax recovery, formal/advisory presentation, and capability advertisement are implemented; broad callable-category fixtures and extension wiring remain.

**Files:** `phalcom-lsp/src/signature_help.rs`, `phalcom-lsp/src/backend.rs`, formal presentation/query adapters, `tools/vsphalcom/src/extension.ts`, signature-help tests.

- [x] Add a read-only LSP signature-help provider backed by published callable surfaces and compiler formal signatures.
- [x] Recover receiver calls, unqualified calls, keyword labels, active parameters, and incomplete argument lists without mutating semantic state.
- [x] Preserve formal parameter/return states; advisory parameter/return shapes render with `≈` only when formal products are unavailable.
- [ ] Add golden expectations and integration tests for inherited, generic, method-local-generic, cross-project, and native callable categories.
- [ ] Wire extension-side signature-help settings and run extension compilation checks.
- [x] Run signature-help recovery/unit tests and capability initialization coverage.

### Tasks 16–18 — Finish and Verify Incremental Formal Ownership

**Status:** Partial; substrate exists, production lifecycle and typed products are incomplete.

**Files:** `phalcom-semantic/src/db/`, `phalcom-semantic/src/session.rs`, `phalcom-semantic/src/workspace.rs`, `phalcom-semantic/src/types/store.rs`, `phalcom-semantic/src/snapshot.rs`, semantic incremental tests.

- [ ] Make one compiler-owned semantic workspace session hold the active `SemanticDb`, revision, stable `TypeStoreId`, source overlay/revision state, and published snapshots.
- [ ] Make parsed modules DB products.
- [ ] Make unlinked interfaces DB products.
- [ ] Make linked interfaces DB products.
- [ ] Make declaration surfaces DB products.
- [ ] Make `CallableAnalysis` a typed DB product and return it from cache hits.
- [ ] Keep `TypeStoreId` stable for one semantic workspace epoch and prove it across source revisions.
- [ ] Replace `analyze_workspace` with a DB-backed cold wrapper when it remains part of the public compiler API.
- [ ] Ensure body-only edits do not rebuild unrelated declaration, interface, project, or callable products.
- [ ] Ensure signature edits invalidate the exact reverse semantic closure.
- [ ] Ensure cancelled generations cannot replace a published `Ready` snapshot.
- [ ] Add structural tests for product fingerprints, cache hits, reverse closure, type-store identity, and last-known-good retention.
- [ ] Run `RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic -- --nocapture`.

### Task 19 — Integrate Compiler SemanticDb and Delete Legacy Formal LSP Path

**Status:** Pending; current production path still constructs fresh project/module/formal state.

**Files:** `phalcom-lsp/src/analysis_service.rs`, `phalcom-lsp/src/semantic/mod.rs`, `phalcom-lsp/src/semantic/snapshot.rs`, `phalcom-lsp/src/semantic/module_graph.rs`, compiler semantic session APIs, `phalcom-lsp/tests/formal_incremental.rs`.

- [ ] Route LSP source overlays and workspace changes into the compiler-owned semantic session.
- [ ] Publish compiler products as immutable LSP snapshots and preserve the last-known-good formal snapshot after failed updates.
- [ ] Read `CallableAnalysis`, declaration surfaces, linked interfaces, formal diagnostics, and provenance from compiler products.
- [ ] Remove production refresh dependence on `run_static_workspace_analysis`, fresh `ProjectUniverse`, fresh `ModuleResolver`, fresh `ModuleLinker`, and direct whole-workspace `analyze_workspace` calls.
- [ ] Keep any LSP `SemanticDb` name confined to the published-snapshot adapter, renamed or documented so it cannot be confused with formal `phalcom_semantic::db::SemanticDb`.
- [ ] Restrict old LSP semantic engine paths to explicitly advisory behavior or delete them after parity coverage passes.
- [ ] Assert compiler diagnostics take precedence over advisory LSP diagnostics.
- [ ] Assert formal non-ready states cannot be upgraded by LSP heuristics.
- [ ] Add tests for source overlay update, body-only reuse, signature invalidation, cancellation, failed update retention, and immutable query behavior.
- [ ] Run `cargo test -p phalcom-lsp --test formal_incremental -- --nocapture`.

---

## Track C — Full Integration, Parity, and Verification

### Task 13 — Module Occurrences and Navigation

**Status:** Pending; current `SemanticTarget` has no `Module(ModuleId)` variant and drops module resolution occurrences.

**Files:** `phalcom-lsp/src/semantic/occurrence.rs`, `phalcom-lsp/src/backend.rs`, canonical module provenance/query code, `phalcom-lsp/tests/module_navigation.rs`.

- [ ] Add `SemanticTarget::Module(ModuleId)` and preserve `NameResolution::Module` occurrences.
- [ ] Index preamble imports, relative/absolute module segments, and selective imports.
- [ ] Resolve definition/reference targets through canonical `ModuleId`, `DeclarationId`, and `CallableId` products.
- [ ] Return physical or virtual source locations from canonical provenance.
- [ ] Test module definition, references, selective import navigation, cross-project navigation, and unresolved-import recovery.
- [ ] Run `cargo test -p phalcom-lsp --test module_navigation -- --nocapture`.

### Task 15 — Legacy Deletion Gate

**Status:** Pending; current production search still finds all prohibited formal ownership paths.

Before marking complete, search production code for:

- [ ] `run_static_workspace_analysis`.
- [ ] Fresh `ProjectUniverse` construction from the LSP edit path.
- [ ] Fresh `ModuleResolver`/`ModuleLinker` construction from the LSP edit path.
- [ ] Direct whole-workspace semantic checker invocation from LSP refresh.
- [ ] URI-derived logical import resolution.
- [ ] LSP-owned formal flow or formal type inference.
- [ ] A second formal incremental cache or type store.

Expected end state:

- [ ] URI translation only maps editor documents to canonical source identity.
- [ ] Module meaning comes from `phalcom-modules` products.
- [ ] Formal meaning comes from compiler `SemanticDb` products.
- [ ] Remaining LSP `ValueShape` code is explicitly advisory.
- [ ] Tests fail if a forbidden fresh formal/module lifecycle is reintroduced.

### Task 20 — Project-Aware Startup and Progressive Readiness

**Status:** Pending; scanner, budgets, and local/workspace modes exist, but startup must use the compiler session and project manifest directly.

**Files:** `phalcom-lsp/src/analysis_service.rs`, `phalcom-lsp/src/workspace_scan.rs`, project/session query APIs, `phalcom-lsp/tests/project_startup.rs`.

- [ ] Ingest root `project.toml` before broad file discovery.
- [ ] Derive project source roots and dependency roots through canonical project products.
- [ ] Prioritize open documents and their required dependency closure.
- [ ] Bound background scanning with existing scan budgets and counters.
- [ ] Publish basic editor-query readiness before full background analysis completes.
- [ ] Assert startup does not construct a second project session or perform query-path disk I/O.
- [ ] Test manifest startup, dependency roots, open-document priority, bounded scanning, and progressive readiness.
- [ ] Run `cargo test -p phalcom-lsp --test project_startup -- --nocapture`.

### Task 21 — Canonical Module Diagnostics

**Status:** Pending; current resolver/load/link errors can be silently continued and canonical module diagnostic codes are absent.

**Files:** `phalcom-modules/src/`, `phalcom-semantic/src/diagnostic.rs`, `phalcom-semantic/src/db/`, `phalcom-lsp/src/diagnostics.rs`, `phalcom-lsp/tests/module_diagnostics.rs`.

- [ ] Produce structured compiler/module facts for unresolved imports, rejected exposure, missing exports, invalid relative roots, and link failures.
- [ ] Assign stable diagnostic codes such as `module.import.unresolved` and `module.exposure.rejected` at the canonical producer.
- [ ] Preserve source ownership and labels for every module diagnostic.
- [ ] Convert canonical diagnostics to the LSP Problems panel without recomputation or information loss.
- [ ] Clear diagnostics incrementally when the import or exposure is fixed.
- [ ] Test unresolved, private, missing-export, invalid-root, cross-project, and repaired-import cases.
- [ ] Run `cargo test -p phalcom-lsp --test module_diagnostics -- --nocapture`.

### Task 22 — VS Code Extension Finishing

**Status:** Pending; Rust analysis notifications exist, but client status/log/source handling is incomplete.

**Files:** `tools/vsphalcom/src/analysisStatus.ts`, `tools/vsphalcom/src/extension.ts`, `tools/vsphalcom/package.json`.

- [ ] Consume monotonic `phalcom/analysisStatus` events and prevent stale status regressions.
- [ ] Stream `phalcom/analysisLog` events into a dedicated output channel with level filtering.
- [ ] Register the `phalcom` virtual-document content provider.
- [ ] Add `phalcom.analysis.logLevel` configuration and reload behavior.
- [ ] Keep server path/restart behavior explicit for local rebuilt servers.
- [ ] Add extension tests for status ordering, log filtering, provider registration, configuration reload, and restart.
- [ ] Run `cd tools/vsphalcom && npm test && npm run compile && npm run package`.
- [ ] Manually verify server path, restart command, output panel, status transitions, virtual core documents, and source navigation.

### Task 23 — Compiler/LSP Canonical Formal Parity

**Status:** Pending; current `ShadowParityHarness` is a no-op recorder.

**Files:** `phalcom-lsp/src/parity.rs`, `phalcom-core/tests/compiler_lsp_parity.rs`, `phalcom-lsp/tests/compiler_parity.rs`, shared test fixtures.

Parity means agreement on canonical formal facts, not byte-identical UI text:

- [ ] Same `ModuleId` and source ownership.
- [ ] Same declaration and callable targets.
- [ ] Same formal `TypeKnowledge` and explicit analysis status.
- [ ] Same callable target and visibility.
- [ ] Same linked exports and exposure decisions.
- [ ] Same diagnostic code, severity, and source labels.
- [ ] Advisory LSP shape/inference remains outside parity unless displayed in a separately labeled section.

- [ ] Replace no-op parity recording with assertions over compiler products and LSP adapters.
- [ ] Add parity fixtures for imports, visibility, generic types, flow narrowing, dynamic/unknown states, callable targets, and diagnostics.
- [ ] Run `cargo test -p phalcom-core --test compiler_lsp_parity -- --nocapture`.
- [ ] Run `cargo test -p phalcom-lsp --test compiler_parity -- --nocapture`.

### Task 24 — Structural Performance Gates and IDE-Golden Acceptance

**Status:** Pending; counters and expectation files exist, but structural assertions and an automated golden runner are incomplete.

**Files:** `phalcom-semantic/tests/performance_structure.rs`, `phalcom-modules/tests/performance_structure.rs`, `phalcom-lsp/tests/performance.rs`, `phalcom-lsp/tests/ide_golden.rs` or an equivalent registered target, `examples/ide-golden/`.

Structural CI assertions:

- [ ] Body-only edit causes zero `ProjectUniverse` rebuilds.
- [ ] Body-only edit causes zero universe bootstrap repeats.
- [ ] Body-only edit causes zero unrelated `CallableBody` recomputes.
- [ ] Stale/cancelled generations never publish over a newer `Ready` snapshot.
- [ ] Startup performs approximately zero eager universe `CallableBody` queries beyond explicit dependencies.
- [ ] Query-path hover/completion performs zero filesystem access and semantic mutation.
- [ ] Type-store identity remains stable within one semantic workspace epoch.
- [ ] Product fingerprints prove exact reverse-closure invalidation.

Golden IDE coverage:

- [ ] Add automated execution for `examples/ide-golden/EXPECTATIONS.md` and expectation TOML files.
- [ ] Cover generic classes and method-local generics.
- [ ] Cover bidirectional and expected-result inference.
- [ ] Cover formal flow narrowing, branch joins, mutation invalidation, and protocol-only iteration.
- [ ] Cover formal `Unknown`, `Dynamic`, `Invalid`, and reproducible `Blocked` states.
- [ ] Cover formal/advisory separation and explicit annotation suppression.
- [ ] Cover signature help, module completion, navigation, references, virtual source text, and compiler/LSP parity.
- [ ] Cover exact DB invalidation behavior and last-known-good publication.

Performance SLOs recorded for reference machines, not hard correctness thresholds:

- [ ] Cold startup target: below one second.
- [ ] Body-only update target: below 100 ms.

- [ ] Run `cargo test -p phalcom-lsp --test performance -- --nocapture` after removing or replacing the current ignored-only harness.
- [ ] Run the registered golden suite and report every expectation by category.

---

## Verification Matrix

### Focused baseline checks

```bash
cargo test -p phalcom-core --test semantic_analysis -- --nocapture
cargo test -p phalcom-core --test modules_compile -- --nocapture
cargo test -p phalcom-semantic --test db -- --nocapture
cargo test -p phalcom-semantic --test spec04_5_flow_graph -- --nocapture
cargo test -p phalcom-lsp --test analysis_status -- --nocapture
cargo test -p phalcom-lsp --test analysis_logging -- --nocapture
```

The two LSP focused suites are separate Cargo integration targets. A single Cargo invocation cannot name both target names.

### Broad Rust checks after focused tests pass

```bash
cargo run -p phalcom-native-surface-gen -- --root . --check
cargo test -p phalcom-native-surface
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-core
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test integration -- --test-threads=2
cargo test -p phalcom-lsp
```

### Extension checks

```bash
cd tools/vsphalcom
npm test
npm run compile
npm run package
```

### Manual compiler checks

```bash
cargo run -p phalcom-core --bin phalcom -- check --source 'const count: String = 1'
cargo run -p phalcom-core --bin phalcom -- check --format json --source 'const count: String = 1'
```

### LSP manual validation

- Rebuild the server.
- Confirm `phalcom.lsp.serverPath` points at rebuilt binary.
- Run `Phalcom: Restart Language Server`.
- Inspect status transitions and analysis-log output.
- Open a core/builtin definition and confirm virtual or physical source content loads.
- Check hover, inlays, completion, signature help, diagnostics, and navigation against the golden fixture.

## Completion and Reporting Rules

Do not claim the integration complete from focused tests alone. Final reporting must separate:

- passing focused and broad checks;
- baseline or unrelated failures;
- deferred reference-machine SLO measurements;
- unverified manual extension behavior;
- remaining implementation work.

Each task should land as a cohesive, reviewable change. Run `git diff --check`, format only owned Rust files, and inspect `git status --short` before staging. Keep implementation commits separate from documentation or workspace-state changes. Run `graphify update .` after source-code modifications so repository relationships stay current.

## Final Acceptance

The plan is complete only when all of the following hold:

- [ ] One compiler-owned `SemanticDb` session serves formal products to compiler and LSP.
- [ ] No legacy LSP formal/module lifecycle remains in production edit refreshes.
- [ ] Callable body products are typed, cached, fingerprinted, status-aware, and incrementally invalidated.
- [ ] Formal `Unknown`, `Dynamic`, `Invalid`, and `Blocked` states are never advisory-upgraded.
- [ ] Compiler and LSP agree on canonical formal facts and diagnostic ownership.
- [ ] Module completion, diagnostics, occurrences, and navigation use canonical module products.
- [ ] Core source definitions are physically or virtually readable in the editor.
- [ ] Status and structured logs remain truthful and monotonic.
- [ ] Structural performance gates and automated IDE-golden expectations pass.
- [ ] Broad Rust, extension, CLI, and manual validation results are reported with scope separation.
