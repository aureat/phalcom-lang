# 16. Numerical stability and probability semantics

Status: recommendation. Float/Int semantics follow Phase 1 files 04–05; shared
math additions are explicit amendments in file 13.

## 16.1 Numerical requirements

| Computation | Required approach | Tempting failure to avoid |
|---|---|---|
| Float sum | Compensated serial sum, scaled overflow fallback, named-order pairwise merge; check final range | Unqualified left fold as a “stable” algorithm |
| Mean | Compensation or central mean with scaled convex-combination fallback | Overflowing sum/n when mean is finite |
| Variance/covariance | Welford central moments; Chan-style pairwise merge; scaled square/cross terms | sum(x²)-sum(x)²/n cancellation |
| Relative weighted mean | Rescale by maximum weight, compensated ratios, scaled values | w*x and sum(w) overflow |
| Probability normalization | Max-scale nonnegative finite weights, compensated total | Divide raw huge weights by a huge raw sum |
| Log normalization | Subtract maximum before exponentiation; retain log outputs | Multiply many probabilities or exponentiate absolute scores |
| Binomial/Poisson mass | Direct log kernel, deviance/asymptotic branches, endpoint cases | Factorial ratios and huge powers; cancellation in three logGamma terms |
| Multinomial mass (later) | Exact count checks, stable log multinomial coefficient, zero-count branches | 0*ln(0), integer-to-Float collapse |
| Normal tails | erfc and direct logErfc/asymptotic forms | 1-cdf(x), or ln of an underflowed erfc |
| Gamma/Beta tails | Regularized incomplete functions, direct complement/log branches | Full gamma/beta mistaken for a CDF |
| Likelihood | Compensated sum of independent observation log contributions | Product of a long sequence; silently mixing measures |

Scaled M2 state should store an exponent/scale and normalized mantissa, so an
intermediate sum of squares may exceed Float range while its divided variance
still fits. Covariance state needs signed scaled cross terms. A result that
cannot be represented even after scaling raises NumericalRangeError. Release
accuracy tests must include large offsets, opposite-signed extremes, subnormals,
degenerate streams, and reordered/merged summaries.

Approximate kernels retain exact Int counts but V1 bounds counts/trials at
`2^53-1` when converting them into binary64 arithmetic. Validate before update,
construction, or query. This is a documented approximate-backend range, not an
Int restriction. Exact sumInts/choose are governed by resource policy instead.
Excessively large Poisson/Gamma parameters or requested count quantiles may also
exceed the supported numerical/count range; report NumericalRangeError. Do not
round an arbitrary Int through Float and claim the same model.

Sampling can encounter representability limits with mathematically unbounded
laws. A finite output outside the supported count range, or an overflowing
continuous sample, raises NumericalRangeError; RNG consumption is retained.
There is no hidden redraw, clipping, or renormalized truncated law.

## 16.2 Probability, logs, odds, density, and likelihood

Use ordinary Float values with explicit contracts in V1:

| Quantity | Domain / interpretation |
|---|---|
| Probability | Finite p in [0,1]; -0.0 accepted as zero |
| Log probability | l <= 0; -infinity represents zero; NaN/+infinity invalid |
| Density | Nonnegative; may exceed 1 and have singular +infinity endpoint value |
| Log density | Any real log value, -infinity for zero, +infinity for a specified singular endpoint |
| Likelihood | Nonnegative model evaluation for fixed data; parameter-space normalization is not required |
| Odds | p/(1-p), [0,+infinity] |
| Log odds | ln(p)-ln1p(-p), [-infinity,+infinity] |
| Relative weights | Nonnegative finite values, not required to sum to one |
| Log weights | Finite or -infinity values; an arbitrary additive constant leaves normalization unchanged |

Validated Probability/UnitInterval wrappers can catch p-versus-log(p) mistakes,
but introduce extraction/conversion at every elementary calculation and do not
make arithmetic closed. Probability<Float> adds little while Float is the only
backend. Defer wrappers until a demonstrated cross-package misuse justifies
them. Keep parameter factories and named normalization functions as checked
boundaries now. A future wrapper must be source-defined with restricted raw
construction (E24), never a new implicit conversion rule.

Likewise defer LogProbability operator algebra. It would be incorrect for
logPdf, positive log-likelihood, arbitrary scores, or positive-infinity endpoint
densities. No shared “probability” type is used for those quantities.

V1 probability helpers: `logit`, `invLogit`, `odds`, `logOdds` (an alias for logit
with a probability-oriented name), `normalizeWeights`, `normalizeLogWeights`,
and `normalizedLogWeights`. All logs are natural. Reuse math ln/ln1p/expm1;
current `math.log(_, base:)` is not the natural-log shorthand.

`logit(0)=-infinity`, `logit(1)=+infinity`; invalid/NaN probability is rejected.
`invLogit(x)` accepts infinities (0/1 limits) and rejects NaN. Evaluate it as
`1/(1+exp(-x))` for x>=0 and `exp(x)/(1+exp(x))` for x<0 to avoid avoidable
overflow. `odds(1)=+infinity`, odds(0)=0. Use checked division and explicit
endpoint branches, never inherit a divide-by-zero exception accidentally.

## 16.3 Log-sum-exp and normalization contracts

Keep `logSumExp` as a math-owned general Float reduction because it is also
useful outside probability. For finite inputs and -infinity entries:

```text
m = maximum(logWeights)
logSumExp = m + ln(sum(exp(w-m)))
normalizedLogWeight[i] = (w[i]-m) - ln(sum(exp(w-m)))
```

The second expression avoids losing the normalizer correction when m is huge.
Use compensated summation and ln1p when the correction is small. Empty
logSumExp = -infinity; all -infinity = -infinity; any +infinity yields +infinity
unless a NaN is present; any NaN yields NaN. These are general reduction rules.

Probability normalization is stricter:

| Input | normalizeWeights | normalizeLogWeights / normalizedLogWeights |
|---|---|---|
| Empty | ArgumentError | ArgumentError |
| All zero / all -infinity | UndefinedNormalizationError | UndefinedNormalizationError |
| Negative linear weight | ArgumentError | Not applicable: negative logs are valid |
| NaN or +infinity | ArgumentError | ArgumentError |
| Zero / -infinity entries beside positive/finite entries | Output probability 0 | Output 0 or log -infinity |
| Huge finite weights / scores | Max scaling | Shifted log evaluation |

Preserve order and length. Return a fresh List<Float>; never mutate the input.
Normalization consumes an input once into a private snapshot, then performs
the maximum/total/output passes on that snapshot. It never asks a single-pass
Iterator to replay itself. Categorical uses the same bounded ownership rule.
Reported probabilities are nonnegative and sum to one within the published
rounding error budget; do not require exact Float equality to one. A small
positive normalized probability may underflow to zero. `normalizedLogWeights`
retains the finite log value and is the preferred classifier score output for
extreme cases. No fallback uniform distribution when every event is impossible.
If even the normalized logarithm is outside binary64 range, raise
NumericalRangeError rather than report a mathematically positive event as log
-infinity. An actual zero weight remains log -infinity.

`Categorical.fromProbabilities` validates each entry and compensated sum within
`16*2^-52*max(1, numberOfCategories)` absolute tolerance (capped at a small
documented tolerance for supported table sizes), then normalizes the small
rounding residual. Arbitrary rescaling belongs to fromWeights. Concrete table
size/resource bounds must prevent this tolerance from accepting a grossly
unnormalized vector; V1 maximum tolerance is 1e-8. Exact p=0 stays zero.

## 16.4 Stable zero and endpoint conventions

Semantic primitive `xlogy(x,y)` returns 0 for x=0 and finite y>=0, including
y=0; positive x with y=0 returns -infinity. Validate domain first; NaN is not
converted into zero. Analogous log1p branches handle counts multiplying
ln(1-p). This may be a private probability helper initially.

Probability/mass underflow to 0 is permitted while corresponding direct log
methods retain a finite result. A mathematically zero event has log -infinity.
This distinction is why paired methods are part of capabilities, not defaults
derived by blindly applying ln.

Endpoint singular continuous densities can produce +infinity. Generic
log-likelihood summation handles extended values explicitly: a zero contribution
(-infinity) and a singular contribution (+infinity) in the same product make the
evaluation undefined, raising NumericalEvaluationError. Density evaluation at
an endpoint does not create an atom or nonzero point-event probability.

## 16.5 Error categories

| Condition | Public behavior |
|---|---|
| Invalid parameter / query / nonfinite statistical observation | ArgumentError, identifying the argument |
| Empty collection statistic | EmptyInputError, proposed Error subclass |
| Valid data but undefined statistic | UndefinedStatisticError; optional summary getters return None only for insufficient data |
| No positive normalization mass | UndefinedNormalizationError |
| Backend nonconvergence / indeterminate extended-value combination | NumericalEvaluationError |
| Unsupported approximate range / unrepresentable result | NumericalRangeError |
| Exact integer allocation/bit budget exhaustion | Existing numeric resource-policy error |
| Impossible event in a valid model | pmf/pdf 0.0, direct log -infinity |

Error subclasses are proposed library types built with ordinary Error behavior.
They do not redefine elementary math's IEEE domain results or add a language
validation system. Source-authored checks are sufficient; proof annotations may
later eliminate checks without changing their meaning.
