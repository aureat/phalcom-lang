import universe.math as math
import universe.math.special as special
from universe.math import NumericalRangeError
from universe.collections.iterator_laws import Iterable
from ..bernoulli import Bernoulli
from ..gamma_beta import Beta, Gamma

const maxApproximateCount: Int = 9007199254740991

data BernoulliCounts private(_ successes: Int, _ failures: Int)

impl BernoulliCounts {
  @class
  fromBools<I>(_ values: I) -> BernoulliCounts
    where I |= Iterable, <I as Iterable>::Item == Bool
  {
    let mut successes = 0
    let mut failures = 0
    let mut iterator = values.iterator
    let mut step = iterator.next
    while step.isSome {
      if successes + failures >= maxApproximateCount {
        NumericalRangeError.new("Bernoulli observation count exceeds exact Float range").raise()
      }
      let value: Bool = step.match(some: |x| { x }, none: || { false })
      if value { successes = successes + 1 } else { failures = failures + 1 }
      step = iterator.next
    }
    return BernoulliCounts(successes, failures)
  }

  @class
  one(_ success: Bool) -> BernoulliCounts {
    if success { return BernoulliCounts(1, 0) }
    return BernoulliCounts(0, 1)
  }

  count -> Int { self.successes + self.failures }

  merged(_ other: BernoulliCounts) -> BernoulliCounts {
    if self.count > maxApproximateCount - other.count {
      NumericalRangeError.new("merged Bernoulli count exceeds exact Float range").raise()
    }
    return BernoulliCounts(self.successes + other.successes, self.failures + other.failures)
  }
}

data BinomialObservation private(_ trials: Int, _ successes: Int)

impl BinomialObservation {
  @class
  new(trials: Int, successes: Int) -> BinomialObservation {
    if trials < 0 or trials > maxApproximateCount or successes < 0 or successes > trials {
      ArgumentError.new("BinomialObservation requires 0 <= successes <= trials <= 2^53-1").raise()
    }
    return BinomialObservation(trials, successes)
  }
}

data PoissonCounts private(_ observations: Int, _ totalCount: Int)

impl PoissonCounts {
  @class
  fromCounts<I>(_ counts: I) -> PoissonCounts
    where I |= Iterable, <I as Iterable>::Item == Int
  {
    let mut observations = 0
    let mut totalCount = 0
    let mut iterator = counts.iterator
    let mut step = iterator.next
    while step.isSome {
      let count: Int = step.match(some: |x| { x }, none: || { 0 })
      if count < 0 { ArgumentError.new("PoissonCounts requires nonnegative observations").raise() }
      if observations >= maxApproximateCount or count > maxApproximateCount - totalCount {
        NumericalRangeError.new("Poisson sufficient statistics exceed exact Float count range").raise()
      }
      observations = observations + 1
      totalCount = totalCount + count
      step = iterator.next
    }
    return PoissonCounts(observations, totalCount)
  }

  merged(_ other: PoissonCounts) -> PoissonCounts {
    if self.observations > maxApproximateCount - other.observations
      or self.totalCount > maxApproximateCount - other.totalCount
    {
      NumericalRangeError.new("merged Poisson sufficient statistics exceed exact Float count range").raise()
    }
    return PoissonCounts(self.observations + other.observations, self.totalCount + other.totalCount)
  }
}

class Bayes {
  @class
  betaBernoulli(prior: Beta, counts: BernoulliCounts) -> Beta {
    if counts.count == 0 { return prior }
    let alpha = prior.alpha + Float.new(counts.successes)
    let beta = prior.beta + Float.new(counts.failures)
    if not alpha.isFinite or not beta.isFinite
      or (counts.successes > 0 and alpha == prior.alpha)
      or (counts.failures > 0 and beta == prior.beta)
    {
      NumericalRangeError.new("Beta-Bernoulli posterior parameters are not representable").raise()
    }
    return Beta.new(alpha: alpha, beta: beta)
  }

  @class
  betaBinomial(prior: Beta, observation: BinomialObservation) -> Beta {
    if observation.trials == 0 { return prior }
    let alpha = prior.alpha + Float.new(observation.successes)
    let failures = observation.trials - observation.successes
    let beta = prior.beta + Float.new(failures)
    if not alpha.isFinite or not beta.isFinite
      or (observation.successes > 0 and alpha == prior.alpha)
      or (failures > 0 and beta == prior.beta)
    {
      NumericalRangeError.new("Beta-Binomial posterior parameters are not representable").raise()
    }
    return Beta.new(alpha: alpha, beta: beta)
  }

  @class
  bernoulliPredictive(_ posterior: Beta) -> Bernoulli {
    return Bernoulli.new(posterior.mean)
  }

  @class
  logBernoulliSequenceEvidence(prior: Beta, counts: BernoulliCounts) -> Float {
    let result = special.logBeta(
      prior.alpha + Float.new(counts.successes),
      prior.beta + Float.new(counts.failures)
    ) - special.logBeta(prior.alpha, prior.beta)
    if not result.isFinite {
      NumericalRangeError.new("Bernoulli sequence evidence is outside Float range").raise()
    }
    return result
  }

  @class
  logBinomialEvidence(prior: Beta, observation: BinomialObservation) -> Float {
    let result = special.logChoose(observation.trials, observation.successes)
      + special.logBeta(
        prior.alpha + Float.new(observation.successes),
        prior.beta + Float.new(observation.trials - observation.successes)
      )
      - special.logBeta(prior.alpha, prior.beta)
    if not result.isFinite {
      NumericalRangeError.new("Binomial evidence is outside Float range").raise()
    }
    return result
  }

  @class
  gammaPoisson(prior: Gamma, counts: PoissonCounts) -> Gamma {
    if counts.observations == 0 and counts.totalCount == 0 { return prior }
    let shape = prior.shape + Float.new(counts.totalCount)
    let rate = prior.rate + Float.new(counts.observations)
    if not shape.isFinite or not rate.isFinite
      or (counts.totalCount > 0 and shape == prior.shape)
      or (counts.observations > 0 and rate == prior.rate)
    {
      NumericalRangeError.new("Gamma-Poisson posterior parameters are not representable").raise()
    }
    return Gamma.new(shape: shape, rate: rate)
  }
}

export BernoulliCounts, BinomialObservation, PoissonCounts, Bayes
