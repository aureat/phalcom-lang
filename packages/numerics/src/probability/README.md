# `numerics.probability`

Status: substantial source implementation, not yet accepted. The generic
`Iterable` helper signatures and single-pass normalization are present, alongside
the mass/density/tail/quantile capabilities, V1 scalar-law source, explicit RNG
sampling helpers, concrete support metadata, likelihood operations, bounded
conjugate updates, and a separate Monte Carlo facade. No Phalcom source compile
or runtime acceptance has been performed for this slice. The earlier note about a
`MATH-GENERIC-ITERABLE-INCIDENT.md` is stale; no such incident record exists in
this checkout.

This optional source module defines checked Float transformations and
normalization from math proposals 16 and 22:

- `logit`, `logOdds`, `odds`, `invLogit`
- `normalizeWeights`, `normalizeLogWeights`, `normalizedLogWeights`
- `UndefinedNormalizationError`

The normalization helpers depend only on Universe math and canonical collection
protocols and carry no RNG state. Distribution implementations use the sibling
`numerics.special` module for advanced kernels.

Inputs are `Iterable` values with Float items. A normalization consumes its input
once into a private List, validates it, and returns a fresh List in input order.
Linear weights must be nonnegative finite Floats; logs may be finite or negative
infinity. Empty input raises ArgumentError; all-zero/all-impossible input raises
UndefinedNormalizationError. Huge finite weights use maximum scaling and
compensated summation. Log normalization shifts before reduction and never
subtracts an absolute log normalizer from an enormous score. Positive outputs
may underflow to zero; direct normalized logs retain finite values or raise
math.NumericalRangeError when an interior log is unrepresentable.

`universe.random::Generator` and `Distribution` remain the canonical sampling
contracts; probability adds no competing RNG or sampling trait. Existing
random-owned Bernoulli is extended through probability-owned evaluation traits
and exposes indicator mean/variance.
Binomial and Poisson currently use inverse-CDF sampling drafts; other laws use
explicit generator draws or the Gamma rejection kernel. These source choices
remain unverified, so no full probability V1 or statistical-quality claim is
made.
`sampleMany` rejects negative counts and reserves its output with
`List.withCapacity` before its first draw. Explicit List reservations share a
64 MiB backing-storage budget and use fallible reservation; oversized requests
fail before generator state can advance. The cap applies to the List element
buffer, not to objects referenced by those elements.

Bayesian helpers live under `numerics.probability.bayes`; fixed-budget
expectation lives under `numerics.probability.monte_carlo`. Monte Carlo consumes
the sibling `numerics.statistics` summaries. Importing `numerics.probability`
does not export those consumer facades.
