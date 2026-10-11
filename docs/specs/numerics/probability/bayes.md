# `numerics.probability.bayes`

## Scope

This child facade provides bounded conjugate updates and evidence calculations
for the supported Bernoulli, binomial, and Poisson observation models. It
reuses the distribution descriptors in
[`../probability.md`](../probability.md), the special-function package, and
canonical `Iterable` inputs. It does not create a general Bayesian modeling
language or hidden random state.

## Sufficient-statistic values

- `BernoulliCounts(successes, failures)` stores nonnegative counts. `from`
  accepts a Bool iterable and counts `true` as success.
- `BinomialObservation(trials, successes)` stores a valid nonnegative trial
  count and a success count in `0..=trials`.
- `PoissonCounts(observations, totalCount)` stores a nonnegative number of
  observations and a nonnegative total count. Construction from counts consumes
  the input once.

These values are immutable snapshots. Their counts are exact `Int` values;
conversion to Float occurs only inside a calculation that explicitly requires
it.

## Operations

| Operation | Contract |
| --- | --- |
| `betaBernoulli(prior, counts)` | Returns the Beta posterior after Bernoulli counts. |
| `betaBinomial(prior, observation)` | Returns the Beta posterior after a binomial observation. |
| `bernoulliPredictive(prior)` | Returns the posterior predictive Bernoulli law. |
| `logBernoulliSequenceEvidence(prior, counts)` | Returns the log marginal evidence for the summarized Bernoulli sequence. |
| `logBinomialEvidence(prior, observation)` | Returns the log marginal evidence for the binomial observation, including its combinatorial multiplicity. |
| `gammaPoisson(prior, counts)` | Returns the Gamma posterior for the Poisson rate under the package's shape/rate parameterization. |
| `equalTailedInterval(distribution, mass)` | Delegates to the quantile capability and returns the equal-tailed interval endpoints. |

Prior parameters and observation counts must satisfy the constructors and
domains of their respective types. Invalid counts or probabilities raise
`ArgumentError`; unrepresentable numerical results use the canonical
`NumericalRangeError`. These helpers do not sample from posterior laws.

## Parameterization and identities

Gamma uses `(shape, rate)` as its primary representation; `fromScale` is an
explicit convenience conversion. Beta parameters are positive concentration
parameters. The prior and posterior remain ordinary `Beta` or `Gamma` values;
the update does not introduce a second distribution identity.
