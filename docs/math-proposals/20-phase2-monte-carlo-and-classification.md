# 20. Monte Carlo, simulation, and classification

Status: recommendation and architectural stress tests. These consumers rely on
the proposed contracts, not a current statistics implementation.

## 20.1 Fixed-budget Monte Carlo

V1 helper: `expectation(distribution, count: n, using: rng, evaluate: f)` in
probability.monte_carlo. It requires Sampleable, a Float-valued callback from
the associated Sample type, and RandomBits. Require 2<=n<=2^53-1, all f outputs
finite. Sample and evaluate serially n times into RunningMoments without
retaining samples.

Result is an immutable `MonteCarloEstimate` with count, mean, sampleVariance,
standardError, and a snapshot of the numerical summary. Standard error estimates
sqrt(sampleVariance/n) under iid finite-variance observations. It is a diagnostic
estimate, not a rigorous error bound or automatic confidence statement. A
constant observed sequence gives zero estimated standard error without proving
zero uncertainty. Heavy tails may make the diagnostic misleading or undefined.

Use a statically checked callback `(Sample)->Float`; both closures and captured
Families fit the existing callable model (E15/E17/E18). A callback may have
effects; it executes exactly once after each draw and in sample order. Failure
does not rewind RNG or external effects. No retries, hidden resampling, or
automatic stopping after an apparently small error.

For a scalar integral over finite a<b, draw Uniform(a,b), estimate E[f(X)], then
scale mean and standard error by b-a using range-aware arithmetic. This is a
composition/example rather than a separate generalized integration framework.
Changing to importance weights requires a new diagnostic contract; self-normalized
importance sampling does not inherit the iid sample-mean error formula.

## 20.2 Bounded simulation

Ordinary functions, explicit RNGs, finite loops, and running summaries suffice.
A multi-variable simulation callback can capture several immutable descriptors
and the same caller-owned RNG, making draw order explicit. Repeated experiments
can use named stream IDs. There is no Simulation<T> hierarchy or simulation DSL.

SampleIterator plus existing map/take/fold supports lazy composition. General
simulation callbacks can aggregate Bool events, count outcomes, or map to Float;
the Monte Carlo helper standardizes only the last case. Retaining traces is a
caller decision and may allocate O(n). Streaming summaries require O(1) state.

Design later adaptive stopping with explicit minimum/maximum budgets and valid
sequential diagnostics. Parallel reductions need task-local streams and a fixed
merge tree; a mergeable summary alone does not establish independent random
streams or scheduler-invariant results. Distributed execution remains deferred.

## 20.3 Naive Bayes belongs above probability

Use classifiers as a stress test and later third-party examples. V1 implements
neither fit/predict framework nor feature matrices. A classifier owns its class
labels, feature schema, smoothing policy, training state, and prediction result.
Probability owns the evaluations and normalization it consumes.

For classes c with priors pi_c and conditionally independent features:

```text
score_c = ln(pi_c) + sum_j logFeatureLikelihood(c,j,x_j)
logPosterior = normalizedLogWeights(scores)
posterior = normalizeLogWeights(scores)
prediction = argmax(score), with declared class-order tie policy
```

| Variant | Foundation used | Classifier-owned decisions |
|---|---|---|
| Bernoulli Naive Bayes | Bernoulli logPmf for every Bool, Beta/count smoothing if selected | Include both present and absent features; schema, smoothing, class priors |
| Multinomial Naive Bayes | Categorical log masses multiplied by nonnegative feature counts; future Multinomial full log mass | Vocabulary/order, token-count interpretation, pseudocount prior |
| Gaussian Naive Bayes | Per-class/per-feature Normal logPdf; RunningMoments for incremental statistics | Variance estimation and positive floor/smoothing, feature independence assumption |

The multinomial coefficient is constant across classes for one fixed feature
count vector, so a classifier may omit it from relative scores. Such scores are
not normalized data likelihoods/evidence. Gaussian class scores are densities
in a common feature space; changing units/reference measures per class breaks
comparison. Never send a Gaussian density through Probability[0,1] validation.

Absent features/missing data must have an explicit classifier policy. Simply
omitting Bernoulli false features changes the model. Zero class priors/log
feature masses produce -infinity; if all classes are impossible, normalization
raises UndefinedNormalizationError. Smoothing is a model choice upstream,
not a foundation fallback to uniform posteriors. Class ordering controls ties
and reproducibility; map iteration order cannot decide the winner silently.

This exercise justifies direct log mass/density, shifted normalization, ordinary
label types, concrete count/moment summaries, and Float results. It does not
justify a classifier API in the foundational package.

## 20.4 Required architecture stress tests

These are analytic acceptance cases for the proposal. They are not executed
tests or claims of current package behavior. Complete Telos usage is in file 21.

| Problem | Expected composition / answer | Failure exposed and resolved |
|---|---|---|
| A: Coin inference | Beta(2,2) + 8 true/2 false -> Beta(10,4); posterior mean 5/7; predictive Bernoulli(5/7) | Typed count summary and Beta return avoid magical Prior/Posterior conversion; future 20-trial predictive is BetaBinomial |
| B: Large binomial P(X>=k) | Binomial(n=1,000,000,p=0.3), survival(k-1), logSurvival(k-1) for k=320,000 | Complement/tail kernel avoids 1-cdf rounding to zero and factorial overflow; exact integer cut preserves off-by-one |
| C: Standard Normal upper tail | survival(10) about 7.619853024e-24; logSurvival(40) about -804.608442 | erfc handles representable tail; direct log tail survives Float probability underflow; ln(survival(40)) fails |
| D: Streaming variance | Values 1e12+1, +2, +3 -> population variance 2/3, sample variance 1, without storing observations | Central moments avoid raw-square cancellation; merge accuracy is tolerance-based |
| E: Monte Carlo pi | Seeded uniform x,y in [0,1); mean of indicator x²+y²<=1, times 4 -> pi; estimated SE scales by 4 | Explicit seed/stream/order, Bool->Float map, O(1) aggregation, no unjustified confidence claim |
| F: Weighted categorical | labels [red,green,blue], weights [1e300,2e300,0] -> masses approximately [1/3,2/3,0] | Max scaling avoids overflow; zero category never drawn; copied table survives caller mutation |
| G: Bayesian classification | scores [-1000,-1001] -> posteriors approximately [0.731058579,0.268941421] | Shifted log normalization avoids exp(-1000) underflow; all -infinity produces explicit error |

Additional composition checks: sampleMany over Categorical<String> returns
List<String>; density-only extensions require no RNG implementation; malformed
Normal parameters fail before publication; empty RunningMoments exposes optional
summary values; a single-pass stream cannot be accidentally reused for a second
statistic. A moments snapshot can support both Monte Carlo and future Gaussian
fitting without introducing competing canonical state.

## 20.5 Performance implications

Reuse one validated descriptor for repeated evaluations. Categorical construction
is O(k) memory/work and draws are O(log k) using a prefix table; a later alias
sampler can make repeated draws O(1) with a distinct consumption/version policy.
Do not rebuild probability arrays per draw. sampleMany necessarily allocates
O(n) outcomes; expectation and running statistics allocate no sample history.
Quantiles explicitly require O(n) materialization.

Generic evidence enables precise checked calls; it does not prove native
monomorphization or allocation-free VM execution. The inspected tests establish
witness transport, not an AOT specialization policy. Performance claims require
measurements in a later implementation, while the API already permits buffered
and specialized kernels without changing distribution semantics.
