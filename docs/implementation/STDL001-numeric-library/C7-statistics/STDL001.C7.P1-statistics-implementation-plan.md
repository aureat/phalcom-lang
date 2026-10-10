---
id: STDL001.C7.P1
kind: implementation-plan
checkpoint: STDL001.C7
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
---

# STDL001.C7.P1 — Statistics implementation

## Objective

Implement the source-owned statistics and stable-reduction slice from proposal
files 13, 15, 16, and 21 without adding a compiler dependency or implicit numeric
conversion.

## Fixed decisions

- Preserve the package split: stable Float sum/mean belong to `universe.math`;
  collection statistics belong to optional `statistics`.
- Inputs are single-pass `Iterable` values. Snapshot only when multiple passes
  or ordering are required.
- Reject nonfinite observations and enforce the exact binary64 count limit.
- Use Type 7 quantile interpolation, strict paired lengths, explicit estimator
  names, and immutable mergeable summaries.

## Tasks

| Task | State | Scope |
| --- | --- | --- |
| T1 | IMPLEMENTED / UNVERIFIED | Add math-owned compensated finite-Float sum and range-aware mean, using magnitude-based Neumaier correction for signed terms. |
| T2 | IMPLEMENTED / UNVERIFIED | Add descriptive reductions, Type 7 quantile, weighted mean, exact integer sum, named Int-to-Rational `meanExact`, and package facade. |
| T3 | IMPLEMENTED / UNVERIFIED | Add scaled Welford/Chan moment and covariance summaries plus mutable running owners. |
| T4 | NOT STARTED | Add and run focused source/runtime acceptance only when the user requests testing or verification. |

## Completion boundary

The implemented source remains unverified. A focused acceptance pass must cover
finite-value rejection, empty and singleton behavior, Type 7 endpoints,
opposite-sign interpolation, strict pair lengths, count limits, stable
large-offset moments, merge equivalence, and exported generic Families before
this plan can be completed.

## Current blocker — 2026-10-11

Source acceptance is paused because the feature branch overflows during
canonical Universe semantic expression analysis of
`SpecialKernel.logGammaOnePlusRatio`; clean `origin/main` completes the same
dependency-free reproducer. Preserve the current architecture pending an
owner-level ruling and a passing minimal-project gate; see the
[incident report](../stack-overflow-incident-2026-10-11.md).
