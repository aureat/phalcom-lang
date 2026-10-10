import universe.math as math
import universe.math.special as special
from universe.math import NumericalRangeError
from universe.random import Distribution, Generator
from .capabilities import HasMass, HasOrderedTails, HasQuantile
from .continuous import DistributionNumeric
from .errors import UndefinedStatisticError
from .support import IntegerSupport

const maxApproximateCount: Int = 9007199254740991

data Binomial private(_ trials: Int, _ probability: Float)

impl Binomial {
  @class
  new(trials: Int, probability: Float) -> Binomial {
    if trials < 0 or trials > maxApproximateCount {
      ArgumentError.new("Binomial trials must be in 0..=2^53-1").raise()
    }
    DistributionNumeric.validateProbability(probability, "Binomial probability")
    return Binomial(trials, probability)
  }

  mean -> Float { Float.new(self.trials) * self.probability }

  variance -> Float { self.mean * (1.0 - self.probability) }

  support -> IntegerSupport {
    if self.trials == 0 or self.probability == 0.0 { return IntegerSupport.singleton(0) }
    if self.probability == 1.0 { return IntegerSupport.singleton(self.trials) }
    return IntegerSupport.closed(0, self.trials)
  }
}

impl HasMass for Binomial {
  type Observation = Int

  pmf(_ value: Int) -> Float {
    let logMass = self.logPmf(value)
    return math.exp(logMass)
  }

  logPmf(_ value: Int) -> Float {
    if value < 0 or value > self.trials { return -1.0 / 0.0 }
    if self.probability == 0.0 { return if value == 0 { 0.0 } else { -1.0 / 0.0 } }
    if self.probability == 1.0 { return if value == self.trials { 0.0 } else { -1.0 / 0.0 } }
    let successes = Float.new(value)
    let failures = Float.new(self.trials - value)
    return special.logChoose(self.trials, value)
      + successes * math.ln(self.probability)
      + failures * math.ln1p(-self.probability)
  }
}

impl HasOrderedTails for Binomial {
  type Cut = Int

  cdf(_ cut: Int) -> Float {
    if cut < 0 { return 0.0 }
    if cut >= self.trials { return 1.0 }
    if self.probability == 0.0 { return 1.0 }
    if self.probability == 1.0 { return 0.0 }
    return special.regularizedBeta(
      Float.new(self.trials - cut), Float.new(cut + 1), 1.0 - self.probability
    )
  }

  survival(_ cut: Int) -> Float {
    if cut < 0 { return 1.0 }
    if cut >= self.trials { return 0.0 }
    if self.probability == 0.0 { return 0.0 }
    if self.probability == 1.0 { return 1.0 }
    return special.regularizedBeta(
      Float.new(cut + 1), Float.new(self.trials - cut), self.probability
    )
  }

  logCdf(_ cut: Int) -> Float {
    if cut < 0 { return -1.0 / 0.0 }
    if cut >= self.trials { return 0.0 }
    if self.probability == 0.0 { return 0.0 }
    if self.probability == 1.0 { return -1.0 / 0.0 }
    return special.logRegularizedBeta(
      Float.new(self.trials - cut), Float.new(cut + 1), 1.0 - self.probability
    )
  }

  logSurvival(_ cut: Int) -> Float {
    if cut < 0 { return 0.0 }
    if cut >= self.trials { return -1.0 / 0.0 }
    if self.probability == 0.0 { return -1.0 / 0.0 }
    if self.probability == 1.0 { return 0.0 }
    return special.logRegularizedBeta(
      Float.new(cut + 1), Float.new(self.trials - cut), self.probability
    )
  }
}

impl HasQuantile for Binomial {
  type Quantile = Int

  quantile(_ q: Float) -> Int {
    DistributionNumeric.validateProbability(q, "Binomial quantile probability")
    if self.probability == 0.0 or self.trials == 0 { return 0 }
    if self.probability == 1.0 { return self.trials }
    if q == 0.0 { return 0 }
    if q == 1.0 { return self.trials }
    let mut low = 0
    let mut high = self.trials
    while low < high {
      let middle = low + ((high - low) ~/ 2)
      if self.cdf(middle) >= q {
        high = middle
      } else {
        low = middle + 1
      }
    }
    return low
  }

  upperQuantile(_ upperProbability: Float) -> Int {
    DistributionNumeric.validateProbability(upperProbability, "Binomial upper quantile probability")
    if self.probability == 0.0 or self.trials == 0 { return 0 }
    if self.probability == 1.0 { return self.trials }
    let mut low = 0
    let mut high = self.trials
    while low < high {
      let middle = low + ((high - low) ~/ 2)
      if self.survival(middle) <= upperProbability {
        high = middle
      } else {
        low = middle + 1
      }
    }
    return low
  }
}

impl Distribution for Binomial {
  type Output = Int

  sample<G>(using generator: G) -> Int where G |= Generator {
    if self.trials == 0 or self.probability == 0.0 { return 0 }
    if self.probability == 1.0 { return self.trials }
    return self.quantile(generator.nextFloat)
  }
}

data Poisson private(_ rate: Float)

impl Poisson {
  @class
  new(rate: Float) -> Poisson {
    if not rate.isFinite or rate < 0.0 or rate > Float.new(maxApproximateCount) {
      ArgumentError.new("Poisson rate must be finite and within the exact Float count range").raise()
    }
    return Poisson(rate)
  }

  mean -> Float { self.rate }
  variance -> Float { self.rate }

  support -> IntegerSupport {
    if self.rate == 0.0 { return IntegerSupport.singleton(0) }
    return IntegerSupport.atLeast(0)
  }
}

impl HasMass for Poisson {
  type Observation = Int

  pmf(_ value: Int) -> Float { math.exp(self.logPmf(value)) }

  logPmf(_ value: Int) -> Float {
    if value < 0 { return -1.0 / 0.0 }
    if self.rate == 0.0 { return if value == 0 { 0.0 } else { -1.0 / 0.0 } }
    let count = Float.new(value)
    let result = count * math.ln(self.rate) - self.rate - special.logFactorial(value)
    if not result.isFinite {
      NumericalRangeError.new("Poisson log mass is outside the finite Float range").raise()
    }
    return result
  }
}

impl HasOrderedTails for Poisson {
  type Cut = Int

  cdf(_ cut: Int) -> Float {
    if cut < 0 { return 0.0 }
    if self.rate == 0.0 { return 1.0 }
    if cut >= maxApproximateCount { return 1.0 }
    return special.regularizedGammaQ(Float.new(cut + 1), self.rate)
  }

  survival(_ cut: Int) -> Float {
    if cut < 0 { return 1.0 }
    if self.rate == 0.0 or cut >= maxApproximateCount { return 0.0 }
    return special.regularizedGammaP(Float.new(cut + 1), self.rate)
  }

  logCdf(_ cut: Int) -> Float {
    if cut < 0 { return -1.0 / 0.0 }
    if self.rate == 0.0 or cut >= maxApproximateCount { return 0.0 }
    return special.logRegularizedGammaQ(Float.new(cut + 1), self.rate)
  }

  logSurvival(_ cut: Int) -> Float {
    if cut < 0 { return 0.0 }
    if self.rate == 0.0 or cut >= maxApproximateCount { return -1.0 / 0.0 }
    return special.logRegularizedGammaP(Float.new(cut + 1), self.rate)
  }
}

impl HasQuantile for Poisson {
  type Quantile = Int

  quantile(_ q: Float) -> Int {
    DistributionNumeric.validateProbability(q, "Poisson quantile probability")
    if self.rate == 0.0 or q == 0.0 { return 0 }
    if q == 1.0 { UndefinedStatisticError.new("Poisson quantile at 1 has no finite Int result").raise() }
    return Poisson.searchQuantile(self, q, upperTail: false)
  }

  upperQuantile(_ upperProbability: Float) -> Int {
    DistributionNumeric.validateProbability(upperProbability, "Poisson upper quantile probability")
    if self.rate == 0.0 { return 0 }
    if upperProbability == 0.0 {
      UndefinedStatisticError.new("Poisson upper quantile at 0 has no finite Int result").raise()
    }
    return Poisson.searchQuantile(self, upperProbability, upperTail: true)
  }
}

impl Distribution for Poisson {
  type Output = Int

  sample<G>(using generator: G) -> Int where G |= Generator {
    if self.rate == 0.0 { return 0 }
    return self.quantile(generator.nextFloat)
  }
}

impl Poisson {
  @class
  private searchQuantile(_ distribution: Poisson, _ target: Float, upperTail: Bool) -> Int {
    let mut low = 0
    let mut high = 1
    let mut found = false
    while not found {
      let probability = if upperTail { distribution.survival(high) } else { distribution.cdf(high) }
      if (upperTail and probability <= target) or (not upperTail and probability >= target) {
        found = true
      } else if high >= maxApproximateCount {
        NumericalRangeError.new("Poisson quantile exceeds the exact Float count range").raise()
      } else {
        low = high + 1
        high = if high > maxApproximateCount ~/ 2 { maxApproximateCount } else { high * 2 }
      }
    }
    while low < high {
      let middle = low + ((high - low) ~/ 2)
      let probability = if upperTail { distribution.survival(middle) } else { distribution.cdf(middle) }
      if (upperTail and probability <= target) or (not upperTail and probability >= target) {
        high = middle
      } else {
        low = middle + 1
      }
    }
    return low
  }
}

export Binomial, Poisson
