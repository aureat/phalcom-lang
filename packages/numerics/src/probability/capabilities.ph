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

export HasMass, HasDensity, HasOrderedTails, HasQuantile
