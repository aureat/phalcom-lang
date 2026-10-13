from .float_kernel import FloatMathKernel
from .numerical_errors import NumericalRangeError
from universe.collections.iteration import Iterable

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
    let result: Option<Float> = FloatReduction.meanOption(values)
    return result.match(
      some: |value| { value },
      none: || {
        ArgumentError.new("mean requires a nonempty input").raise()
        0.0
      }
    )
  }

  /// Returns None for an empty input; otherwise computes the stable mean in one pass.
  @class
  meanOption<I>(_ values: I) -> Option<Float>
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
    if count == 0 { return Option::None }
    if scale == 0.0 { return Option::Some(0.0) }
    let normalized = (total + correction) / Float.new(count)
    let result = scale * normalized
    if not result.isFinite {
      NumericalRangeError.new("Float mean is outside the finite binary64 range").raise()
    }
    return Option::Some(result)
  }

  /// Stable two-term log-domain addition with explicit IEEE edge handling.
  @class
  logAddExp(_ a: Float, _ b: Float) -> Float {
    if a != a { return a }
    if b != b { return b }
    let infinity = 1.0 / 0.0
    if a == infinity or b == infinity { return infinity }
    if a == -infinity { return b }
    if b == -infinity { return a }
    let maximum = if a >= b { a } else { b }
    return maximum + FloatMathKernel.ln1p(FloatMathKernel.exp(-(a - b).abs))
  }

  /// Stable logarithm of 1 - exp(x), defined for real x <= 0.
  @class
  log1mexp(_ x: Float) -> Float {
    if x != x { return x }
    let infinity = 1.0 / 0.0
    if x > 0.0 { return 0.0 / 0.0 }
    if x == 0.0 { return -infinity }
    if x == -infinity { return 0.0 }
    if x < -0.6931471805599453 {
      return FloatMathKernel.ln1p(-FloatMathKernel.exp(x))
    }
    return FloatMathKernel.ln(-FloatMathKernel.expm1(x))
  }

  @class
  logSumExp<I>(_ values: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let infinity = 1.0 / 0.0
    let mut maximum = -infinity
    let mut total = 0.0
    let mut correction = 0.0
    let mut iterator = values.iterator
    let mut step = iterator.next
    while step.isSome {
      let x: Float = step.match(some: |value| { value }, none: || { 0.0 })
      if x != x { return x }
      if x == infinity {
        maximum = infinity
        total = 0.0
        correction = 0.0
      } else if maximum != infinity and x != -infinity {
        if maximum == -infinity {
          maximum = x
        } else if x > maximum {
          let oldMaximum = maximum
          let ratio = FloatMathKernel.exp(oldMaximum - x)
          total = total * ratio
          correction = correction * ratio
          maximum = x
          let term = ratio
          let next = total + term
          if total.abs >= term.abs {
            correction = correction + ((total - next) + term)
          } else {
            correction = correction + ((term - next) + total)
          }
          total = next
        } else {
          let term = FloatMathKernel.exp(x - maximum)
          let next = total + term
          if total.abs >= term.abs {
            correction = correction + ((total - next) + term)
          } else {
            correction = correction + ((term - next) + total)
          }
          total = next
        }
      }
      step = iterator.next
    }
    if maximum == infinity or maximum == -infinity { return maximum }
    return maximum + FloatMathKernel.ln1p(total + correction)
  }
}

export FloatReduction
