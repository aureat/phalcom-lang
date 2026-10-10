---
id: STDL001.C7
category: STDL
program: STDL001
checkpoint: STDL001.C7
kind: checkpoint-record
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
active_plan: STDL001.C7.P1
---

# STDL001.C7 — Descriptive statistics

## Objective

Implement the bounded Phase 2 descriptive-statistics foundation in the optional
`statistics` package: finite-Float collection reductions, explicit sample and
population estimators, strict paired covariance, quantiles, weighted means,
mergeable streaming summaries, and a separate exact Int-to-Rational `meanExact`.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| `STDL001.C7.P1` | Stable reductions and descriptive/statistical summary APIs | BLOCKED | PARTIAL | UNVERIFIED | source draft is present; acceptance is blocked by a branch-only stack overflow in canonical Universe semantic expression analysis; clean `origin/main` completes the reproducer; see [incident report](../stack-overflow-incident-2026-10-11.md) |

## Established invariants

- Approximate moment counts are capped at `2^53 - 1` before conversion to Float.
- Statistical observations must be finite; missing values must be handled by
  callers before entering these reductions.
- Sample and population denominators have distinct names.
- Paired inputs require equal lengths and never silently truncate.
- Running states are mutable owners; published summaries are immutable.
- Moment state uses a scale and normalized central moments; merge uses the
  Chan combination formula without converting an arbitrary-precision count.
- Quantiles use Type 7 on a private copy; caller input is not reordered.
- `universe.math.sum` and `mean` are math-owned Float reductions; the optional
  statistics package may delegate but the numeric foundation does not depend on
  the consumer package.

## Verification

The statistics source remains unverified. The dependency-free CLI reproducer
overflows during branch-only canonical Universe semantic expression analysis,
so the failure is not isolated to the statistics package. Source acceptance
remains blocked pending an owner-level correction; the focused Rust checks for
the semantic/compiler algebraic-literal seam do not establish source-package
acceptance. See the
[incident report](../stack-overflow-incident-2026-10-11.md).

## Next action

Wait for an owner-level architectural ruling and a passing minimal-project CLI
regression. Then resume focused source acceptance; parsing, generic capture,
numerical behavior, and merge laws remain unverified.
