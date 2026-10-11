import universe.math as math
from universe.collections.iterator_laws import Iterable
from universe.math import NumericalRangeError

class UndefinedNormalizationError is Error {}

/// Checked probability transformations and private-snapshot normalization.
class ProbabilityHelpers {
  @class
  probability(_ p: Float) {
    if not p.isFinite or p < 0.0 or p > 1.0 {
      ArgumentError.new("probability must be finite and in [0, 1]").raise()
    }
  }

  @class
  logit(_ p: Float) -> Float {
    ProbabilityHelpers.probability(p)
    if p == 0.0 { return -1.0 / 0.0 }
    if p == 1.0 { return 1.0 / 0.0 }
    return math.ln(p) - math.ln1p(-p)
  }

  @class
  odds(_ p: Float) -> Float {
    ProbabilityHelpers.probability(p)
    if p == 1.0 { return 1.0 / 0.0 }
    if p == 0.0 { return 0.0 }
    return p / (1.0 - p)
  }

  @class
  invLogit(_ x: Float) -> Float {
    if x.isNaN { ArgumentError.new("invLogit does not accept NaN").raise() }
    if x >= 0.0 { return 1.0 / (1.0 + math.exp(-x)) }
    let e = math.exp(x)
    return e / (1.0 + e)
  }

  @class
  snapshot<I>(_ input: I) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let result = List<Float>.new()
    let iterator = input.iterator
    while let Option::Some(value) = iterator.next { result.append(value) }
    if result.size == 0 { ArgumentError.new("normalization requires a nonempty input").raise() }
    return result
  }

  @class
  normalizeWeights<I>(_ input: I) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = ProbabilityHelpers.snapshot(input)
    let mut maximum = 0.0
    let mut i = 0
    while i < values.size {
      let x: Float = values.at(i)
      if not x.isFinite or x < 0.0 { ArgumentError.new("weights must be non-negative and finite").raise() }
      if x > maximum { maximum = x }
      i = i + 1
    }
    if maximum == 0.0 { UndefinedNormalizationError.new("weights have no positive mass").raise() }
    let mut sum = 0.0
    let mut correction = 0.0
    i = 0
    while i < values.size {
      let x: Float = values.at(i)
      let term = x / maximum
      let next = sum + term
      if sum >= term {
        correction = correction + ((sum - next) + term)
      } else {
        correction = correction + ((term - next) + sum)
      }
      sum = next
      i = i + 1
    }
    let total = sum + correction
    let result = List<Float>.new()
    i = 0
    while i < values.size {
      let x: Float = values.at(i)
      result.append((x / maximum) / total)
      i = i + 1
    }
    return result
  }

  @class
  normalized<I>(_ input: I, logs: Bool) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = ProbabilityHelpers.snapshot(input)
    let mut maximum = -1.0 / 0.0
    let mut i = 0
    while i < values.size {
      let x: Float = values.at(i)
      if x.isNaN or x == 1.0 / 0.0 { ArgumentError.new("log weights must be finite or negative infinity").raise() }
      if x > maximum { maximum = x }
      i = i + 1
    }
    if maximum == -1.0 / 0.0 { UndefinedNormalizationError.new("log weights have no possible event").raise() }
    let shifted = List<Float>.new()
    i = 0
    while i < values.size {
      let x: Float = values.at(i)
      let delta = x - maximum
      if logs and x.isFinite and not delta.isFinite {
        NumericalRangeError.new("normalized log weight is outside Float range").raise()
      }
      shifted.append(delta)
      i = i + 1
    }
    let normalizer = math.logSumExp(shifted)
    let result = List<Float>.new()
    i = 0
    while i < shifted.size {
      let x: Float = shifted.at(i)
      let normalized = x - normalizer
      if logs {
        if x.isFinite and not normalized.isFinite { NumericalRangeError.new("normalized log weight is outside Float range").raise() }
        result.append(normalized)
      } else {
        result.append(math.exp(normalized))
      }
      i = i + 1
    }
    return result
  }

  @class
  normalizeLogWeights<I>(_ input: I) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  { ProbabilityHelpers.normalized(input, logs: false) }

  @class
  normalizedLogWeights<I>(_ input: I) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  { ProbabilityHelpers.normalized(input, logs: true) }
}

export ProbabilityHelpers, UndefinedNormalizationError
