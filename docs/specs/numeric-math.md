# `universe.math`

## Ownership

This document specifies the public `universe.math` module. It owns Float
elementary functions, integer algorithms, Float reductions, angle conversion,
and the shared `NumericalRangeError` and `NumericalEvaluationError` types.
Numeric value representation and conversion rules belong to
[`numeric-types.md`](numeric-types.md); arithmetic capability traits belong to
[`numeric-capabilities.md`](numeric-capabilities.md). Advanced positive-real
special functions live in the optional
[`numerics.special`](numerics/special.md) package. Complex mathematics is
specified by [`complex-numbers.md`](complex-numbers.md).

`universe.math` is Float-oriented. It does not define implicit conversion from
`Int` or `Rational` to `Float`, nor a general numeric promotion ladder.

## Constants and angles

The module exports `pi`, `tau`, `e`, `ln2`, `ln10`, and `sqrt2` as Float
constants. Trigonometric arguments and results use radians. `degrees` and
`radians` explicitly convert between the two units.

## Float functions

The public unary functions are `sqrt`, `cbrt`, `exp`, `exp2`, `expm1`, `ln`,
`ln1p`, `log2`, `log10`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`,
`cosh`, `tanh`, `asinh`, `acosh`, and `atanh`. `log(value, base)`, `atan2(y, x)`,
and `hypot(x, y)` are binary. Their argument and result domains are Float.

They preserve the Float domain and follow the Rust `f64` backend's IEEE
binary64 behavior for invalid domains, infinities, NaNs, and signed zero. They
do not convert backend domain results into language-specific exception
wrappers. The contract does not promise identical last bits across hosts.

## Reductions and log-domain operations

The iterable reductions accept a single-pass `Iterable<Float>` and consume it
in encounter order. They do not mutate the input.

| Operation | Contract |
| --- | --- |
| `sum(values)` | Compensated finite-Float sum; empty input returns zero. Non-finite input raises `ArgumentError`; a non-finite result raises `NumericalRangeError`. |
| `mean(values)` | Finite-Float mean; empty input raises `ArgumentError`. |
| `meanOption(values)` | Finite-Float mean as `Option<Float>`; empty input returns `None`. |
| `logSumExp(values)` | Stable log of the sum of exponentials. Empty input and an all-negative-infinity input return negative infinity; any NaN input propagates, otherwise positive infinity dominates. |
| `logAddExp(a, b)` | Stable two-term log-sum-exp. A NaN input propagates; otherwise positive infinity dominates and negative infinity is the additive identity. |
| `log1mexp(x)` | Stable `ln(1 - exp(x))` for `x <= 0`; `x > 0` returns NaN. NaN propagates, zero returns negative infinity, and negative infinity returns zero. |

The reduction inputs are consumed once in encounter order. The sum and mean
operations require finite observations and may raise `NumericalRangeError`
when the finite result is outside the binary64 range. These operations may
round as Float operations; they do not promise exact summation or order
independence. `numerics.statistics` owns statistical estimators, paired
reductions, and mergeable summaries.

## Integer algorithms

The exact `Int` operations are:

| Operation | Contract |
| --- | --- |
| `gcd(a, b)` | Nonnegative greatest common divisor; `gcd(0, 0) == 0`. |
| `lcm(a, b)` | Nonnegative least common multiple; either zero argument gives zero. |
| `factorial(n)` | Exact product from 1 through `n`; negative input is invalid. |
| `choose(n, k)` | Exact binomial coefficient; negative `n` is invalid, while `k < 0` or `k > n` returns zero. |
| `fallingFactorial(n, k)` | Exact falling product; negative arguments are invalid and `k > n` returns zero. |
| `risingFactorial(n, k)` | Exact rising product; negative arguments are invalid and `k == 0` returns one. |
| `isqrt(n)` | Floor of the nonnegative square root; negative input is invalid. |

The mathematical results are exact `Int` values, subject to the implementation's
documented integer resource policy. Logarithmic factorial and combination
functions have a separate owner in `numerics.special` because they return Float.

## Errors

`NumericalRangeError` reports a valid argument or result outside the supported
numeric range. `NumericalEvaluationError` reports failure of a bounded
numerical evaluation to converge. Neither error permits returning a guessed
fallback value. Ordinary invalid arguments use `ArgumentError` where the
individual operation specifies that response.

## Separation from implementation

The module's function set and result/error behavior are source-visible. This
specification does not require a particular polynomial, approximation,
iteration scheme, or native backend unless that choice is needed to preserve
the stated behavior.
