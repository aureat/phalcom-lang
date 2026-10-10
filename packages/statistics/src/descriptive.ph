import universe.math as math
from universe.numbers import Rational
from universe.math import NumericalRangeError
from universe.collections.iterator_laws import Iterable
from .moments import MomentKernel, MomentSummary
from .covariance import CovarianceKernel

class EmptyInputError is Error {}
class UndefinedStatisticError is Error {}

const maxApproximateCount: Int = 9007199254740991

/// Descriptive Float statistics. Inputs are consumed once and never mutated.
class Descriptive {
  @class
  private snapshot<I>(_ input: I) -> List<Float>
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let result = List<Float>.new()
    let mut iterator = input.iterator
    let mut ordinal = 0
    let mut step = iterator.next
    while step.isSome {
      let value: Float = step.match(some: |x| { x }, none: || { 0.0 })
      if not value.isFinite {
        ArgumentError.new("statistics require finite Float observations").raise()
      }
      if ordinal >= maxApproximateCount {
        NumericalRangeError.new("observation count exceeds the exact Float count range").raise()
      }
      result.append(value)
      ordinal = ordinal + 1
      step = iterator.next
    }
    return result
  }

  @class
  sum<I>(_ values: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  { math.sum(values) }

  @class
  sumInts<I>(_ values: I) -> Int
    where I |= Iterable, <I as Iterable>::Item == Int
  {
    let mut total = 0
    let mut iterator = values.iterator
    let mut step = iterator.next
    while step.isSome {
      let value: Int = step.match(some: |x| { x }, none: || { 0 })
      total = total + value
      step = iterator.next
    }
    return total
  }

  @class
  meanExact<I>(_ values: I) -> Rational
    where I |= Iterable, <I as Iterable>::Item == Int
  {
    let mut total = 0
    let mut count = 0
    let mut iterator = values.iterator
    let mut step = iterator.next
    while step.isSome {
      let value: Int = step.match(some: |x| { x }, none: || { 0 })
      total = total + value
      count = count + 1
      step = iterator.next
    }
    if count == 0 { EmptyInputError.new("meanExact requires a nonempty input").raise() }
    return Rational.new(total, count)
  }

  @class
  mean<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = Descriptive.snapshot(input)
    if values.size == 0 { EmptyInputError.new("mean requires a nonempty input").raise() }
    return math.meanList(values)
  }

  @class
  minimum<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = Descriptive.snapshot(input)
    if values.size == 0 { EmptyInputError.new("minimum requires a nonempty input").raise() }
    let mut result: Float = values.at(0)
    let mut i = 1
    while i < values.size {
      let value: Float = values.at(i)
      if value < result { result = value }
      i = i + 1
    }
    return result
  }

  @class
  maximum<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = Descriptive.snapshot(input)
    if values.size == 0 { EmptyInputError.new("maximum requires a nonempty input").raise() }
    let mut result: Float = values.at(0)
    let mut i = 1
    while i < values.size {
      let value: Float = values.at(i)
      if value > result { result = value }
      i = i + 1
    }
    return result
  }

  @class
  extrema<I>(_ input: I) -> Tuple<***(Float, Float)>
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let values = Descriptive.snapshot(input)
    if values.size == 0 { EmptyInputError.new("extrema requires a nonempty input").raise() }
    let mut low: Float = values.at(0)
    let mut high: Float = values.at(0)
    let mut i = 1
    while i < values.size {
      let value: Float = values.at(i)
      if value < low { low = value }
      if value > high { high = value }
      i = i + 1
    }
    return (low, high)
  }

  @class
  span<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let bounds = Descriptive.extrema(input)
    let result = bounds[1] - bounds[0]
    if not result.isFinite {
      NumericalRangeError.new("span is outside the finite Float range").raise()
    }
    return result
  }

  @class
  populationVariance<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let summary = MomentKernel.from(input)
    let result: Float = match summary.populationVariance {
      Option::Some(value) => value
      Option::None => {
        EmptyInputError.new("population variance requires a nonempty input").raise()
        0.0
      }
    }
    return result
  }

  @class
  sampleVariance<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let summary = MomentKernel.from(input)
    let result: Float = match summary.sampleVariance {
      Option::Some(value) => value
      Option::None => {
        UndefinedStatisticError.new("sample variance requires at least two values").raise()
        0.0
      }
    }
    return result
  }

  @class
  populationStddev<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  { math.sqrt(Descriptive.populationVariance(input)) }

  @class
  sampleStddev<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  { math.sqrt(Descriptive.sampleVariance(input)) }

  @class
  populationCovariance<I, J>(_ xs: I, with ys: J) -> Float
    where I |= Iterable, J |= Iterable,
      <I as Iterable>::Item == Float, <J as Iterable>::Item == Float
  {
    let summary = CovarianceKernel.from(xs, with: ys)
    let result: Float = match summary.populationCovariance {
      Option::Some(value) => value
      Option::None => {
        EmptyInputError.new("population covariance requires a nonempty input").raise()
        0.0
      }
    }
    return result
  }

  @class
  sampleCovariance<I, J>(_ xs: I, with ys: J) -> Float
    where I |= Iterable, J |= Iterable,
      <I as Iterable>::Item == Float, <J as Iterable>::Item == Float
  {
    let summary = CovarianceKernel.from(xs, with: ys)
    let result: Float = match summary.sampleCovariance {
      Option::Some(value) => value
      Option::None => {
        UndefinedStatisticError.new("sample covariance requires at least two pairs").raise()
        0.0
      }
    }
    return result
  }

  @class
  correlation<I, J>(_ xs: I, with ys: J) -> Float
    where I |= Iterable, J |= Iterable,
      <I as Iterable>::Item == Float, <J as Iterable>::Item == Float
  {
    let summary = CovarianceKernel.from(xs, with: ys)
    let result: Float = match summary.correlation {
      Option::Some(value) => value
      Option::None => {
        UndefinedStatisticError.new("correlation requires at least two pairs and positive variance on both sides").raise()
        0.0
      }
    }
    return result
  }

  @class
  median<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  { Descriptive.quantile(input, probability: 0.5) }

  @class
  quantile<I>(_ input: I, probability q: Float) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    if not q.isFinite or q < 0.0 or q > 1.0 {
      ArgumentError.new("quantile probability must be finite and in [0, 1]").raise()
    }
    let values = Descriptive.snapshot(input)
    if values.size == 0 { EmptyInputError.new("quantile requires a nonempty input").raise() }
    Descriptive.heapSort(values)
    if values.size == 1 or q == 0.0 { return values.at(0) }
    if q == 1.0 { return values.at(values.size - 1) }

    let position = Float.new(values.size - 1) * q
    let lowerIndex = position.floor
    let upperIndex = lowerIndex + 1
    let fraction = position - Float.new(lowerIndex)
    let lower: Float = values.at(lowerIndex)
    let upper: Float = values.at(upperIndex)
    if lower <= 0.0 and upper >= 0.0 or lower >= 0.0 and upper <= 0.0 {
      return lower * (1.0 - fraction) + upper * fraction
    }
    return lower + (upper - lower) * fraction
  }

  @class
  weightedMean<I, W>(_ input: I, weights weightInput: W) -> Float
    where I |= Iterable, W |= Iterable,
      <I as Iterable>::Item == Float, <W as Iterable>::Item == Float
  {
    let values = Descriptive.snapshot(input)
    let weights = Descriptive.snapshot(weightInput)
    if values.size != weights.size {
      ArgumentError.new("weightedMean requires equal-length values and weights").raise()
    }
    if values.size == 0 { EmptyInputError.new("weightedMean requires a nonempty input").raise() }

    let mut maxWeight = 0.0
    let mut scale = 0.0
    let mut i = 0
    while i < values.size {
      let weight: Float = weights.at(i)
      let value: Float = values.at(i)
      if weight < 0.0 { ArgumentError.new("weights must be nonnegative").raise() }
      if weight > maxWeight { maxWeight = weight }
      if value.abs > scale { scale = value.abs }
      i = i + 1
    }
    if maxWeight == 0.0 {
      UndefinedStatisticError.new("weightedMean requires at least one positive weight").raise()
    }

    let mut denominator = 0.0
    let mut denominatorCorrection = 0.0
    let mut numerator = 0.0
    let mut numeratorCorrection = 0.0
    i = 0
    while i < values.size {
      let weight: Float = weights.at(i) / maxWeight
      let value: Float = values.at(i)
      let normalizedValue = if scale == 0.0 { 0.0 } else { value / scale }
      let denominatorNext = denominator + weight
      if denominator >= weight {
        denominatorCorrection = denominatorCorrection + ((denominator - denominatorNext) + weight)
      } else {
        denominatorCorrection = denominatorCorrection + ((weight - denominatorNext) + denominator)
      }
      denominator = denominatorNext

      let term = weight * normalizedValue
      let numeratorNext = numerator + term
      if numerator.abs >= term.abs {
        numeratorCorrection = numeratorCorrection + ((numerator - numeratorNext) + term)
      } else {
        numeratorCorrection = numeratorCorrection + ((term - numeratorNext) + numerator)
      }
      numerator = numeratorNext
      i = i + 1
    }
    if scale == 0.0 { return 0.0 }
    let normalized = (numerator + numeratorCorrection) / (denominator + denominatorCorrection)
    let result = scale * normalized
    if not result.isFinite {
      NumericalRangeError.new("weighted mean is outside the finite Float range").raise()
    }
    return result
  }

  @class
  private heapSort(_ values: List<Float>) -> Unit {
    let mut start = values.size / 2 - 1
    while start >= 0 {
      Descriptive.siftDown(values, start, values.size)
      start = start - 1
    }
    let mut end = values.size - 1
    while end > 0 {
      values.swap(first: 0, second: end).unwrap
      Descriptive.siftDown(values, 0, end)
      end = end - 1
    }
    return ()
  }

  @class
  private siftDown(_ values: List<Float>, _ rootIndex: Int, _ end: Int) -> Unit {
    let mut root = rootIndex
    let mut active = true
    while active {
      let child = root * 2 + 1
      if child >= end {
        active = false
      } else {
        let mut selected = child
        if child + 1 < end and values.at(child) < values.at(child + 1) {
          selected = child + 1
        }
        if values.at(root) < values.at(selected) {
          values.swap(first: root, second: selected).unwrap
          root = selected
        } else {
          active = false
        }
      }
    }
    return ()
  }
}

export Descriptive, MomentSummary, EmptyInputError, UndefinedStatisticError
