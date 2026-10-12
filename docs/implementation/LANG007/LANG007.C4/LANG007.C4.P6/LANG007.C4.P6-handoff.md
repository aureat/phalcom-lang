---
id: LANG007.C4.P6
kind: implementation-handoff
title: "Static Control Flow, Structural Iteration, Matching, and Proof Predicates — Handoff"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-12
updated: 2026-10-12
branch: main
---

# LANG007.C4.P6 — Handoff

`LANG007.C4.P6` is complete (`IMPLEMENTED / FOCUSED_TESTED`).
This handoff records the established interfaces, invariants, and instructions for downstream plans (`LANG007.C4.P7`, `LANG007.C4.P8`).

## Read First

1. [C4 Checkpoint Record](../LANG007.C4-CHECKPOINT.md) for checkpoint-wide authority, status, and ledger.
2. [P6 Implementation Plan](LANG007.C4.P6-luna-patch-grade-implementation-plan.md) for the architecture and task breakdown.
3. [P6 Walkthrough](LANG007.C4.P6-walkthrough.md) for verification evidence and changes made.

## Established Takeover Interfaces and Invariants

1. **Canonical Match and Pattern Products:**
   - `CheckedStaticBody` and `CheckedStaticExpression` carry canonical `match_resolutions: Arc<MatchResolutionIndex>` and `pattern_site_resolutions: Arc<PatternSiteResolutionIndex>`.
   - Accessors `EvaluatorBody::match_resolutions()` and `EvaluatorBody::pattern_site_resolutions()` expose these to `StaticEvaluator`.
   - No duplicate pattern parsing, pattern binding resolution, or match checking is performed during static evaluation.

2. **Static Structural Iteration Carrier:**
   - Located in `phalcom-semantic/src/checker/static_iteration.rs`.
   - `StaticIterationLaneResolution` records `mode: StaticIterationMode` and `item_domain: StaticIterationItemDomain`.
   - `Statement::For` branches on `EvaluationPhase::Static`, resolving structural lanes via `resolve_static_iteration_lane`.
   - Structural iterations are guaranteed finite and reject open tails or non-structural carriers with `static.iteration_not_structural` and `static.iteration_open_tail_not_finite`.

3. **Static Pattern Matching Engine:**
   - Centralized in `phalcom-semantic/src/static_eval/pattern.rs` (`match_pattern_against_value`).
   - Supports `Wildcard`, `Binding`, `Char`, `Tuple`, `Variant` (`VariantId` canonical equality), `Or`, `Record`, and `List` (with rest slices).
   - Produces `StaticPatternMatchOutcome::Matched(Vec<(BindingId, StaticValueRef)>)` or `StaticPatternMatchOutcome::Mismatch`.

4. **Static Match Execution & Branch Proof Transport:**
   - Evaluated via `Work::AfterMatchScrutinee`, `Work::TryMatchArm`, and `Work::FinishMatchArm`.
   - Untaken match arms are never evaluated (immunity from unreduced/invalid operations in dead branches).
   - During arm evaluation, the arm's canonical `BranchProofEnvironment` is pushed onto `StaticEvaluator.active_branch_proofs` and restored upon arm completion.
   - Overridden locals from arm pattern bindings are restored cleanly in `FinishMatchArm`.

5. **Structural For Execution:**
   - `StatementContainer` unifies execution between `LexicalBlock` and `ForStatement.body`.
   - Iteration sources supported: concrete `Pack`, `Tuple`, `Sequence`, `Record`, `Data` (with closed tail), and `Type` elements.
   - Empty collections (including empty tuple `StaticValueData::Unit`) produce 0 iterations without error.
   - Linear cursor advancement in $O(n)$ steps with element pattern matching and optional `at i` index binding.
   - Integrates `ControlBoundary::Loop` so `break`, `continue`, and `return` execute with standard semantics.

6. **Proof Predicate Analysis and Evaluation:**
   - `ProofPredicateResolutionIndex` records checked proof predicate sites (`Equivalent`, `Subtype`, `Conforms`).
   - Evaluated in `Expr::TypeForm`:
     - `Equivalent<T1, T2>`: Resolved across active branch proofs and kinded arguments.
     - `Subtype<T1, T2>`: Checked via bounded subtype analysis (`check_subtype_bounded`).
     - `Conforms<T, Trait>`: Checked via `prove_conformance_with_context` against compiler-derived structural capabilities (`Product`, `OrderedProduct`, `UnorderedProduct`, `Sum`) and user traits via `StaticEvaluator::with_conformance`.
   - Fail-closed: returns `true` on proven, `false` on refutation, and `Stop::Symbolic` on open/symbolic/blocked states.

## Downstream Consumers

- **P7 (`LANG007.C4.P7`):** Consumes static control flow, `StaticEvaluator.active_branch_proofs`, `Stop::Symbolic` propagation, and structural reflection for recursive generic static computation, memoization, and residualization (`ResidualMatch`, `ResidualFor`, `ResidualProof`).
- **P8 (`LANG007.C4.P8`):** Serializes and cross-module imports static definitions containing `CheckedStaticBody` with canonical match, iteration, and proof predicate indices.
