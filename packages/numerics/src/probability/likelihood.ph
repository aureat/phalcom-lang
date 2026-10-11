import universe.math as math
from universe.math import NumericalRangeError, NumericalEvaluationError
from universe.collections.iterator_laws import Iterable
from .capabilities import HasMass, HasDensity, HasOrderedTails, HasQuantile

class Likelihood {
  @class
  iidLogMassLikelihood<D, I>(_ distribution: D, observations: I) -> Float
    where D |= HasMass, I |= Iterable,
      <I as Iterable>::Item == <D as HasMass>::Observation
  {
    let mut iterator = observations.iterator
    let mut sum = 0.0
    let mut correction = 0.0
    let mut impossible = false
    let mut step = iterator.next
    while step.isSome {
      let observation = step.match(some: |value| { value }, none: || { None })
      let contribution = distribution.logPmf(observation)
      if contribution.isNaN or contribution == 1.0 / 0.0 {
        NumericalEvaluationError.new("mass log likelihood returned NaN or positive infinity").raise()
      }
      if contribution == -1.0 / 0.0 {
        impossible = true
      } else if not impossible {
        let next = sum + contribution
        if sum.abs >= contribution.abs {
          correction = correction + ((sum - next) + contribution)
        } else {
          correction = correction + ((contribution - next) + sum)
        }
        sum = next
      }
      step = iterator.next
    }
    if impossible { return -1.0 / 0.0 }
    let result = sum + correction
    if not result.isFinite {
      NumericalRangeError.new("mass log likelihood is outside the finite Float range").raise()
    }
    return result
  }

  @class
  iidLogDensityLikelihood<D, I>(_ distribution: D, observations: I) -> Float
    where D |= HasDensity, I |= Iterable,
      <I as Iterable>::Item == <D as HasDensity>::Observation
  {
    let mut iterator = observations.iterator
    let mut sum = 0.0
    let mut correction = 0.0
    let mut positiveInfinity = false
    let mut negativeInfinity = false
    let mut step = iterator.next
    while step.isSome {
      let observation = step.match(some: |value| { value }, none: || { None })
      let contribution = distribution.logPdf(observation)
      if contribution.isNaN {
        NumericalEvaluationError.new("density log likelihood produced an indeterminate NaN").raise()
      }
      if contribution == 1.0 / 0.0 {
        positiveInfinity = true
      } else if contribution == -1.0 / 0.0 {
        negativeInfinity = true
      } else {
        let next = sum + contribution
        if sum.abs >= contribution.abs {
          correction = correction + ((sum - next) + contribution)
        } else {
          correction = correction + ((contribution - next) + sum)
        }
        sum = next
      }
      step = iterator.next
    }
    if positiveInfinity and negativeInfinity {
      NumericalEvaluationError.new("density log likelihood combines positive and negative infinity").raise()
    }
    if positiveInfinity { return 1.0 / 0.0 }
    if negativeInfinity { return -1.0 / 0.0 }
    let result = sum + correction
    if not result.isFinite {
      NumericalRangeError.new("density log likelihood is outside the finite Float range").raise()
    }
    return result
  }

  @class
  tailDiagnostics<D>(_ distribution: D, at cut: <D as HasOrderedTails>::Cut) -> Tuple<***(Float, Float, Float, Float)>
    where D |= HasOrderedTails
  {
    return (
      distribution.cdf(cut),
      distribution.survival(cut),
      distribution.logCdf(cut),
      distribution.logSurvival(cut)
    )
  }

  @class
  equalTailedInterval<D>(_ distribution: D, mass: Float) -> Tuple<***(<D as HasQuantile>::Quantile, <D as HasQuantile>::Quantile)>
    where D |= HasQuantile
  {
    if not mass.isFinite or mass <= 0.0 or mass >= 1.0 {
      ArgumentError.new("interval mass must be finite and in (0, 1)").raise()
    }
    let tail = (1.0 - mass) / 2.0
    return (distribution.quantile(tail), distribution.upperQuantile(tail))
  }
}

export Likelihood
