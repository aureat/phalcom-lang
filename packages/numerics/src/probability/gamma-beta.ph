import universe.math as math
import ..special as special
from universe.math import NumericalRangeError, NumericalEvaluationError
from universe.random import Distribution, Generator
from .capabilities import HasDensity, HasOrderedTails, HasQuantile
from .continuous import DistributionNumeric, Normal
from .support import RealSupport

data Gamma private(_ shape: Float, _ rate: Float)

impl Gamma {
  @class
  new(shape: Float, rate: Float) -> Gamma {
    if not shape.isFinite or not rate.isFinite or shape <= 0.0 or rate <= 0.0 {
      ArgumentError.new("Gamma requires positive finite shape and rate").raise()
    }
    return Gamma(shape, rate)
  }

  mean -> Float {
    let result = self.shape / self.rate
    if not result.isFinite { NumericalRangeError.new("Gamma mean is outside the finite Float range").raise() }
    return result
  }

  variance -> Float {
    let result = (self.shape / self.rate) / self.rate
    if not result.isFinite { NumericalRangeError.new("Gamma variance is outside the finite Float range").raise() }
    return result
  }

  support -> RealSupport { RealSupport.atLeast(0.0) }

  @class
  fromScale(shape: Float, scale: Float) -> Gamma {
    if not scale.isFinite or scale <= 0.0 {
      ArgumentError.new("Gamma scale must be positive and finite").raise()
    }
    let rate = 1.0 / scale
    if not rate.isFinite or rate <= 0.0 {
      NumericalRangeError.new("Gamma scale reciprocal is outside the finite Float range").raise()
    }
    return Gamma.new(shape: shape, rate: rate)
  }
}

impl HasDensity for Gamma {
  type Observation = Float

  pdf(_ value: Float) -> Float {
    let result = math.exp(self.logPdf(value))
    if result.isInfinite and value != 0.0 {
      NumericalRangeError.new("Gamma density is outside the finite Float range").raise()
    }
    return result
  }

  logPdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Gamma log density does not accept NaN").raise() }
    if value < 0.0 or value == 1.0 / 0.0 { return -1.0 / 0.0 }
    if value == 0.0 {
      if self.shape < 1.0 { return 1.0 / 0.0 }
      if self.shape == 1.0 { return math.ln(self.rate) }
      return -1.0 / 0.0
    }
    let scaled = self.rate * value
    if not scaled.isFinite {
      NumericalRangeError.new("Gamma log density is outside the finite Float range").raise()
    }
    let result = self.shape * math.ln(self.rate)
      - special.logGamma(self.shape)
      + (self.shape - 1.0) * math.ln(value)
      - scaled
    if result.isNaN {
      NumericalRangeError.new("Gamma log density is numerically indeterminate").raise()
    }
    return result
  }
}

impl HasOrderedTails for Gamma {
  type Cut = Float

  cdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Gamma CDF does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    if value == 1.0 / 0.0 { return 1.0 }
    return special.regularizedGammaP(self.shape, self.rate * value)
  }

  survival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Gamma survival does not accept NaN").raise() }
    if value <= 0.0 { return 1.0 }
    if value == 1.0 / 0.0 { return 0.0 }
    return special.regularizedGammaQ(self.shape, self.rate * value)
  }

  logCdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Gamma log CDF does not accept NaN").raise() }
    if value <= 0.0 { return -1.0 / 0.0 }
    if value == 1.0 / 0.0 { return 0.0 }
    return special.logRegularizedGammaP(self.shape, self.rate * value)
  }

  logSurvival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Gamma log survival does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    if value == 1.0 / 0.0 { return -1.0 / 0.0 }
    return special.logRegularizedGammaQ(self.shape, self.rate * value)
  }
}

impl HasQuantile for Gamma {
  type Quantile = Float

  quantile(_ q: Float) -> Float {
    DistributionNumeric.validateProbability(q, "Gamma quantile probability")
    if q == 0.0 { return 0.0 }
    if q == 1.0 { return 1.0 / 0.0 }
    let result = special.inverseRegularizedGammaP(self.shape, q) / self.rate
    if not result.isFinite or result <= 0.0 {
      NumericalRangeError.new("Gamma quantile is outside the finite Float range").raise()
    }
    return result
  }

  upperQuantile(_ t: Float) -> Float {
    DistributionNumeric.validateProbability(t, "Gamma upper quantile probability")
    if t == 0.0 { return 1.0 / 0.0 }
    if t == 1.0 { return 0.0 }
    return Gamma.searchUpperQuantile(self, t)
  }
}

impl Distribution for Gamma {
  type Output = Float

  sample<G>(using generator: G) -> Float where G |= Generator {
    let logUnit = GammaKernel.sampleLog(self.shape, using: generator)
    let result = math.exp(logUnit - math.ln(self.rate))
    if not result.isFinite or result == 0.0 {
      NumericalRangeError.new("Gamma sample is outside the positive finite Float range").raise()
    }
    return result
  }
}

impl Gamma {
  @class
  private searchUpperQuantile(_ distribution: Gamma, _ target: Float) -> Float {
    let targetLog = math.ln(target)
    let mut high = distribution.shape / distribution.rate
    if not high.isFinite { high = Float.maxFinite }
    if high == 0.0 { high = Float.minPositiveSubnormal }
    let mut bracketed = false
    let mut attempts = 0
    while not bracketed and attempts < 1100 {
      if distribution.logSurvival(high) <= targetLog {
        bracketed = true
      } else if high >= Float.maxFinite / 2.0 {
        high = Float.maxFinite
        bracketed = distribution.logSurvival(high) <= targetLog
        if not bracketed {
          NumericalRangeError.new("Gamma upper quantile exceeds finite Float range").raise()
        }
      } else {
        high = high * 2.0
      }
      attempts = attempts + 1
    }
    if not bracketed { NumericalEvaluationError.new("Gamma upper quantile bracket did not converge").raise() }
    let mut low = 0.0
    let mut steps = 0
    while steps < 1100 {
      let middle = low + (high - low) / 2.0
      if middle == low or middle == high { return high }
      if distribution.logSurvival(middle) <= targetLog {
        high = middle
      } else {
        low = middle
      }
      steps = steps + 1
    }
    return high
  }
}

data Beta private(_ alpha: Float, _ beta: Float)

impl Beta {
  @class
  new(alpha: Float, beta: Float) -> Beta {
    if not alpha.isFinite or not beta.isFinite or alpha <= 0.0 or beta <= 0.0 {
      ArgumentError.new("Beta requires positive finite alpha and beta").raise()
    }
    return Beta(alpha, beta)
  }

  mean -> Float {
    if self.alpha >= self.beta {
      return 1.0 / (1.0 + self.beta / self.alpha)
    }
    let ratio = self.alpha / self.beta
    return ratio / (1.0 + ratio)
  }

  variance -> Float {
    let mean = self.mean
    let largest = if self.alpha >= self.beta { self.alpha } else { self.beta }
    if largest < 1.0 {
      return mean * (1.0 - mean) / (self.alpha + self.beta + 1.0)
    }
    let normalizedTotal = self.alpha / largest + self.beta / largest
    let inverseTotal = (1.0 / largest) / normalizedTotal
    return mean * (1.0 - mean) * inverseTotal / (1.0 + inverseTotal)
  }

  support -> RealSupport { RealSupport.closed(0.0, 1.0) }
}

impl HasDensity for Beta {
  type Observation = Float

  pdf(_ value: Float) -> Float {
    let result = math.exp(self.logPdf(value))
    if result.isInfinite and value > 0.0 and value < 1.0 {
      NumericalRangeError.new("Beta density is outside the finite Float range").raise()
    }
    return result
  }

  logPdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Beta log density does not accept NaN").raise() }
    if value < 0.0 or value > 1.0 { return -1.0 / 0.0 }
    if value == 0.0 {
      if self.alpha < 1.0 { return 1.0 / 0.0 }
      if self.alpha == 1.0 { return math.ln(self.beta) }
      return -1.0 / 0.0
    }
    if value == 1.0 {
      if self.beta < 1.0 { return 1.0 / 0.0 }
      if self.beta == 1.0 { return math.ln(self.alpha) }
      return -1.0 / 0.0
    }
    return (self.alpha - 1.0) * math.ln(value)
      + (self.beta - 1.0) * math.ln1p(-value)
      - special.logBeta(self.alpha, self.beta)
  }
}

impl HasOrderedTails for Beta {
  type Cut = Float

  cdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Beta CDF does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    if value >= 1.0 { return 1.0 }
    return special.regularizedBeta(self.alpha, self.beta, value)
  }

  survival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Beta survival does not accept NaN").raise() }
    if value <= 0.0 { return 1.0 }
    if value >= 1.0 { return 0.0 }
    return special.regularizedBetaComplement(self.alpha, self.beta, value)
  }

  logCdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Beta log CDF does not accept NaN").raise() }
    if value <= 0.0 { return -1.0 / 0.0 }
    if value >= 1.0 { return 0.0 }
    return special.logRegularizedBeta(self.alpha, self.beta, value)
  }

  logSurvival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Beta log survival does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    if value >= 1.0 { return -1.0 / 0.0 }
    return special.logRegularizedBetaComplement(self.alpha, self.beta, value)
  }
}

impl HasQuantile for Beta {
  type Quantile = Float

  quantile(_ q: Float) -> Float {
    DistributionNumeric.validateProbability(q, "Beta quantile probability")
    if q == 0.0 { return 0.0 }
    if q == 1.0 { return 1.0 }
    return special.inverseRegularizedBeta(self.alpha, self.beta, q)
  }

  upperQuantile(_ t: Float) -> Float {
    DistributionNumeric.validateProbability(t, "Beta upper quantile probability")
    if t == 0.0 { return 1.0 }
    if t == 1.0 { return 0.0 }
    return 1.0 - special.inverseRegularizedBeta(self.beta, self.alpha, t)
  }
}

impl Distribution for Beta {
  type Output = Float

  sample<G>(using generator: G) -> Float where G |= Generator {
    let logX = GammaKernel.sampleLog(self.alpha, using: generator)
    let logY = GammaKernel.sampleLog(self.beta, using: generator)
    if logX >= logY {
      return 1.0 / (1.0 + math.exp(logY - logX))
    }
    let ratio = math.exp(logX - logY)
    return ratio / (1.0 + ratio)
  }
}

/// Stateless source-level Marsaglia–Tsang gamma variate kernel.
class GammaKernel {
  @class
  sampleLog<G>(_ shape: Float, using generator: G) -> Float where G |= Generator {
    if shape < 1.0 {
      let boosted = GammaKernel.sampleLog(shape + 1.0, using: generator)
      let result = boosted + math.ln(generator.openUnit) / shape
      if not result.isFinite {
        NumericalRangeError.new("Gamma log sample is outside finite Float range").raise()
      }
      return result
    }
    let d = shape - 1.0 / 3.0
    let c = 1.0 / math.sqrt(9.0 * d)
    let normal = Normal.new(mean: 0.0, stddev: 1.0)
    let mut attempts = 0
    while attempts < 1024 {
      let x = normal.sample(using: generator)
      let base = 1.0 + c * x
      if base > 0.0 {
        let v = base * base * base
        let u = generator.openUnit
        let x2 = x * x
        let quickAccept = u < 1.0 - 0.0331 * x2 * x2
        let logAccept = math.ln(u) < 0.5 * x2 + d * (1.0 - v + math.ln(v))
        if quickAccept or logAccept {
          return math.ln(d) + 3.0 * math.ln(base)
        }
      }
      attempts = attempts + 1
    }
    NumericalEvaluationError.new("Gamma rejection sampler did not accept within its attempt budget").raise()
  }
}

export Gamma, Beta, GammaKernel
