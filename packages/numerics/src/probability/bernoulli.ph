import universe.math as math
from universe.random import Bernoulli
from .capabilities import HasMass, HasOrderedTails, HasQuantile
from .support import FiniteSupport

impl Bernoulli {
  /// Mean and variance of the Bool indicator (false = 0, true = 1).
  mean -> Float { self.probability }
  variance -> Float { self.probability * (1.0 - self.probability) }

  support -> FiniteSupport<Bool> {
    let values = List<Bool>.new()
    if self.probability < 1.0 { values.append(false) }
    if self.probability > 0.0 { values.append(true) }
    return FiniteSupport.from(values)
  }
}

impl HasMass for Bernoulli {
  type Observation = Bool

  pmf(_ value: Bool) -> Float {
    if value { return self.probability }
    return 1.0 - self.probability
  }

  logPmf(_ value: Bool) -> Float {
    if value {
      if self.probability == 0.0 { return -1.0 / 0.0 }
      return math.ln(self.probability)
    }
    if self.probability == 1.0 { return -1.0 / 0.0 }
    return math.ln1p(-self.probability)
  }
}

impl HasOrderedTails for Bernoulli {
  type Cut = Bool

  cdf(_ cut: Bool) -> Float {
    if cut { return 1.0 }
    return 1.0 - self.probability
  }

  survival(_ cut: Bool) -> Float {
    if cut { return 0.0 }
    return self.probability
  }

  logCdf(_ cut: Bool) -> Float {
    if cut { return 0.0 }
    if self.probability == 1.0 { return -1.0 / 0.0 }
    return math.ln1p(-self.probability)
  }

  logSurvival(_ cut: Bool) -> Float {
    if cut { return -1.0 / 0.0 }
    if self.probability == 0.0 { return -1.0 / 0.0 }
    return math.ln(self.probability)
  }
}

impl HasQuantile for Bernoulli {
  type Quantile = Bool

  quantile(_ probability: Float) -> Bool {
    if not probability.isFinite or probability < 0.0 or probability > 1.0 {
      ArgumentError.new("Bernoulli quantile probability must be finite and in [0, 1]").raise()
    }
    if self.probability == 0.0 { return false }
    if self.probability == 1.0 { return true }
    if probability == 0.0 { return false }
    if probability == 1.0 { return true }
    return probability > 1.0 - self.probability
  }

  upperQuantile(_ upperProbability: Float) -> Bool {
    if not upperProbability.isFinite or upperProbability < 0.0 or upperProbability > 1.0 {
      ArgumentError.new("Bernoulli upper quantile probability must be finite and in [0, 1]").raise()
    }
    if self.probability == 0.0 { return false }
    if self.probability == 1.0 { return true }
    if upperProbability == 0.0 { return true }
    if upperProbability == 1.0 { return false }
    return upperProbability < self.probability
  }
}

export Bernoulli
