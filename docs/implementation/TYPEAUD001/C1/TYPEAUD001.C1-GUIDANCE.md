---
id: TYPEAUD001.C1
category: TYPEAUD
program: TYPEAUD001
checkpoint: TYPEAUD001.C1
kind: checkpoint-guidance
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: main
prepared: 2026-10-12
prepared_against_revision: b00d75b3ac9f50d37cf9abfddfde00917e099031
---

# TYPEAUD001.C1 — Implementation Guidance
## Architecture, execution authority, testing, and lifecycle controls

> **This document is guidance, not checkpoint state authority.** The single durable C1 state and verification ledger is [`TYPEAUD001.C1-CHECKPOINT.md`](TYPEAUD001.C1-CHECKPOINT.md). The P1 patch-grade plan owns exact T0–T11 tasks, G0–G9 gates, and CV-01–CV-30 requirements. Proposed P2–P5 scope is not an authorization to implement unplanned architecture. Live repository location: `docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1-GUIDANCE.md`.

## 0. Executor doctrine

This checkpoint contains soundness-critical semantic changes. An implementation agent may adapt mechanical code details but **must not invent or weaken** formal semantic rules, authority boundaries, identity schemes, test oracles, or scope limits.

**Do:** repair the canonical owner, preserve historical proof provenance, write counterexamples before broad changes where feasible, propagate infrastructure failure explicitly, use narrow selected tests, classify baseline failures, and produce verifiable lifecycle evidence.

**Do not:** convert blocked or Dynamic proof outcomes into static success; manufacture a `Complete` snapshot after failure; replace lossless type reconstruction with a closed shape; duplicate the semantic checker in compiler/LSP; alter unrelated tests to obtain green gates; run broad workspace suites after every edit; or claim completion from source diff alone.

### Canonical priority of authority

1. `AGENTS.md` and applicable workflow/implementation constraints.
2. Current normative `docs/specs/` and ratified accepted decisions for semantic meaning.
3. User-ratified program/checkpoint architectural rulings.
4. Shared C1 checkpoint for actual lifecycle and verified state.
5. Active patch-grade plan for bounded task mechanics and gates.
6. Current source/tests as evidence of implementation behavior, not normative overrides.
7. Audit reports and historical plans as incident evidence/provenance, not independent semantic authority.

Read `docs/specs/README.md` before resolving language-semantic conflicts. A source/spec conflict is an incident to repair or escalate, never an excuse to silently edit the specification.

## 1. Canonical directory and read order

```text
docs/implementation/TYPEAUD001/
  PROGRAM.md
  STATUS.md
  C1/
    TYPEAUD001.C1-CHECKPOINT.md
    TYPEAUD001.C1-GUIDANCE.md
    TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md
    # P2–P5 plans, task evidence and handoffs are created when actually ready.
```

**Required P1 read order:**

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/specs/README.md
docs/wiki/semantic/workspace-incrementality.md

TYPEAUD001/PROGRAM.md
TYPEAUD001/STATUS.md
TYPEAUD001/C1/TYPEAUD001.C1-CHECKPOINT.md
TYPEAUD001/C1/TYPEAUD001.C1-GUIDANCE.md
TYPEAUD001/C1/
  TYPEAUD001.C1.P1-semantic-publication-luna-patch-grade-implementation-plan.md

Audit INC-007 and INC-011 (from audit report/handoff or supplied source)

phalcom-semantic/src/session.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/{mod,state,dependency,query,scheduler}.rs
phalcom-core/src/modules/{compile,compile_session,canonical_universe}.rs
phalcom-lsp/src/{analysis_service,request_context}.rs
```

Verify the location of source/tests against live repository state; file offsets and private symbols may have drifted. Do not assume locally downloaded records have been committed to the repository.

**Suggested preflight commands in implementer's own checkout:**

```sh
git status --short
git rev-parse HEAD
git rev-parse origin/main
git log -1 --oneline
cat rust-toolchain.toml
rg -n 'update_with_delta|update_with_budget_and_cancel_and_delta|update_module_workspace|apply_module_mutations|analyze_workspace' \
  phalcom-semantic/src phalcom-core/src phalcom-lsp/src
```

Proceed from the verified live tree. Mechanical drift is adaptable; changed authority, identity, or architecture is a consultation trigger.

## 2. Cross-plan architectural invariants

The following are checkpoint acceptance requirements, **not claims already implemented**.

| ID | Invariant | Source of authority / test obligation |
|---|---|---|
| AUTH-01 | Only a successfully committed semantic generation may satisfy authoritative compiler analysis | P1 / semantic workspace contract |
| AUTH-02 | Accepted module/source changes persist despite failed semantic attempts | P1 / module-session contract |
| AUTH-03 | Successful retry is cold-equivalent; no candidate cache can poison later publication | P1 / incremental contract |
| REL-01 | Union subtype conjunction/disjunction honor quantified proof outcomes | P2 / normative type algebra |
| REL-02 | `Blocked`, `Refuted`, `DynamicBoundary`, `Cancelled`, `BudgetExceeded`, `Failed` remain distinct | P2/P3 and P1 failure representation |
| SHAPE-01 | Tuple labeled entries are ordered and label identity is significant | P2 / structural-products.md |
| SHAPE-02 | Positional and labeled open tails are independent and survive all type transformations | P4 / structural-products.md |
| SHAPE-03 | Record field presence, key identity, and tail bounds survive all type transformations | P4 / structural-products.md |
| DY-01 | Explicit Dynamic argument remains a boundary and cannot be converted to assumed static evidence | P3 / accepted generic typing |
| INTR-01 | Intrinsic type members preserve open shapes or remain symbolic when exact reduction is not provable | P5 / TDR-0086, TDR-0087 |
| CONS-01 | Compiler/LSP consume semantic-layer products; they do not recreate canonical type rules | All plans |
| EVID-01 | Every claimed gate has exact command, nonzero selected tests, actual results, and revision | All plans / workflow |

### Architectural rulings for P1

1. **Do not roll back the module session merely because the semantic query was cancelled.** The source/module state and semantic publication have distinct commits.
2. Use an **explicit fallible** semantic authority interface. A stale snapshot is a recovery/presentation resource, not an `Ok` formal publication.
3. Stage, journal, or otherwise isolate **all mutable candidate semantic state**, not just `last_snapshot`.
4. Query-level Ready and last-known-good products are not automatically committed workspace authority.
5. Preserve a durable ledger of all module deltas since last successful semantic publication. Empty-mutation retry must be supported.
6. Preserve successfully published `Partial` snapshots where the existing consumer-specific policy allows them. An infrastructure failure is neither `Complete` nor an ordinary `Partial` success.
7. Compiler and Universe bootstrap reject failure before executable lowering; LSP interactive and scan paths must not publish failed candidates as fresh.
8. Prefer copy-on-write/persistent candidate roots or a complete transaction journal; **no routine unconditional deep clone** of the entire DB or TypeStore on ordinary edits.
9. Preserve existing TypeStore identity, TypeIds, QueryOutcome categories, and incremental dependency reuse.
10. P1 must not repair P2–P5 type laws opportunistically.

## 3. P1 execution handrails

### FIXED — cannot be delegated to implementer discretion

- Canonical semantic-publication authority and strict error boundary.
- Independent source/module and semantic commit points.
- Durable pending semantic delta across all failed attempts.
- Atomic commitment of every authoritative semantic state root.
- Source revision/provenance on stale presentation snapshots.
- Cancellation and LSP epoch semantics as two separate publication decisions.
- Correct retry equivalence, TypeStore identity, and normal incremental precision.
- No misleading synthetic `SnapshotStatus::Complete` or empty-diagnostics success.

### MECHANICALLY FLEXIBLE

- Private helper names, file-local refactors, trait implementations and adapters.
- Specific staging/journal internals **if** every mutated state owner is covered and no invariant is weakened.
- Choice of unit versus feature-enabled integration tests for controlled failure injection, subject to no production exposure.
- Narrow test-registration locations matching current harness conventions.

### VERIFY-FIRST (T0/T1)

- All API callers and re-exports; include `analyze_workspace`, `analyze_single_module`, `update`, module batch convenience APIs, compiler bootstrap, LSP scans, and tests.
- Full per-update write set: database revision, query products, LKG, dependency edges, retirement, structural shards, sources/fingerprints, static values, interner, diagnostics, alias/signature graphs, and deep frontier.
- Query/session mutation escape hatches (`db_mut`, `scheduler_mut`, `Arc::make_mut`) and ownership boundaries.
- Exact source generation and LSP epoch contracts; legitimate partial publication.
- Relevant test harness and toolchain override; baseline failure classification.
- Current `main` status and unrelated user edits before any change.

### Task-to-gate sequence

The P1 plan owns T0–T11 and local G0–G9. Main steps:

```text
T0  Establish repository/test/lifecycle baseline.
T1  Inventory mutable write set and caller graph; choose safe transaction implementation.
T2  Create RED failure-injection and authority tests.
T3  Migrate explicit success/failure and recovery API contracts.
T4  Make SemanticDb candidate operations transactional.
T5  Stage all remaining session-owned candidate state.
T6  Make pending delta and deep frontier durable across failed updates.
T7  Integrate one atomic semantic commit coordinator.
T8  Migrate compiler and canonical Universe paths.
T9  Migrate LSP interactive/scan and stale provenance.
T10 Differential retry, resource reuse and selected stabilization gates.
T11 Close incidents, checkpoint records, walkthrough, and handoff.
```

Do not shortcut T4–T7 by changing only `unwrap_or_else` and `pending_semantic_delta.take`. API correctness without transactional state isolation is insufficient to close P1.

## 4. Testing doctrine and verification policy

### Test budget

- **BUILD:** Select the smallest discriminating regression (T0/T1), then directly affected tests; rerun unchanged failing commands only after a relevant change.
- **STABILIZE:** Run owning crate/suite and affected compiler/LSP integration after the implementation becomes coherent.
- **CERTIFY:** Run broad gates only when P1 G9 or C1 G6 explicitly requires them, recording commands, exact revision and selected/pass/failed counts.
- Do not claim executing tests from the plan, audit, or checkpoint text. Historical audit observations are not execution evidence for this remediation.

### Required P1 failure classes

`Cancelled`, `BudgetExceeded`, `Blocked`, and internal `Failed` must remain distinguishable. Test failure after DB revision, shard staging, query invalidation/retirement, source fingerprint staging, candidate Ready/LKG insertion, static value/type interning, and just before the commit boundary. Verify repeated failures, provider/consumer invalidation, module identity changes, empty-batch retry, and compiler/LSP non-publication.

### Differential oracle

```text
ColdAnalyze(final workspace)
  ≡
Publish(G1); fail(G2); fail(G3); succeed(G4 with same final workspace)
```

Compare formal semantic products, diagnostics, dependencies, source indexes and stable semantic identities—not incidental interner allocation order. Retain and record both correctness and incremental reuse metrics.

### Historical baseline

INC-001 reported 300 passed, 7 failed, 1 ignored in a prior AST/parser test binary, with toolchain override uncertainty. Re-establish the current baseline; do not erase the historical observation or classify it as repaired merely because unrelated tests now pass. It is not blanket permission to ignore P1 requirements.

### Failure classification

Use the Luna schema classifications: **A** definitely patch-caused; **B** probably patch-caused; **C** unclear, one bounded classification pass; **D** unrelated/baseline, record and defer unless acceptance-blocking. Preserve exact reproduction/evidence and revisit gate.

## 5. Mandatory STOP / CONSULT protocol

Stop source modifications and prepare a concise consultation packet if:

1. Preserving the module/source commit boundary appears impossible.
2. Complete transaction isolation requires changing public `SemanticDb`, TypeStore, or query-identity architecture unexpectedly.
3. No acceptable transactional design can preserve normal incremental precision.
4. Query Ready/LKG ownership or cancellation semantics conflict with an established normative contract.
5. Source semantics or test oracle contradict accepted specifications.
6. Compiler/bootstrap or LSP consumer contracts require a materially different interpretation of semantic authority.
7. New data races, lifetime hazards, TypeId instability, or unbounded candidate/journal growth arise.
8. A new unrelated subsystem must be rewritten, or a second source of truth appears unavoidable.
9. The same soundness-critical test failure persists after one serious evidenced correction.
10. A plan invariant must be weakened or tests altered merely to pass.

Packet must include: current HEAD/worktree; task/gate; minimal observed behavior; relevant source with line numbers; hypothesis and discriminating test; scope alternatives; recommended ruling; proposed plan amendment; tests needed to prove it. Do not self-waive a consultation trigger.

### Debugging budgets

Mechanical errors: up to three coherent inspect → fix → smallest-test cycles if narrowing. Semantic errors: record observation/hypothesis/evidence/prediction/test **before** changing code, permit one serious correction for the same failure, then consult if it persists. Architecture uncertainty: consult rather than trying speculative workarounds.

## 6. Working-tree, branch and documentation discipline

- Expected execution target: `main`, but verify user authorization, branch, HEAD, `origin/main` and dirty tree at implementation start.
- Preserve all unrelated modified/untracked files. Do not broadly reset/clean/reformat.
- Stage only the coherent authorized work unit; do not assert commits or pushes from plan documents.
- On actual plan start, update `TYPEAUD001.C1-CHECKPOINT.md` from `PROPOSED` to `IN_PROGRESS` and set `active_plan`.
- Update program `STATUS.md` only to reflect the checkpoint; it is a summary, never an alternative ledger.
- Write walkthrough and handoff for each completed plan; record real tests, known residuals, new interfaces, and exact revision.
- If accepted semantics change, obtain an explicit architectural/spec decision; do not silently modify `docs/specs/` to make a regression pass.

## 7. Subsequent-plan boundaries

| Plan | Must consume | Must not redefine |
|---|---|---|
| P2 | P1 authoritative semantic publication; normative type laws | Query transaction/cancellation system |
| P3 | P2 relation outcome reducers and DynamicBoundary semantics | Relation aggregation independently from P2 |
| P4 | P1 session/interner consistency and canonical structural representations | New structural domain identities or separate type universe |
| P5 | P4 lossless structural reconstruction, accepted TDR-0086/0087 | Reinterpret finite projections contrary to cleared INC-006 |

The corrected semantic publication substrate must be delivered with stable interface and invariant descriptions in the P1 handoff, so P2–P5 need not rediscover or rewrite it.

## 8. Ready-to-start / ready-to-close criteria

**Ready to start P1:** lifecycle records installed; ID and HEAD verified; P1 plan read; existing dirty work acknowledged; T0/T1 baseline and write-set census begun. **No architectural implementation changes before T1 concludes.**

**Ready to close P1:** every required CV-01–CV-30 outcome and local G0–G9 disposition is evidenced; strict compiler/LSP paths audited; retry correctness proven; performance impact characterized; INC-007/011 individually dispositioned; walkthrough/handoff and checkpoint updated. A RED test that could not be run is documented transparently rather than invented.

**Ready to close C1:** P1–P5 complete or formally superseded with preserved invariants; C1 G6 executed and classified; the cumulative incident register and checkpoint state agree; the program/status documents reflect actual evidence. `RELEASE_COMPLETE` is only used after genuinely executed release-level verification.

**Current next action:** register these files and the existing P1 plan, then perform P1 T0/T1 in the authorized checkout. No implementation or verification is claimed in this guidance.
