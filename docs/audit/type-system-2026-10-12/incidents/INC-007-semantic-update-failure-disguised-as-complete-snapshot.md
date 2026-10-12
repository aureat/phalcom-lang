# INC-007 — Semantic update failure can be published as an apparently complete snapshot

**Finding ID:** INC-007  
**Severity:** High / P1; potential P0 if failure is provably user-triggerable and a compiler consumer accepts the fallback as successful semantic authority  
**Confidence:** High for the fallback/consumer control flow; trigger frequency and executable exploitability unverified  
**Status:** Source-confirmed conditional failure-containment defect; forced-failure and end-to-end reproduction pending  
**Audit slice:** 03 — incremental snapshot / failure publication  
**Repository:** `aureat/phalcom-lang`, `main` revision `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac`, verified against `main`  
**Classification:** D — failure containment, E — semantic consistency; possible A/C effects **not** claimed  
**Invariant focus:** T1, T5, T7, T8, T9.

## Ownership and production consumer

- `phalcom-semantic/src/session.rs`: `SemanticWorkspaceSession::update` (923–925), private `update_with_delta` (927–960), and fallible `update_with_budget_and_cancel` (963–969).
- `phalcom-semantic/src/workspace.rs`: `analyze_workspace` (123–127) constructs a **new** session and invokes `session.update(input)`; only the returned snapshot is exposed in `SemanticAnalysis`.
- `phalcom-semantic/src/snapshot.rs`: `SnapshotStatus::{Complete, Partial}` (around 128–145), `new_with_callable_analyses` (295–356) initializes `status: Complete`; `has_errors` (869–873) checks only diagnostics for severity Error.
- `phalcom-core/src/modules/compile.rs`: concrete `analyze_workspace` consumers at lines ~618 and ~726 store its result in `AnalyzedProgram.semantic`; `ProgramCompiler::compile_analyzed` at ~827–831 rejects `has_errors()` but not a missing formal publication.
- **Important unaffected-by-this-exact-fallback route:** `ProgramAnalyzer::discover_and_analyze` at ~797–805 calls `update_with_budget_and_cancel` and maps `Err` into `ProgramCompileError::SemanticUpdate`; `SemanticWorkspaceSession::update_module_workspace_with_cancel` similarly returns `Result` explicitly. This incident is **not** a claim that the normal project compilation entry path always swallows failures.

## Contract

The audit's `INSTRUCTIONS.md` establishes T5 (non-proof must not become success), T7 (fresh/incremental formal equivalence), T8 (failure containment) and compiler evidence provenance requirements. A query failure is not an ordinary source diagnostic nor proof of successful analysis. The source-level `SnapshotStatus` claims semantic completeness; `SemanticAnalysis` exposes no `QueryOutcome` alongside the snapshot.

## Reproducible source-level contradiction

`session.rs:927–960`:

```rust
self.update_with_budget_and_cancel_and_delta(
    input, QueryBudget::default(), &CancellationToken::new(), module_delta,
)
.unwrap_or_else(|_| {
    let snapshot = self.last_known_good.clone().unwrap_or_else(|| {
        Arc::new(SemanticSnapshot::new_with_callable_analyses(
            /* current input generation and sources, but base tables and */
            /* diagnostics = empty BTreeMap, analyses = empty HashMap */
        ))
    });
    SemanticWorkspaceUpdate {
        snapshot,
        invalidated: Arc::from(Vec::new()),
        recomputed: Arc::from(Vec::new()),
        stats: SemanticUpdateStats::default(),
        module_stats,
        effects: SemanticPublicationEffects::default(),
    }
})
```

The closure discards the `QueryOutcome` (`Cancelled`, `Blocked`, `BudgetExceeded`, `Failed`). Two consequences exist:

1. **No prior successful snapshot:** it constructs a synthetic snapshot from current sources but only base semantic tables and **empty diagnostics and callable analyses**. `new_with_callable_analyses` marks it `SnapshotStatus::Complete`. `has_errors()` consequently reports false despite unsuccessful analysis. `analyze_workspace` exposes the snapshot without error status.
2. **Prior successful snapshot:** it returns `last_known_good` unchanged, including previous generation/source contents/status; the returned `SemanticWorkspaceUpdate` has no failure/stale flag. A caller can mistake the result for an analysis of the requested revision. The preserved `snapshot.generation` may permit *some* consumers to reject it, but the API itself does not require a generation check and does not distinguish failure.

This is **conditionally provable from source control flow**: if the fallible update returns `Err`, the emitted result cannot reliably communicate that failure. The audit has **not** yet demonstrated an ordinary source example that drives this error path under `QueryBudget::default`.

`session.rs` has concrete `Err` propagation points for `QueryOutcome::{Cancelled,BudgetExceeded,Blocked,Failed}` while publishing formal queries, e.g. around lines 1178–1185 (static-binding catalog), 1224–1231 (unlinked interfaces), 2328–2336 (associated surfaces), and many others. These are real error paths rather than an impossible `Result` type.

## Production reachability, bounded and qualified

`workspace.rs:123–127` directly uses the infallible fallback and returns only `SemanticAnalysis { snapshot }`. `compile.rs` has at least two callers at lines ~618 and ~726. The `compile_analyzed` guard at ~827–831 only checks the snapshot's error diagnostics; a default-Complete empty-diagnostics fallback is not rejected by that guard. However, the exact public entry variants reaching those sites, any other checks inside `project_analyzed`, and whether fallible updates can be provoked with supported source were not executed or completely traced. This justifies an immediate high-priority incident, not a confirmed executable soundness exploit.

## Root cause

A convenience API has the return type for a successful `SemanticWorkspaceUpdate` but implements error recovery by manufacturing or reusing a formally authoritative `SemanticSnapshot`, rather than modeling publication failure as a separate outcome. A two-state completeness enum (`Complete`/`Partial`) lacks a failed/stale state, and `SemanticAnalysis` drops any update-level status. Recovery and formal authority are conflated.

## Required correction architecture

1. Make authoritative fresh/workspace publication return `Result<SemanticWorkspaceUpdate, SemanticUpdateError>` (or an equivalent explicit outcome) for cancelled, budget-exhausted, blocked, and internal-failure cases. `analyze_workspace` should propagate this through an explicit result-bearing API; migrate compiler consumers accordingly.
2. Keep last-known-good data as **separately typed advisory/editor fallback**, carrying source/generation identity and an explicit `Stale/Failed { reason, attempted_generation }` marker; never substitute it for a formal current snapshot.
3. If compatibility requires an infallible editor helper, return `PresentationRecovery { attempted_revision, snapshot: Option<...>, failure }`, not a success-shaped `SemanticWorkspaceUpdate` that pretends formal publication succeeded. Document the authority boundary.
4. Audit all consumers of `session.update`, `update_module_workspace`, `analyze_workspace`, and `compile_analyzed`. Require current generation/identity checks for any published compilation artifact and forbid treating empty diagnostics as evidence of successful analysis.
5. Preserve atomicity: failed updates must not advance `last_snapshot`, `last_known_good`, or dependent authoritative state; test retry after failure.

## Required tests and reproduction strategy

- **Deterministic forced-failure test** on a fresh session: make a canonical query return `QueryOutcome::Failed`/`BudgetExceeded` via test-only query injection or a bounded test entry. Assert no successful `Complete` snapshot is returned; no forged empty-diagnostics success.
- **Stale LKG test**: publish generation 1, force failure on generation 2, assert returned recovery is explicitly stale, source/generation identities remain generation 1, and no formal generation-2 claims are made.
- **Compiler tests**: for each production entry calling `analyze_workspace`, propagate the semantic update error before `compile_analyzed`; no runtime artifact may be emitted from synthetic or stale formal products.
- **LSP tests**: failure permits recovery for presentation only; no incompatible source-range attachments; generations never cross.
- **Cancellation/budget tests**: distinguish `Cancelled`, `BudgetExceeded`, `Blocked`, and internal `Failed` rather than treating them as interchangeable.
- **Retry and incremental equivalence**: recovered update after failed generation agrees with fresh analysis and does not reuse invalid dependency identities.

## Verification status and exclusions

Source control flow and consumer references were inspected in the connected `main` repository; no Rust tests run, no edits performed. The prior handoff had only identified the convenience fallback as a candidate; this slice establishes a concrete production consumer. **Do not claim `compile.rs:797` or the strict `update_module_workspace_with_cancel` path is affected by this particular fallback.** Reassess severity after deterministic forced failure and source-reachable reproduction.
