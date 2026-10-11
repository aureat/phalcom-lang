# `numerics.statistics`

## Scope and input rules

The module provides finite-Float descriptive statistics over a single-pass
`Iterable<Float>`, paired statistics over two such inputs, and one exact
integer-valued operation. Inputs are consumed once in encounter order and are
not mutated. Paired inputs must have equal lengths. Non-finite observations,
invalid weights, and unsupported counts are rejected rather than silently
filtered.

`sumInts` sums `Int` values exactly. `meanExact` returns the exact `Rational`
mean of a nonempty `Iterable<Int>`. Float-based operations use Float arithmetic
and may round; see [`numeric-types.md`](../numeric-types.md) and
[`numeric-math.md`](../numeric-math.md).

## Public operations

| Operation group | Exports | Empty or insufficient data |
| --- | --- | --- |
| Totals and location | `sum`, `sumInts`, `mean`, `meanExact`, `minimum`, `maximum`, `extrema`, `span` | Operations that require an observation raise `EmptyInputError`; `meanExact` also requires a nonempty input. |
| Dispersion | `populationVariance`, `sampleVariance`, `populationStddev`, `sampleStddev` | Population forms require at least one observation and raise `EmptyInputError` when empty; sample forms require at least two and raise `UndefinedStatisticError` otherwise. |
| Paired data | `populationCovariance`, `sampleCovariance`, `correlation` | Inputs must have equal lengths; population covariance requires at least one pair, sample covariance requires at least two, and correlation requires at least two pairs with positive variance on both sides. |
| Order statistics | `median`, `quantile` | Require nonempty input; `quantile` requires a probability in `[0,1]` and uses the Type 7 estimator. |
| Relative weights | `weightedMean` | Values and weights have equal length; weights are finite, nonnegative, and must have positive total weight. |
| Online accumulation | `RunningMoments`, `RunningCovariance` | Mutable accumulators reject non-finite observations; snapshots represent the accumulated data so far. |

Float-based observation counts are limited to the largest integer exactly
representable in binary64, `2^53 - 1`. `sumInts` and `meanExact` do not use
that Float-count limit.

## Summary values and accumulators

### `MomentSummary`

An immutable snapshot contains the count, mean, population and sample variance
when defined, extrema, and private scaled accumulation state. `mean`, `minimum`,
`maximum`, and `populationVariance` return `None` when the summary is empty;
`sampleVariance` returns `None` when fewer than two values are represented.
`populationStddev` and `sampleStddev` preserve those corresponding optional
results. `merged(other)` returns a new summary for the union of the represented
observations; neither input is mutated.

### `RunningMoments`

A mutable online accumulator starts empty. `push(x)` incorporates one finite
Float. `snapshot` returns an immutable `MomentSummary`. `merge(summary)` merges a
previously captured summary into the receiver. Merging summaries is the
parallelizable operation; no claim is made that arbitrary reordering of the
original Float stream yields bit-identical results.

### `CovarianceSummary`

An immutable paired-data snapshot records a common observation count, paired
means, covariance state, and corresponding summary statistics. `merged(other)`
combines summaries only when their dimensions and count arithmetic are valid.
`meanX`, `meanY`, and `populationCovariance` return `None` for an empty
summary. `sampleCovariance` returns `None` with fewer than two pairs, and
`correlation` returns `None` with fewer than two pairs or zero variance on
either side. Population and sample covariance remain distinct estimators.

### `RunningCovariance`

A mutable online paired accumulator accepts `(x,y)` observations through
`push(x, with: y)`. Both values must be finite. Its snapshot is immutable, and
`merge(summary)` combines a previously captured paired summary.

## Errors

`EmptyInputError` is owned by this module and identifies an operation whose
contract requires at least one input. `UndefinedStatisticError` is owned by the
root `numerics` package and re-exported by statistics and probability; all
facades refer to that same type identity. `NumericalRangeError` is owned by
`universe.math` and reports Float range or supported-count failures.

The package's internal reduction kernels are not public API. This specification
does not mandate a particular stable summation or merge algorithm, only the
input, estimator, mutation, result, and failure contracts above.
