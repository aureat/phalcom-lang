# `numerics.probability.monte_carlo`

## Scope

This child facade estimates a finite-expectation quantity by drawing a fixed
number of samples from a `Distribution` using an explicit `Generator`. It
reuses [`RunningMoments`](../statistics.md) and does not manage a global RNG or
adaptively stop based on a target error.

## API and result

`expectation(distribution, count:, using:, evaluate:)` draws exactly `count`
samples, evaluates the supplied callable once per sample in draw order, and
returns a `MonteCarloEstimate`. The count must be at least two and no greater
than `2^53 - 1`. The callback must return a finite `Float` for every draw. If
evaluation, sampling, or the finite-result check raises, the operation
propagates the failure; it does not return a partial estimate.

`MonteCarloEstimate` is an immutable snapshot containing the sample count,
estimated mean, sample variance, and standard error. Standard error is
`sqrt(sampleVariance / count)`. The estimate records the observed result, not a
confidence interval or a guarantee that the true expectation lies within a
particular distance.

The supplied generator's state advances according to the distribution's
documented sample behavior for each requested draw. Reproducibility and
secure-random boundaries are owned by [`randomness.md`](../../randomness.md).

This module's result is statistical evidence only. It must not be presented as
an exact integral or as a certified error bound.
