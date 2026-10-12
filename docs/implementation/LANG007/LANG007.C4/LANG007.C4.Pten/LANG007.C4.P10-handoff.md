---
id: LANG007.C4.P10
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
implementation_revision: 6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
checkpoint: LANG007.C4
program: LANG007
prepared: 2026-10-11
---

# LANG007.C4.P10 — Completion Handoff

## Repository and Worktree

- Worktree: `/Users/altunhasanli/.codex/worktrees/lang007-c4-p10-5099/phalcom`
- Branch: `codex/lang007-c4-p10`
- Base: `6e5287af46e7bde7a13fb2cecd37a10f42d6ee80`
- `LANG007.C4.P4` remains the checkpoint's primary active plan.
- P10 is complete as `IMPLEMENTED` + `FOCUSED_TESTED`.

## Stable Interfaces and Invariants

- `Expr::While(WhileExpr)` owns the ordinary source `while` condition and
  lexical body. `WhileLet` remains distinct.
- Explicit `Block.whileTrue` source remains an ordinary method/send, not static-control
  identity.
- Semantic checker implementation analyzes `Expr::While` directly with `solve_loop_header`.
- Static evaluation reuses `EvalControl`, `ControlBoundary::Loop`, `QueryBudget`,
  cancellation, and the Work machine. Finite loops, break, continue, return,
  and budget/cancellation handling are verified and passing.
- Runtime compiler lowering compiles `Expr::While` via direct jump-loop with
  `Block.whileTrue` deoptimization fallback compatibility preserved.
- Structural classifier/reflection (`StructuralClassifier`, `OrderedProductStructure`,
  `SumStructure`, `SpecializedVariantStructure`) and fixed-call/no-Pack invariants
  remain intact and passing.

## Downstream Ownership

- **P5:** may consume existing `StructuralClassifier`, `OrderedProductStructure`,
  `SumStructure`, and `SpecializedVariantStructure`.
- **P6:** may consume `Expr::While`, `EvalControl`, and `ControlBoundary::Loop`.
  Ordinary checker, static evaluation, and runtime behavior are fully verified.
- **P9:** may reuse the ontology census and Pack/evaluator resource harnesses.

## Baseline Failures Recorded

- `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`: confirmed as pre-existing
  on `main` (commit 6e5287af). Root cause is AUD-SEM-002, where `invalidate_opaque_calls()`
  in `apply_resolved_callable` erases mutable bindings with inferred contracts across
  method calls (such as primitive arithmetic operators `+` and `<`).
