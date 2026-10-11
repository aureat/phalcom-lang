# `numerics.special`

## Scope

`numerics.special` is the optional home for positive-real special functions
that are not part of the foundational `universe.math` module. Its functions
accept `Float` or `Int` as shown below and return `Float`; it does not add
numeric promotion. Core elementary functions and the shared numerical error
types are owned by [`numeric-math.md`](../numeric-math.md).

## Public function families

| Public family | Operations |
| --- | --- |
| `Gamma` | `log(x)`, regularized lower/upper tails `p(a,x)` / `q(a,x)`, their logarithms `logP` / `logQ`, and `inverseP(a,p)`. |
| `Beta` | `log(a,b)`, `regularized(x,a,b)`, `complement(x,a,b)`, `logRegularized(x,a,b)`, `logComplement(x,a,b)`, and `inverse(p,a,b)`. |
| `Normal` | `quantile(p)`, `cdf(x)`, `survival(x)`, `logCdf(x)`, and `logSurvival(x)` for the standard normal law. |
| `Combinatorics` | `logFactorial(n)` and `logChoose(n,k)`, matching the top-level functions. |
| Top-level functions | `erf(x)`, `erfc(x)`, `logErfc(x)`, `logFactorial(n)`, and `logChoose(n,k)`. |

`Gamma`, `Beta`, `Normal`, and `Combinatorics` are stateless family namespaces;
they are distinct from the distribution descriptor types named `Gamma` and
`Beta` in [`probability.md`](probability.md). The top-level combinatorial
functions and the `Combinatorics` methods have the same mathematical contract.

## Domains and endpoints

- `Gamma.log(x)` requires positive finite `x`.
- Gamma and beta shape parameters are positive finite values no greater than
  1,000,000. Exceeding a documented supported range raises
  `NumericalRangeError` rather than returning a guessed result.
- Incomplete beta arguments and inverse probabilities are finite values in
  `[0,1]`. Incomplete gamma `x` is nonnegative finite or positive infinity.
- `erf`, `erfc`, and `logErfc` accept infinities and reject NaN. All logarithms
  are natural logarithms.
- Endpoint tail values are exact: probabilities return their mathematical
  0/1 endpoints, and log probabilities return `-infinity` or zero as
  appropriate. Interior probabilities may underflow to zero; a direct
  logarithmic-tail operation is provided where available.
- Inverse beta maps probabilities zero and one to endpoints zero and one.
  Inverse gamma maps them to zero and positive infinity. The normal quantile
  maps them to negative and positive infinity.
- Log combinatorial functions require `0 <= n <= 2^53 - 1`. A negative `n` or
  a larger count raises `ArgumentError` or `NumericalRangeError`, respectively.
  `logChoose(n,k)` returns negative infinity when `k < 0` or `k > n`.

Invalid input domains raise `ArgumentError`. A range failure raises the
canonical `universe.math.NumericalRangeError`; failure to converge within the
specified bounded evaluation raises `NumericalEvaluationError`. Both errors
are re-exported through this facade without acquiring a new type identity.

## Numerical guarantees

The contract requires stable tail-oriented operations where separately exposed
and a failure rather than an arbitrary fallback when bounded evaluation cannot
justify a result. It does not promise cross-platform last-bit identity or an
exhaustive error bound. The recurrence, series, continued-fraction, and inverse
search implementations are replaceable mechanisms as long as the domain,
endpoint, error, and documented numerical behavior remain satisfied.

The detailed current approximation provenance and verification limits are
maintained with the implementation, not used as a second normative contract.
