from .generator import Generator
from .distribution import Distribution

data Bernoulli private(_ probability: Float)

impl Bernoulli {
  @class
  new(_ probability: Float) -> Bernoulli {
    if probability.isNaN or probability < 0.0 or probability > 1.0 {
      ArgumentError.new("Bernoulli.new: probability must be in 0.0..=1.0").raise()
    }
    Bernoulli(probability)
  }

  /// Mean of the Bool indicator (false = 0, true = 1).
  mean -> Float { self.probability }

  /// Variance of the Bool indicator (false = 0, true = 1).
  variance -> Float { self.probability * (1.0 - self.probability) }
}

impl Distribution for Bernoulli {
  type Output = Bool
  sample<G>(using generator: G) -> Bool where G |= Generator {
    if self.probability == 0.0 { return false }
    if self.probability == 1.0 { return true }
    generator.nextFloat < self.probability
  }
}

export Bernoulli
