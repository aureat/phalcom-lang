# 01 — Projection solver soundness

## Issue

Non-injective projection equalities were treated as `SolveEffect::Unchanged`, conflating “no substitution was produced” with “the relation was proven”. This permitted `finish_frame` / `finish_root` to return `Solved` when all inference variables had substitutions even though a required projection equality remained symbolic.

## Starting revision

`0d6a6e9bbf74c82e5c3587eb4153fde589cbedd5`

## Architectural correction

The existing `InferenceSession` fixed-point solver now owns an explicit residual-obligation set keyed by the original constraint index. Projection comparison does not gain reverse-inference authority.

During each constraint retry:

1. the previous residual state for that constraint is cleared;
2. the constraint is evaluated through the existing solver;
3. a stuck non-injective projection records a residual obligation;
4. independent substitutions may make forward materialization/normalization possible on a later pass;
5. a later successful retry leaves the obligation discharged;
6. a later mismatch produces the ordinary conflicting outcome.

`finish_frame` and `finish_root` now refuse `Solved` while a relevant mandatory residual remains. Closed symbolic projection equalities with no remaining `InferVarId` therefore cannot falsely succeed.

Pack `Positionals` / `Labeled` projections first attempt forward materialization after their subjects become concrete. Equality of their results never reverse-solves the complete-pack subjects. The `Complete` lane remains an identity reduction and delegates to ordinary equality of the reduced subjects.

## Direct solver regressions

Law-level tests were added in `phalcom-semantic/tests/semantic/foundations/inference.rs` for:

- type-member projection non-injectivity;
- associated projection non-injectivity;
- pack-lane projection non-injectivity;
- zero-variable residual blocking false `Solved`;
- delayed positive proof after independent subject substitutions;
- delayed negative proof after independent subject substitutions;
- projection-vs-concrete residual behavior;
- identical projection immediate proof;
- complete-pack identity-lane forward reduction.

These tests exercise `InferenceSession` directly rather than only `TypeStore`.

## Invariants

- Projection-result equality never implies subject equality for non-injective operations.
- Residual equality is proof state, not solver progress.
- Residuals are retried by the existing fixed point; no second solver was introduced.
- Completion depends on both substitution completeness and mandatory-obligation discharge.
- `Dynamic` is not introduced as an escape hatch.

## Verification

Focused verification is recorded against the resulting commit/CI run after the repository update. P4 remains open; this evidence closes only the projection-solver soundness finding.
