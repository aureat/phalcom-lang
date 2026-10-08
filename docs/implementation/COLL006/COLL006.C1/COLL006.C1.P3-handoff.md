---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 continuation handoff

## Current state — 2026-10-08 publication interruption

P3 and its inference-publication supplement remain `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Current inspected HEAD is `298084a4`, following `e5e4ac20` and the historical delivered foundation `e3dfe492`. Branch: `feat/telos-collection-upgrade`; worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/telos-collections-upgrade`.

The working tree contains uncommitted checker/session/test continuation. The two later commits appeared during implementation; do not assume either HEAD or the uncommitted diff contains the entire work unit. No commit/push or remote-state verification was performed by the records update. Preserve all unrelated work and inherited formatting.

The user interrupted implementation and requested lifecycle updates plus a detailed report. This handoff records that interruption; no further source edits or Cargo validation were performed during the records-only update.

The [checkpoint](COLL006.C1-CHECKPOINT.md) is the state authority. The [publication report](COLL006.C1.P3-inference-publication-report.md) contains the complete findings, adopted solutions, remaining implementation steps, test ledger and dependency/complexity ordering. Read it before resuming; this handoff is the compact operational map.

## Must-read and inherited interfaces

Read the checkpoint's current publication section, [active supplement](COLL006.C1.P3-inference-publication-plan.md), report S1–S4, [parent plan](COLL006.C1.P3-implementation-plan.md), requirements analysis as needed, and current scoped diff. The walkthrough retains earlier evidence with explicit snapshot limits.

Established foundations:

- Shared exact `phalcom_type_meta::StaticInt`, canonical `ConstInt`, heterogeneous static arguments and metadata schema 12.
- `checker/sequence_literal.rs` is the sole bracket checker; inactive legacy synthesis was removed. Ordinary elements use checked-expression validation. Family explanations consume checked List/Array/Vector identities.
- A lone symbolic native Vector spread can use `SequenceConstructionPlan::VectorIdentitySpread`; compiler verifies extent/site/storage proof and emits `CopyVector`. No runtime interpretation of N or general symbolic arithmetic.
- Immutable native dense Vector storage, precise tracing, read-only API, equality/hash, conversions and private builder transfer are inherited. Earlier focused runtime evidence exists.

Current publication takeover map:

- `sequence_publication.rs`: private pending resolutions, conjunctive context/frame gates, terminal receipts, fail-closed registry lookup and probe metadata copy.
- `context.rs`: registers owners, stages/looks up candidates, validates saved application readiness before acceptance, rejects/disposes unresolved owners, and checks extraction settlement.
- `expression.rs`, `typed_expr.rs`, `composition.rs`: actual contextual provenance through expected types and required operands; no ambient lexical transaction.
- `sequence_literal.rs`, `statement.rs`: final head/declaration settlement after local checks and reconciliation.
- `call.rs`: explicit application provenance even when expectation is Proper; final call/union settlement; exact mandatory constraint intervals and deferred conformance inputs.
- `row_inference.rs`: owned row requirements; existing proof/materialization helpers shared with deferred validation.
- `static_body.rs`: settles expected owner before extracting maps; invalid static owners are rejected and acceptance failures affect checked status.
- `session.rs`: same-checker static annotation repair, focused-tested with Vector/List partial-owner regressions.

Keep pending state outside shared solver Rc handles and durable artifacts. A known fallback return, intermediate solve, frame close or disposal never authorizes publication. Deferred children must be revalidated against the existing final graph before terminal receipts; independent products survive unrelated rejection.

## Verification inherited by this handoff

Latest completed publication filter: **14 passed**. Registry unit selector: **3 passed**. Record-row filter: **6 passed**. Current semantic/core all-targets check passed; Vector and Array consumer filters passed 8 and 10 tests. These do not constitute final P3 acceptance.

Earlier continuation results: collection semantics 27 passed, Vector runtime 8 passed, Array runtime 10 passed, native surface check current at 406 records, workspace all-targets check passed. All predate the publication supplement. Run the supplement's consumer/build gates after semantic stabilization rather than claiming current coverage from those results.

No new incremental publication module exists. Generic proof-integrity has one existing range-oracle failure; static-constants has three outside-slice computed-key failures. Union/control/exact-case coverage, direct injected missing-owner coverage, scoped formatting and final diff review remain incomplete. Commands/counts and deliberate deferrals are in the report.

## Exact first action — static boundary closure

Adviser diagnosis confirmed a pre-existing caller defect: session elaborated partial static annotations in a temporary checker, dropped its graph, then checked the initializer in a fresh checker. `const values: Vector<_, _> = [1]` exposed this through a misleading extent diagnostic.

Adopted Level 1 T5 clarification: elaborate the annotation once in the same configured checker that analyzes the initializer. The `session.rs` edit is now present; do not redesign or transfer/recreate inference graphs. Computed-key and canonical annotation paths retain established ownership. This does not authorize P4 collection evaluation.

Resume in this order:

1. Add direct injected missing-owner API coverage. Expression/static entry now rejects missing owners with internal status/diagnostics; no fallback acceptance is used.
2. Preserve the focused static owner regressions and keep acceptance/settlement failures reflected in status and diagnostics.
3. Separate checked-static acceptance from unavailable P4 evaluation/materialization while extending exact-case, deferred row, union and control evidence.
4. Add exact-case conformance and deferred row/incomplete-child discriminators, union shared-argument valid/invalid evidence, failure/control laws and pending-family explanation evidence.
5. Create registered invalid→valid→invalid incremental/cold parity coverage, including unchanged independent callable Arc reuse. Compare canonical meanings, not cross-session raw TypeIds.
6. Run the supplement's focused semantic/core/build gates serially, scoped formatting and diff review; reconcile records before returning to parent P3 work.

The List<_> static fixture passed, but it avoids the exact-extent operation and does not prove sound partial Vector ownership. The exact-case conformance helper compiled in the 10-test run but still needs a case-only conformance positive/negative regression.

## Parent P3 remaining after publication closure

- Centralize concrete negative extent formation across canonical complete/scoped/materialized/publication paths. Start at `TypeStore::apply_static_form_with_arena` and scoped annotation applications; raw interning is not a fallible validation boundary. Retain symbolic N and exact large positive identity.
- Correct Array homogeneous APIs. Adopted direction: preserve `at` behavior as `T | None`, guarded private T reads that throw out of bounds, `get -> Option<T>`, typed replacements. Validate union support and dependent operations, then regenerate/check native surface if native signatures change. This is diagnosed, not implemented.
- Complete Vector metadata/reflection/import-export, extent/exactness revision invalidation, remaining formation/Iterable/contextual matrix and runtime effect/failure/resource edges.
- Reconcile final P3 acceptance gates and required records. P4 stays deferred.

## Residuals and boundaries

Broader metadata D1 (32 passed, 1 failure) and reflection D2 (25 passed, 4 failures) remain suspected unrelated by scoped inspection without clean-predecessor confirmation. Preserve those evidence limits; do not repair them opportunistically. The static caller defect is baseline-confirmed but actively required for T5 acceptance.

Do not re-audit exact static identity, reopen the resolved Object-layout oracle, add a second solver, weaken proof requirements, infer ownership from source containment, or silently certify dropped products. Bare brackets remain List; P4 owns static materialization/default cutover. Focused evidence does not imply workspace/release certification.
