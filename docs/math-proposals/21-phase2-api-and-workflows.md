# 21. API sketches and complete workflows

Every Telos block in this file is a **proposed library API in current syntax**.
The new packages/members do not exist. Imports, `let`/`let mut`, loops,
closures, class methods, trait bounds, associated projections, and `.new`
versus primary data construction follow evidence E02, E07–E19, E24–E26.
The blocks were reviewed against those precedents; they were not compiled.
No trailing DSL blocks, Rust syntax, optional/default parameter mechanism,
free-function declarations, or compiler extension is required.

## 21.1 Generic terminals and public Family facade

A collection terminal is a method on a hidden owner and an exported captured
Family. Contract signatures inside that owner can be shaped as follows; bodies
are specified in files 15–16 rather than included as dummy implementations:

```text
@class mean<S>(_ values: S) -> Float
  where S |= Iterable, <S as Iterable>::Item == Float
@class sampleVariance<S>(_ values: S) -> Float
  where S |= Iterable, <S as Iterable>::Item == Float
@class quantile<S>(_ values: S, probability q: Float) -> Float
  where S |= Iterable, <S as Iterable>::Item == Float
@class weightedMean<S,W>(_ values: S, weights weights: W) -> Float
  where S |= Iterable, W |= Iterable,
        <S as Iterable>::Item == Float, <W as Iterable>::Item == Float
```

This is non-executable contract notation. Exact parameter labels are part of
the proposal. Generic inputs preserve iterator/lazy composition rather than
requiring List<Float> or an existential Iterable value.

Facade shape (proposed package source):

```telos
from .descriptive import Descriptive
let mean = &Descriptive.mean(_)
let sampleVariance = &Descriptive.sampleVariance(_)
let quantile = &Descriptive.quantile(_, probability)
let weightedMean = &Descriptive.weightedMean(_, weights)
export mean, sampleVariance, quantile, weightedMean
```

Other named statistical terminals use the same facade pattern; the public owner
class is not necessary. E02 gives live math captures; E17–E18 give generic
method/witness precedents. Capturing/exporting each new generic family still
requires a focused check during later implementation; the design does not
claim every inference combination was tested here.

## 21.2 Descriptive and weighted statistics

```telos
import statistics as stats

let xs = [1.2, 4.0, 3.8, 7.1]
let mean = stats.mean(xs)
let population = stats.populationVariance(xs)
let sample = stats.sampleVariance(xs)
let spread = stats.sampleStddev(xs)
let median = stats.median(xs)
let q95 = stats.quantile(xs, probability: 0.95)
let weighted = stats.weightedMean(xs, weights: [1.0, 2.0, 1.0, 1.0])
let r = stats.correlation(xs, with: [2.4, 8.0, 7.6, 14.2])

let integers = [1, 2, 3, 4]
let exactSum: Int = stats.sumInts(integers)
let approximateMean: Float = stats.mean(integers.iter.map |x| { Float.new(x) })
```

The last line states the user's rounding choice. No `mean([Int]) -> Float`
overload silently changes exactness. Named covariance/correlation wrappers use
`with:`; paired streams can also be passed to pair-specific terminals.

## 21.3 Streaming state and merge

```telos
from statistics import RunningMoments

let left = RunningMoments.new()
for x in [1000000000001.0, 1000000000002.0] {
  left.push(x)
}
let right = RunningMoments.new()
right.push(1000000000003.0)

left.merge(right.snapshot)
let summary = left.snapshot
let n: Int = summary.count
let sampleVariance = summary.sampleVariance.match(
  some: |value| { value },
  none: || { throw ArgumentError.new("sample variance requires two values") }
)
```

The expected count is 3 and sample variance is approximately 1. Snapshot values
are immutable; left/right bindings need not be reassignable to mutate their
objects. The same loop accepts a finite stream with Item Float and stores no
observations. Empty summary.mean is None; it does not poison later pushes.

Mutable state contracts (non-executable notation):

```text
RunningMoments.new() -> RunningMoments
push(Float) -> Unit; merge(MomentSummary) -> Unit; snapshot -> MomentSummary
MomentSummary: count Int; mean Option<Float>; minimum/maximum Option<Float>
populationVariance/sampleVariance Option<Float>; merged(MomentSummary) -> MomentSummary

RunningCovariance.new() -> RunningCovariance
push(Float, with: Float) -> Unit; merge(CovarianceSummary) -> Unit
snapshot -> CovarianceSummary
```

## 21.4 Distribution evaluation and tails

```telos
from probability import Normal, Binomial, Beta, Gamma, Exponential, Uniform

let normal = Normal.new(mean: 0.0, stddev: 1.0)
let density: Float = normal.pdf(1.0)
let logDensity: Float = normal.logPdf(1.0)
let below: Float = normal.cdf(1.96)
let q: Float = normal.quantile(0.975)
let tinyTail: Float = normal.survival(10.0)
let logTinyTail: Float = normal.logSurvival(40.0)
let upper: Float = normal.upperQuantile(0.000000000001)

let binomial = Binomial.new(trials: 20, probability: 0.3)
let exactlyFive = binomial.pmf(5)
let atLeastFive = binomial.survival(4)

let large = Binomial.new(trials: 1000000, probability: 0.3)
let logAtLeast320000 = large.logSurvival(319999)

let beta = Beta.new(alpha: 2.0, beta: 2.0)
let gamma = Gamma.new(shape: 3.0, rate: 2.0)
let exponential = Exponential.new(rate: 2.0)
let uniform = Uniform.new(lower: -1.0, upper: 1.0)
```

All constructors are checked factories. Normal(0,1) is not a current/proposed
public direct constructor shortcut because it would bypass the restricted data
factory. Density is an ordinary Float and may exceed one. Integer tail cuts
preserve strict survival semantics. Beta/Gamma require the special-function
foundation amendments before these operations can be implemented accurately.

## 21.5 Generic sampling helper and typed extension

The following complete proposed method body demonstrates why Sampleable and
RandomBits earn public interfaces:

```telos
from universe.collections import Iterable, Iterator
from universe.random import RandomBits
from probability import Sampleable

class Sampling {
  @class
  sampleMany<D,R>(_ d: D, count n: Int, using rng: R)
      -> List<<D as Sampleable>::Sample>
      where D |= Sampleable, R |= RandomBits
  {
    if n < 0 { throw ArgumentError.new("sample count must be nonnegative") }
    let result = List<<D as Sampleable>::Sample>.new()
    let mut remaining = n
    while remaining > 0 {
      result.append(d.sample(using: rng))
      remaining = remaining - 1
    }
    result
  }
}

data ConstantSampler<T>(_ value: T)
impl<T> Sampleable for ConstantSampler<T> {
  type Sample = T
  sample<R>(using rng: R) -> T where R |= RandomBits { value }
}
```

ConstantSampler is a legitimate point-law sample-only extension; it consumes
no bits. It need not implement a density, mass, CDF, universal Distribution root,
or moments. The helper body illustrates typed composition; production
sampleMany also validates the allocation/resource bound before drawing, as
specified in file 18.

```telos
from universe.random import Pcg32
from probability import Normal, Categorical, sampleMany, samples
import statistics as stats

let rng = Pcg32.new(seed: 42, stream: 7)
let normal = Normal.new(mean: 0.0, stddev: 1.0)
let one: Float = normal.sample(using: rng)
let batch: List<Float> = sampleMany(normal, count: 1000, using: rng)
let stream = samples(normal, using: rng).take(1000)
let streamMean = stats.mean(stream)

let colors = Categorical.fromWeights(
  labels: ["red", "green", "blue"],
  weights: [1e300, 2e300, 0.0]
)
let color: String = colors.sample(using: rng)
let massRed: Float = colors.pmf("red")
let names: List<String> = sampleMany(colors, count: 10, using: rng)
```

The lazy stream consumes the same RNG after the batch. It neither replays nor
promises the same samples as a fresh generator. An implementation's sample stream binds
Item to Sampleable::Sample using the existing canonical Iterator conformance.

## 21.6 Likelihood and conjugate updates

```telos
import probability as probability
import probability.bayes as bayes
from probability import Beta, Gamma, Bernoulli, Poisson
from probability.bayes import BernoulliCounts, BinomialObservation, PoissonCounts

let observations = [true, true, false, true, true, true, false, true, true, true]
let counts = BernoulliCounts.fromBools(observations)
let prior = Beta.new(alpha: 2.0, beta: 2.0)
let posterior: Beta = bayes.betaBernoulli(prior: prior, counts: counts)
let predictive: Bernoulli = bayes.bernoulliPredictive(posterior: posterior)
let estimate: Float = posterior.mean
let interval = bayes.equalTailedInterval(posterior, mass: 0.95)
let evidence = bayes.logBernoulliSequenceEvidence(prior: prior, counts: counts)

let grouped = BinomialObservation.new(trials: 10, successes: 8)
let groupedPosterior = bayes.betaBinomial(prior: prior, observation: grouped)
let groupedEvidence = bayes.logBinomialEvidence(prior: prior, observation: grouped)

let logL = probability.iidLogMassLikelihood(
  Bernoulli.new(probability: 0.7), observations: observations
)

let poissonCounts = PoissonCounts.fromCounts([2, 0, 3])
let ratePrior = Gamma.new(shape: 2.0, rate: 1.0)
let ratePosterior: Gamma = bayes.gammaPoisson(prior: ratePrior, counts: poissonCounts)
```

The posterior is Beta(10,4); groupedPosterior has the same parameters while
groupedEvidence differs by logChoose(10,8). ratePosterior is Gamma(shape=7,
rate=4). A future predictive count descriptor must marginalize uncertainty;
Poisson(ratePosterior.mean) is only a plug-in model.

Generic likelihood shape (non-executable contract notation):

```text
@class iidLogMassLikelihood<D,S>(_ d: D, observations xs: S) -> Float
  where D |= HasMass, S |= Iterable,
        <S as Iterable>::Item == <D as HasMass>::Observation
@class iidLogDensityLikelihood<D,S>(_ d: D, observations xs: S) -> Float
  where D |= HasDensity, S |= Iterable,
        <S as Iterable>::Item == <D as HasDensity>::Observation
```

## 21.7 Monte Carlo expectation and pi

```telos
import probability.monte_carlo as mc
from universe.random import Pcg32
from probability import Normal

let rng = Pcg32.new(seed: 42, stream: 11)
let normal = Normal.new(mean: 0.0, stddev: 1.0)
let result = mc.expectation(
  normal, count: 100000, using: rng,
  evaluate: |x| { x * x }
)
let expectation = result.mean
let diagnosticError = result.standardError
```

Expected target E[X²]=1, with simulation uncertainty. Signature uses the
Sampleable associated type and callback `(Sample)->Float`, requiring no
constructor/type lookup at runtime.

An explicit multi-variable simulation needs only ordinary loops:

```telos
from universe.random import Pcg32
from probability import Uniform
from statistics import RunningMoments
import universe.math as math

let rng = Pcg32.new(seed: 42, stream: 12)
let unit = Uniform.new(lower: 0.0, upper: 1.0)
let moments = RunningMoments.new()
let mut remaining = 100000
while remaining > 0 {
  let x = unit.sample(using: rng)
  let y = unit.sample(using: rng)
  let hit = if x*x + y*y <= 1.0 { 1.0 } else { 0.0 }
  moments.push(hit)
  remaining = remaining - 1
}
let summary = moments.snapshot
let mean = summary.mean.match(
  some: |x| { x }, none: || { throw ArgumentError.new("no simulated observations") }
)
let variance = summary.sampleVariance.match(
  some: |x| { x }, none: || { throw ArgumentError.new("insufficient simulated observations") }
)
let piEstimate = 4.0 * mean
let standardError = 4.0 * math.sqrt(variance / Float.new(summary.count))
```

The error branches are unreachable for this known count and keep the example's
insufficient-data policy explicit using current Option elimination.
No deterministic decimal result is asserted for unimplemented samplers.

## 21.8 Log-domain classifier example without an ML framework

```telos
import probability as probability
from probability import Bernoulli, Normal
import universe.math as math

let classPriorA = 0.6
let classPriorB = 0.4
let featureA = Bernoulli.new(probability: 0.8)
let featureB = Bernoulli.new(probability: 0.2)
let magnitudeA = Normal.new(mean: 1.0, stddev: 0.5)
let magnitudeB = Normal.new(mean: 2.0, stddev: 1.0)

let scoreA = math.ln(classPriorA) + featureA.logPmf(true) + magnitudeA.logPdf(1.2)
let scoreB = math.ln(classPriorB) + featureB.logPmf(true) + magnitudeB.logPdf(1.2)
let logPosteriors = probability.normalizedLogWeights([scoreA, scoreB])
let posteriors = probability.normalizeLogWeights([scoreA, scoreB])
let label = if scoreA >= scoreB { "A" } else { "B" }
```

The tie convention explicitly favors A. This is a mixed-feature conditional
model in a common observation space; file 20 separately treats Bernoulli,
Multinomial, and Gaussian Naive Bayes. The foundation requires no classifier
class, training-table format, or Distribution inheritance tree.
