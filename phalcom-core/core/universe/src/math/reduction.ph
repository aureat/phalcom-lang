from .float_kernel import FloatMathKernel

/// Stable Float reductions. The input List is read without mutation.
class FloatReduction {
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
