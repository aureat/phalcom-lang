# INC-011 — Infallible module update loses an uncommitted semantic delta after a failed publication

**Finding ID:** INC-011\
**Priority:** P1 — incremental consistency / conditional stale-authority risk\
**Confidence:** High for the path-specific lost-delta logic; runtime failure/retry probe pending\
**Status:** SOURCE-CONFIRMED conditional invariant violation; user-facing trigger and current caller coverage unverified\
**Audit:** Telos type-system audit, Slice 05, 2026-10-12\
**Repository:** `aureat/phalcom-lang` / `main` at `0651b762ac2a0c5cb1a1abd38d01c14e3afe8297`\
**Class:** incremental invalidation integrity / failed-publication atomicity\
**Scope:** `phalcom-semantic/src/session.rs` (module workspace update API, pending semantic delta, fingerprint shortcuts).

## 1. Distinction from INC-007

INC-007 reports that `SemanticWorkspaceSession::update_with_delta` hides failed semantic updates by returning a success-shaped fallback snapshot. This incident isolates a *separate state transition defect*: the **infallible module-workspace entrypoint discards the set of source changes that must be replayed on a subsequent successful semantic update**. Even if callers recognize a stale snapshot, the next incremental calculation can use the wrong invalidation scope.

Do not infer that all editor/compiler call paths use this entrypoint. The compiler's `phalcom-core/src/modules/compile_session.rs` uses the stricter `update_module_workspace_with_cancel` variant, which preserves a failed delta. The infallible method is public and source-reachable by API; direct production callers were not established from current code search.

## 2. Source-level evidence

**A. Correct failure preservation in the fallible route:** `session.rs:811–849`

```rust
let delta = self.merge_pending_semantic_delta(delta);
let result = self.update_with_budget_and_cancel_and_delta(..., Some(delta.clone()));
if result.is_err() {
    self.pending_semantic_delta = Some(delta);
} else {
    self.pending_semantic_delta = None;
}
```

**B. Lost delta in the infallible route:** `session.rs:852–888`

```rust
fn merge_pending_semantic_delta(&mut self, mut current: SemanticModuleDelta) -> SemanticModuleDelta {
    let Some(previous) = self.pending_semantic_delta.take() else { return current };
    current.changed_modules.extend(previous.changed_modules);
    // ... merges removed_modules, identity_changes and graph flag ...
    current
}

pub fn update_module_workspace(&mut self, update: WorkspaceModuleUpdate) -> SemanticWorkspaceUpdate {
    let delta = /* changed_modules/removed_modules/identity_changes from update */;
    let delta = self.merge_pending_semantic_delta(delta);
    self.update_with_delta(/* input */, Some(delta))
    // no success/failure branch, no requeue
}
```

**C. The callee can fail:** `session.rs:927–960` wraps the fallible update in `.unwrap_or_else(|_| ...)`, returning a previous or manufactured snapshot without conveying the query failure or restoring the delta. The pending delta was consumed before this call.

**D. The next delta-driven update trusts absence from `changed_modules` without checking source content:** `session.rs:1017–1055` uses

```rust
let unchanged = if delta_driven && explicitly_changed && structural_unchanged { /* fast reuse */ }
    else if delta_driven && !explicitly_changed { true }
    else { /* compare fingerprint */ };
```

and `session.rs:1197–1214` similarly reuses an old source fingerprint whenever `delta_driven && !changed_modules.contains(module_id)`. `session.rs:1263–1268` uses the same trust for field-lifecycle fingerprinting. Thus a missing formerly failed delta can bypass required invalidation even when a module's source differs from the successfully published semantic revision.

## 3. Conditional failure sequence and distinct impact

1. Publish semantic generation `g1` with module `M` at source content `S1`. The session has source fingerprints and callable products for `S1`.
2. Apply a module update containing `M: S1 -> S2`, with `changed_modules={M}`, via the public `update_module_workspace`.
3. Force an ordinary fallible semantic query **after** the attempt started to return `QueryOutcome::Failed`, `Blocked`, or `BudgetExceeded`. The infallible callee returns a last-known-good/synthetic fallback; it never requeues `{M}`. Several internal session fields are updated *before* publication, including `self.semantic_structure_shards` (`session.rs:1055`) and `self.sources/source_fingerprints` (`:1279–1280`), which increases the need for transaction-boundary verification.
4. Apply a *different module* update `N` and retry with `changed_modules={N}` while `M` remains `S2`. Because `{M}` was dropped, delta-driven shortcuts treat `M` as unchanged and may retain or recompute products according to partial mutated state rather than the last successful formal publication. This can preserve stale or inconsistent formal products for `M` despite source `S2`.

The branch-level lost-delta behavior is certain **if a query returns `Err`**; proving a stale end product requires an injected failure and an actual retry in the Rust environment. No normal source-driven trigger has been demonstrated. This is not a claim of observed corrupted compiler output.

## 4. Root cause and architectural correction

There are two incompatible publication contracts:

- Fallible update: retains an uncommitted delta and reports the failure.
- Infallible update: consumes that delta and returns a success-shaped fallback; it cannot tell whether the delta committed.

**Required model:** transactional incremental revisions. A semantic delta is acknowledged only when **all canonical products of the requested generation commit atomically**. Preserve the distinction among (a) module resolver input generation, (b) successful semantic generation, and (c) last-known-good presentation.

Implementation requirements:

1. Route module updates through a fallible core with explicit success/failure (`Result<SemanticWorkspacePublication, QueryOutcome<_>>`). The failure path must retain the complete merged pending delta including removals, identity changes, graph changes, and changed modules.
2. Treat `update_module_workspace` as a deprecated presentation compatibility entrypoint or return a structured failed/stale publication; do not hide the error while dropping its delta.
3. Commit semantic-structure shards, cached source fingerprints, query revisions, dependency graphs, and snapshot only as one coherent revision; otherwise use a rollback/staging log or force an entire fresh recomputation after partial mutation.
4. Make optimistic `delta_driven && !explicitly_changed` reuse conditional on **proven source fingerprint and semantic revision agreement**, not merely omission from the current mutation batch, especially after previous failures.
5. Preserve the strict compiler and LSP update paths; no speculative last-good data may satisfy compiler authority.

## 5. Required regressions and gates

- `infallible_module_failed_publication_preserves_changed_delta`: successful `g1`; forced failed `M` update at `g2`; inspect pending delta and ensure `{M}` is retained. Expected to fail on audited main.
- `retry_new_module_edit_recomputes_uncommitted_prior_module`: after failed `{M}`, apply `{N}`; next publication must include `M` *and* `N` invalidation and agree with a cold `S2`/`N2` analysis in all formal products and diagnostics.
- `failed_update_does_not_commit_structural_shards`: failure injected before/after `session.rs:1055` and after `:1279`; check rollback or explicit dirty staging state.
- `pending_delta_keeps_removed_and_identity_changes`: removed modules, reparented module identities, and graph changes persist through retry.
- `strict_update_result_keeps_pending_delta`: positive control for existing `update_module_workspace_with_cancel` error branch; test both cancellation and failure.
- `publish_generation_exactly_once`: source generation/semantic revision and fingerprint authority agree; old snapshot stays read-only during rollback.
- Resource/cancellation gate: no unlimited retry loops or silent budget inflation.

## 6. Verification status

Inspected remote `main` source and traced the exact pending-delta `take`/fallback and delta-fast-path conditions. This is a **conditional, source-confirmed invariant violation**, not an executed failure/retry reproduction. No file changes or tests were performed in the repository.

**Suggested owner:** semantic session update transaction and module-workspace delta lifecycle; coordinate with INC-007 remediation rather than applying isolated fallback patches.

## Implementation follow-up — 2026-10-12

**Disposition:** implementation present; incident remains OPEN pending P1 G4–G9 acceptance.

Module update APIs now return semantic failures explicitly. Pending deltas merge without destructive consumption, are reconciled against the latest source membership, and clear only after semantic commit. The session transaction restores DB query/dependency state, candidate interner roots, and structural shards; accepted module source state remains committed independently. Failed deep-analysis work also remains queued.

Focused evidence: 13 publication tests pass, covering repeated failed edits, remove/re-add, empty-batch retry, cold-analysis projection parity, retained deep frontier, and a real zero-step budget exhaustion followed by successful empty-batch retry with both accepted module edits. `workspace_indexing` selected 19 tests and passed all 19. Reuse-suite failures were reproduced unchanged on base `b00d75b3` and are classified D. **Remaining:** complete G6/G7 cross-layer evidence and G8 baseline-qualified reuse/performance review before closing INC-011.
