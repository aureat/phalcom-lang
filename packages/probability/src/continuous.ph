import universe.math as math
import numerics.special as special
from universe.math import NumericalRangeError
from universe.random import Distribution, Generator
from .capabilities import HasDensity, HasOrderedTails, HasQuantile
from .support import RealSupport

class DistributionNumeric {
  @class
  validateProbability(_ q: Float, _ name: String) -> Unit {
    if not q.isFinite or q < 0.0 or q > 1.0 {
      ArgumentError.new(name + " must be finite and in [0, 1]").raise()
    }
    return ()
  }

  @class
  interpolate(_ lower: Float, _ upper: Float, _ fraction: Float) -> Float {
    if lower <= 0.0 and upper >= 0.0 or lower >= 0.0 and upper <= 0.0 {
      return lower * (1.0 - fraction) + upper * fraction
    }
    return lower + (upper - lower) * fraction
  }

  @class
  normalZ(_ value: Float, mean: Float, stddev: Float) -> Float {
    if value.isInfinite { return value }
    let scale = if value.abs >= mean.abs { value.abs } else { mean.abs }
    if scale == 0.0 { return 0.0 }
    let difference = value / scale - mean / scale
    if difference == 0.0 { return 0.0 }
    if scale <= stddev { return difference * (scale / stddev) }
    return difference / (stddev / scale)
  }

  @class
  normalAffine(_ mean: Float, _ stddev: Float, _ z: Float) -> Float {
    let scale = if mean.abs >= stddev { mean.abs } else { stddev }
    let normalized = mean / scale + (stddev / scale) * z
    let result = scale * normalized
    if not result.isFinite {
      NumericalRangeError.new("Normal result is outside the finite Float range").raise()
    }
    return result
  }
}

data Uniform private(_ lower: Float, _ upper: Float)

impl Uniform {
  @class
  new(lower: Float, upper: Float) -> Uniform {
    if not lower.isFinite or not upper.isFinite or lower >= upper {
      ArgumentError.new("Uniform requires finite lower < upper").raise()
    }
    return Uniform(lower, upper)
  }

  mean -> Float { DistributionNumeric.interpolate(self.lower, self.upper, 0.5) }

  support -> RealSupport { RealSupport.closed(self.lower, self.upper) }

  variance -> Float {
    let width = self.upper - self.lower
    let result = if width.isFinite {
      let half = width / 2.0
      (half * half) / 3.0
    } else {
      let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
      let normalizedWidth = self.upper / scale - self.lower / scale
      let halfNormalized = normalizedWidth / 2.0
      (scale * ((scale * halfNormalized) * halfNormalized)) / 3.0
    }
    if not result.isFinite {
      NumericalRangeError.new("Uniform variance is outside the finite Float range").raise()
    }
    return result
  }
}

impl Distribution for Uniform {
  type Output = Float

  sample<G>(using generator: G) -> Float where G |= Generator {
    let unit = generator.nextFloat
    let mut value = DistributionNumeric.interpolate(self.lower, self.upper, unit)
    if value < self.lower { value = self.lower }
    if value >= self.upper { value = self.upper.nextDown }
    if not value.isFinite or value < self.lower or value >= self.upper {
      NumericalRangeError.new("Uniform sample is not representable in the validated interval").raise()
    }
    return value
  }
}

impl HasDensity for Uniform {
  type Observation = Float

  pdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform density does not accept NaN").raise() }
    if value < self.lower or value > self.upper { return 0.0 }
    let width = self.upper - self.lower
    if width.isFinite {
      let result = 1.0 / width
      if not result.isFinite {
        NumericalRangeError.new("Uniform density is outside the finite Float range").raise()
      }
      return result
    }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    return (1.0 / scale) / normalizedWidth
  }

  logPdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform log density does not accept NaN").raise() }
    if value < self.lower or value > self.upper { return -1.0 / 0.0 }
    let width = self.upper - self.lower
    if width.isFinite { return -math.ln(width) }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    return -math.ln(scale) - math.ln(normalizedWidth)
  }
}

impl HasOrderedTails for Uniform {
  type Cut = Float

  cdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform CDF does not accept NaN").raise() }
    if value <= self.lower { return 0.0 }
    if value >= self.upper { return 1.0 }
    let width = self.upper - self.lower
    if width.isFinite { return (value - self.lower) / width }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    return (value / scale - self.lower / scale) / normalizedWidth
  }

  survival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform survival does not accept NaN").raise() }
    if value < self.lower { return 1.0 }
    if value >= self.upper { return 0.0 }
    let width = self.upper - self.lower
    if width.isFinite { return (self.upper - value) / width }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    return (self.upper / scale - value / scale) / normalizedWidth
  }

  logCdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform log CDF does not accept NaN").raise() }
    if value <= self.lower { return -1.0 / 0.0 }
    if value >= self.upper { return 0.0 }
    let width = self.upper - self.lower
    if width.isFinite { return math.ln(value - self.lower) - math.ln(width) }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    let normalizedDistance = value / scale - self.lower / scale
    return math.ln(normalizedDistance) - math.ln(normalizedWidth)
  }

  logSurvival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Uniform log survival does not accept NaN").raise() }
    if value < self.lower { return 0.0 }
    if value >= self.upper { return -1.0 / 0.0 }
    let width = self.upper - self.lower
    if width.isFinite { return math.ln(self.upper - value) - math.ln(width) }
    let scale = if self.lower.abs >= self.upper.abs { self.lower.abs } else { self.upper.abs }
    let normalizedWidth = self.upper / scale - self.lower / scale
    let normalizedDistance = self.upper / scale - value / scale
    return math.ln(normalizedDistance) - math.ln(normalizedWidth)
  }
}

impl HasQuantile for Uniform {
  type Quantile = Float

  quantile(_ probability: Float) -> Float {
    DistributionNumeric.validateProbability(probability, "Uniform quantile probability")
    return DistributionNumeric.interpolate(self.lower, self.upper, probability)
  }

  upperQuantile(_ upperProbability: Float) -> Float {
    DistributionNumeric.validateProbability(upperProbability, "Uniform upper quantile probability")
    return DistributionNumeric.interpolate(self.upper, self.lower, upperProbability)
  }
}

data Exponential private(_ rate: Float)

impl Exponential {
  @class
  new(rate: Float) -> Exponential {
    if not rate.isFinite or rate <= 0.0 {
      ArgumentError.new("Exponential requires a positive finite rate").raise()
    }
    return Exponential(rate)
  }

  mean -> Float {
    let result = 1.0 / self.rate
    if not result.isFinite { NumericalRangeError.new("Exponential mean is outside the finite Float range").raise() }
    return result
  }

  support -> RealSupport { RealSupport.atLeast(0.0) }

  variance -> Float {
    let mean = self.mean
    let result = mean * mean
    if not result.isFinite { NumericalRangeError.new("Exponential variance is outside the finite Float range").raise() }
    return result
  }
}

impl Distribution for Exponential {
  type Output = Float

  sample<G>(using generator: G) -> Float where G |= Generator {
    let value = -math.ln1p(-generator.nextFloat) / self.rate
    if not value.isFinite or value < 0.0 {
      NumericalRangeError.new("Exponential sample is outside the finite Float range").raise()
    }
    return value
  }
}

impl HasDensity for Exponential {
  type Observation = Float

  pdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential density does not accept NaN").raise() }
    if value < 0.0 or value == 1.0 / 0.0 { return 0.0 }
    if value == 0.0 { return self.rate }
    let exponent = -self.rate * value
    let result = self.rate * math.exp(exponent)
    if not result.isFinite {
      NumericalRangeError.new("Exponential density is outside the finite Float range").raise()
    }
    return result
  }

  logPdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential log density does not accept NaN").raise() }
    if value < 0.0 or value == 1.0 / 0.0 { return -1.0 / 0.0 }
    let product = self.rate * value
    if not product.isFinite {
      NumericalRangeError.new("Exponential log density is outside the finite Float range").raise()
    }
    return math.ln(self.rate) - product
  }
}

impl HasOrderedTails for Exponential {
  type Cut = Float

  cdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential CDF does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    return -math.expm1(-self.rate * value)
  }

  survival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential survival does not accept NaN").raise() }
    if value <= 0.0 { return 1.0 }
    return math.exp(-self.rate * value)
  }

  logCdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential log CDF does not accept NaN").raise() }
    if value <= 0.0 { return -1.0 / 0.0 }
    return math.ln(-math.expm1(-self.rate * value))
  }

  logSurvival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Exponential log survival does not accept NaN").raise() }
    if value <= 0.0 { return 0.0 }
    let result = -self.rate * value
    if value.isFinite and not result.isFinite {
      NumericalRangeError.new("Exponential log tail is outside the finite Float range").raise()
    }
    return result
  }
}

impl HasQuantile for Exponential {
  type Quantile = Float

  quantile(_ probability: Float) -> Float {
    DistributionNumeric.validateProbability(probability, "Exponential quantile probability")
    if probability == 1.0 { return 1.0 / 0.0 }
    if probability == 0.0 { return 0.0 }
    let result = -math.ln1p(-probability) / self.rate
    if not result.isFinite {
      NumericalRangeError.new("Exponential quantile is outside the finite Float range").raise()
    }
    return result
  }

  upperQuantile(_ upperProbability: Float) -> Float {
    DistributionNumeric.validateProbability(upperProbability, "Exponential upper quantile probability")
    if upperProbability == 0.0 { return 1.0 / 0.0 }
    if upperProbability == 1.0 { return 0.0 }
    let result = -math.ln(upperProbability) / self.rate
    if not result.isFinite {
      NumericalRangeError.new("Exponential upper quantile is outside the finite Float range").raise()
    }
    return result
  }
}

data Normal private(_ mean: Float, _ stddev: Float)

impl Normal {
  @class
  new(mean: Float, stddev: Float) -> Normal {
    if not mean.isFinite or not stddev.isFinite or stddev <= 0.0 {
      ArgumentError.new("Normal requires finite mean and positive finite stddev").raise()
    }
    return Normal(mean, stddev)
  }

  @class
  fromVariance(mean: Float, variance: Float) -> Normal {
    if not variance.isFinite or variance <= 0.0 {
      ArgumentError.new("Normal variance must be positive and finite").raise()
    }
    let stddev = math.sqrt(variance)
    if not stddev.isFinite or stddev <= 0.0 {
      NumericalRangeError.new("Normal variance cannot be represented as a positive stddev").raise()
    }
    return Normal.new(mean: mean, stddev: stddev)
  }

  support -> RealSupport { RealSupport.allReal() }

  variance -> Float {
    let result = self.stddev * self.stddev
    if not result.isFinite { NumericalRangeError.new("Normal variance is outside the finite Float range").raise() }
    return result
  }
}

impl Distribution for Normal {
  type Output = Float

  sample<G>(using generator: G) -> Float where G |= Generator {
    let first = generator.openUnit
    let second = generator.nextFloat
    let radius = math.sqrt(-2.0 * math.ln(first))
    let standard = radius * math.cos(6.283185307179586 * second)
    return DistributionNumeric.normalAffine(self.mean, self.stddev, standard)
  }
}

impl HasDensity for Normal {
  type Observation = Float

  pdf(_ value: Float) -> Float {
    let logDensity = self.logPdf(value)
    let result = math.exp(logDensity)
    if not result.isFinite {
      NumericalRangeError.new("Normal density is outside the finite Float range").raise()
    }
    return result
  }

  logPdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Normal log density does not accept NaN").raise() }
    if value.isInfinite { return -1.0 / 0.0 }
    let z = DistributionNumeric.normalZ(value, mean: self.mean, stddev: self.stddev)
    let square = z * z
    if not square.isFinite {
      NumericalRangeError.new("Normal log density is outside the finite Float range").raise()
    }
    return -0.5 * square - math.ln(self.stddev) - 0.9189385332046727
  }
}

impl HasOrderedTails for Normal {
  type Cut = Float

  cdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Normal CDF does not accept NaN").raise() }
    if value == 1.0 / 0.0 { return 1.0 }
    if value == -1.0 / 0.0 { return 0.0 }
    let z = DistributionNumeric.normalZ(value, mean: self.mean, stddev: self.stddev)
    return 0.5 * special.erfc(-z / 1.4142135623730951)
  }

  survival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Normal survival does not accept NaN").raise() }
    if value == 1.0 / 0.0 { return 0.0 }
    if value == -1.0 / 0.0 { return 1.0 }
    let z = DistributionNumeric.normalZ(value, mean: self.mean, stddev: self.stddev)
    return 0.5 * special.erfc(z / 1.4142135623730951)
  }

  logCdf(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Normal log CDF does not accept NaN").raise() }
    if value == 1.0 / 0.0 { return 0.0 }
    if value == -1.0 / 0.0 { return -1.0 / 0.0 }
    let z = DistributionNumeric.normalZ(value, mean: self.mean, stddev: self.stddev)
    return -0.6931471805599453 + special.logErfc(-z / 1.4142135623730951)
  }

  logSurvival(_ value: Float) -> Float {
    if value.isNaN { ArgumentError.new("Normal log survival does not accept NaN").raise() }
    if value == 1.0 / 0.0 { return -1.0 / 0.0 }
    if value == -1.0 / 0.0 { return 0.0 }
    let z = DistributionNumeric.normalZ(value, mean: self.mean, stddev: self.stddev)
    return -0.6931471805599453 + special.logErfc(z / 1.4142135623730951)
  }
}

impl HasQuantile for Normal {
  type Quantile = Float

  quantile(_ probability: Float) -> Float {
    DistributionNumeric.validateProbability(probability, "Normal quantile probability")
    if probability == 0.0 { return -1.0 / 0.0 }
    if probability == 1.0 { return 1.0 / 0.0 }
    let z = special.inverseNormal(probability)
    return DistributionNumeric.normalAffine(self.mean, self.stddev, z)
  }

  upperQuantile(_ upperProbability: Float) -> Float {
    DistributionNumeric.validateProbability(upperProbability, "Normal upper quantile probability")
    if upperProbability == 0.0 { return 1.0 / 0.0 }
    if upperProbability == 1.0 { return -1.0 / 0.0 }
    let z = -special.inverseNormal(upperProbability)
    return DistributionNumeric.normalAffine(self.mean, self.stddev, z)
  }
}

export Uniform, Exponential, Normal, DistributionNumeric
