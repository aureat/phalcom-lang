---
id: STDL001.C8.P1
kind: implementation-plan
checkpoint: STDL001.C8
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
---

# STDL001.C8.P1 — Probability implementation

## Objective

Advance the V1 proposal scope in files 14 and 16–21 while preserving the live
`universe.random` contract and dependency direction.

## Fixed architecture

- Keep sampling on `universe.random::Distribution` and its `Output` associated
  type; keep explicit state on `Generator`.
- Use separate nominal evaluation capabilities for mass, density, ordered
  tails, and quantiles. Do not infer one capability from another.
- Keep descriptors immutable with private raw constructors and checked factories.
- Keep statistics out of probability distribution evaluation; only
  `numerics.probability.monte_carlo` consumes `numerics.statistics` summaries.
- Keep Bayesian updates in `numerics.probability.bayes` and Monte Carlo in
  `numerics.probability.monte_carlo`, not in the probability root facade.

## Tasks

| Task | State | Scope |
| --- | --- | --- |
| T1 | IMPLEMENTED / UNVERIFIED | Add evaluation capabilities and extend the existing Bernoulli law. |
| T2 | IMPLEMENTED / UNVERIFIED | Add scalar continuous and discrete descriptors, categorical tables, and direct tails/log tails. |
| T3 | PARTIAL / UNVERIFIED | Add explicit RNG sampling, `openUnit`, `sampleMany`, and lazy sample iteration. `List.withCapacity` now applies the shared reservation budget before `sampleMany` draws; focused source/runtime acceptance remains. |
| T4 | IMPLEMENTED / UNVERIFIED | Add likelihood helpers, validated sufficient statistics, Beta/Gamma updates, and fixed-budget Monte Carlo. |
| T5 | NOT STARTED | Focused source acceptance and algorithm/stream review; no tests were requested in this continuation. |

## Open acceptance requirements

Review the current CDF-inversion algorithms for discrete sampling against the
proposal's performance and stream-consumption contracts. Validate direct
tail/log-tail edge branches, endpoint density conventions, quantile endpoints,
categorical small/zero masses and copied storage, rejection termination,
representability errors, posterior formulas, and Monte Carlo single-callback
ordering before completing this plan.

## Current blocker — 2026-10-11

Source acceptance is paused because the feature branch overflows during
canonical Universe semantic expression analysis of
`SpecialKernel.logGammaOnePlusRatio`; clean `origin/main` completes the same
dependency-free reproducer. Preserve the current architecture pending an
owner-level ruling and a passing minimal-project gate; see the
[incident report](../stack-overflow-incident-2026-10-11.md).
