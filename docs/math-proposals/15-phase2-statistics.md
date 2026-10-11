# 15. Collection and streaming statistics

Status: recommendation. E07–E13 establish the iteration and error substrate;
Phase 1 files 04–05 establish exact Int versus binary64 boundaries.

## 15.1 Result-type policy

Keep exact counts and integer sums exact. Make approximate statistical kernels
Float-only, with explicit conversion at the call site. V1 `mean(xs)` accepts an
Iterable whose associated Item is Float; it does not accept Number, silently
convert arbitrary-size Int, or infer a global promotion. Users can write
`xs.iter.map |x| { Float.new(x) }` when rounding is acceptable.

| Input / operation | Result policy |
|---|---|
| Any Iterable, count | Existing count -> exact Int |
| Int values, sumInts | Exact Int, numeric resource policy applies |
| Float values, sum/mean/variance/deviation/covariance/correlation | Float; stable kernel; no implicit exactness claim |
| Float median/interpolated quantile | Float, including singleton result |
| Int median/interpolated quantile | Explicit Float conversion in V1; future exact mean/median may return Rational |
| Int nearest-rank order statistic | Future selection helper preserves Int; cannot promise interpolation |
| Arbitrary labels, frequencies/modes | Future counts stay Int and labels stay T |
| Running summary | Immutable concrete summary with count Int and optional mean/variance accessors |

The distinct `meanExact(Iterable<Int>) -> Rational` source Family is now present
in `packages/numerics/src/statistics`; its Phalcom source acceptance remains
open. Exact
second moments remain later work. Standard deviation generally has no Rational
result. Decimal/BigFloat-specific packages
may provide their own explicit capabilities and kernels; V1 does not require a
universal associated accumulation type before users can compute a Float mean.

## 15.2 Collection operation matrix

Contracts use natural-language signatures; file 21 gives Telos declarations.
All Float statistical inputs must be finite. A terminal consumes its input once
unless materialization is explicitly specified.

| Operation | V1 decision | Algorithm / result / edge semantics |
|---|---|---|
| count | Reuse existing Iterable.count | Exact Int; empty 0; consumes a single-pass iterator |
| sumInts | Include | Exact accumulation; empty 0 |
| sum | Export math-owned stable Float sum | Compensated accumulation; empty 0.0 |
| mean | Include | Range-aware incremental or compensated mean; empty raises EmptyInputError |
| weightedMean | Include | Strict paired iterable, validated nonnegative weights, stable scaled normalization; empty or all-zero weights invalid |
| minimum / maximum | Include Float terminals | Finite comparison; empty raises EmptyInputError; preserves Float |
| extrema / span | Include | Tuple (minimum, maximum); span=max-min, may overflow as a numerical condition; no ambiguity with collection Range |
| populationVariance | Include | M2/n; empty error; singleton 0.0 |
| sampleVariance | Include | M2/(n-1); n<2 raises UndefinedStatisticError |
| populationStddev / sampleStddev | Include | Square root of corresponding variance; same minimum n |
| variance / standardDeviation | Omit ambiguous aliases | Callers select the denominator explicitly |
| median / quantile | Include Float | Copy/materialize then sort, Type 7 interpolation below; empty error |
| percentile | Defer alias | Quantile uses q in [0,1]; avoids duplicate probability scales |
| mode / modes / frequencies | Design later | Equality/hash policy, tie ordering, and NaN-key behavior require explicit contracts; no arbitrary first hash-table winner |
| populationCovariance / sampleCovariance | Include | Strict zip or pairs, central co-moment; n and n-1 denominators respectively |
| correlation | Include | Pearson; n>=2 and both dispersions positive; no divide-by-zero NaN fallback |
| skewness / kurtosis | Design later | Specify bias correction and excess versus raw before publishing; M3/M4 state not V1 |
| geometricMean | Defer | Positive observations, stable sum of logs; separate zero policy required |
| harmonicMean | Defer | Positive observations, scaled reciprocal sum; no inherited signed/zero convention |

No arbitrary ddof argument in V1. The two common denominators are discoverable
names. A future named `varianceWithDof` must require integer ddof >= 0 and n >
ddof and state its estimation interpretation; ddof alone never establishes an
unbiased estimator for every population.

### Quantile convention

For a sorted copy x[0..n-1] and finite q in [0,1], use
`h=(n-1)*q`, `j=floor(h)`, `t=h-j`, then interpolate x[j] and x[j+1]. Handle
q=0, q=1, and singleton n directly. Use a range-aware weighted interpolation
which avoids overflowing `b-a` for opposite-signed extremes. This is the usual
Type 7 sample quantile, not the generalized inverse of an empirical step CDF.
Future empirical CDF quantiles need a distinct convention/name.

The input is never reordered. Materialization is O(n) memory; comparison sorting
is O(n log n). An internal selection algorithm may reduce work without changing
the chosen convention. The inspected collection source has no sort terminal;
V1 can implement a private sort on a copied List. A general public sort API is
an optional collection improvement, not a compiler prerequisite.

## 15.3 Weighted statistics

`weightedMean(values, weights: weights)` treats weights as nonnegative relative
importance, not replicated observation counts or inverse-variance evidence.
Require equal lengths and at least one positive finite weight. Zero-weight
observations still undergo finite-input validation; they then contribute
nothing. No dedicated Weights class is needed for this single interpretation.

Scale by the largest weight before summing, or rescale online when a new largest
weight arrives. Compute normalized contributions with compensated/scaled
accumulation. Do not form `sum(w*x)/sum(w)` with unbounded intermediate products.

Design later two explicitly separate weighted-variance contracts:

- frequency weights: nonnegative Int repetitions, sample denominator W-1;
- reliability weights: nonnegative Float relative weights, corrected denominator
  W-W2/W, where W2=sum(w^2), only for the stated independent-observation model.

Population weighted variance divides by W. Weight rescaling preserves relative
weighted mean/population variance and reliability correction; it changes the
meaning of frequency weights. Negative survey/calibration weights require a
specialist layer. Generic weighted variance is deferred to prevent confusing
these interpretations.

## 15.4 Running summaries

V1 concrete states:

| Mutable owner | Minimal state / observation | Immutable snapshot |
|---|---|---|
| RunningMoments | n, mean, scaled M2, min/max; Float push | MomentSummary |
| RunningCovariance | n, meanX/meanY, scaled M2X/M2Y/C; paired Float push | CovarianceSummary |

Public mutable methods: `push(_) -> Unit`, `merge(_ summary) -> Unit`,
`snapshot -> MomentSummary` (or covariance equivalent). Snapshot data uses
restricted raw construction so malformed moment state cannot be fabricated.
`merged(_)` on summaries returns a new summary. Merge never aliases a mutable
state. An empty summary is the identity; merging a snapshot of oneself means
duplicating observations, not a no-op. Concurrent mutation during consumption
is outside the reduction contract.

Accessors: count always exists; mean/extrema/populationVariance are Option<Float>
on snapshots because n may be zero; sampleVariance is None for n<2. Named
collection terminals raise on undefined quantities for concise checked usage.
For example `stats.snapshot.sampleVariance` makes a new stream's empty state
visible rather than manufacturing zero variance. Numeric failure is an error,
not None. Construction and numeric errors are separate from data insufficiency.

### Central-moment update and merge laws

Mathematical recurrence (non-Telos notation):

```text
n' = n + 1
delta = x - mean
mean' = mean + delta/n'
M2' = M2 + delta*(x - mean')

n = na + nb
delta = meanB - meanA
mean = meanA + delta*(nb/n)
M2 = M2A + M2B + delta^2*(na*nb/n)
```

Empty states are special-cased. Do not multiply arbitrary-size na*nb and convert
the entire product through Float. Compute scaled ratios and check n against the
approximate count range in file 16. Opposite-sign mean differences and squared
differences need scaled representations; this recurrence is the mathematical
law, not permission to execute every product literally in binary64.

For covariance, update with `C += (x-oldMeanX)*(y-newMeanY)` alongside both
second moments. Merge adds
`(meanXB-meanXA)*(meanYB-meanYA)*(na*nb/n)`. Correlation uses scaled ratios to
avoid overflow in `sqrt(M2X*M2Y)`.

Build collection variance through the same summary kernel, so batch and stream
share semantics. Floating results may differ with merge tree/order; tests should
check tolerance and mathematical equivalence, not promise bitwise associativity.
Parallel reductions require explicit partitioning, separate RNGs if present,
and a reproducible merge tree. Thread/distributed scheduling remains deferred.

## 15.5 Missing, NaN, infinity, and mutation policy

V1 statistical kernels reject nonfinite observations with ArgumentError and
their ordinal when available. This is a deliberately stricter statistical
boundary than elementary math's IEEE propagation. It makes NaN sorting, invalid
moment arithmetic, and inconsistent min/max behavior predictable. No implicit
skip-NaN option, missing-value class, or imputation policy is added.

Absence is upstream composition: an Iterable<Option<Float>> must be explicitly
filtered/mapped into Float observations. Dropping paired values must happen at
the pair level, preserving alignment. Strict zip raises a length mismatch rather
than silently truncating covariance or weighted calculations.

A failed push validates before mutating, including numerical-range checks;
the previous summary remains usable. Batch consumption may have advanced a
single-pass iterator before an error; no rollback of an arbitrary input stream
is promised. Lists/iterators follow their existing mutation semantics; callers
must not mutate an observation source during a statistical reduction.
