---
id: STDL001.C7
category: STDL
program: STDL001
checkpoint: STDL001.C7
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
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
| `STDL001.C7.P1` | Stable reductions and descriptive/statistical summary APIs | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | source draft is present; semantic stack regression and canonical bootstrap both pass; see [incident report](../stack-overflow-incident-2026-10-11.md), §16–17 |

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

The statistics source draft is present. Both the architect-directed carrier/frame
repair and the canonical bootstrap repair pass their focused regressions
(`algebraic_realization.rs` and `canonical_bootstrap_stack.rs`).
Source-package acceptance remains open for parsing, generic capture, numerical
behavior, and merge laws. See the
[incident report](../stack-overflow-incident-2026-10-11.md), §16–17.

## Next action

Resume focused source acceptance for the descriptive statistics package now
that canonical bootstrap passes.
