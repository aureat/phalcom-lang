import universe.math as math
from universe.random import Bernoulli

/// Independent law capabilities; sampling stays on universe.random::Distribution.
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

/// Bernoulli law capabilities are owned here, alongside the traits they
/// implement. The descriptor itself remains universe.random::Bernoulli.
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

export HasMass, HasDensity, HasOrderedTails, HasQuantile
