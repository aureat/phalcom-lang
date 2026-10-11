# `numerics.statistics`

Status: first implementation slice present; source acceptance is unverified.

This optional module implements finite-Float descriptive reductions with
explicit population/sample names, strict paired covariance, Type 7 quantiles,
relative-weight means, and mergeable scaled central-moment summaries. Input
streams are consumed once. Numeric statistics reject nonfinite observations;
approximate moment counts are bounded at the exact-integer limit for binary64.
The separate `meanExact` terminal returns `Rational` for Int observations and
does not use Float accumulation.

`RunningMoments` and `RunningCovariance` hold mutable online state and publish
immutable `MomentSummary` and `CovarianceSummary` snapshots. Snapshots merge
through scaled Chan formulas. Empty and insufficient-data behavior uses the
package's `EmptyInputError` and `UndefinedStatisticError` types.

Weighted variance, modes/frequencies, higher moments, parallel reductions, and
distribution consumers remain outside this slice. This README does not claim
the Phalcom source has compiled or passed runtime acceptance.
