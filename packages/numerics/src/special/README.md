# Optional positive-real special-function package

This is a source-authored binary64 implementation migrated from the former
`universe.math.special` surface. The public facade is `numerics.special`; the
implementation child (`kernel`) is hidden.
No native primitive, host special-function dependency, or numerical promotion
is introduced. Ordinary `universe.math` does not import this backend.

## Public surface

The public API is structured around mathematical families rather than a flat namespace
of compound identifiers:

```telos
from numerics.special import (
  Gamma,
  Beta,
  Normal,
  Combinatorics,
  erf,
  erfc,
  logErfc,
  logFactorial,
  logChoose,
  NumericalRangeError,
  NumericalEvaluationError,
)

// Gamma family
Gamma.log(x)
Gamma.p(a, x)
Gamma.q(a, x)
Gamma.logP(a, x)
Gamma.logQ(a, x)
Gamma.inverseP(a, p)

// Beta family
Beta.log(a, b)
Beta.regularized(x, a, b)
Beta.complement(x, a, b)
Beta.logRegularized(x, a, b)
Beta.logComplement(x, a, b)
Beta.inverse(p, a, b)

// Normal family
Normal.quantile(p)
Normal.cdf(x)
Normal.survival(x)
Normal.logCdf(x)
Normal.logSurvival(x)

// Error functions
erf(x)
erfc(x)
logErfc(x)

// Combinatorics
logFactorial(n)
logChoose(n, k)
Combinatorics.logFactorial(n)
Combinatorics.logChoose(n, k)
```

The underlying `SpecialKernel` retains long, explicit identifiers (e.g.
`SpecialKernel.regularizedBetaComplement`, `SpecialKernel.inverseNormal`) as an internal
implementation primitive; the public surface projects those into clean mathematical families.

## Algorithms and ownership

- `logGamma`: recurrence to x >= 16 and eight Bernoulli correction terms
  from [DLMF 5.11.1](https://dlmf.nist.gov/5.11.E1). Near 1 and 2, and for tiny
  positive values, a 24-term zeta Taylor expansion avoids recurrence cancellation.
- `logBeta`: recurrence to a,b >= 16, entropy-form leading terms and Stirling
  corrections. It does not subtract three large logGamma values.
- Incomplete gamma: positive lower-tail series
  ([DLMF 8.7](https://dlmf.nist.gov/8.7)) and modified-Lentz upper-tail continued
  fraction ([DLMF 8.9.2](https://dlmf.nist.gov/8.9.E2)). A separate small-shape
  upper series factors out the shape and preserves an upper tail when P rounds
  to one. The large-shape log prefactor uses a deviance expansion near x=a.
- Incomplete beta: symmetry-selected modified-Lentz continued fraction
  ([DLMF 8.17.22](https://dlmf.nist.gov/8.17.E22)).
- `logErfc`: incomplete gamma Q(1/2,x*x), a local erf series near zero, and a
  stable negative-argument complement. Direct logs remain finite after erfc
  underflows. `erfc` permits probability underflow; `logErfc` raises when the
  mathematical logarithm exceeds binary64 range.
- `logFactorial`/`logChoose`: exact Int count validation through 2^53-1,
  compensated small-k log products, and entropy-form Stirling corrections for
  larger k. No exact factorial is allocated to evaluate a log mass.
- Inverse beta/gamma: bounded bisection seeded by lower-tail asymptotics, log-tail comparisons, gamma bracket
  doubling. Inverse normal uses the direct log-erfc tail and a [0,40] magnitude
  bracket. No fixed absolute tolerance truncates small quantiles. Beta/gamma
  return the representable upper bracket when the endpoints become adjacent.

Continued fractions and gamma series have a 100000-iteration cap. Bisections
have a 1100-iteration cap (normal: 100); failure raises
`NumericalEvaluationError`. There is no guessed fallback result.

## Domains and limits

`logGamma` requires a positive finite Float; an unrepresentable result raises
`NumericalRangeError`. Beta and incomplete gamma/beta shape parameters must be
positive finite Floats <= 1000000; larger shapes raise `NumericalRangeError`.
Incomplete beta x and inverse probabilities must be finite and in [0,1].
Incomplete gamma x may be nonnegative finite or +infinity. erfc/logErfc accept
infinities and reject NaN. All logarithms are natural.

Endpoint tails have their exact 0/1 and -infinity/0 log values. Inverse beta
returns 0/1 at probability 0/1; inverse gamma returns 0/+infinity; inverse normal
returns -infinity/+infinity. Interior ordinary tails may underflow to zero.

`NumericalRangeError` and `NumericalEvaluationError` are source-owned by the
shared math numerical-error module and re-exported through both math facades.
Invalid domains raise `ArgumentError`.

## Verification scope

The checked-in reference fixture is generated independently with mpmath 1.4.1
at 100 decimal digits by `phalcom-core/tests/reference/generate_math_special.py`.
Cargo consumes the constants without Python or mpmath. Tests compare ordinary
tails by relative error (including tiny probabilities) and logarithms by scaled
absolute error. The fixture includes extreme tails, subnormal inputs, gamma/beta
shape regimes, and inverse references. It is a focused regression matrix, not
an exhaustive error bound or a cross-platform last-bit guarantee. Shapes near
one million and all possible small-shape/endpoint combinations still require
wider numerical certification before distribution release claims.
