# `numerics.probability`

## Scope and boundaries

This optional module adds probability evaluation to descriptors and helper
operations. It reuses the canonical explicit-RNG
[`Generator` and `Distribution`](../randomness.md) contracts, the canonical
collections and `Iterable` protocol, and the special-function owner
[`numerics.special`](special.md). It has no hidden generator state, automatic
numeric conversion, or competing sampling trait.

## Independent evaluation capabilities

The module defines four independent traits:

| Trait | Associated type | Operations |
| --- | --- | --- |
| `HasMass` | `Observation` | `pmf(value)`, `logPmf(value)` |
| `HasDensity` | `Observation` | `pdf(value)`, `logPdf(value)` |
| `HasOrderedTails` | `Cut` | `cdf`, `survival`, `logCdf`, `logSurvival` |
| `HasQuantile` | `Quantile` | `quantile(p)`, `upperQuantile(q)` |

One trait does not imply another. A law may expose a mass without an ordered
tail, a density without a sampler, or a quantile with a result type distinct
from Float. Implementations must define endpoint and invalid-domain behavior
for each supplied operation.

## Distribution descriptors

| Type | Construction | Evaluation and output |
| --- | --- | --- |
| `Bernoulli` | Canonical `universe.random.Bernoulli(probability)` | Bool mass/log-mass, ordered tails, quantiles, and explicit-generator sampling. `mean` and `variance` describe the Bool indicator. |
| `Uniform` | `new(lower:, upper:)` | Continuous density, log-density, ordered tails, quantiles, and sampling on its finite interval. |
| `Normal` | `new(mean:, stddev:)` or `fromVariance(mean:, variance:)` | Continuous density, log-density, ordered tails, quantiles, and sampling; standard deviation must be positive and finite. |
| `Exponential` | `new(rate:)` | Continuous density, log-density, ordered tails, quantiles, and sampling for a positive finite rate. |
| `Binomial` | `new(trials:, probability:)` | Integer mass, ordered tails, integer quantiles, and sampling; `trials` is in `[0, 2^53 - 1]` and probability lies in `[0,1]`. |
| `Poisson` | `new(rate:)` | Integer mass, ordered tails, integer quantiles, and sampling for a finite rate in `[0, 2^53 - 1]`. |
| `Gamma` | `new(shape:, rate:)` or `fromScale(shape:, scale:)` | Continuous density, log-density, ordered tails, quantiles, and sampling for positive parameters. |
| `Beta` | `new(alpha:, beta:)` | Continuous density, log-density, ordered tails, quantiles, and sampling for positive parameters. |
| `Categorical<T>` | See constructors below | Mass/log-mass over unique labels and explicit-generator sampling of `T`. |

`Gamma` and `Beta` above are random-variable descriptors. They are not the
stateless special-function families of the same names in
[`special.md`](special.md).

`Categorical<T>` supports construction from linear weights, normalized
probabilities, or log weights. Labels are copied in input order and must be
unique under stable equality. Linear weights are finite and nonnegative with a
positive total; probabilities are finite, nonnegative, and sum to one within
`1e-8`; log weights may be finite or negative infinity and must have at least
one possible label. Zero-probability labels
remain part of the declared labels but are never sampled. Normalization uses a
stable shifted/scaled calculation; a positive normalized Float probability may
underflow to zero.

Sampling behavior and random-state consumption are owned by
[`randomness.md`](../randomness.md). The probability contract does not promise
statistical-quality bounds for the currently unaccepted package implementation.

## Support descriptors

- `RealSupport` represents a Float interval with explicit lower and upper
  endpoint closure. It supports closed finite intervals, intervals at least a
  finite lower bound, and all-real support. Its `contains` operation rejects
  non-finite candidate values.
- `IntegerSupport` represents a lower-bounded Int interval with an optional
  upper bound. Constructors cover closed intervals, unbounded-above intervals,
  and singletons.
- `FiniteSupport<T>` contains a nonempty copied ordered label list and
  membership set. Duplicate labels under stable equality are invalid; `values`
  returns a fresh List so callers cannot mutate the descriptor's stored order.

Support metadata describes mathematical membership. It does not promise that a
finite-precision sampler can realize every point in the support.

## Transformations, normalization, and sampling helpers

The module-level transformations are `logit`, `logOdds`, `odds`, and
`invLogit`. `logit` and `logOdds` are aliases for the log-odds transformation.
Normalization helpers are `normalizeWeights`, `normalizeLogWeights`, and
`normalizedLogWeights`. They consume one `Iterable<Float>` once and return a
fresh List in input order. Empty input raises `ArgumentError`; all-zero or
all-impossible weights raise `UndefinedNormalizationError`. Invalid negative,
non-finite, or otherwise unsupported weights are rejected. Positive normalized
weights may underflow; direct log results report an unrepresentable interior
result through `NumericalRangeError`.

`SampleIterator<D,G>` yields successive `Distribution.Output` values by
repeatedly calling the supplied distribution with the same explicit generator.
`samples(of:using:)` constructs it. `sampleMany(_:count:using:)` rejects a
negative count and reserves its List output before drawing; a reservation
  failure occurs before the generator advances.

## Likelihood operations

`iidLogMassLikelihood(distribution, observations)` sums log masses for an
`Iterable` whose item type matches `HasMass.Observation`.
`iidLogDensityLikelihood` does the same for a `HasDensity` law. Inputs are
consumed once. A zero-likelihood observation yields negative infinity; an
invalid observation follows the law's own evaluation contract.
`tailDiagnostics(distribution, at:)` returns the tuple
`(cdf, survival, logCdf, logSurvival)` for a `HasOrderedTails` law.

## Errors and nested modules

`UndefinedNormalizationError` is owned by this module. `UndefinedStatisticError`
is re-exported from the root `numerics` package. General invalid parameters use
`ArgumentError`; numerical range and evaluation errors retain their canonical
`universe.math` identities.

The `bayes` child and `monte_carlo` child have separate owners:
[`bayes.md`](probability/bayes.md) and
[`monte-carlo.md`](probability/monte-carlo.md). Their public operations do not
become root probability exports unless listed by the corresponding facade.
