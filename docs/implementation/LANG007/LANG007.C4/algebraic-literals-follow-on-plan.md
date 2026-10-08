---
id: LANG007.C4.algebraic-literals-follow-on
kind: implementation-plan
checkpoint: LANG007.C4
status: BLOCKED
completion: PARTIAL
verification: FOCUSED_TESTED
prepared_against_revision: 43a4f466
last_updated: 2026-10-08
---

# Contextual algebraic literals — execution plan and stop state

This tracked record preserves the task contract originally written in
`docs/superpowers/plans/2026-10-08-contextual-algebraic-literals.md`. That directory
is ignored by Git. The C4 checkpoint owns lifecycle state; this file owns the
remaining execution contract.

The follow-on has no assigned numbered plan ID. `LANG007.C4.P12` already belongs
to String–Char work and must not be reused. `LANG007.C4.P3` belongs to Pack and
call/rest convergence. Phase 3 of `cliteral.md` is contextual realization and
remains incomplete.

## Objective and fixed architecture

Implement the runtime algebraic-value and suffix-literal contract in
`cliteral.md`, including contextual Complex/Quaternion realization. Literals
remain required work. Static realization is deferred to the existing static
eligibility/materialization contract.

Keep mathematical behavior and nominal declarations source-owned. Preserve
canonical declaration identity, exact generic data types, capability evidence,
ordinary selector identity, single evaluation, and source order. Publish checked
realization facts in the semantic layer for compiler consumption. Do not select
units by source spelling or infer formal facts from advisory shapes.

## Task ledger

| Task | State | Established result / remaining acceptance |
| --- | --- | --- |
| T1 — Failing syntax and runtime probes | PARTIAL | Syntax, basis identity, Quaternion laws/unavailability, typed realization, and projection probes exist. Add `2i` versus `2*i`, context propagation, shadowing, effect-order, and rejection coverage. |
| T2 — Lexer/parser and AST consumers | IMPLEMENTED / FOCUSED_TESTED | Dedicated coefficient/axis nodes, numeric grammar integration, atomic errors, and exhaustive consumers compile. Two focused syntax tests pass; all four new syntax tests passed in the larger AST run. |
| T3 — Source-owned basis/Quaternion algebra | BLOCKED / PARTIAL | Basis declarations, immutable runtime bindings, Quaternion operations, provider registration, and law probes exist. Identity capabilities, explicit realization/embedding APIs, and generic `complexPart` identity acceptance are unfinished. |
| T4 — Canonical semantic realization | NOT_STARTED | Temporary direct-construction prototypes were removed. The checker returns `Unknown(UncheckedExpression)` for imaginary nodes; bare basis expressions retain their existing types. |
| T5 — Compiler projection and lowering | NOT_STARTED | Compiler rejects imaginary expressions with an explicit missing-realization error. No semantic realization descriptor is retained. |
| T6 — Acceptance and lifecycle | PARTIAL | Interim evidence and blocker records are delivered at the user-requested stop. Full feature acceptance is red. |

## Resume gates

1. Resolve the generic reification incident before changing the affected seam.
   Keep the failing projection regression and its exact identity oracle.
2. Establish scalar identities and explicit target-side realization capabilities
   in the existing numeric source modules. Validate coherence and exact callable
   selection, especially multiple Quaternion axes under one generic capability.
3. Add canonical semantic regressions for unconstrained Basis identity, typed
   uses, aliases/qualified access, same-spelled unrelated declarations, expected
   types, inference boundaries, and cold/incremental parity.
4. Implement bounded arithmetic-region target selection before committing nested
   operations: I-only defaults to Complex, J/K or mixed axes to Quaternion; Int
   and Float coefficient domains remain distinct. A completed binding is a
   boundary. Scalar-by-basis action places the coefficient directly.
5. Publish checked realization evidence per expression, including source and
   exact target types, selected capability/callable, coefficient source, and
   evaluation obligations. Extend publication, fingerprints, dependency tracking,
   and compiler projection at their current owners.
6. Lower checked realization with ordinary data/capability machinery. Preserve
   reads, side effects, single evaluation, exceptional Float components, large
   exact Int coefficients, and GC rooting. Fail closed without sufficient facts.
7. Complete focused AST, semantic, compiler, runtime, and incremental gates;
   review the scoped diff and update the checkpoint and completion records.

Run Cargo gates serially. Broader crate/workspace/release gates require an
explicit acceptance need. The current stop does not authorize a scope reduction.
