# 22. Packages, gaps, alternatives, and V1 boundary

Status: recommendation with its package namespace adopted on 2026-10-11. This
closes the Phase 2 architecture report. The report is a design deliverable; no
implementation completion or behavioral certification is implied by its V1
list.

## 22.1 Bounded package architecture

Arrows below mean “imports/depends on,” not execution order:

```text
numerics.statistics ---------------------> universe.math, universe.collections
numerics.probability.distributions ------> universe.math, numerics.special,
                                            universe.random, universe.collections
numerics.probability.helpers ------------> universe.math, universe.collections
numerics.probability.bayes --------------> numerics.probability.distributions,
                                            numerics.special, universe.collections
numerics.probability.monte_carlo --------> probability sampling capabilities,
                                            numerics.statistics, universe.random
```

`numerics.statistics` never imports probability or random. Distribution
evaluation does not import statistics; formulas for model moments remain
distribution-owned.
`universe.random` supplies bit generation, uniform helpers, and future entropy
adapters, and never imports probability. `numerics.special` is optional
numerical infrastructure and never imports consumer packages.

One optional `numerics` project with public module facades is sufficient:

```text
numerics/project.toml                          one optional package manifest
numerics/src/package.ph                        exposes special/statistics/probability
numerics/src/special/package.ph                special-function facade
numerics/src/statistics/package.ph             descriptive Families and summaries
numerics/src/statistics/{descriptive,moments,covariance}.ph
numerics/src/probability/package.ph            capabilities, descriptors, normalization, sampling
numerics/src/probability/{bernoulli,continuous,discrete,gamma-beta}.ph
numerics/src/probability/{bayes,monte_carlo}/package.ph
```

The live topology uses one project manifest with these public submodules. The live branch currently places
the `Distribution` sampling trait in `universe.random`; keep that canonical
identity or provide a compatibility-preserving facade if ownership changes.
Do not define a duplicate probability trait with the same role. Importing
probability's root must not implicitly load Monte Carlo/statistics or Bayesian
helpers. Hidden children follow E02/E10 facade precedent. Shared errors belong to their owning package;
NumericalRangeError/EvaluationError should be math-owned and re-exported where
needed, rather than duplicate incompatible error identities.

The source tree now has a deterministic random implementation, so the old
random-stub annotation and G1 evidence are stale. RNG mechanics remain in
`universe.random`; probability owns laws and their evaluation. The proposed
`RandomBits`/`Pcg32` remains an alternative to the existing `Generator`/
`Xoshiro256PlusPlus` contract and needs a deliberate compatibility decision.
Namespace details may adapt to current package loading, while dependency
direction remains the recommendation.

## 22.2 Selective prior art: four ecosystems

These sources were consulted on 2026-10-07 only for specific architecture
decisions. They are external lessons, never evidence for Telos support.

| Source | Extracted lesson | Telos decision / cost |
|---|---|---|
| [Julia Distributions.jl univariate interface](https://juliastats.org/Distributions.jl/stable/univariate/) | Common log/tail/quantile methods and parameter descriptors support generic numerical consumers; some laws lack computable moments | Adopt direct operations and concrete values; split capabilities rather than its full Distribution hierarchy or one pdf for both measures |
| [SciPy rv_continuous](https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.rv_continuous.html) | Frozen parameters are reusable; sf/logsf overrides matter because 1-cdf loses precision; generic fallback inversion/integration can be expensive | Immutable descriptors, mandatory direct tails where promised; no enormous base class with automatic fit/entropy/integration defaults |
| [Rust rand StdRng 0.10.3](https://docs.rs/rand/0.10.3/rand/rngs/struct.StdRng.html) | Fixed seeding alone does not promise portable or release-stable output | Named pinned generator and separate sampler/backend version; no blanket seeded reproducibility promise |
| [Boost incomplete beta](https://www.boost.org/doc/libs/latest/libs/math/doc/html/math_toolkit/sf_beta/ibeta_function.html) and [erf/erfc](https://www.boost.org/doc/libs/latest/libs/math/doc/html/math_toolkit/sf_erf/error_function.html) | Regularized/complementary kernels and region-specific algorithms matter; accuracy needs independent high-precision evidence | Foundation amendments include direct complementary/log paths; no textbook factorial/tail formulas as universal algorithms |

The [PCG32 primary algorithm reference](https://www.pcg-random.org/using-pcg-c-basic.html)
settles the selected generator's transition, stream, seed warm-up, and biased
remainder warning. This is a focused algorithm reference, not a fifth library
survey. Do not inherit stronger “independent stream” claims without an explicit
statistical argument. Network retrieval of Stan and older rand book paths failed;
no design claim depends on those uninspected pages.

## 22.3 Error/edge-case decision table

| Example / condition | Required proposed behavior |
|---|---|
| mean([]), quantile([], q), minimum([]) | EmptyInputError; no manufactured zero/NaN |
| populationVariance([]) | EmptyInputError |
| populationVariance([x]) with finite x | 0.0 |
| sampleVariance([x]) | UndefinedStatisticError |
| correlation with constant lane / n<2 | UndefinedStatisticError |
| quantile(xs,-0.1), NaN q | ArgumentError |
| quantile singleton, q in [0,1] | The single Float value |
| Normal stddev 0/negative/NaN | ArgumentError, including zero degeneracy |
| Beta alpha 0/negative; Gamma shape/rate nonpositive | ArgumentError |
| Uniform reversed/equal bounds or infinite endpoints | ArgumentError |
| Bernoulli p NaN/outside [0,1] | ArgumentError; 0/1 are valid |
| Binomial negative n; fractional count at typed API | ArgumentError / static argument mismatch; no rounding |
| Binomial n=0 or Poisson rate=0 | Valid point mass at 0 |
| Categorical empty, unequal tables, duplicates | ArgumentError |
| Categorical all-zero weights | UndefinedNormalizationError |
| Categorical negative/NaN/infinite weights | ArgumentError |
| HasMass impossible matching-type outcome | 0.0 / -infinity, not a parameter error |
| Continuous finite value outside support | 0.0 / -infinity |
| Continuous singular endpoint density | +infinity with specified log endpoint; no point mass |
| Any statistical observation NaN/infinite | ArgumentError; caller filters intentionally |
| Sample count -1 / 0 | ArgumentError / empty with no RNG draws |
| Infinite-count law quantile(1) | UndefinedStatisticError when no Int limit exists |
| normalizeLogWeights all -infinity | UndefinedNormalizationError |
| Empty iid log likelihood | 0.0 |
| Empty sufficient statistics | Valid identity update |
| Backend nonconvergence | NumericalEvaluationError, not a guessed tail/quantile |
| Approximate count/parameter/result outside supported range | NumericalRangeError; exact Int itself remains unrestricted except resource policy |
| Invalid push/merge | Error before mutation; prior summary retained |

V1 exports no bare variance alias. Checked factories return descriptors directly
and raise; named tryNew adapters may return Result. Undefined data quantity,
invalid model construction, zero probability, and numerical failure remain
distinct. Option is used for insufficient-data snapshot access, not as a
catch-all error channel.

## 22.4 Gap ledger

| ID / category | Concrete need and evidence | Minimum useful action | V1 blocking? |
|---|---|---|---|
| G1 stdlib | Live branch has Generator/Xoshiro256PlusPlus and Distribution; probability has source samplers, but `sampleMany` lacks a preflight allocation bound | Preserve existing RNG identities; decide a shared List allocation limit, validate it before drawing, and review inverse-CDF Binomial/Poisson drafts | Source acceptance and sampler policy/review remain open |
| G2 stdlib | Live branch has `numerics.special` gamma/beta, incomplete/tail, and inverse kernels; acceptance accuracy still belongs to their checkpoint | Reuse the canonical special-function surface and validate distribution-specific tails | Remaining distribution contracts depend on source acceptance |
| G3 stdlib | `universe.math.sum`/`mean` and a first `statistics` source implementation now exist; Phalcom source acceptance remains open | Complete focused generic and numerical acceptance for stable reductions and moment owners | Source acceptance remains required |
| G4 stdlib | Live `IntegerMath` already provides factorial/choose/falling/rising factorial and logarithmic helpers | Reuse existing exact and log kernels; retain distribution-specific accuracy work | Existing helper slice; source acceptance governs claims |
| G5 stdlib | No public collection sort terminal; statistics now uses a private heap sort on a copied List | Keep sorting private until a general collection API is justified | No compiler blocker |
| G6 typing | Probability reuses `universe.random::Distribution::Output` in generic sampling and uses associated types in evaluation capabilities | Exercise canonical evidence/projection through exported Families; keep one sampling trait identity | Verification prerequisite; source acceptance remains open |
| G7 language | No supertraits/trait objects, E25 | Use independent nominal traits and concrete/callable adapters | No; no enhancement proposed |
| G8 compiler/runtime | No cross-platform bitwise transcendental guarantee, Phase 1 file 05 | Document backend sensitivity; deterministic backend only separately funded | No for version/platform-scoped sampling |
| G9 ergonomic | Validated data factory is more verbose than direct construction, E24 | Explicit .new/tryNew; preserve inaccessible raw constructor | No new constructor validation syntax |
| G10 stdlib | Float representability helpers were absent in the inspected baseline; they are now declared and implemented on `feat/complex-math` | Keep scalar-owned nextDown and machine metadata; source acceptance remains open | Implemented slice; source verification pending |
| G11 stdlib | No dedicated split/counter-based generator | Later source-defined generator/capability with explicit stream laws | No, serial explicit streams suffice |
| G12 typing/ergonomic | Heterogeneous feature products do not imply homogeneous Float iteration, E27–E28 | User-authored extraction/normalization callback; later schema libraries | No |
| G13 stdlib | Rational source and exact `Complex<Rational>` division/powers now exist but are unverified; statistics exposes named Int-to-Rational `meanExact` | Keep exact mean separate from Float `mean`; source acceptance still gates claims | No |

No compiler/language enhancement is proposed for foundational V1. Every listed
API has an existing language mechanism or an explicit stdlib dependency. If a
later generic integration check disproves G6, the minimum remedy must be an
implementation correction to the current generic-method/evidence contract or a
specific documented typing amendment, with a reproducer. It cannot become a
distribution-specific runtime tag dispatcher or a second inference authority.

## 22.5 Rejected alternatives and criteria

The primary recommendation in file 14 wins on mathematical precision,
composability, and implementability. Its costs are explicit Float conversion,
several independent bounds, and checked factory spelling. Those costs are
visible at API boundaries and leave ordinary evaluation concise.

- A giant Distribution root forces moments/order/sampling on models that lack
  them. Separate capabilities permit third-party evaluation-only descriptors.
- One `pdf` for mass and density hides different measures. pmf/logPmf and
  pdf/logPdf are intentionally distinct; shared likelihood code states which
  reference measure it uses.
- Runtime/static Probability wrappers everywhere burden elementary arithmetic
  and still fail to capture density/likelihood semantics. Validation and named
  operations address the actual V1 boundaries.
- A generic Statistic/Estimator/Simulation family adds taxonomy without a
  demonstrated generic consumer. Current fold, callbacks, and concrete state
  suffice.
- Uncontrolled global or descriptor-owned RNG hides order and prevents clean
  replay. Explicit state costs one label and keeps effects observable.
- Universal conjugacy inference guesses observation models and evidence
  constants. Typed model-specific methods preserve return laws.
- Automatically deriving survival as 1-cdf and logs as ln(probability) fails
  the required normal/binomial tail cases. Direct backend contracts are required.
- Making all statistics scalar-generic before an accumulation policy exists
  obscures exactness. Float V1 plus distinct future exact/precision kernels is a
  deliberate boundary, not implicit promotion.
- A package per textbook noun causes needless dependency management. Two
  consumers with optional higher-level modules preserve a comprehensible DAG.

## 22.6 Open questions that remain bounded

1. Which special-function backend/source algorithms meet the specified
   parameter-region accuracy, license, and convergence requirements? Decide
   using independent high-precision tail/quantile data and resource measurements.
   This blocks an implementation's numerical certification, not the ownership
   or direct-log semantic choice.
2. Which binomial/Poisson sampler thresholds and exact backend constants define
   sampler version 1? Decide with validation and focused profiles; pin the
   completed version before publishing sequence expectations.
3. Do all new exported generic Families infer associated Sample and quantified
   RNG evidence as the current tests suggest? A minimal cross-module prototype
   must answer this during implementation; no speculative compiler feature is
   included now.
4. What package names fit the project's eventual distribution mechanism?
   `statistics`/`probability` are recommended names; dependency direction and
   canonical capability identity must survive any mechanical namespace change.

Decisions already resolved here: Float result policy, finite statistical input
policy, strict survival, canonical parameterizations, Bool Bernoulli, explicit
RNG state, count ranges, private descriptor factories, typed updates, and V1
family selection. These are not reopened by the questions above.

## 22.7 Future work streams

Continue source acceptance and numeric-foundation work on Rational/Complex
independently; the current Rational and exact Complex<Rational> source remains
unverified. Probability follow-ups may add Geometric/Multinomial, Dirichlet and
conjugate predictive laws, StudentT/ChiSquared, weighted-variance models and
higher moments. Future inference can then design estimators, confidence
procedures, hypothesis tests, and differentiable log evaluators on the
established measure/tail contracts.
Advanced simulation needs splittable/counter-based streams and valid sequential
diagnostics before parallel/adaptive APIs. No full ML or probabilistic-program
runtime is implied.

## 22.8 Decisive V1 boundary

### Implement in V1

- Preserve exact Int and Float boundaries, current math facade and canonical
  Iterable/Iterator; add only the bounded A1–A4 foundation dependencies.
- Float mean/sum/extrema/span, explicit population/sample variance/stddev,
  covariance/correlation, Type 7 median/quantile, weighted mean, exact sumInts.
- RunningMoments/RunningCovariance and validated mergeable snapshots.
- Named log/odds helpers, linear/log normalization, direct normalized log outputs.
- Preserve `universe.random::Generator` and `Distribution` as the canonical
  sampling contracts; add independent `HasMass`, `HasDensity`,
  `HasOrderedTails`, and `HasQuantile` evaluation capabilities.
- Bernoulli, Binomial, Categorical, Poisson, Uniform, Normal, Exponential, Gamma,
  Beta; validated immutable parameters, direct evaluation/log/tail methods,
  Bernoulli indicator mean/variance, and explicit sampling with the caller-owned
  canonical Generator. The proposal's Pcg32 stream contract is not the current
  implementation contract.
- sampleMany and lazy SampleIterator; no hidden default generator.
- iid mass/density log likelihood, BernoulliCounts, BinomialObservation,
  PoissonCounts; Beta–Bernoulli/Binomial and unit-exposure Gamma–Poisson updates.
- Bernoulli posterior predictive, named Beta evidence operations, equal-tailed
  quantile interval, fixed-budget Monte Carlo expectation/standard-error estimate.

This is the recommended target after its prerequisites, not a claim that the
current random and math layers already supply every listed distribution or
consumer API.

### Design now, implement later

- Precision-specific distribution kernels.
- Frequency/reliability weighted variance, M3/M4 and explicit skewness/kurtosis,
  modes/frequencies, geometric/harmonic means, percentile convenience.
- Geometric, Multinomial, Dirichlet, BetaBinomial, NegativeBinomial predictive
  laws, DirichletMultinomial, StudentT/ChiSquared.
- Exposure-aware Gamma–Poisson summaries/evidence and richer loss/MAP helpers.
- Inverse log tails, cancellation-aware interval-mass subtraction, highest-density sets,
  confidence/prediction procedures with their own semantics.
- Deterministic split/counter-based RNGs, buffer sampling, alias tables, adaptive
  stopping with valid diagnostics, ordered parallel reductions.
- Finite discrete entropy/cross-entropy/KL utility contracts in a later
  probability.information module: natural-log units, 0*ln(0)=0, KL(P||Q)
  direction explicit, Q=0 with P>0 gives +infinity. Require a shared outcome
  support/order; continuous differential entropy/divergence is a separate design.

### Explicitly defer

- Dataframes, visualization, general ML/training frameworks and neural networks.
- Regression/optimization/autodiff/tensor/large-scale linear algebra frameworks.
- Probabilistic programming, arbitrary graphical models, MCMC/HMC/NUTS,
  variational inference, Gaussian processes.
- Time-series/survival-analysis/causal/experimental-design systems, distributed
  statistical computing, specialist distribution catalogs.
- Global numeric promotion, distribution parameter refinements/type-level
  probability mathematics, automatic conjugacy, universal Estimator/Simulation
  objects, and language/compiler changes without a concrete unmet contract.
