# Phalcom LSP Async Performance Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use - [ ] checkboxes for tracking.

**Goal:** Complete Tasks 8–13 of the asynchronous Phalcom LSP performance specification so editor request latency stays independent from background semantic convergence while preserving Specs 1–4 behavior.

**Architecture:** Keep one dedicated worker as the only owner of mutable deep semantic state. Reuse immutable source products and one unified flow result, narrow invalidation through forward/reverse module and callable dependencies, publish complete immutable generations, and answer requests from live source plus the newest compatible snapshot without waiting or reading closed files from disk. Execute disjoint implementation slices in isolated worktrees, then serialize shared engine.rs/backend.rs integration.

**Tech Stack:** Rust, Cargo, Tokio synchronization, tower-lsp, immutable Arc snapshots, Phalcom AST, TypeScript, vscode-languageclient 8.x, existing VS Code Extension Host tests.

---

## Current checkpoint and constraints

- Starting commit: 195ef13 (wip: checkpoint async LSP implementation); tree was clean after commit.
- Authority: patchwork/PHALCOM_LSP_ASYNC_PERFORMANCE_IMPLEMENTATION_SPEC.md, especially Sections 2, 8–29, Tasks 8–13, and Section 36.
- Handoff: /Users/altunhasanli/.codex/attachments/16a6e792-4528-4d8a-95f6-5d1dac109c65/pasted-text.txt.
- Existing scaffolding: async worker/snapshot publication, progressive scan, FileSourceSnapshot, reverse module graph, SourceChangeKind, callable worklist, parameter-contribution storage, closed-source metadata cache, performance counters, and a restart helper.
- Known unresolved evidence: cross_file_hover_resolves_the_doc_from_the_declaring_file was null after the prior scheduling patch. Treat this as unknown until reproduced at the checkpoint; do not assume it is fixed or caused by the checkpoint.
- Preserve: no analysis.flush() in request handlers, no synchronous closed-file reads in hover/definition, latest-wins epoch cancellation, open-document priority, immutable publication, one-writer ownership, and unrelated committed artifacts already included in 195ef13.
- Non-goals: incremental parsing, persistent semantic cache, new type system, compiler IR, VM changes, distributed analysis, and speculative parallel semantic solving.

## Parallel execution and review protocol

1. Before implementation, create one codex/ branch/worktree per write-owning agent from 195ef13. Never let two agents edit the same file in parallel. Keep the main checkout clean and integrate commits intentionally.
2. Wave 0 is a short baseline worker: reproduce only the focused hover test and the narrow semantic tests needed to establish current behavior. Do not run workspace-wide gates yet.
3. Wave 1 may run these disjoint workstreams in parallel. Ownership is exact; workers must not edit files outside their row:
   - Flow-product worker: `phalcom-lsp/src/semantic/flow.rs` only, including its `cfg(test)` module.
   - Module-invalidation worker: `phalcom-lsp/src/semantic/module_graph.rs` and `phalcom-lsp/src/semantic/invalidation.rs` only, including their `cfg(test)` modules.
   - Query-index worker: `phalcom-lsp/src/semantic/occurrence.rs` and `phalcom-lsp/src/semantic/scope.rs` only, including their `cfg(test)` modules. `completion.rs` is a later serialized slice if profiling justifies it.
   - Extension worker: `tools/vsphalcom/src/extension.ts` and `tools/vsphalcom/src/test/suite/extension.test.ts` only.
   - No Wave 1 worker edits `engine.rs`, `backend.rs`, `analysis_service.rs`, `infer.rs`, `facts.rs`, `semantic/mod.rs`, shared integration tests, or test support.
4. After Wave 1, integrate shared seams serially:
   - flow/infer/engine unified-pass integration;
   - module graph/engine/backend/worker batch integration;
   - callable worklist and contribution solver;
   - backend closed-source cache and hover/definition integration.
   The controller owns these serial seam files during integration: `phalcom-lsp/src/semantic/engine.rs`, `phalcom-lsp/src/semantic/mod.rs`, `phalcom-lsp/src/analysis_service.rs`, `phalcom-lsp/src/backend.rs`, `phalcom-lsp/src/documents.rs`, and shared integration-test support.
5. After every implementation task or integrated slice, dispatch two fresh reviewers in order: spec-compliance review, then code-quality review using requesting-code-review. Fix Critical and Important findings and re-review before advancing. A worker self-review does not replace either review.
6. Use targeted commands with CARGO_TARGET_DIR=target. Run formatting only after a cohesive Rust slice changes. Defer broad crate/workspace and extension gates to Task 13.

## File ownership map

- phalcom-lsp/src/semantic/flow.rs: one source-backed structured flow traversal and all SurfaceFlowAnalysis products.
- phalcom-lsp/src/semantic/infer.rs: solver context construction and callable-granular convergence; no duplicate extraction wrappers.
- phalcom-lsp/src/semantic/engine.rs: worker-owned mutation, invalidation frontier, contribution replacement, and snapshot construction.
- phalcom-lsp/src/semantic/module_graph.rs and invalidation.rs: forward/reverse module edges and change classification.
- phalcom-lsp/src/semantic/callable.rs and facts.rs: callable work queue, summaries, and contribution provenance.
- phalcom-lsp/src/semantic/occurrence.rs and scope.rs: bounded source-query indexes.
- phalcom-lsp/src/analysis_service.rs: scheduling, batch coalescing, worker test gates, and closed-source scan cache.
- phalcom-lsp/src/backend.rs: LSP lifecycle, event publication, request-path cache reads, and batched filesystem events.
- phalcom-lsp/src/perf.rs: low-overhead counters/spans with deterministic test access.
- phalcom-lsp/src/semantic/mod.rs and phalcom-lsp/src/documents.rs: serialized integration-controller ownership for the public query facade, snapshot pinning, and document snapshot extraction; never edit these from Wave 1.
- phalcom-lsp/tests and Rust cfg(test) modules: behavior and deterministic concurrency tests.
- tools/vsphalcom/src/extension.ts plus tools/vsphalcom/src/test: restart orchestration and Extension Host coverage.

### Task 0: Establish focused baseline and worktree handoff

**Files:**
- Read only: patchwork/PHALCOM_LSP_ASYNC_PERFORMANCE_IMPLEMENTATION_SPEC.md
- Read only: phalcom-lsp/src/analysis_service.rs, backend.rs, and targeted semantic modules
- Test: existing phalcom-lsp/tests/stage4_hover.rs and semantic unit tests

- [ ] Step 1: Verify clean starting state and record checkpoint.

Run:

    git status --short
    git rev-parse HEAD

Expected: no status output and 195ef13.

- [ ] Step 2: Reproduce only the known cross-file hover case.

Run:

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration cross_file_hover_resolves_the_doc_from_the_declaring_file --quiet

Expected: record PASS or the known null hover failure; do not label either result as full acceptance.

- [ ] Step 3: Capture focused semantic baseline with reproducible exact filters.

Run:

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::three_step_return_forwarding_converges -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::recursive_scc_with_concrete_evidence_converges -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::nine_incompatible_return_shapes_widen_to_unknown -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::leaf_edit_does_not_recompute_unrelated_module -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::provider_edit_recomputes_transitive_consumers -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::provider_creation_repairs_previously_unresolved_import -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::provider_removal_invalidates_existing_importer -- --exact
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::tests::caller_edit_removes_stale_parameter_contribution -- --exact

Record exact pass/fail names and classify failures as checkpoint, baseline, or unknown.

- [ ] Step 4: Create isolated worktrees and assign disjoint ownership.

Use the superpowers:using-git-worktrees workflow. No implementation begins until each worker has exact file ownership from the parallel matrix above.

### Task 8: Reuse source structures and run one unified flow pass

**Files:**
- Modify: phalcom-lsp/src/semantic/flow.rs
- Modify: phalcom-lsp/src/semantic/infer.rs
- Modify: phalcom-lsp/src/semantic/engine.rs
- Test: flow and semantic unit tests near flow.rs and semantic/mod.rs

- [ ] Step 1: Add a regression fixture for one-pass products.

Create a small source with a local binding, field initializer/write, callable return, and parameter call evidence. Assert one SurfaceFlowAnalysis contains local, field, parameter, and summary products. Add a test-visible flow-pass count.

- [ ] Step 2: Prove current duplication before changing it.

Use the counter and targeted test to show which calls currently enter analyze_surface or analyze_callable. The test must fail if the engine extracts parameters, locals, fields, or summaries by independently rerunning the same source traversal.

- [ ] Step 3: Make flow consume existing FileSourceSnapshot products.

Keep Program, ModuleSurface, ScopeGraph, and OccurrenceIndex source-owned. analyze_surface must read the snapshot and must not rebuild scopes or clone AST bodies. Keep callable-specific analysis only as the bounded unit used by the callable worklist; do not reintroduce module-wide wrappers that independently traverse the same body.

- [ ] Step 4: Consume SurfaceFlowAnalysis directly in inference and engine.

Remove selector wrappers whose only purpose is to rerun analyze_surface. Store one result per solver unit and project local/field/parameter/summary facts from it. Permit at most one final stabilized pass when local or field facts change after convergence; make the counter assertion explicit.

- [ ] Step 5: Run focused flow and semantic gates.

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic:: --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage3_completion --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage4_hover --quiet

Expected: targeted tests and flow-pass assertion pass, or failures are reported with baseline classification.

- [ ] Step 6: Commit the cohesive slice.

    git add phalcom-lsp/src/semantic/flow.rs phalcom-lsp/src/semantic/infer.rs phalcom-lsp/src/semantic/engine.rs
    git commit -m "perf(lsp): reuse source graphs and unify flow passes"

### Task 9: Make module/class invalidation incremental and batch mutations

**Files:**
- Modify: phalcom-lsp/src/semantic/module_graph.rs
- Modify: phalcom-lsp/src/semantic/invalidation.rs
- Integrate serially: phalcom-lsp/src/semantic/engine.rs, backend.rs, analysis_service.rs
- Test: module graph, invalidation, watched-file, and workspace-folder tests

- [ ] Step 1: Test forward/reverse edge maintenance.

Add tests for import insertion, replacement, unresolved-provider creation, provider removal, and stale reverse-edge removal. Assert only affected importers are returned by repair operations.

- [ ] Step 2: Test change classification at the semantic boundary.

Assert body-only edits preserve declaration/import fingerprints; import edits produce ImportSurface; class/member/field/signature edits produce DeclarationSurface; add/remove and logical core changes retain their dedicated kinds.

- [ ] Step 3: Narrow graph repair.

Update the reverse index when a module's own import surface changes. For provider add/remove, repair only importers whose retained paths can resolve to that provider. Do not call full refresh_resolutions for body-only edits. Preserve deterministic BTreeMap/BTreeSet behavior and avoid cloning unrelated graph state.

- [ ] Step 4: Integrate engine invalidation.

Use SourceChangeKind to retain class surfaces and module edges for body-only updates, replace only changed declaration surfaces, and extend the invalidation frontier through reverse module dependents. Keep the physical selected core represented by one logical core module.

- [ ] Step 5: Batch watcher, root-removal, and configuration mutations.

Ensure one didChangeWatchedFiles or workspace-root removal call produces one coalesced removal/update transaction. Add a combined worker API if needed; do not enqueue removals and updates as separate epochs. Preserve open-document priority and latest-wins replacement.

- [ ] Step 6: Enforce revision and epoch ordering across every worker mutation.

When a pending URI already has a newer FileRevision, reject older updates, including core updates. Run removals, core replacement, scan publication, and semantic updates through one epoch-aware candidate transaction; stale work must not publish semantic generations, workspace-index entries, closed-source cache events, or source-catalog entries. Evict deleted modules and every canonical URI alias before import-closure expansion. Add cancellation checkpoints between removal items and scan files, and cap one directory-expansion turn so a flat directory cannot monopolize the worker.

- [ ] Step 7: Keep document handlers bounded.

Move semantic recovery parsing that is not required for immediate syntax diagnostics off didOpen/didChange, or prove it is bounded current-document work. Watched-file and close handlers must enqueue disk refresh/removal descriptions rather than synchronously reading/parsing large files on the LSP executor. Publish syntax diagnostics immediately, enqueue the recovered program, and never run deep solving or wait under the LSP handler.

- [ ] Step 8: Run focused invalidation gates.

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::module_graph --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic::invalidation --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic:: --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration workspace_semantics --quiet

Expected: graph and invalidation tests pass; body-only traces do not rebuild unrelated module/class state.

- [ ] Step 9: Commit the cohesive slice.

    git add phalcom-lsp/src/semantic/module_graph.rs phalcom-lsp/src/semantic/invalidation.rs phalcom-lsp/src/semantic/engine.rs phalcom-lsp/src/backend.rs phalcom-lsp/src/analysis_service.rs phalcom-lsp/src/documents.rs phalcom-lsp/src/semantic/mod.rs
    git commit -m "perf(lsp): narrow module graph and class invalidation"

### Task 10: Replace module-wide solving with callable worklists and contribution tracking

**Files:**
- Modify: phalcom-lsp/src/semantic/infer.rs
- Modify: phalcom-lsp/src/semantic/engine.rs
- Modify: phalcom-lsp/src/semantic/callable.rs
- Modify: phalcom-lsp/src/semantic/facts.rs
- Test: semantic dependency/invalidation tests in semantic/mod.rs or a focused test module

This task starts only after Task 8's one-pass contract and Task 9's invalidation frontier are approved.

- [ ] Step 1: Add worklist behavior tests before changing solver code.

Add fixtures asserting: a leaf body edit recomputes that callable only when its summary is unchanged; a changed return summary recomputes exactly true reverse dependents; changed parameter evidence reaches only consumers; recursion converges; bottom/unknown distinction and shape widening remain unchanged.

- [ ] Step 2: Track callable dependency edge diffs.

Maintain direct dependency and reverse-dependent edges per callable. Remove stale edges and insert new edges when a summary changes. Seed the queue from changed callables, affected dependents, and changed contribution slots rather than every callable in each affected module.

- [ ] Step 3: Make parameter evidence contribution-based.

Record source/call-site contributions by callable and parameter, replacing one source contribution without cloning or losing other evidence. Propagate only when the joined fact changes. Apply widening coherently to summaries and parameter facts when the bounded work budget is exhausted.

- [ ] Step 4: Add cooperative cancellation at callable boundaries.

Check shutdown/epoch cancellation before and after each work item. Return no partial candidate for publication. Keep one running stale solve bounded, and let the worker discard it when a newer epoch exists.

- [ ] Step 5: Remove module-wide solving from the normal path.

Retain any full-workspace solver only as a clearly named regression reference if tests need it. Normal engine updates must use the callable worklist and consume the unified analysis result from Task 8.

- [ ] Step 6: Run focused solver gates.

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic:: --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration workspace_semantics --quiet

Expected: leaf/dependent/parameter/recursive fixtures pass and RebuildTrace.callables_recomputed matches the true frontier.

- [ ] Step 7: Commit the cohesive slice.

    git add phalcom-lsp/src/semantic/infer.rs phalcom-lsp/src/semantic/engine.rs phalcom-lsp/src/semantic/callable.rs phalcom-lsp/src/semantic/facts.rs phalcom-lsp/src/semantic/mod.rs
    git commit -m "perf(lsp): add callable-grained incremental semantic solving"

### Task 11: Cache closed source and PhalDoc data and accelerate hot queries

**Files:**
- Modify: phalcom-lsp/src/analysis_service.rs or existing source-cache definitions
- Modify: phalcom-lsp/src/backend.rs
- Modify: phalcom-lsp/src/semantic/occurrence.rs
- Modify: phalcom-lsp/src/semantic/scope.rs
- Modify: phalcom-lsp/src/completion.rs only if profiling proves member lookup material
- Test: hover/navigation/query tests and test-only disk-read guard

- [ ] Step 1: Add a request-path disk-I/O regression guard.

Provide a test-only read hook or cache-only fixture. Assert closed-file hover/definition succeeds from indexed cache and fails if it attempts read_to_string, reparsing, or filesystem canonicalization on the request path. Reproduce and resolve the handoff's cross-file hover fixture.

- [ ] Step 2: Make the cache authoritative for indexed closed files.

Retain Arc<str>, LineIndex, parsed Program, shallow declaration metadata, and PhalDoc/source ranges together with the indexed revision. Update and remove entries atomically with scan, watcher, close, root-removal, and open-document transitions. Do not introduce persistent on-disk caching.

- [ ] Step 3: Remove closed-file reads from hover and navigation.

Route with_source_snapshot, definition locations, PhalDoc harvesting, and fallback hover through cache or live document. Replace query-time path canonicalization with canonical identities recorded during indexing; hover, completion, definition, and inlay paths must not call filesystem canonicalize. A request may return a conservative result during a publication gap, but must not wait or read disk.

- [ ] Step 4: Pin one immutable snapshot per request.

Make hover, completion, inlay, definition, references, and semantic-token handlers clone one Arc<SemanticSnapshot> at entry and use it for the request. Validate the snapshot file revision against the live DocumentSnapshot before using occurrence/binding ranges; return a conservative result when revisions differ. Extract an owned DocumentSnapshot and never hold DashMap guards while invoking semantic code.

- [ ] Step 5: Bound occurrence, scope, and declaration lookup.

Keep occurrence lookup sorted by start with prefix maximum-end pruning and add a test-only candidate-count assertion for a pathological fixture. Keep scope_at interval-bounded and binding_for_declaration direct. Add the smallest immutable-generation completion-member cache only if focused counters show hierarchy walks are material.

- [ ] Step 6: Run focused query gates.

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage4_hover --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage2_index --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage3_completion --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic:: --quiet

Expected: cross-file hover returns declaring-file PhalDoc, closed-file navigation is cache-only, and existing hover/completion behavior stays green.

- [ ] Step 7: Avoid duplicate source products and workspace-wide query clones.

Pass retained indexed source products through publication events instead of reparsing scanned text in the event task. Keep snapshot lookup methods reference-based and avoid cloning all class/summary maps for one receiver query. Remove or gate any debug eprintln path behind PHALCOM_LSP_PERF=1.

- [ ] Step 8: Commit the cohesive slice.

    git add phalcom-lsp/src/analysis_service.rs phalcom-lsp/src/backend.rs phalcom-lsp/src/semantic/occurrence.rs phalcom-lsp/src/semantic/scope.rs phalcom-lsp/src/completion.rs phalcom-lsp/tests/stage4_hover.rs phalcom-lsp/tests/stage2_index.rs phalcom-lsp/tests/stage3_completion.rs
    git commit -m "perf(lsp): cache source metadata and accelerate semantic queries"

### Task 12: Make VS Code restart resilient

**Files:**
- Modify: tools/vsphalcom/src/extension.ts
- Create or modify: a small lifecycle helper only if needed to unit-test stop/start orchestration without starting a real server
- Test: tools/vsphalcom/src/test/suite/extension.test.ts or focused lifecycle test

- [ ] Step 1: Add a stop-failure test.

Use the existing TypeScript setup or a pure injected lifecycle helper to model a client whose stop() rejects. Assert the old client is disposed, the error is logged, and a replacement starts when phalcom.lsp.enabled remains true.

- [ ] Step 2: Centralize restart and stop behavior.

Keep restartLspClient(context) as the public orchestration entry point. Set lspClient to undefined before stopping, dispose on stop failure, and check enabled before starting a replacement. Reuse the safe stop helper for disabling and configuration/server-path restarts; do not duplicate stop/catch/dispose code.

- [ ] Step 3: Serialize lifecycle transitions.

Prevent overlapping command/configuration restarts from starting duplicate clients. Ensure each replaced client is disposed and is not retained indefinitely through repeated context-subscription registration.

- [ ] Step 4: Run focused extension gates.

From tools/vsphalcom:

    npm run lint
    npm run compile
    npm test -- --grep "language-server|restart|lifecycle"

Expected: lint/compile pass and failure-path test proves replacement startup is not suppressed by rejected graceful stop. Run full E2E only in Task 13.

- [ ] Step 5: Commit the cohesive slice.

    git add tools/vsphalcom/src/extension.ts tools/vsphalcom/src/test/suite/extension.test.ts
    git commit -m "fix(vsphalcom): make language server restart resilient"

## Section 36 acceptance-to-test matrix

The implementation workers must maintain this matrix while adding tests. A timing test alone is insufficient for a structural acceptance item.

| Spec acceptance | Required proof | Planned owner/test |
|---|---|---|
| 1–2 | Backend construction has zero core analysis; initialize has zero recursive scan/deep solve before response | `backend.rs` construction counter test; `performance.rs::initialize_returns_while_worker_is_blocked` |
| 3–8 | Worker-only deep mutation, immutable snapshots, no freshness wait, latest-wins, stale publication rejection | `performance.rs` blocked hover/inlay, 100-revision coalescing, stale-generation tests |
| 9–10 | Syntax diagnostics publish independently; inlay rejects stale revision/ranges and refreshes after compatible publication | `stage6_inlay_hints` plus new blocked/revision-mismatch performance test |
| 11–12 | `analysis.mode` works end to end; local is default; progressive scan yields to open work | config/unit test plus `performance.rs` local/workspace scan tests |
| 13–15 | One logical core namespace; presentation-only config does not rebuild core; watched/root batches publish one transaction | backend/core/config tests and watched/root batch tests |
| 16–18 | Reverse module dependencies, body-only narrow invalidation, source products reused | module graph/invalidation tests, flow-pass bound test, semantic trace tests |
| 19–22 | No AST-body cloning; one unified flow result; callable-granular dependency/contribution propagation | dispatch clone test, flow-pass test, callable frontier/contribution tests |
| 23–25 | Closed-file query path is cache-only; restart starts replacement after stop failure | disk-read hook hover/definition test and extension lifecycle failure test |
| 26–27 | Counters/spans report required phases; repository-root manual use remains responsive | `perf.rs`/worker counters, ignored benchmark harness, manual VS Code check |

### Task 13: Add deterministic acceptance harness and complete verification

**Files:**
- Modify: phalcom-lsp/src/perf.rs
- Modify: phalcom-lsp/src/analysis_service.rs
- Modify: phalcom-lsp/src/backend.rs only for missing phase instrumentation/event refresh wiring
- Modify: phalcom-lsp/tests/support/lsp_client.rs and tests/integration.rs if protocol helpers are needed
- Create: phalcom-lsp/tests/performance.rs only if public test APIs make it appropriate
- Docs: benchmark/verification notes only if required to record measurements

- [ ] Step 1: Make counters test-safe and truthful.

Prefer counters owned by an AnalysisService/test fixture rather than resetting one global mutable counter across parallel tests. Retain cheap atomics and one PHALCOM_LSP_PERF=1 output gate. Instrument construction, initialize, source parse/shallow work, workspace discovery/parse, core selection/analyze, semantic batch/solve/flow/publish, and hover/completion/inlay. Do not emit per-node logs or add a tracing dependency.

- [ ] Step 2: Add deterministic worker gates.

Under cfg(test), add a barrier/channel hook before and after a semantic batch plus a test-only idle/join helper. The hook must block worker progress without making production shutdown join the worker.

- [ ] Step 3: Add concurrency regression tests.

Cover the spec cases: initialize while worker is blocked; hover and inlay while a new generation is blocked; revisions 1–100 coalescing to 100; stale generation 10 never publishing over 11; shutdown returning before a blocked worker is released; watched-file batches producing one transaction; presentation-only configuration not rebuilding core; one logical core namespace; and flow-pass count bounded by solver rounds plus the allowed final stabilized pass. Add explicit assertions that `Backend::new` performs no core analysis, `initialize` performs no recursive scan/deep solve before its response, `analysis.mode` defaults to `local` and changes behavior end to end, and inlay hints return no stale-range results when the document revision mismatches.

- [ ] Step 4: Coalesce publication refresh notifications.

Ensure workspace scanning cannot flood the client with duplicate inlay/semantic-token refresh requests. Preserve refresh-after-compatible-publication semantics while batching or deduplicating refresh events.

- [ ] Step 5: Wire truthful phase spans and publication counters.

Add production call sites for `PerfSpan` around construction, initialize, source parse/shallow work, discovery/parse, core selection/analyze, semantic batch/solve/flow/publish, and hover/completion/inlay. Count scan publications and every accepted/discarded mutation consistently. Ensure perf output includes generation/epoch and compact counters without source text or per-node logs.

- [ ] Step 6: Run targeted acceptance gates in dependency order.

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --lib semantic:: --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage4_hover --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration workspace_semantics --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage3_completion --quiet
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration stage6_inlay_hints --quiet
    cargo fmt --all -- --check

Expected: focused semantic/editor gates and formatting pass. If a focused gate fails, stop broad verification and fix/review that slice.

- [ ] Step 7: Run required broad gates once.

    CARGO_TARGET_DIR=target cargo test -p phalcom-ast
    CARGO_TARGET_DIR=target cargo test -p phalcom-native-surface
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp
    CARGO_TARGET_DIR=target cargo test --workspace
    cd tools/vsphalcom
    npm ci
    npm run lint
    npm run compile
    npm test
    npm run test:lsp:e2e

Record each result separately. Do not hide unrelated baseline failures behind a green focused test.

- [ ] Step 8: Run ignored/manual performance checks.

Run debug and release versions of an ignored perf_ harness covering backend construction, initialize response, shallow open-document work, local/workspace convergence, leaf edit, dependent return-shape edit, class-surface edit, 20 rapid edits, and hover while the worker is blocked:

    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --test integration perf_ -- --ignored --nocapture
    CARGO_TARGET_DIR=target cargo test -p phalcom-lsp --release --test integration perf_ -- --ignored --nocapture

Do not encode supplied historical timings as strict thresholds. Record before/after timings, counter snapshots, host/configuration, and limitations.

- [ ] Step 9: Perform only required manual checks.

In a repository-root VS Code workspace, verify rapid typing remains responsive during progressive indexing, hover/definition/inlay return while the worker is deliberately busy, and restart during busy analysis starts a usable replacement client. Capture failures as reproducible evidence.

- [ ] Step 10: Run documentation and hygiene checks.

Run `cargo doc --workspace --no-deps` and treat new rustdoc warnings, stale public documentation, unused public fields, and ungated debug output as defects. Correct documentation such as the empty `SemanticDb::new` contract before final review.

- [ ] Step 11: Final review and checkpoint commit.

Request one final code review over the complete implementation range. Resolve all Critical/Important findings, run git diff --cached --check, and commit only cohesive verification harness/docs if they changed:

    git add phalcom-lsp/src/perf.rs phalcom-lsp/src/analysis_service.rs phalcom-lsp/src/backend.rs phalcom-lsp/tests/performance.rs phalcom-lsp/tests/integration.rs phalcom-lsp/tests/support/lsp_client.rs docs/superpowers/plans/2026-08-13-phalcom-lsp-async-performance.md
    git diff --cached --check
    git commit -m "test(lsp): lock asynchronous performance regressions"

Final report must separate focused passes, broad passes, baseline/unrelated failures, expected-negative results, manual evidence, and unverified work. Completion requires the Section 36 acceptance gate, not merely passing one focused test.
