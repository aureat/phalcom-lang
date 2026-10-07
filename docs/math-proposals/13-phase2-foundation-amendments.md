# 13. Phase 1 dependencies and amendments

Status: recommendation. These are amendments to the foundation's proposed scope,
not additions secretly owned by statistics. The Phase 1 capability/conversion and
facade decisions in files 01, 05, 07, and 08 remain in force.

## 13.1 Required substrate

| Facility | Live status | Phase 2 consumer | Owner / required action |
|---|---|---|---|
| Exact Int, true division, explicit Float.new | Present in `U/scalar/number.ph` and Phase 1 evidence | Counts, combinations, sufficient statistics | Preserve scalar semantics and numeric resource policy |
| sqrt, hypot, ln, ln1p, exp, expm1 | Present, E02–E03 | Dispersion, transforms, stable complements | Preserve existing Float-only math Families |
| Finite/NaN/infinite predicates | Present on Float in scalar source | Parameter and observation validation | Reuse; no stats-local Float classifier |
| Machine epsilon, representable extrema | Proposed in Phase 1, not in inspected public scalar surface | Error budgets and range checks | Publish Float metadata; binary64 constants may be explicit private constants meanwhile |
| Stable Float summation/mean reduction | No dedicated public facility in E02/E07 | Means, normalization, log-likelihood | A1, math-owned algorithms with Iterable inputs |
| Positive-domain logGamma/logBeta | Absent from E02–E03 | Binomial/Poisson/Gamma/Beta evaluations, evidence | A2, small special-function substrate |
| erfc and direct logErfc | Absent | Normal small tails beyond erf cancellation or erfc underflow | A2 |
| Regularized incomplete beta and gamma, complementary/log forms | Absent | Binomial/Beta and Poisson/Gamma tails | A2; complete gamma/beta alone cannot supply CDFs |
| Safeguarded inversion / inverse-normal kernel | Absent | Distribution quantiles | A2; distribution-specific result/support policy remains in probability |
| Exact factorial/choose | Absent from IntegerMath | Probability problems; exact combinatorial counts | A3, math-owned Int algorithms |
| Uniform random bits, range rejection, seeded state | Random facade is a stub | Every sampler and Monte Carlo helper | A4, universe.random |
| Iteration/fold/reduce/strict zip | Present, E07/E20 | Generic one-pass inputs and paired observations | Reuse without a statistics-specific iterator system |

`U` and evidence identifiers refer to [12](12-phase2-recommendation-and-evidence.md).

## 13.2 A1 — stable reduction is shared mathematics

Add a math-owned compensated sum and range-aware mean kernel over finite Floats.
Use Neumaier-style compensation for serial sum and explicitly ordered pairwise
combination for buffered reductions. Use scaled fallback state when an
intermediate sum overflows despite a representable final sum. Neither claims
Float associativity. A
streaming mean needs scaled/convex-combination fallbacks when a difference or
sum overflows even though the final mean is representable.

Also add math-owned `logSumExp` with the extended-value reduction semantics in
file 16. Probability normalization delegates its numerical core to that owner
while applying stricter weight validation and shifted output construction.

Statistics owns central-moment state and degrees of freedom. Probability owns
weight validation and normalization. Both consume math's accumulation kernels.
No universal numeric `Accumulator<T>` trait is needed. Existing fold accepts
ordinary callbacks and initial state.

## 13.3 A2 — bounded special functions become a V1 dependency

Phase 1 deferred all special functions. Amend that scheduling boundary for the
specific probability consumers below, while leaving Bessel functions and broad
scientific catalogs deferred:

| Required kernel contract | Consumer |
|---|---|
| `logGamma(x)` for finite x > 0; stable `logBeta(a,b)` for a,b > 0 | Log densities, log masses, conjugate evidence |
| `erfc(x)` and `logErfc(x)` with direct large-positive-x computation | Normal CDF/survival and logs |
| `regularizedBeta(a,b,x)` and complement, plus direct logarithms | Beta CDF and binomial tails |
| `regularizedGammaP(a,x)` and Q, plus direct logarithms | Gamma CDF and Poisson tails |
| Tail-aware inverse normal; safeguarded beta/gamma inversion | Quantiles and equal-tailed credible limits |
| Stable `logFactorial(n)`/`logChoose(n,k)` using exact count validation | Count log probabilities without exact factorial construction |

Names are proposed library contracts. Use `universe.math.special` as an optional
public facade, backed by hidden source owners. Do not make importing the current
`universe.math` require all special functions or any probability module.

The kernel contract must specify domains, endpoint values, convergence failure,
accuracy by parameter region, and supported range. Direct log tails must survive
ordinary probability underflow; `ln(cdf(...))` is insufficient. Complementary
pairs share validated numerical machinery, with the smaller tail evaluated
directly. Probability supplies distribution identities and endpoint semantics.

Backend choice remains an evidence-based engineering decision. Candidate
source-authored approximations or licensed reusable algorithms require
independent high-precision reference data, tail-relative error measurements,
and licensing review. This report selects the semantic contract, not a new native
primitive or host dependency. Speed alone does not justify a runtime floor
addition under the Phase 1 native-boundary recommendation.

## 13.4 A3 — exact combinatorics and approximate logs

Promote `factorial(Int)` and `choose(Int,Int)` to the Phase 2-required numerical
foundation. Return exact Int subject to the existing numeric resource policy.

- `factorial(0) = 1`; negative n raises ArgumentError.
- `choose(n,k)` requires n >= 0; for k < 0 or k > n return 0.
- Use k = min(k,n-k), multiplication with exact division/cancellation, and
  resource checks. Never convert the exact result through Float.
- `logFactorial(0) = 0.0`; negative n raises ArgumentError.
- `logChoose(n,k)` returns negative infinity for impossible k, with invalid n
  rejected. Its result is an approximation in natural-log units.

Do not compute logChoose solely as three huge logGamma values when subtraction
loses significant digits. Small-k compensated products, symmetry, and
large-parameter asymptotic/deviance branches need region-specific accuracy tests.
Factorial is for exact counting; distribution evaluation never requires it.

Design now: `fallingFactorial(n,k)` for integer n >= 0, k >= 0 (zero when k > n),
and `risingFactorial(n,k)` for n >= 0, k >= 0, with the empty product equal to 1.
Defer real/complex Pochhammer symbols. “Permutations”/“combinations” enumeration
belongs in collections/combinatorial iterators; it is distinct from counting.

## 13.5 A4 — RNG is an independent foundation

The exported random namespace is not an RNG implementation (E05–E06).
[18](18-phase2-random-and-sampling.md) recommends a source-expressible, pinned
PCG32 generator plus a uniform-bit capability. Distribution samplers live in
probability, which consumes random; random never imports distributions.

The Hypothesis SeededRandom supplies state/seeding syntax evidence, but its
small period and remainder mapping do not establish unbiased arbitrary bounded
draws. Likewise, `seed + exampleIndex` in its factory is reproducible indexing,
not a proof of independent streams. Do not silently designate it the simulation
backend.

## 13.6 Dependencies that do not block approximate V1

Rational, Complex, arbitrary-precision Float, numeric promotion relations,
generic constants, differentiation, and static parameter refinements are not
prerequisites for the recommended Float probability layer. Preserve extension
space for them through explicit conversion and separate observation/sample
types. `meanExact` over Int waits for Rational; no tuple masquerades as a new
rational-number implementation.
