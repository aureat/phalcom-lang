# `numerics` Package

`numerics` is an optional source package built on the canonical
[`universe.math`](../numeric-math.md), numeric types, collection protocols, and
[`universe.random`](../randomness.md). It does not replace those owners and
introduces no implicit numeric promotion or hidden random state.

## Package map

| Import path | Owner | Public surface |
| --- | --- | --- |
| `numerics.special` | [`special.md`](special.md) | Positive-real special functions and structured function families. |
| `numerics.statistics` | [`statistics.md`](statistics.md) | Descriptive Float statistics, exact integer mean, and online summaries. |
| `numerics.probability` | [`probability.md`](probability.md) | Probability evaluation traits, distribution descriptors, explicit-RNG sampling helpers, and support values. |
| `numerics.probability.bayes` | [`probability/bayes.md`](probability/bayes.md) | Conjugate updates, sufficient-statistic values, predictive laws, and evidence. |
| `numerics.probability.monte_carlo` | [`probability/monte-carlo.md`](probability/monte-carlo.md) | Fixed-count Monte Carlo expectation and its estimate value. |

The root `numerics` package exposes these three main modules and re-exports
`UndefinedStatisticError`. A nested facade is public only when the package
explicitly exposes it; implementation kernels and helper classes are not
automatically part of the API.

## Shared contracts

- Public iterables use the canonical `Iterable` protocol and are consumed in
  encounter order. A single-pass input is not silently traversed twice.
- Float-based estimators and probability functions have explicitly documented
  domains. They do not imply exact arithmetic.
- `Rational` is used where the API explicitly promises an exact result, such as
  `meanExact`; it is not an intermediate promotion target for Float APIs.
- Sampling requires an explicit `Generator` argument. Probability evaluation
  traits do not change sampling state or algorithm.
- Numerical failures use `universe.math.NumericalRangeError` or
  `NumericalEvaluationError` where stated. Package-specific undefined-result
  cases use the error owner identified by the relevant module.

## Status boundary

This package specification describes the public source contract. It does not
claim that every package implementation has passed compile-time or runtime
acceptance. Implementation and verification status belongs in the package and
implementation records, not in this normative contract.
