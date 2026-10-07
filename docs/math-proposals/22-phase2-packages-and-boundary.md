# 22. Packages, gaps, alternatives, and V1 boundary

Status: recommendation. This closes the Phase 2 architecture report. The report
is a design deliverable; no implementation completion or behavioral certification
is implied by its V1 list.

## 22.1 Bounded package architecture

Arrows below mean “imports/depends on,” not execution order:

```text
statistics ---------------------> universe.math, universe.collections
probability.distributions ------> universe.math, universe.math.special,
                                  universe.random, universe.collections
probability.probability_helpers -> universe.math, universe.collections
probability.bayes ---------------> probability.distributions,
                                  universe.math.special, universe.collections
probability.monte_carlo ---------> probability sampling capabilities,
                                  statistics, universe.random
```

`statistics` never imports probability or random. Distribution evaluation does
not import statistics; formulas for model moments remain distribution-owned.
`universe.random` supplies bit generation, uniform helpers, and future entropy
adapters, and never imports probability. `universe.math.special` is optional
numerical infrastructure and never imports consumer packages.

Two optional packages with public facades are sufficient:

```text
statistics/package.ph            descriptive Families, summaries, errors
statistics/descriptive.ph        hidden terminal owner
statistics/moments.ph            concrete streaming owners/snapshots

probability/package.ph           capabilities, descriptors, normalization, sampling
probability/capabilities.ph      shared exported contracts
probability/distributions/       hidden family implementations
probability/probability_helpers.ph hidden weight/log utility owner
probability/sampling.ph          sampleMany and sample stream owner
probability/bayes/package.ph     public bounded conjugate/interval facade
probability/monte_carlo/package.ph public expectation facade
```

This is proposed topology, not new manifests. Importing probability's root must
not implicitly load Monte Carlo/statistics or Bayesian helpers. Hidden children
follow E02/E10 facade precedent. Shared errors belong to their owning package;
NumericalRangeError/EvaluationError should be math-owned and re-exported where
needed, rather than duplicate incompatible error identities.

The random stub's annotation mentions distributions. Amend that description
when implementation is authorized: RNG mechanics remain in random; probability
owns probability laws and their samplers. This report makes no source edit to
the stub. Namespace details may adapt to current package loading, while these
dependency/ownership boundaries remain fixed recommendations.

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
| G1 stdlib | random stub E05 cannot feed any sampler | RandomBits + pinned Pcg32 + unbiased uniform helpers | Yes, sampling/Monte Carlo |
| G2 stdlib | E02–E03 lack gamma/beta/incomplete/tail functions | A2's bounded math.special and accuracy contracts | Yes, full recommended distribution evaluation |
| G3 stdlib | Fold E07 is not a stable numerical sum/moment kernel | A1 stable reductions plus concrete moment owners | Yes, numerical stats guarantees |
| G4 stdlib | Current IntegerMath E04 has no choose/factorial | A3 exact/log combinatorial helpers | Exact helpers separate; log kernels needed for probability |
| G5 stdlib | No inspected sort API; quantile must order a copy | Private sort/selection now; optional general collection sort later | No compiler blocker |
| G6 typing | New generic Sampleable method quantified over RNG and associated sample returned through exported Families | Exercise existing canonical evidence/projection machinery; preserve the declared contract | Verification prerequisite later; no new type feature assumed |
| G7 language | No supertraits/trait objects, E25 | Use independent nominal traits and concrete/callable adapters | No; no enhancement proposed |
| G8 compiler/runtime | No cross-platform bitwise transcendental guarantee, Phase 1 file 05 | Document backend sensitivity; deterministic backend only separately funded | No for version/platform-scoped sampling |
| G9 ergonomic | Validated data factory is more verbose than direct construction, E24 | Explicit .new/tryNew; preserve inaccessible raw constructor | No new constructor validation syntax |
| G10 stdlib | No nextDown/machine metadata in inspected Float surface | Scalar-owned representability helper and constants, Phase 1 extension | nextDown needed for Uniform rounding policy |
| G11 stdlib | No dedicated split/counter-based generator | Later source-defined generator/capability with explicit stream laws | No, serial explicit streams suffice |
| G12 typing/ergonomic | Heterogeneous feature products do not imply homogeneous Float iteration, E27–E28 | User-authored extraction/normalization callback; later schema libraries | No |
| G13 stdlib | Exact statistical means would need Rational | Keep named future meanExact; approximate Float API is explicit | No |

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

Continue numeric-foundation work on Rational/Complex independently. Probability
follow-ups may add Geometric/Multinomial, Dirichlet and conjugate predictive
laws, StudentT/ChiSquared, weighted-variance models and higher moments. Future
inference can then design estimators, confidence procedures, hypothesis tests,
and differentiable log evaluators on the established measure/tail contracts.
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
- Independent RandomBits, Sampleable, HasMass, HasDensity, HasOrderedTails, and
  HasQuantile contracts; no hierarchy or existential Distribution.
- Bernoulli, Binomial, Categorical, Poisson, Uniform, Normal, Exponential, Gamma,
  Beta; validated immutable parameters, direct evaluation/log/tail methods, and
  explicit seeded sampling using pinned Pcg32.
- sampleMany and lazy SampleIterator; no hidden default generator.
- iid mass/density log likelihood, BernoulliCounts, BinomialObservation,
  PoissonCounts; Beta–Bernoulli/Binomial and unit-exposure Gamma–Poisson updates.
- Bernoulli posterior predictive, named Beta evidence operations, equal-tailed
  quantile interval, fixed-budget Monte Carlo expectation/standard-error estimate.

This is the recommended target after its prerequisites, not a claim that the
current random stub and elementary math suffice to ship it immediately.

### Design now, implement later

- Rational-returning meanExact, precision-specific distribution kernels.
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
