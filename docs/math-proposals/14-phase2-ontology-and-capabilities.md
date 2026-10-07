# 14. Ontology and capability architecture

Status: recommendation. Syntax precedents are E15–E19 and E24–E26 in file 12.
Every trait/member name below is a proposed library addition.

## 14.1 Shared vocabulary

| Concept | Representation and semantic role |
|---|---|
| Distribution | Immutable parameterized description of a probability law; concrete data/class value, no universal nominal root |
| Sample | One ordinary outcome value; no Sample<T> wrapper |
| Observation | Value supplied to an evaluation/update; often same type as sample, but a tail cut and a sufficient statistic can differ |
| Sample collection | List<T>, existing Iterable, or a single-pass Iterator |
| Parameter | Ordinary runtime scalar/collection; shape/rate/count are validated values |
| Probability | Finite Float in [0,1] at checked boundaries |
| Log probability | Float <= 0, including negative infinity; separate from log density |
| Density | Nonnegative Float, possibly greater than 1 or positive infinity at an endpoint |
| Likelihood | Function of parameters for fixed observations; realized evaluation is an unconstrained nonnegative scalar |
| Prior / posterior | Roles of a distribution in an explicit inference calculation |
| Statistic / estimator | Ordinary function/callable over observations; no runtime taxonomy |
| Running moments | Mutable class with stable numerical state and immutable summary snapshots |
| Sufficient statistics | Validated concrete data for a specific observation model |
| RNG | Mutable identity-bearing object satisfying a uniform-bit capability |
| Support | Runtime domain metadata or a concrete membership operation |
| Random variable | Mathematical documentation concept; no executable graph node in V1 |
| Static information | Associated sample/observation/result types and trait evidence; runtime parameters stay outside the type classifier |

## 14.2 Independent capabilities that earn their existence

Contract sketch, proposed library declarations in current syntax:

```telos
trait RandomBits {
  nextBits32 -> Int
}

trait Sampleable {
  type Sample
  sample<R>(using rng: R) -> Self::Sample where R |= RandomBits
}

trait HasMass {
  type Observation
  pmf(_ value: Self::Observation) -> Float
  logPmf(_ value: Self::Observation) -> Float
}

trait HasDensity {
  type Observation
  pdf(_ value: Self::Observation) -> Float
  logPdf(_ value: Self::Observation) -> Float
}

trait HasOrderedTails {
  type Cut
  cdf(_ cut: Self::Cut) -> Float
  survival(_ cut: Self::Cut) -> Float
  logCdf(_ cut: Self::Cut) -> Float
  logSurvival(_ cut: Self::Cut) -> Float
}

trait HasQuantile {
  type Quantile
  quantile(_ probability: Float) -> Self::Quantile
  upperQuantile(_ upperProbability: Float) -> Self::Quantile
}
```

These are independent nominal contracts. `HasQuantile` does not inherit tails;
an algorithm needing both asks for both. Method names and labels are selector
identity, not runtime type overload keys. Types constrain checked calls and
selected witnesses. There is no `dyn Distribution` or trait-valued list.

| Capability | Concrete generic consumer | Required semantic law |
|---|---|---|
| RandomBits | Uniform integer rejection and Float unit draws | Returned Int lies in [0,2^32), with uniform-bit source semantics; PRNG quality documented separately |
| Sampleable | sampleMany / SampleIterator / Monte Carlo expectation | Returned values follow the descriptor's law up to specified RNG/numerical approximation; consumes supplied RNG only |
| HasMass | iidLogMassLikelihood over Iterable<Observation> | Normalized counting measure; logPmf directly handles zero as -infinity |
| HasDensity | iidLogDensityLikelihood and Gaussian classifier feature score | Named reference measure; log density can be positive |
| HasOrderedTails | tailDiagnostics returning lower/upper probabilities and direct logs for one cut; future calibrated p-value calculations | F(c)=P(X<=c), S(c)=P(X>c), direct logs |
| HasQuantile | Equal-tailed interval and quantile-based prediction limits | Generalized inverse / explicit endpoint policy; upperQuantile accepts a small upper-tail probability directly |

Do not add HasMoments now. Concrete distribution mean/variance getters are
useful, but finite, divergent, undefined, vector-valued, and numeric embeddings
need different result contracts. Likewise no Support trait is required by the
listed generic V1 algorithms. Concrete support metadata suffices.

`tailDiagnostics<D>(d, at: cut)` derives the cut type from
`<D as HasOrderedTails>::Cut` and returns an ordinary labeled tuple of cdf,
survival, logCdf, and logSurvival results. It gives tail-reporting consumers one
generic entry point without reconstructing a complement or losing a small log
tail. This is a library helper, not another capability or a proof-producing API.

## 14.3 Associated sample type versus Distribution<T>

Choose `Sampleable::Sample` because the sample type belongs to the conforming
descriptor. Generic consumers derive it, as Iterable derives Item. Binomial
binds Int, Normal binds Float, Bernoulli binds Bool, and Categorical<T> binds T.
`HasMass::Observation`, `HasOrderedTails::Cut`, and `HasQuantile::Quantile` are
separate associations; one universal T would hide meaningful differences.

`Sampleable<T>` is a viable alternative, demonstrated by Hypothesis Strategy<T>.
It is less convenient for a helper whose output must be inferred solely from its
distribution argument. Associated Sample expresses one canonical output per
descriptor. A deliberately convertible sampler can be a separate mapped
descriptor. No associated type silently converts observations or changes
runtime class identity.

## 14.4 Alternatives and tradeoffs

| Question | Alternatives | Primary choice and consequence |
|---|---|---|
| Distribution abstraction | Giant nominal root; one structural method bundle; independent explicit traits | Independent traits admit sample-only and evaluation-only extensions; no universal distribution registry |
| Discrete/continuous distinction | Inheritance tags; one ambiguous pdf; mass/density capabilities | Distinct pmf/pdf names preserve measure semantics; mixed measures need a later explicit contract |
| Sample type | Generic Distribution<T>; associated Sample; runtime tags | Associated Sample gives precise sampleMany return types without classifier inspection |
| Probability values | Float; validated wrapper; static unit-interval domain | Float plus boundary validation in V1; ordinary arithmetic does not preserve an invariant automatically |
| Log values | Float; LogProbability operator algebra; one log-density wrapper | Strong names and Float, retaining +infinity/log density; avoids a false <=0 bound on all scores |
| RNG ownership | Global; immutable state threading; mutable explicit object | Explicit state matches class/iterator precedent; aliases share stream consumption |
| Reducers | New generic Statistic/Accumulator; existing fold plus concrete states | Reuse fold; merge is a concrete law, never an assertion of bitwise associativity |
| Bayesian update | Universal Conjugate trait; runtime registry; typed model-specific methods | Explicit methods preserve Beta/Gamma return types and distinguish observation models |
| Packages | Huge math; six noun packages; two consumers with modules | Two consumers plus existing math/random owners; optional higher-level modules avoid cycles |

The design permits later precision-specific distributions, differentiable
evaluators, foreign backend adapters, and explicit mixed measures. It rules out
automatic implicit promotion, hidden default RNG consumption, and automatic
conjugacy inference as V1 contracts. None is required for the concrete workflows.

## 14.5 Third-party extension contract

A package owns its descriptor and explicitly implements the canonical
capabilities it can satisfy. It need not inherit Number or modify compiler
tables. A sampling-only simulator implements Sampleable; a custom likelihood
can provide a typed callable; an evaluable distribution implements mass or
density and optionally tails/quantiles. No fallback numerically integrates an
arbitrary density merely to manufacture a promised CDF.

Trait-or-target ownership in E25 permits an external package to implement
standard traits for its own distribution. It cannot add inherent methods or a
conformance between two foreign declarations. Wrap a foreign descriptor in an
owned adapter instead. If an adapter stores a callable, preserve its exact
labels and result type. Exported Families are public API capabilities; keep
kernel owner classes hidden as the math facade does.

A useful conformance review checks normalized measure, support/endpoints,
observation type, direct log methods, explicit RNG effects, error semantics, and
documented numerical range. These are library laws, not claims that nominal
trait evidence statically proves probability normalization.
