from .float_kernel import FloatMathKernel
from .numerical_errors import NumericalRangeError
from universe.collections.iterator_laws import Iterable

/// Stable Float reductions over one-pass inputs; no source value is mutated.
class FloatReduction {
  /// Compensated, range-aware sum for one finite Float stream.
  @class
  sum<I>(_ values: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let mut iterator = values.iterator
    let mut scale = 0.0
    let mut total = 0.0
    let mut correction = 0.0
    let mut step = iterator.next
    while step.isSome {
      let x: Float = step.match(some: |value| { value }, none: || { 0.0 })
      if not x.isFinite { ArgumentError.new("Float sum requires finite values").raise() }
      let magnitude = x.abs
      if magnitude > scale {
        if scale != 0.0 {
          let ratio = scale / magnitude
          total = total * ratio
          correction = correction * ratio
        }
        scale = magnitude
      }
      if scale != 0.0 {
        let term = x / scale
        let next = total + term
        if total.abs >= term.abs {
          correction = correction + ((total - next) + term)
        } else {
          correction = correction + ((term - next) + total)
        }
        total = next
      }
      step = iterator.next
    }
    if scale == 0.0 { return 0.0 }
    let result = scale * (total + correction)
    if not result.isFinite {
      NumericalRangeError.new("Float sum is outside the finite binary64 range").raise()
    }
    return result
  }

  /// Range-aware mean for one finite Float stream.
  @class
  mean<I>(_ values: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let mut iterator = values.iterator
    let mut scale = 0.0
    let mut total = 0.0
    let mut correction = 0.0
    let mut count = 0
    let mut step = iterator.next
    while step.isSome {
      let x: Float = step.match(some: |value| { value }, none: || { 0.0 })
      if not x.isFinite { ArgumentError.new("Float mean requires finite values").raise() }
      if count >= 9007199254740991 {
        NumericalRangeError.new("mean count exceeds the exact Float integer range").raise()
      }
      let magnitude = x.abs
      if magnitude > scale {
        if scale != 0.0 {
          let ratio = scale / magnitude
          total = total * ratio
          correction = correction * ratio
        }
        scale = magnitude
      }
      if scale != 0.0 {
        let term = x / scale
        let next = total + term
        if total.abs >= term.abs {
          correction = correction + ((total - next) + term)
        } else {
          correction = correction + ((term - next) + total)
        }
        total = next
      }
      count = count + 1
      step = iterator.next
    }
    if count == 0 { ArgumentError.new("mean requires a nonempty input").raise() }
    if scale == 0.0 { return 0.0 }
    let normalized = (total + correction) / Float.new(count)
    let result = scale * normalized
    if not result.isFinite {
      NumericalRangeError.new("Float mean is outside the finite binary64 range").raise()
    }
    return result
  }

  @class
  logSumExp(_ values: List<Float>) -> Float {
    let infinity = 1.0 / 0.0
    let mut maximum = -infinity
    let mut maximumIndex = -1
    let mut i = 0
    // Scan the entire input so NaN takes precedence over positive infinity.
    while i < values.size {
      let x: Float = values.at(i)
      if x != x { return x }
      if x > maximum {
        maximum = x
        maximumIndex = i
      }
      i = i + 1
    }
    if maximum == infinity or maximum == -infinity { return maximum }

    // Omit one maximum term (exactly 1) and accumulate the residual using
    // Neumaier compensation. ln1p preserves tiny corrections to that term.
    let mut total = 0.0
    let mut correction = 0.0
    i = 0
    while i < values.size {
      if i != maximumIndex {
        let x: Float = values.at(i)
        let term = FloatMathKernel.exp(x - maximum)
        let next = total + term
        if total >= term {
          correction = correction + ((total - next) + term)
        } else {
          correction = correction + ((term - next) + total)
        }
        total = next
      }
      i = i + 1
    }
    return maximum + FloatMathKernel.ln1p(total + correction)
  }
}

export FloatReduction
