import universe.math as math
from universe.math import NumericalRangeError
from universe.random import Distribution, Generator
from universe.collections.iterator_laws import Iterable
from .capabilities import HasMass
from .probability_helpers import UndefinedNormalizationError
from .support import FiniteSupport

const maxCategoricalEntries: Int = 1000000

/// Copied immutable-by-interface finite label law. Internal tables are private.
class Categorical<T> {
  _labels: List<T>
  _probabilities: List<Float>
  _logMasses: List<Float>
  _prefixes: List<Float>
  _lastPositive: Int
  _support: FiniteSupport<T>

  @constructor
  private create(
    _ labels: List<T>,
    _ probabilities: List<Float>,
    _ logMasses: List<Float>,
    _ prefixes: List<Float>,
    _ lastPositive: Int,
    _ support: FiniteSupport<T>
  ) {
    _labels = labels
    _probabilities = probabilities
    _logMasses = logMasses
    _prefixes = prefixes
    _lastPositive = lastPositive
    _support = support
  }

  @class
  fromWeights<L, W>(_ labels: L, weights weightInput: W) -> Categorical<T>
    where L |= Iterable, W |= Iterable,
      <L as Iterable>::Item == T, <W as Iterable>::Item == Float
  {
    let labelValues = Categorical.copyLabels(labels)
    let weights = Categorical.copyWeights(weightInput)
    if labelValues.size == 0 { ArgumentError.new("Categorical requires a nonempty label table").raise() }
    if labelValues.size != weights.size {
      ArgumentError.new("Categorical labels and weights must have equal lengths").raise()
    }
    Categorical.requireUnique(labelValues)

    let mut maximum = 0.0
    let mut i = 0
    while i < weights.size {
      let weight: Float = weights.at(i)
      if not weight.isFinite or weight < 0.0 {
        ArgumentError.new("Categorical weights must be finite and nonnegative").raise()
      }
      if weight > maximum { maximum = weight }
      i = i + 1
    }
    if maximum == 0.0 {
      UndefinedNormalizationError.new("Categorical weights have no positive mass").raise()
    }

    let mut total = 0.0
    let mut correction = 0.0
    i = 0
    while i < weights.size {
      let term: Float = weights.at(i) / maximum
      let next = total + term
      if total >= term {
        correction = correction + ((total - next) + term)
      } else {
        correction = correction + ((term - next) + total)
      }
      total = next
      i = i + 1
    }
    let normalizedTotal = total + correction
    let probabilities = List<Float>.new()
    let logMasses = List<Float>.new()
    let positiveLabels = List<T>.new()
    let mut lastPositive = -1
    i = 0
    while i < weights.size {
      let weight: Float = weights.at(i)
      if weight == 0.0 {
        probabilities.append(0.0)
        logMasses.append(-1.0 / 0.0)
      } else {
        let probability = (weight / maximum) / normalizedTotal
        let logMass = math.ln(weight) - math.ln(maximum) - math.ln(normalizedTotal)
        if not logMass.isFinite {
          NumericalRangeError.new("Categorical normalized log mass is outside Float range").raise()
        }
        probabilities.append(probability)
        logMasses.append(logMass)
        positiveLabels.append(labelValues.at(i))
        lastPositive = i
      }
      i = i + 1
    }
    return Categorical.create(
      labelValues,
      probabilities,
      logMasses,
      Categorical.prefixes(probabilities, lastPositive),
      lastPositive,
      FiniteSupport.from(positiveLabels)
    )
  }

  @class
  fromProbabilities<L, P>(_ labels: L, probabilities probabilityInput: P) -> Categorical<T>
    where L |= Iterable, P |= Iterable,
      <L as Iterable>::Item == T, <P as Iterable>::Item == Float
  {
    let probabilities = Categorical.copyWeights(probabilityInput)
    if probabilities.size == 0 { ArgumentError.new("Categorical requires a nonempty probability table").raise() }
    let mut total = 0.0
    let mut correction = 0.0
    let mut i = 0
    while i < probabilities.size {
      let value: Float = probabilities.at(i)
      if not value.isFinite or value < 0.0 {
        ArgumentError.new("Categorical probabilities must be finite and nonnegative").raise()
      }
      let next = total + value
      if total >= value {
        correction = correction + ((total - next) + value)
      } else {
        correction = correction + ((value - next) + total)
      }
      total = next
      i = i + 1
    }
    let sum = total + correction
    if not sum.isFinite or (sum - 1.0).abs > 0.00000001 {
      ArgumentError.new("Categorical probabilities must sum to one within 1e-8").raise()
    }
    return Categorical.fromWeights(labels, weights: probabilities)
  }

  @class
  fromLogWeights<L, W>(_ labels: L, logWeights weightInput: W) -> Categorical<T>
    where L |= Iterable, W |= Iterable,
      <L as Iterable>::Item == T, <W as Iterable>::Item == Float
  {
    let labelValues = Categorical.copyLabels(labels)
    let logWeights = Categorical.copyWeights(weightInput)
    if labelValues.size == 0 { ArgumentError.new("Categorical requires a nonempty label table").raise() }
    if labelValues.size != logWeights.size {
      ArgumentError.new("Categorical labels and log weights must have equal lengths").raise()
    }
    Categorical.requireUnique(labelValues)
    let mut maximum = -1.0 / 0.0
    let mut i = 0
    while i < logWeights.size {
      let value: Float = logWeights.at(i)
      if value.isNaN or value == 1.0 / 0.0 {
        ArgumentError.new("Categorical log weights must be finite or negative infinity").raise()
      }
      if value > maximum { maximum = value }
      i = i + 1
    }
    if maximum == -1.0 / 0.0 {
      UndefinedNormalizationError.new("Categorical log weights have no possible event").raise()
    }

    let shifted = List<Float>.new()
    i = 0
    while i < logWeights.size {
      let value: Float = logWeights.at(i)
      shifted.append(value - maximum)
      i = i + 1
    }
    let correction = math.logSumExp(shifted)
    let probabilities = List<Float>.new()
    let normalizedLogs = List<Float>.new()
    let positiveLabels = List<T>.new()
    let mut lastPositive = -1
    i = 0
    while i < logWeights.size {
      let value: Float = logWeights.at(i)
      if value == -1.0 / 0.0 {
        probabilities.append(0.0)
        normalizedLogs.append(-1.0 / 0.0)
      } else {
        let logMass = (value - maximum) - correction
        if not logMass.isFinite {
          NumericalRangeError.new("Categorical normalized log mass is outside Float range").raise()
        }
        probabilities.append(math.exp(logMass))
        normalizedLogs.append(logMass)
        positiveLabels.append(labelValues.at(i))
        lastPositive = i
      }
      i = i + 1
    }
    return Categorical.create(
      labelValues,
      probabilities,
      normalizedLogs,
      Categorical.prefixes(probabilities, lastPositive),
      lastPositive,
      FiniteSupport.from(positiveLabels)
    )
  }

  /// A copied view of every label in construction order, including zero-mass labels.
  labels -> List<T> {
    let result = List<T>.new()
    let mut i = 0
    while i < _labels.size {
      result.append(_labels.at(i))
      i = i + 1
    }
    return result
  }

  /// Mathematical support, preserving positive source weights after Float underflow.
  support -> FiniteSupport<T> { _support }

  @class
  private copyLabels<L>(_ input: L) -> List<T>
    where L |= Iterable, <L as Iterable>::Item == T
  {
    let result = List<T>.new()
    let mut iterator = input.iterator
    let mut step = iterator.next
    while step.isSome {
      if result.size >= maxCategoricalEntries {
        NumericalRangeError.new("Categorical table exceeds its entry limit").raise()
      }
      result.append(step.match(some: |value| { value }, none: || { None }))
      step = iterator.next
    }
    return result
  }

  @class
  private copyWeights<W>(_ input: W) -> List<Float>
    where W |= Iterable, <W as Iterable>::Item == Float
  {
    let result = List<Float>.new()
    let mut iterator = input.iterator
    let mut step = iterator.next
    while step.isSome {
      if result.size >= maxCategoricalEntries {
        NumericalRangeError.new("Categorical table exceeds its entry limit").raise()
      }
      result.append(step.match(some: |value| { value }, none: || { 0.0 }))
      step = iterator.next
    }
    return result
  }

  @class
  private requireUnique(_ labels: List<T>) -> Unit {
    let seen = Set<T>.new()
    let mut i = 0
    while i < labels.size {
      let label: T = labels.at(i)
      if seen.contains(label) {
        ArgumentError.new("Categorical labels must be unique under stable equality").raise()
      }
      seen.add(label)
      i = i + 1
    }
    return ()
  }

  @class
  private prefixes(_ probabilities: List<Float>, _ lastPositive: Int) -> List<Float> {
    let result = List<Float>.new()
    let mut total = 0.0
    let mut i = 0
    while i < probabilities.size {
      if i >= lastPositive {
        total = 1.0
      } else {
        total = total + probabilities.at(i)
      }
      result.append(total)
      i = i + 1
    }
    return result
  }

  private find(_ value: T) -> Int {
    let mut i = 0
    while i < _labels.size {
      if _labels.at(i) == value { return i }
      i = i + 1
    }
    return -1
  }
}

impl HasMass for Categorical<T> {
  type Observation = T

  pmf(_ value: T) -> Float {
    let index = self.find(value)
    if index < 0 { return 0.0 }
    return _probabilities.at(index)
  }

  logPmf(_ value: T) -> Float {
    let index = self.find(value)
    if index < 0 { return -1.0 / 0.0 }
    return _logMasses.at(index)
  }
}

impl Distribution for Categorical<T> {
  type Output = T

  sample<G>(using generator: G) -> T where G |= Generator {
    let target = generator.nextFloat
    let mut low = 0
    let mut high = _prefixes.size
    while low < high {
      let middle = low + ((high - low) ~/ 2)
      if target < _prefixes.at(middle) {
        high = middle
      } else {
        low = middle + 1
      }
    }
    if low >= _labels.size { low = _lastPositive }
    return _labels.at(low)
  }
}

export Categorical
