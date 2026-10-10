---
id: STDL001.C8
category: STDL
program: STDL001
checkpoint: STDL001.C8
kind: checkpoint-record
status: BLOCKED
completion: PARTIAL
verification: UNVERIFIED
last_updated: 2026-10-11
active_plan: STDL001.C8.P1
---

# STDL001.C8 — Probability foundations

## Objective

Implement the Phase 2 probability foundation using the live random API while
preserving package boundaries: independent probability-evaluation capabilities,
validated immutable law descriptors, explicit caller-owned RNG sampling,
likelihood helpers, bounded conjugate updates, and fixed-budget Monte Carlo.

## Plan ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| `STDL001.C8.P1` | V1 laws, evaluation, sampling, inference, and Monte Carlo source | BLOCKED | PARTIAL | UNVERIFIED | source is present; source acceptance is blocked by the dependency-free CLI stack overflow in [incident report](../stack-overflow-incident-2026-10-11.md) |

## Established compatibility decisions

- Keep `universe.random::Generator` and `universe.random::Distribution` as the
  canonical RNG and sampling identities. `Distribution::Output` remains the
  associated sample type; do not add a competing `RandomBits` or `Sampleable`
  trait in this slice.
- Keep the existing `universe.random::Bernoulli` identity. Probability-owned
  mass, tail, and quantile traits extend it; no second Bernoulli descriptor is
  introduced.
- Distribution descriptors and evaluation live in `probability`; the existing
  generator and distribution trait remain in `universe.random`.
- `probability.monte_carlo` depends on the canonical `statistics` summaries.
  Probability root exports remain separate from Bayesian and Monte Carlo
  facades.
- The branch's `Generator.nextFloat` is a 53-bit grid draw. Samplers consume the
  caller's generator explicitly and retain this backend's reproducibility
  contract; the proposal's `Pcg32` stream contract is not substituted here.
- Explicit List capacity requests use a shared 64 MiB backing-storage budget.
  `List.withCapacity` validates and reserves before returning; `sampleMany`
  uses it before entering the sampling loop. This limits only the element
  buffer, not memory retained by sampled values.

## Implemented source scope

The current source adds independent mass, density, ordered-tail, and quantile
traits; extends Bernoulli with indicator mean/variance; adds Binomial, Poisson, Categorical, Uniform, Normal,
Exponential, Gamma, and Beta; adds `openUnit`, `sampleMany`, and a lazy sampler
iterator; adds mass/density likelihood and tail/interval helpers; adds
Bernoulli/Binomial/Poisson sufficient statistics and Beta/Gamma conjugate
updates; adds fixed-budget Monte Carlo estimates; and adds concrete finite-label,
integer-interval, and real-interval support metadata. Categorical support tracks
positive source weights independently of rounded Float probabilities.

`sampleMany` now checks negative counts and preflights a List reservation before
the first RNG draw. Its source and runtime contract remain unverified. The
current Binomial/Poisson sampler drafts use binary-search CDF inversion, while
Gamma uses Marsaglia–Tsang rejection and Beta combines two log-gamma draws.
These are implementation choices requiring focused distributional, endpoint,
and stream-consumption review before acceptance.

## Verification

No probability behavior is verified. A dependency-free CLI project reproduces
a stack overflow, so the failure is not isolated to this package; source
acceptance is blocked pending phase-specific diagnosis. Rust checks from earlier
math/compiler work do not verify these source packages. See the
[incident report](../stack-overflow-incident-2026-10-11.md).

## Remaining scope

Wait for architectural guidance and identification of the overflowing phase.
Then resume source acceptance. Distribution-specific numerical accuracy, the
full sampler contract, categorical copy/equality guarantees, focused generic
Family projection, likelihood edge cases, Bayesian invariant checks, and Monte
Carlo callback/RNG ordering remain unverified. The entire probability plan must
not be promoted on source presence alone.
