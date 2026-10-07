```markdown
# Task: Design Telos Statistics, Probability, Distributions, and Bayesian Foundations — Phase 2

You are acting as a **language/library architect** for Telos.

Your task is to investigate the existing Telos language, standard library, mathematical/numerical facilities, and the Phase 1 numerical architecture, then design a coherent **statistics and probability foundation** for the Telos ecosystem.

This is a **design and architecture task**, not an implementation task.

The objective is not merely to collect statistical functions. The objective is to establish a principled set of abstractions, interfaces, types, algorithms, package boundaries, and shared vocabulary that can support:

- ordinary descriptive statistics;
- streaming/online statistics;
- probability calculations;
- combinatorics needed for probability;
- discrete and continuous probability distributions;
- random sampling;
- likelihoods and log-likelihoods;
- Bayesian priors, posteriors, and updates;
- common conjugate models;
- binomial/Bernoulli/multinomial concepts;
- Monte Carlo estimation;
- simulation;
- Bayesian classification;
- future statistical inference and probabilistic computing libraries.

The resulting architecture should be suitable both for **Telos-native libraries** and for third-party packages that need to interoperate through common statistical and probabilistic abstractions.

Do not assume that conventional APIs from Python, R, Julia, SciPy, Stan, or other ecosystems are automatically appropriate for Telos. Use them selectively as prior art, but design around Telos's actual semantics and advanced type system.

---

# 1. Relationship to Phase 1

Phase 1 concerns the numerical and mathematical foundation, including concepts such as:

- integers;
- rational values;
- floating-point semantics;
- complex numbers;
- exact versus approximate arithmetic;
- elementary mathematical functions;
- special functions;
- numerical classification;
- generic scalar abstractions;
- numeric conversions and promotions;
- stable mathematical operations.

Treat that design as the mathematical substrate for this phase.

Before designing statistical abstractions, determine what Phase 2 **requires from Phase 1**.

Examples may include:

- `log`, `log1p`, `exp`, `expm1`;
- `sqrt`;
- gamma/log-gamma;
- beta/log-beta;
- error functions;
- stable summation;
- combinatorial integer functions;
- machine precision information;
- exact integer arithmetic;
- generic accumulation;
- random number generation.

If Phase 2 exposes missing requirements in Phase 1, record them explicitly as **foundation amendments** rather than silently duplicating them inside statistics packages.

---

# 2. Scope of This Turn

This phase should investigate and design the foundations for:

1. descriptive statistics;
2. measures of location and dispersion;
3. moments;
4. covariance and correlation;
5. weighted statistics;
6. streaming/online statistics;
7. numerically stable statistical algorithms;
8. probability primitives;
9. combinatorics relevant to probability;
10. probability values and probability-domain safety where useful;
11. discrete probability distributions;
12. continuous probability distributions;
13. distribution parameterization;
14. density/mass functions;
15. cumulative distribution functions;
16. survival functions;
17. inverse CDFs / quantiles;
18. moments and summary properties of distributions;
19. random sampling;
20. sample collections;
21. likelihood and log-likelihood;
22. priors and posteriors;
23. Bayesian updating;
24. conjugate Bayesian models;
25. Bernoulli/binomial concepts;
26. categorical/multinomial concepts;
27. Poisson/count models;
28. common continuous distributions;
29. Monte Carlo estimation;
30. bounded simulation infrastructure;
31. Bayesian classifiers, especially the architectural needs exposed by Naive Bayes;
32. APIs needed by future statistical inference libraries;
33. package/module boundaries;
34. language, type-system, compiler, runtime, and standard-library gaps discovered during the work.

These are investigation areas rather than a mandate to expose every concept in V1.

A core responsibility of this task is determining **which abstractions deserve to exist at all**.

---

# 3. Explicitly Deferred Areas

Do not let this phase turn into designing an entire scientific-computing or machine-learning ecosystem.

The following are outside the primary scope:

- dataframe/dataframe-query systems;
- plotting and visualization;
- general machine learning;
- neural networks;
- regression frameworks beyond requirements analysis;
- general optimization;
- automatic differentiation;
- tensor frameworks;
- large-scale linear algebra;
- probabilistic programming languages;
- arbitrary graphical models;
- HMC/NUTS implementation;
- advanced MCMC frameworks;
- variational inference frameworks;
- Gaussian processes;
- time-series frameworks;
- survival-analysis frameworks;
- causal inference;
- hypothesis-testing frameworks beyond foundational requirements;
- full experimental-design systems;
- distributed statistical computing.

You may mention these where they impose constraints on the foundation.

For example, a future inference package may require:

- distributions to expose differentiable log densities;
- parameter support information;
- stable log probability APIs;
- composable likelihoods;
- deterministic RNG splitting;
- reusable sufficient statistics.

Record those as extension requirements without designing the entire subsystem.

---

# 4. Repository Grounding Is Mandatory

Do not design this entirely from generic mathematical knowledge.

Perform a **bounded investigation of the current Telos repository**.

Determine how contemporary Telos actually expresses:

- generic data structures;
- `data` declarations;
- method families;
- overloads;
- generic constraints;
- static parameters;
- domains and other advanced typing mechanisms where relevant;
- packs and heterogeneous structures;
- records and tuples;
- structural matching;
- iterator/iteration facilities;
- collections;
- optional/error/result-like behavior;
- mutable state where relevant;
- numeric operations;
- current `math`;
- current `random`;
- RNG state and random generation;
- callable objects;
- higher-order functions;
- package exports;
- annotations such as constructors;
- pattern matching;
- existing examples of mathematical or generic APIs.

Prefer:

1. current compiler behavior;
2. current tests;
3. recent Telos packages;
4. current implementation;
5. recent architectural decisions;
6. only then older documentation.

Older Telos examples may be obsolete.

Do not invent syntax or semantic capabilities unsupported by repository evidence.

---

# 5. Bound the Repository Investigation

This is not an exhaustive repository audit.

Use a targeted search budget of approximately:

- **15–30 highly relevant files**, plus
- a small number of additional files when required to settle a specific uncertainty.

Prioritize:

- the current `math` implementation;
- the current random-number facilities;
- collection abstractions;
- iterator/fold/reduction facilities;
- generic method-family examples;
- modern type-system tests;
- recent packages demonstrating advanced Telos typing.

Do not recursively inspect entire source trees.

Maintain a concise evidence ledger:

| Area | File / Symbol / Test | Observed Behavior | Design Consequence |
|---|---|---|---|

For important architectural assumptions, establish repository evidence or explicitly mark them as unresolved.

---

# 6. Start From Statistical Semantics, Not API Names

Before producing API sketches, establish the semantic ontology.

Determine which concepts are:

- ordinary values;
- data types;
- generic families;
- semantic constraints;
- capabilities/interfaces;
- algorithms;
- statistical accumulators;
- immutable descriptors;
- random generators;
- parameter objects;
- static type-level information.

Do not create nominal concepts merely because they appear in textbooks.

For example, investigate whether Telos should have abstractions such as:

```text
Distribution
DiscreteDistribution
ContinuousDistribution
Sampleable
Probability
Likelihood
Prior
Posterior
Statistic
Estimator
Accumulator
RandomVariable
Observation
Sample
```

None of these names are requirements.

Each abstraction must earn its existence through concrete generic programming use cases.

---

# 7. Statistics Over Collections

Design ordinary statistical APIs carefully.

At minimum, analyze:

```text
count
sum
mean
weighted_mean

min
max
range

variance
population_variance
sample_variance
standard_deviation

median
quantile
percentile

mode

covariance
correlation

skewness
kurtosis

geometric_mean
harmonic_mean
```

Determine:

- which belong in V1;
- which require sorting;
- which work over generic iterables;
- which require numeric conversion;
- appropriate result types;
- empty-input semantics;
- singleton semantics;
- degrees-of-freedom handling;
- exactness versus floating-point output;
- whether weights need a dedicated abstraction;
- how NaN values should behave;
- whether missing-data policy belongs here or elsewhere.

Do not expose ambiguous APIs such as:

```text
variance(xs)
```

without deciding whether that means population variance or an estimator of sample variance.

Prefer semantics that are explicit and difficult to misuse.

---

# 8. Streaming and Online Statistics

Do not assume that all statistics operate over materialized collections.

Investigate reusable online accumulators for quantities such as:

- count;
- mean;
- variance;
- covariance;
- extrema;
- moments;
- weighted statistics.

Consider algorithms such as Welford-style stable variance accumulation and pairwise/mergeable summaries.

Ask whether APIs should support:

```telos
let stats = RunningStats()

for x in observations {
    stats.push(x)
}

stats.mean
stats.variance
```

and/or immutable/reducer-oriented forms.

Important design questions:

- Can accumulators be merged?
- Are they numerically stable?
- Can they support parallel reductions?
- Should there be a generic `Statistic` or `Accumulator` abstraction?
- Would such an abstraction be useful outside statistics?
- Does Telos already have a reduction abstraction that should be reused instead?

Do not introduce a statistics-specific protocol when an existing general reduction concept is sufficient.

---

# 9. Numerical Stability Is a First-Class Requirement

Statistical APIs must not encode naive textbook formulas when stable alternatives are known.

Investigate and specify appropriate algorithms for:

- means;
- sums;
- variance;
- covariance;
- weighted accumulation;
- probability normalization;
- log probabilities;
- likelihood products;
- tail probabilities;
- binomial probabilities;
- multinomial probabilities;
- normal distribution tails;
- entropy-like quantities where included.

Pay particular attention to:

```text
log-sum-exp
log1p
expm1
log-gamma
log-beta
complementary CDF / survival function
```

Do not define probability calculations solely through naive combinations of factorials, powers, and density formulas when doing so is numerically fragile.

The API should make stable computations straightforward.

---

# 10. Probability Representation

Analyze whether probability values require a dedicated abstraction.

Possible approaches include:

```text
Float
Probability
Probability<Float>
UnitInterval<Float>
```

Do not assume a dedicated `Probability` type is desirable.

Evaluate whether such an abstraction could enforce:

```text
0 <= p <= 1
```

and whether that additional type complexity actually improves real APIs.

Also distinguish:

- probability;
- log probability;
- odds;
- log odds;
- likelihood;
- normalized probability mass.

These are mathematically related but semantically different.

Determine whether Telos should provide helpers such as:

```text
logit
inv_logit
odds
log_odds
```

and whether these belong in foundational probability utilities.

---

# 11. Distribution Architecture

A major output of this phase is the design of probability distributions.

Determine how a distribution should be represented.

Explore candidate APIs conceptually similar to:

```telos
let d = Normal(mean: 0.0, stddev: 1.0)

d.pdf(0.5)
d.log_pdf(0.5)
d.cdf(0.5)
d.quantile(0.95)
d.sample(using: rng)
```

or an alternative more natural to Telos.

Do not assume object-oriented member APIs are correct.

Determine the correct conceptual vocabulary for:

### Discrete distributions

Possible operations:

```text
pmf
log_pmf
cdf
survival
quantile
sample
mean
variance
support
```

### Continuous distributions

Possible operations:

```text
pdf
log_pdf
cdf
survival
quantile
sample
mean
variance
support
```

Investigate whether there should be one common `Distribution` abstraction or smaller orthogonal capabilities such as:

```text
Sampleable
HasCdf
HasQuantile
HasDensity
HasMass
HasMoments
```

Do not construct a giant interface merely for taxonomic neatness.

---

# 12. Distribution Support

A mathematically serious distribution API needs a coherent notion of support.

Examples:

```text
Bernoulli     -> {0, 1}
Binomial      -> integers 0..n
Poisson       -> nonnegative integers
Uniform       -> interval [a, b]
Normal        -> real line
Exponential   -> nonnegative real values
Beta          -> [0, 1]
Gamma         -> positive values
```

Determine whether support should be:

- documentation only;
- runtime metadata;
- a reusable range/set abstraction;
- encoded through types;
- partially encoded through constructors;
- exposed through an explicit `support` API.

Do not overuse advanced typing when runtime parameter values determine the support.

For example, a `Uniform(a, b)` distribution has parameter-dependent support that may not sensibly be encoded in the static classifier.

---

# 13. Distribution Construction and Parameter Validation

Determine behavior for invalid parameters.

Examples:

```text
Normal(stddev: -1)
Binomial(n: -4, p: 0.7)
Beta(alpha: 0, beta: 2)
Uniform(min: 5, max: 2)
Categorical(weights: ...)
```

Possible approaches include:

- validated constructors;
- runtime exceptions;
- result-returning construction;
- separate unchecked internal constructors;
- refinement-like types;
- constructor preconditions.

Use actual Telos error-handling mechanisms.

Do not invent a validation system unsupported by the language.

The public API should make invalid statistical states difficult to create without making ordinary construction cumbersome.

---

# 14. Core V1 Distribution Set

Investigate a bounded foundational set.

Strong candidates include:

### Discrete

- Bernoulli;
- Binomial;
- Categorical;
- Multinomial;
- Poisson;
- Geometric.

### Continuous

- Uniform;
- Normal / Gaussian;
- Exponential;
- Gamma;
- Beta.

Potential later additions:

- Negative Binomial;
- Hypergeometric;
- Chi-squared;
- Student's t;
- F;
- Log-normal;
- Weibull;
- Dirichlet;
- Cauchy;
- Pareto.

Do not include every classical distribution.

Determine which distributions create the smallest useful basis for:

- ordinary probability problems;
- Bayesian conjugacy;
- statistical simulation;
- future inference packages.

Clearly distinguish:

- V1;
- near-term extensions;
- specialist distributions.

---

# 15. Bernoulli, Binomial, Categorical, and Multinomial Semantics

Treat these carefully because they establish the discrete probability vocabulary.

Investigate distinctions among:

```text
Bernoulli
Binomial
Categorical
Multinomial
```

and concepts such as:

```text
Trial
successes
trials
category
counts
probabilities
weights
```

Do not conflate:

- a Bernoulli random variable;
- a single observed Boolean;
- a binomial count;
- a collection of Bernoulli observations.

Determine whether observations should naturally use:

```text
Bool
Int
```

or purpose-specific values.

Prefer interoperability with ordinary Telos values unless a semantic wrapper materially improves correctness.

---

# 16. Sampling and Randomness

Ground sampling architecture in Telos's actual RNG facilities.

Investigate how users should write deterministic code such as:

```telos
let rng = Random(seed: 42)
let normal = Normal(0.0, 1.0)

let x = normal.sample(using: rng)
```

The exact syntax is not predetermined.

Resolve:

- mutable versus immutable RNG state;
- explicit RNG parameters;
- default/global generators, if any;
- deterministic seeding;
- reproducibility;
- stream splitting/forking;
- sampling one value;
- sampling many values;
- iterator/lazy sampling;
- distribution-specific sampling algorithms;
- cryptographic versus simulation randomness.

Do not mix cryptographic security requirements with statistical simulation unless the existing random architecture requires it.

Make reproducibility an explicit design concern.

---

# 17. Distribution Sampling Versus Distribution Evaluation

Separate the concepts of:

```text
sampling
probability evaluation
parameter estimation
inference
```

A distribution should not become a dumping ground for every operation associated with that family.

For example:

```text
Normal(...)
```

may reasonably support density evaluation and sampling.

But determining a maximum-likelihood estimate for Normal parameters may belong in a different statistical layer.

Design clean ownership boundaries.

---

# 18. Likelihood

Likelihood is fundamental to both frequentist and Bayesian inference.

Design a coherent representation or API for computing:

```text
likelihood
log_likelihood
```

given:

- a distribution/model;
- one observation;
- many observations.

For example, conceptually:

```text
log_likelihood(model, observations)
```

or:

```text
model.log_likelihood(observations)
```

Determine whether likelihood deserves a first-class abstraction or whether it is simply a reusable operation.

Avoid representing likelihood as an ordinary probability value: likelihood is a function of parameters conditioned on observed data and does not generally sum/integrate to one over parameter space.

The API and terminology must respect this distinction.

---

# 19. Bayesian Foundation

Design a **bounded Bayesian layer**.

The purpose of V1 is not to build Stan, PyMC, Turing, or a probabilistic programming language.

Instead, establish the concepts needed for straightforward Bayesian calculations.

Investigate:

- priors;
- likelihoods;
- posterior distributions;
- posterior predictive distributions;
- evidence / marginal likelihood where practical;
- conjugate updating;
- Bayesian point estimates;
- credible intervals;
- MAP versus posterior mean;
- Bayesian classification.

Ask whether these require new abstractions or can largely be expressed through ordinary distributions and functions.

Do not invent a generic `Bayes` object without demonstrating a useful semantic role.

---

# 20. Conjugate Bayesian Models

Conjugate models are an excellent stress test because they expose whether the distribution architecture composes naturally.

At minimum, investigate:

### Beta–Bernoulli

```text
prior:
    p ~ Beta(alpha, beta)

observation:
    x ~ Bernoulli(p)

posterior:
    p | data ~ Beta(alpha + successes,
                    beta + failures)
```

### Beta–Binomial

```text
p ~ Beta(alpha, beta)
k ~ Binomial(n, p)
```

### Gamma–Poisson

Potentially:

```text
lambda ~ Gamma(...)
k ~ Poisson(lambda)
```

### Dirichlet–Categorical / Multinomial

Potential future extension if Dirichlet is not in V1.

Use these models to test:

- parameterization;
- observations;
- sufficient statistics;
- update APIs;
- posterior return types;
- batch versus incremental updates;
- predictive distributions.

Determine whether a reusable conjugacy abstraction is justified or whether explicit typed update functions are clearer.

---

# 21. Sufficient Statistics

Investigate whether sufficient statistics deserve explicit representation.

For Beta–Bernoulli, for example, the entire observation set can be reduced to:

```text
successes
failures
```

For Normal-family inference, useful sufficient statistics may include:

```text
count
sum
sum_of_squares
```

or stable equivalent accumulators.

Potential advantages include:

- streaming Bayesian updates;
- reduced memory use;
- mergeable computations;
- distributed calculations;
- clearer conjugate inference.

Do not generalize prematurely.

If there is a useful common abstraction, demonstrate at least two or three concrete models benefiting from it.

---

# 22. Bayesian Updating API

Explore what good Telos usage should look like.

For example, compare conceptual approaches such as:

```telos
let prior = Beta(2.0, 2.0)

let posterior = update(
    prior,
    BernoulliObservations(successes: 8, failures: 2)
)
```

versus:

```telos
let posterior = prior.observe(
    Bernoulli,
    successes: 8,
    failures: 2,
)
```

versus model-specific functions.

Do not choose syntax before understanding the semantic structure.

Prefer explicit, composable APIs over magical inference based on runtime type inspection.

---

# 23. Credible Intervals and Statistical Intervals

Establish terminology carefully.

A Bayesian credible interval is not the same thing as a frequentist confidence interval.

If interval helpers are introduced, their semantics must be explicit.

Potential concepts include:

```text
credible_interval
equal_tailed_interval
highest_density_interval
confidence_interval
prediction_interval
```

Do not collapse these under a generic `interval` function.

For V1, it may be appropriate to implement only distribution quantiles and defer higher-level interval constructs.

Make that decision explicitly.

---

# 24. Frequentist Statistical Inference

This phase should identify the requirements for future inferential statistics without necessarily implementing a large inference package.

Investigate foundations needed later for:

- standard errors;
- confidence intervals;
- hypothesis tests;
- test statistics;
- p-values;
- estimators;
- sampling distributions.

Determine what V1 must expose now to avoid future architectural incompatibility.

Do not design an exhaustive hypothesis-testing library in this phase unless necessary for the architecture.

---

# 25. Estimators

Determine whether Telos needs a generic concept of an estimator.

Potential distinction:

```text
Statistic:
    function of observed data

Estimator:
    statistic used to estimate an unknown parameter

Estimate:
    realized value produced by an estimator
```

These distinctions are mathematically meaningful, but they may not warrant runtime abstractions.

Investigate actual generic use cases before introducing types named `Statistic`, `Estimator`, or `Estimate`.

---

# 26. Monte Carlo Foundations

Design a bounded Monte Carlo layer sufficient for ordinary numerical estimation and simulation.

At minimum investigate:

```text
Monte Carlo expectation
Monte Carlo integration
repeated simulation
sample mean estimation
variance / standard error estimates
confidence diagnostics
```

Conceptually:

```telos
let estimate = monte_carlo(
    samples: 100_000,
    using: rng,
) {
    let x = distribution.sample(using: rng)
    f(x)
}
```

The exact API is not prescribed.

Consider:

- deterministic RNG ownership;
- reproducibility;
- aggregation without storing every sample;
- parallelizability;
- mergeable accumulators;
- estimation error;
- callback/generator ergonomics;
- multi-variable simulation;
- stopping criteria.

Keep V1 modest.

Do not design a large simulation DSL unless repository evidence suggests Telos has unusually strong facilities that justify one.

---

# 27. Generic Simulation

Determine whether simulation requires its own abstraction or is naturally expressed through:

- functions;
- iterators;
- generators;
- RNGs;
- distributions;
- reducers.

A good result may simply be that Telos already has sufficient compositional primitives.

Avoid creating types such as `Simulation<T>` merely because the name sounds appropriate.

Demonstrate concrete benefits before adding abstraction.

---

# 28. Bayesian Classification

Use Bayesian classification as an architectural stress test.

In particular, investigate a basic **Naive Bayes** classifier.

Relevant variants include:

- Bernoulli Naive Bayes;
- Multinomial Naive Bayes;
- Gaussian Naive Bayes.

This is not a request to design a general ML framework.

The goal is to see whether the probability/statistics foundation naturally supports:

- class priors;
- conditional feature distributions;
- likelihood accumulation;
- log-domain probability calculations;
- posterior normalization;
- categorical labels;
- incremental fitting;
- prediction.

A conceptual usage might resemble:

```telos
let model = NaiveBayes.fit(training_data)

let prediction = model.predict(features)
let probabilities = model.predict_proba(features)
```

but do not assume this is the correct Telos API.

Determine whether Naive Bayes belongs:

- in the statistics package;
- in a later ML package;
- or merely as an example proving the foundation is sufficient.

The likely result is that classifier implementations belong above the foundational probability layer.

---

# 29. Log-Domain Probability

This requires explicit treatment.

Repeated multiplication of probabilities is numerically fragile.

The architecture should support common operations such as:

```text
log probability
log likelihood
log-sum-exp
normalization from log weights
softmax-style normalization
```

Investigate whether Telos should provide an abstraction such as a log-domain probability value, or whether ordinary `Float` plus strongly named functions is better.

Compare the ergonomic and safety implications.

A dedicated wrapper may prevent accidental mixing of:

```text
p
log(p)
```

but may also create substantial operator complexity.

Do not decide based on aesthetics alone.

---

# 30. Weighted and Categorical Probability

Categorical sampling often begins from unnormalized weights rather than probabilities.

Distinguish:

```text
probabilities
weights
log_weights
```

Determine what APIs should accept each.

A useful design should make operations such as:

```text
normalize(weights)
normalize_log_weights(log_weights)
sample_weighted(...)
```

numerically robust and semantically clear.

Avoid requiring users to manually divide by `sum(weights)` everywhere.

---

# 31. Entropy and Information-Theoretic Utilities

Investigate whether basic information-theoretic functions belong in this layer.

Potential candidates:

```text
entropy
cross_entropy
KL divergence
```

These are useful for:

- distributions;
- Bayesian inference;
- information theory;
- machine learning.

However, they may belong in a later package.

Determine the correct boundary rather than automatically including them.

If included, specify directionality and domain semantics carefully, particularly for KL divergence.

---

# 32. Combinatorics Needed by Probability

Probability APIs frequently need exact combinatorial operations.

Investigate foundational operations such as:

```text
factorial
falling_factorial
rising_factorial

choose / binomial_coefficient
permutations
combinations

log_factorial
log_binomial_coefficient
```

Determine which belong in Phase 1 `math` versus Phase 2 probability utilities.

Exact integer operations and floating log-domain counterparts should not be conflated.

For example:

```text
choose(100, 20)
```

should ideally produce an exact integer when the integer model permits it, while:

```text
log_choose(1_000_000, 200_000)
```

serves a different numerical purpose.

---

# 33. Distribution Parameterization Must Be Explicit

Many distributions have multiple common parameterizations.

Examples:

### Normal

```text
mean + standard deviation
mean + variance
precision
```

### Gamma

```text
shape + scale
shape + rate
```

### Geometric

Whether support starts at:

```text
0
```

or:

```text
1
```

### Negative Binomial

Multiple conventions exist.

Do not expose ambiguous constructors.

Choose canonical parameterizations and provide clearly named alternatives where useful.

Repository/API documentation should make parameter semantics mathematically explicit.

---

# 34. Statistical Result Types

Determine when a statistical function should return:

- the input scalar type;
- a floating result;
- an exact rational;
- a wider accumulator type;
- an optional result;
- an error/result;
- a structured result.

Examples requiring careful treatment:

```text
mean([Int])
variance([Int])
median([Int])
quantile([Int], 0.5)
correlation(...)
```

Do not casually force every operation into `Float` without analyzing exactness, ergonomics, and interoperability.

At the same time, do not make basic statistics unusably generic solely to preserve theoretical exactness.

Seek a coherent policy.

---

# 35. Missing Data and NaN Policy

Determine whether V1 statistics should define behavior around:

- NaN;
- infinities;
- absent values;
- missing values.

Do not automatically invent an R-style missing-value system.

Possible approaches include:

- statistics operate only on values actually supplied;
- users filter optionals before calling;
- NaN propagates according to numeric semantics;
- explicit `skip_nan` helpers;
- policy arguments.

Prefer simple compositional behavior unless stronger semantics are justified.

---

# 36. Type-Level Opportunities

Telos has an advanced type system. Use it where it improves APIs materially.

Investigate opportunities such as:

- distinguishing distribution sample types;
- preserving return distribution types after conjugate updates;
- parameterized observations;
- statically distinguishing discrete versus continuous distributions;
- generic sampling over result type;
- heterogeneous composition where useful;
- statically deriving result types from distribution families.

However:

**Do not type-theoretically encode mathematics merely because Telos can.**

Avoid designs that require users to understand elaborate type machinery for ordinary statistics.

The best type-system use should remove ambiguity, prevent misuse, or improve inference without burdening common code.

---

# 37. Distribution Type Relationships

Investigate questions like:

```text
What is the relationship between Distribution<T> and T?
```

For example:

```text
Bernoulli -> Bool?
Bernoulli -> Int?
Binomial -> Int
Normal -> Float
Categorical<T> -> T
```

A generic `Distribution<T>` may be attractive, but determine whether Telos can express it naturally and whether it captures enough semantic information.

Similarly investigate whether something like:

```text
DiscreteDistribution<T>
ContinuousDistribution<T>
```

is useful or over-taxonomized.

Show actual generic algorithms that would consume these abstractions.

---

# 38. Generic Algorithms Must Justify Interfaces

For each proposed interface/capability, provide at least one useful generic function.

For example, if proposing a `Sampleable<T>` concept, demonstrate something like:

```text
sample_many(distribution, n, rng)
```

If proposing a CDF capability, demonstrate a generic quantile or tail-analysis operation that benefits from it.

If proposing `Distribution<T>`, demonstrate how it enables more than merely attaching a name to unrelated methods.

Delete abstractions whose only purpose is taxonomy.

---

# 39. External Prior Art — Bounded and Selective

Study relevant ecosystems where useful.

Potential references:

- Julia `Distributions.jl` and `StatsBase`;
- Python `statistics`, NumPy, SciPy, `statistics`, scikit-learn;
- R's probability/distribution conventions;
- Rust crates such as `rand`, `rand_distr`, and `statrs`;
- Haskell probability/statistics libraries;
- Stan's distribution semantics;
- Boost.Math;
- Apache Commons Math;
- probabilistic programming systems only where they reveal useful abstraction boundaries.

Do not write a language/library survey.

For each prior-art system, extract only specific architectural lessons.

Useful questions include:

- How are distributions represented?
- How are random sampling and probability evaluation separated?
- Are log probability methods first-class?
- How are parameter errors handled?
- How are continuous/discrete distinctions represented?
- How do APIs handle reproducibility?
- How are sufficient statistics modeled?
- What abstractions proved too broad or too rigid?

Aim for no more than roughly **4–6 ecosystems** unless a concrete unresolved issue warrants another comparison.

---

# 40. Package Architecture

Develop a bounded package layout.

Possible concepts might include:

```text
statistics
probability
distributions
random
simulation
bayes
```

These names are not prescribed.

Avoid both extremes:

### Bad extreme A

A single enormous `math` namespace containing everything.

### Bad extreme B

A package for every mathematical noun.

Determine sensible dependency direction.

For example, an architecture could conceptually resemble:

```text
math
  ↓
random
  ↓
statistics
  ↓
probability / distributions
  ↓
bayes / simulation
```

But do not adopt this without investigating actual dependency needs.

In particular, distributions often depend simultaneously on:

- math;
- random sampling;
- probability utilities.

Avoid dependency cycles.

---

# 41. Candidate V1 User Experience

The final proposal must contain substantial usage examples.

Test common workflows such as:

## Descriptive statistics

```telos
let xs = [1.2, 4.0, 3.8, 7.1]

mean(xs)
standard_deviation(xs)
quantile(xs, 0.95)
```

## Streaming statistics

```telos
let stats = RunningStats()

for x in stream {
    stats.push(x)
}

stats.mean()
stats.standard_deviation()
```

## Distribution calculations

```telos
let normal = Normal(mean: 0.0, stddev: 1.0)

normal.pdf(1.0)
normal.log_pdf(1.0)
normal.cdf(1.96)
normal.quantile(0.975)
```

## Reproducible sampling

```telos
let rng = Random(seed: 42)

let samples = normal.sample(1000, using: rng)
```

## Binomial probability

```telos
let d = Binomial(trials: 20, probability: 0.3)

d.pmf(5)
d.cdf(5)
```

## Bayesian update

```telos
let prior = Beta(alpha: 2.0, beta: 2.0)

let posterior = ...
```

## Monte Carlo

```telos
let result = monte_carlo(...) {
    ...
}
```

Use syntax confirmed by the repository wherever possible.

If examples rely on proposed syntax or compiler extensions, label them explicitly.

---

# 42. Stress-Test the Architecture

Test the proposed abstractions against realistic tasks.

At minimum include:

### Problem A — Coin inference

Start from:

```text
Beta prior
+
Bernoulli observations
```

and derive a posterior.

### Problem B — Binomial probability

Compute:

```text
P(X >= k)
```

robustly for a large `n`.

### Problem C — Normal tail

Compute a very small upper-tail probability without catastrophic cancellation.

### Problem D — Streaming variance

Process a large observation stream without retaining values and without naive cancellation.

### Problem E — Monte Carlo π / integral / expectation

Use deterministic seeded randomness.

### Problem F — Categorical sampling

Sample categories from unnormalized weights.

### Problem G — Bayesian classification

Compute class posterior scores using log probabilities without underflow.

For each, inspect whether the API feels composable and whether the types clarify or obstruct the code.

---

# 43. Error and Edge-Case Semantics

Document behavior for statistical corner cases.

Examples:

```text
mean([])
variance([])
sample_variance([x])

quantile([], q)
quantile(xs, -0.1)

Normal(stddev: 0)
Beta(alpha: -1, beta: 2)

Bernoulli(p: NaN)
Bernoulli(p: 1.2)

Categorical([])
Categorical([0, 0, 0])

sample(distribution, count: -1)
```

Do not leave these as "implementation details."

Separate:

- invalid construction;
- undefined statistical quantity;
- empty input;
- numerical exceptional condition;
- impossible event;
- zero probability.

---

# 44. Performance and Allocation

The architecture must not require unnecessary allocation.

Consider:

- iterators instead of materialized collections;
- streaming reducers;
- reusable distribution values;
- batched sampling;
- avoiding temporary probability arrays;
- generic monomorphization where Telos supports it;
- immutable parameter objects;
- efficient categorical sampling structures where repeated sampling is expected.

However, do not prematurely optimize specialist workloads.

Design obvious extension points for optimized implementations.

---

# 45. Reproducibility and Determinism

Probability and simulation APIs require more explicit reproducibility semantics than ordinary math APIs.

Investigate:

- seeded generators;
- deterministic algorithm choice;
- platform reproducibility;
- random stream splitting;
- concurrency;
- iteration-order dependence;
- whether distribution samplers may change algorithms across library versions.

Distinguish:

- deterministic given the same runtime/library version;
- bit-for-bit stable across versions;
- statistically equivalent but not sequence-identical.

Do not promise stronger reproducibility than the architecture can realistically guarantee.

---

# 46. Language / Standard-Library Gap Ledger

Maintain a separate ledger of gaps discovered during the work.

Classify each as:

1. **stdlib gap**
2. **typing gap**
3. **language gap**
4. **compiler/runtime gap**
5. **ergonomic gap**

Potential examples may include:

- insufficient iterator/reduction support;
- inability to express generic distribution result types;
- weak constructor validation mechanisms;
- missing numeric functions;
- missing random stream control;
- insufficient overload resolution;
- inability to express associated sample types cleanly.

Do not turn every inconvenience into a compiler feature.

For every proposed language/compiler enhancement explain:

- the concrete statistical API requiring it;
- why existing mechanisms are inadequate;
- the minimum useful change;
- whether V1 blocks on it.

---

# 47. Design Criteria

Evaluate major decisions against:

| Criterion | Question |
|---|---|
| Mathematical correctness | Does the API reflect correct probability/statistical semantics? |
| Numerical robustness | Does it avoid avoidable instability and underflow? |
| Telos-native design | Does it exploit actual Telos capabilities rather than copy another language? |
| Ergonomics | Are common statistical tasks concise and understandable? |
| Type precision | Can invalid or nonsensical combinations be rejected where useful? |
| Generic composability | Can third-party distributions and algorithms interoperate? |
| Reproducibility | Are RNG behavior and ownership explicit? |
| Extensibility | Can future inference and simulation packages build on this foundation? |
| Performance | Can common workloads avoid unnecessary allocation and dispatch? |
| Minimality | Does each abstraction justify its existence? |
| Discoverability | Can users predict where operations belong? |
| Implementability | Can contemporary Telos implement this without speculative compiler work? |

When criteria conflict, explain the tradeoff.

---

# 48. Avoid These Failure Modes

Do **not**:

- reproduce SciPy/R APIs mechanically;
- make every textbook concept a runtime type;
- build a giant inheritance-style distribution hierarchy;
- assume Telos has Rust-style traits;
- invent unsupported syntax;
- use naive probability formulas where stable versions are known;
- multiply long sequences of probabilities directly;
- conflate probability with likelihood;
- conflate confidence intervals with credible intervals;
- conflate observations with distributions;
- treat sampling and density evaluation as the same capability;
- hide RNG state behind uncontrolled global randomness;
- design a probabilistic programming language in this phase;
- build a full ML framework around Naive Bayes;
- create `Distribution`, `Statistic`, `Estimator`, or `Simulation` abstractions without demonstrated generic value;
- encode runtime distribution parameters excessively at the type level;
- put every special function into the statistics package;
- overfit the architecture to Normal distributions;
- optimize solely for mathematical elegance at the expense of practical use.

---

# 49. Bounded Reasoning Procedure

Use a disciplined reasoning process.

## Stage A — Establish the substrate

Determine the actual current state of:

- Phase 1 numerical facilities;
- Telos random facilities;
- collections;
- iteration;
- generic typing;
- constructors;
- errors;
- method families.

Produce a concise evidence ledger.

---

## Stage B — Identify core use cases

Start from concrete workflows rather than abstraction names.

At minimum:

1. descriptive statistics over a collection;
2. streaming statistics;
3. binomial probability calculation;
4. continuous distribution evaluation;
5. deterministic sampling;
6. Bayesian conjugate update;
7. Monte Carlo expectation;
8. categorical weighted sampling;
9. log-domain Bayesian classification.

Derive requirements from these.

---

## Stage C — Develop alternative architectures

For major questions, compare credible alternatives.

At minimum evaluate alternatives for:

- `Distribution` abstraction;
- discrete/continuous separation;
- sample-type representation;
- probability representation;
- log probability representation;
- RNG ownership;
- statistical reducers;
- Bayesian updating;
- package structure.

Do not generate artificial alternatives for trivial decisions.

---

## Stage D — Stress-test

Apply candidate architectures to the concrete workflows.

Look actively for:

- type explosion;
- unnecessary wrappers;
- numeric instability;
- allocation;
- ambiguous parameterization;
- awkward generic signatures;
- impossible extension by third-party distributions;
- dependency cycles;
- hidden RNG state;
- runtime errors that could cheaply be prevented.

---

## Stage E — Converge

Produce one primary recommended architecture.

Where uncertainty remains, clearly state:

- the unresolved question;
- the viable alternatives;
- what evidence would decide between them.

Avoid ending with several equally weighted designs.

---

# 50. Required Final Deliverable

Produce a **Phase 2 architecture and library design report**, not an implementation plan.

Structure it approximately as follows:

1. **Executive architectural recommendation**
2. **Repository findings**
3. **Dependencies on and amendments to Phase 1**
4. **Statistical/probabilistic design principles**
5. **Proposed ontology and shared vocabulary**
6. **Collection and streaming statistics**
7. **Numerical stability requirements**
8. **Probability representation**
9. **Distribution architecture**
10. **Discrete distributions**
11. **Continuous distributions**
12. **Distribution support and parameter validation**
13. **Sampling and RNG integration**
14. **Likelihood and log-likelihood**
15. **Bernoulli/binomial/categorical/multinomial design**
16. **Bayesian foundation**
17. **Conjugate Bayesian updates**
18. **Sufficient statistics**
19. **Monte Carlo and simulation foundation**
20. **Bayesian-classifier stress test**
21. **Combinatorial probability utilities**
22. **Information-theoretic utilities, if justified**
23. **Statistical result-type policy**
24. **Error and edge-case semantics**
25. **Package/module architecture**
26. **Third-party extension model**
27. **Detailed Telos API sketches**
28. **Complete usage examples**
29. **Language/compiler/stdlib gaps**
30. **Rejected alternatives**
31. **V1 boundary**
32. **Future phases**
33. **Open architectural questions**

---

# 51. V1 Boundary Is Mandatory

End with a decisive classification.

## Implement in V1

Identify the smallest high-value foundation that Telos should implement immediately.

Likely candidates to investigate include:

- basic descriptive statistics;
- numerically stable running statistics;
- probability helpers;
- foundational distributions;
- sampling;
- log-probability APIs;
- Bernoulli/binomial/categorical concepts;
- limited conjugate Bayesian updates;
- basic Monte Carlo helpers.

Do not assume all belong in V1.

## Design now, implement later

Identify concepts whose interfaces must be anticipated now even if implementation can wait.

Potential examples:

- Dirichlet;
- Student's t;
- advanced Bayesian helpers;
- generalized estimators;
- additional Monte Carlo machinery.

## Explicitly defer

Identify functionality that should remain out of the foundational package.

Examples may include:

- MCMC frameworks;
- HMC/NUTS;
- probabilistic programming;
- generalized linear models;
- full ML;
- causal inference;
- large-scale statistical modeling.

This boundary should prevent Phase 2 from becoming an unbounded statistics ecosystem project.

---

# 52. Evidence and Confidence

For claims about current Telos behavior, cite:

- repository files;
- symbols;
- tests;
- current documentation where authoritative.

Clearly distinguish:

- **observed current behavior**;
- **existing architectural intent**;
- **Phase 1 assumptions**;
- **new Phase 2 proposal**;
- **future-compatible speculation**.

Where repository evidence is contradictory, report the contradiction rather than silently choosing one interpretation.

---

# 53. Quality Bar

The goal is not "a stats package."

The goal is a **small, coherent statistical and probabilistic foundation that can become shared vocabulary across the Telos ecosystem**.

It should make simple things simple:

```text
mean(xs)
Normal(0, 1).cdf(x)
Binomial(n, p).pmf(k)
```

while providing enough semantic and numerical discipline for serious code:

```text
stable online variance
log-likelihood accumulation
tail probabilities
reproducible sampling
Bayesian conjugate updates
Monte Carlo estimation
third-party distribution implementations
```

Favor a compact set of powerful composable abstractions over a broad catalog of unrelated functions.

Use Telos's advanced type system where it materially improves correctness or developer ergonomics, but resist type-theoretic ornamentation.

Challenge conventional statistical-library design where appropriate.

If an abstraction common in other languages is unnecessary in Telos, remove it.

If Telos enables a materially stronger design, exploit that capability.

The resulting report should be sufficiently concrete and architecturally resolved that a later phase can turn it into a detailed implementation plan without reopening the foundational semantic questions.
```