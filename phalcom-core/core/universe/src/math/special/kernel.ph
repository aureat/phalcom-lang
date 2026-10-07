from ..float_kernel import FloatMathKernel as F

from ..numerical_errors import NumericalRangeError, NumericalEvaluationError

/// Positive-real binary64 special functions. No host-library or native floor.
/// Formula sources and supported parameter region: see math/special/README.md.
class SpecialKernel {
  @class
  positive(_ x: Float) {
    if not x.isFinite or x <= 0.0 {
      ArgumentError.new("special function requires a positive finite parameter").raise()
    }
  }

  @class
  shape(_ x: Float) {
    SpecialKernel.positive(x)
    if x > 1000000.0 {
      NumericalRangeError.new("special-function shape exceeds 1000000").raise()
    }
  }

  @class
  unit(_ x: Float) {
    if not x.isFinite or x < 0.0 or x > 1.0 {
      ArgumentError.new("special function requires a probability in [0, 1]").raise()
    }
  }

  // DLMF 5.11.1, through B16; used only at x >= 16.
  @class
  correction(_ x: Float) -> Float {
    let r = 1.0 / x
    let z = r * r
    return r * (1.0 / 12.0 + z * (-1.0 / 360.0 + z * (1.0 / 1260.0 + z * (-1.0 / 1680.0 + z * (1.0 / 1188.0 + z * (-691.0 / 360360.0 + z * (1.0 / 156.0 + z * (-3617.0 / 122400.0))))))))
  }

  // Taylor expansion of lnGamma(1+x)/x, |x| <= 1/8. Coefficients
  // are (-1)^n*zeta(n)/n; retaining the ratio preserves subnormal inputs.
  @class
  logGammaOnePlusRatio(_ x: Float) -> Float {
    return (-0.5772156649015329 + x * (0.8224670334241132 + x * (-0.40068563438653143 + x * (0.27058080842778454 + x * (-0.20738555102867398 + x * (0.1695571769974082 + x * (-0.1440498967688461 + x * (0.12550966952474304 + x * (-0.11133426586956469 + x * (0.1000994575127818 + x * (-0.09095401714582904 + x * (0.083353840546109 + x * (-0.0769325164113522 + x * (0.07143294629536133 + x * (-0.06666870588242046 + x * (0.06250095514121304 + x * (-0.058823978658684585 + x * (0.055555767627403614 + x * (-0.05263167937961666 + x * (0.05000004769810169 + x * (-0.047619070330142226 + x * (0.04545455629320467 + x * (-0.04347826605304026 + x * 0.04166666915034121)))))))))))))))))))))))
  }

  @class
  logGammaOnePlus(_ x: Float) -> Float {
    if x.abs <= 0.125 { return x * SpecialKernel.logGammaOnePlusRatio(x) }
    return SpecialKernel.logGamma(1.0 + x)
  }

  @class
  logGamma(_ value: Float) -> Float {
    SpecialKernel.positive(value)
    if value == 1.0 or value == 2.0 { return 0.0 }
    if value < 0.125 { return SpecialKernel.logGammaOnePlus(value) - F.ln(value) }
    if (value - 1.0).abs <= 0.125 { return SpecialKernel.logGammaOnePlus(value - 1.0) }
    if (value - 2.0).abs <= 0.125 {
      return F.ln1p(value - 2.0) + SpecialKernel.logGammaOnePlus(value - 2.0)
    }
    let mut x = value
    let mut shift = 0.0
    while x < 16.0 {
      shift = shift - F.ln(x)
      x = x + 1.0
    }
    let result = (x - 0.5) * F.ln(x) - x + 0.9189385332046727 + SpecialKernel.correction(x) + shift
    if not result.isFinite { NumericalRangeError.new("logGamma result is outside Float range").raise() }
    return result
  }

  @class
  logBeta(_ a: Float, _ b: Float) -> Float {
    SpecialKernel.shape(a)
    SpecialKernel.shape(b)
    let mut x = a
    let mut y = b
    let mut shift = 0.0
    while x < 16.0 {
      // ln((x+y)/x), without overflowing y/x for subnormal x.
      shift = shift + F.ln(x + y) - F.ln(x)
      x = x + 1.0
    }
    while y < 16.0 {
      shift = shift + F.ln(x + y) - F.ln(y)
      y = y + 1.0
    }
    let total = x + y
    // Evaluate the entropy terms directly: avoid subtracting three logGammas.
    return -x * F.ln1p(y / x) - y * F.ln1p(x / y) + 0.5 * (F.ln(total) - F.ln(x) - F.ln(y)) + 0.9189385332046727 + SpecialKernel.correction(x) + SpecialKernel.correction(y) - SpecialKernel.correction(total) + shift
  }

  @class
  count(_ n: Int) {
    if n < 0 { ArgumentError.new("log combinatorics requires non-negative n").raise() }
    if n > 9007199254740991 { NumericalRangeError.new("log combinatorics count exceeds exact binary64 count range").raise() }
  }

  @class
  logFactorial(_ n: Int) -> Float {
    SpecialKernel.count(n)
    if n < 2 { return 0.0 }
    if n < 32 {
      let mut result = 0.0
      let mut i = 2
      while i <= n {
        result = result + F.ln(Float.new(i))
        i = i + 1
      }
      return result
    }
    let x = Float.new(n)
    return x * (F.ln(x) - 1.0) + 0.5 * F.ln(x) + 0.9189385332046727 + SpecialKernel.correction(x)
  }

  @class
  logChoose(_ n: Int, _ k: Int) -> Float {
    SpecialKernel.count(n)
    if k < 0 or k > n { return -1.0 / 0.0 }
    let count = if k < n - k { k } else { n - k }
    if count == 0 { return 0.0 }
    if count < 32 {
      let mut sum = 0.0
      let mut correction = 0.0
      let mut i = 1
      while i <= count {
        let term = F.ln(Float.new(n - count + i)) - F.ln(Float.new(i))
        let next = sum + term
        if sum >= term {
          correction = correction + ((sum - next) + term)
        } else {
          correction = correction + ((term - next) + sum)
        }
        sum = next
        i = i + 1
      }
      return sum + correction
    }
    let x = Float.new(count)
    let y = Float.new(n - count)
    let total = Float.new(n)
    // Entropy form with Stirling remainders: no subtraction of huge lgammas.
    return x * (F.ln(total) - F.ln(x)) + y * F.ln1p(x / y) + 0.5 * (F.ln(total) - F.ln(x) - F.ln(y)) - 0.9189385332046727 + SpecialKernel.correction(total) - SpecialKernel.correction(x) - SpecialKernel.correction(y)
  }

  // log(1-exp(logValue)), stable on both sides of ln(1/2).
  @class
  logComplement(_ logValue: Float) -> Float {
    if logValue < -0.6931471805599453 { return F.ln1p(-F.exp(logValue)) }
    return F.ln(-F.expm1(logValue))
  }

  @class
  boundedLog(_ value: Float) -> Float {
    if value.isNaN or value > 0.00000001 {
      NumericalEvaluationError.new("special-function tail evaluation lost its probability range").raise()
    }
    if value > 0.0 { return 0.0 }
    return value
  }

  // Stable a*ln(x)-x-lnGamma(a), avoiding cancellation near x=a.
  @class
  gammaLogScale(_ a: Float, _ x: Float) -> Float {
    if a < 16.0 { return a * F.ln(x) - x - SpecialKernel.logGamma(a) }
    let t = (x - a) / a
    let mut deviance = 0.0
    if t.abs < 0.125 {
      let mut power = t * t
      let mut k = 2
      let mut sum = 0.0
      while k < 100 {
        let term = power / Float.new(k)
        sum = sum + term
        if term.abs <= sum.abs * 0.0000000000000002 { break }
        power = power * -t
        k = k + 1
      }
      deviance = a * sum
    } else {
      deviance = (x - a) - a * (F.ln(x) - F.ln(a))
    }
    return -deviance + 0.5 * F.ln(a) - 0.9189385332046727 - SpecialKernel.correction(a)
  }

  // Positive power series for P(a,x), with its leading term factored in logs.
  @class
  gammaSeries(_ a: Float, _ x: Float) -> Float {
    let mut term = 1.0
    let mut total = 1.0
    let mut denominator = a
    let mut i = 0
    while i < 100000 {
      denominator = denominator + 1.0
      term = term * (x / denominator)
      total = total + term
      if term <= total * 0.0000000000000004 {
        return SpecialKernel.boundedLog(SpecialKernel.gammaLogScale(a, x) - F.ln(a) + F.ln(total))
      }
      i = i + 1
    }
    NumericalEvaluationError.new("incomplete gamma series did not converge").raise()
    return 0.0
  }

  @class
  nonzero(_ x: Float) -> Float {
    if x.abs >= 1.0e-300 { return x }
    if x < 0.0 { return -1.0e-300 }
    return 1.0e-300
  }

  // DLMF 8.9.2, modified Lentz evaluation of the upper-tail fraction.
  @class
  gammaFraction(_ a: Float, _ x: Float) -> Float {
    let mut b = x + 1.0 - a
    let mut c = 1.0e300
    let mut d = 1.0 / SpecialKernel.nonzero(b)
    let mut h = d
    let mut i = 1
    while i <= 100000 {
      let fi = Float.new(i)
      let coefficient = fi * (a - fi)
      b = b + 2.0
      d = 1.0 / SpecialKernel.nonzero(coefficient * d + b)
      c = SpecialKernel.nonzero(b + coefficient / c)
      let delta = c * d
      h = h * delta
      if (delta - 1.0).abs <= 0.0000000000000004 {
        return SpecialKernel.boundedLog(SpecialKernel.gammaLogScale(a, x) + F.ln(h))
      }
      i = i + 1
    }
    NumericalEvaluationError.new("incomplete gamma fraction did not converge").raise()
    return 0.0
  }

  // Direct upper tail for small shape and x <= 1. Factoring out a
  // retains Q when P rounds to one, including subnormal shape parameters.
  @class
  gammaSmallUpperLog(_ a: Float, _ x: Float) -> Float {
    let ratio = if a <= 0.125 { SpecialKernel.logGammaOnePlusRatio(a) } else { SpecialKernel.logGammaOnePlus(a) / a }
    let lRatio = F.ln(x) - ratio
    let logLeading = a * lRatio
    let first = if logLeading.abs < 0.00000001 {
      -lRatio * (1.0 + logLeading * (0.5 + logLeading / 6.0))
    } else {
      -F.expm1(logLeading) / a
    }
    let mut term = -x
    let mut sum = term / (a + 1.0)
    let mut n = 2
    while n < 1000 {
      let floatN = Float.new(n)
      term = term * (-x / floatN)
      let next = term / (a + floatN)
      sum = sum + next
      if next.abs <= sum.abs * 0.0000000000000002 {
        return SpecialKernel.boundedLog(F.ln(a) + F.ln(first - F.exp(logLeading) * sum))
      }
      n = n + 1
    }
    NumericalEvaluationError.new("small-shape gamma upper series did not converge").raise()
    return 0.0
  }

  @class
  logRegularizedGammaP(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.shape(a)
    if x.isNaN or x < 0.0 { ArgumentError.new("incomplete gamma requires x >= 0").raise() }
    if x == 0.0 { return -1.0 / 0.0 }
    if x.isInfinite { return 0.0 }
    if a <= 1.0 and x <= 1.0 {
      let upper = SpecialKernel.gammaSmallUpperLog(a, x)
      if upper < -0.6931471805599453 { return SpecialKernel.logComplement(upper) }
      return SpecialKernel.gammaSeries(a, x)
    }
    if a < 1.0 { return SpecialKernel.logComplement(SpecialKernel.gammaFraction(a, x)) }
    if x < a + 1.0 { return SpecialKernel.gammaSeries(a, x) }
    return SpecialKernel.logComplement(SpecialKernel.gammaFraction(a, x))
  }

  @class
  logRegularizedGammaQ(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.shape(a)
    if x.isNaN or x < 0.0 { ArgumentError.new("incomplete gamma requires x >= 0").raise() }
    if x == 0.0 { return 0.0 }
    if x.isInfinite { return -1.0 / 0.0 }
    if a <= 1.0 and x <= 1.0 {
      let upper = SpecialKernel.gammaSmallUpperLog(a, x)
      if upper < -0.6931471805599453 { return upper }
      return SpecialKernel.logComplement(SpecialKernel.gammaSeries(a, x))
    }
    if a < 1.0 { return SpecialKernel.gammaFraction(a, x) }
    if x < a + 1.0 { return SpecialKernel.logComplement(SpecialKernel.gammaSeries(a, x)) }
    return SpecialKernel.gammaFraction(a, x)
  }

  @class
  regularizedGammaP(_ a: Float, _ x: Float) -> Float { F.exp(SpecialKernel.logRegularizedGammaP(a, x)) }
  @class
  regularizedGammaQ(_ a: Float, _ x: Float) -> Float { F.exp(SpecialKernel.logRegularizedGammaQ(a, x)) }

  // DLMF 8.17.22, alternating even/odd coefficients, modified Lentz.
  @class
  betaFraction(_ a: Float, _ b: Float, _ x: Float) -> Float {
    let ab = a + b
    let ap = a + 1.0
    let am = a - 1.0
    let mut c = 1.0
    let mut d = 1.0 / SpecialKernel.nonzero(1.0 - ab * x / ap)
    let mut h = d
    let mut m = 1
    while m <= 100000 {
      let fm = Float.new(m)
      let twice = 2.0 * fm
      let first = fm * (b - fm) * x / ((am + twice) * (a + twice))
      d = 1.0 / SpecialKernel.nonzero(1.0 + first * d)
      c = SpecialKernel.nonzero(1.0 + first / c)
      h = h * d * c
      let second = -(a + fm) * (ab + fm) * x / ((a + twice) * (ap + twice))
      d = 1.0 / SpecialKernel.nonzero(1.0 + second * d)
      c = SpecialKernel.nonzero(1.0 + second / c)
      let delta = d * c
      h = h * delta
      if (delta - 1.0).abs <= 0.0000000000000004 { return h }
      m = m + 1
    }
    NumericalEvaluationError.new("incomplete beta fraction did not converge").raise()
    return 0.0
  }

  @class
  betaLowerLog(_ a: Float, _ b: Float, _ x: Float) -> Float {
    return SpecialKernel.boundedLog(a * F.ln(x) + b * F.ln1p(-x) - SpecialKernel.logBeta(a, b) + F.ln(SpecialKernel.betaFraction(a, b, x)) - F.ln(a))
  }

  @class
  logRegularizedBeta(_ a: Float, _ b: Float, _ x: Float) -> Float {
    SpecialKernel.shape(a)
    SpecialKernel.shape(b)
    SpecialKernel.unit(x)
    if x == 0.0 { return -1.0 / 0.0 }
    if x == 1.0 { return 0.0 }
    if x < (a + 1.0) / (a + b + 2.0) { return SpecialKernel.betaLowerLog(a, b, x) }
    return SpecialKernel.logComplement(SpecialKernel.betaLowerLog(b, a, 1.0 - x))
  }

  @class
  logRegularizedBetaComplement(_ a: Float, _ b: Float, _ x: Float) -> Float {
    SpecialKernel.shape(a)
    SpecialKernel.shape(b)
    SpecialKernel.unit(x)
    if x == 0.0 { return 0.0 }
    if x == 1.0 { return -1.0 / 0.0 }
    if x < (a + 1.0) / (a + b + 2.0) {
      return SpecialKernel.logComplement(SpecialKernel.betaLowerLog(a, b, x))
    }
    return SpecialKernel.betaLowerLog(b, a, 1.0 - x)
  }

  @class
  regularizedBeta(_ a: Float, _ b: Float, _ x: Float) -> Float { F.exp(SpecialKernel.logRegularizedBeta(a, b, x)) }
  @class
  regularizedBetaComplement(_ a: Float, _ b: Float, _ x: Float) -> Float { F.exp(SpecialKernel.logRegularizedBetaComplement(a, b, x)) }

  @class
  logErfc(_ x: Float) -> Float {
    if x.isNaN { ArgumentError.new("logErfc requires a non-NaN value").raise() }
    if x == 0.0 { return 0.0 }
    if x == 1.0 / 0.0 { return -1.0 / 0.0 }
    if x == -1.0 / 0.0 { return 0.6931471805599453 }
    if x.abs <= 0.5 {
      let square = x * x
      let mut term = 1.0
      let mut sum = 1.0
      let mut n = 1
      while n < 100 {
        let floatN = Float.new(n)
        term = term * (-square / floatN)
        let next = term / (2.0 * floatN + 1.0)
        sum = sum + next
        if next.abs <= sum.abs * 0.0000000000000002 { break }
        n = n + 1
      }
      return F.ln1p(-1.1283791670955126 * x * sum)
    }
    let square = x * x
    if square.isInfinite {
      if x < 0.0 { return 0.6931471805599453 }
      NumericalRangeError.new("logErfc result is outside Float range").raise()
    }
    let tail = SpecialKernel.logRegularizedGammaQ(0.5, square)
    if x > 0.0 { return tail }
    return 0.6931471805599453 + F.ln1p(-0.5 * F.exp(tail))
  }

  @class
  erfc(_ x: Float) -> Float {
    if x > 28.0 { return 0.0 }
    return F.exp(SpecialKernel.logErfc(x))
  }

  @class
  inverseRegularizedBeta(_ a: Float, _ b: Float, _ p: Float) -> Float {
    SpecialKernel.shape(a)
    SpecialKernel.shape(b)
    SpecialKernel.unit(p)
    if p == 0.0 or p == 1.0 { return p }
    let upper = p > 0.5
    let target = if upper { F.ln1p(-p) } else { F.ln(p) }
    let mut low = 0.0
    let mut high = 1.0
    if not upper {
      let estimate = F.exp((target + F.ln(a) + SpecialKernel.logBeta(a, b)) / a)
      if estimate > 0.0 and estimate < 0.5 {
        high = estimate * 2.0
        while high < 1.0 and SpecialKernel.logRegularizedBeta(a, b, high) < target {
          high = high * 2.0
          if high > 1.0 { high = 1.0 }
        }
      }
    }
    let mut i = 0
    while i < 1100 {
      let middle = low + (high - low) * 0.5
      if middle == low or middle == high { return high }
      let logTail = if upper { SpecialKernel.logRegularizedBetaComplement(a, b, middle) } else { SpecialKernel.logRegularizedBeta(a, b, middle) }
      if (upper and logTail > target) or (not upper and logTail < target) {
        low = middle
      } else {
        high = middle
      }
      i = i + 1
    }
    NumericalEvaluationError.new("incomplete beta inverse did not converge").raise()
    return 0.0
  }

  @class
  inverseRegularizedGammaP(_ a: Float, _ p: Float) -> Float {
    SpecialKernel.shape(a)
    SpecialKernel.unit(p)
    if p == 0.0 { return 0.0 }
    if p == 1.0 { return 1.0 / 0.0 }
    let upper = p > 0.5
    let target = if upper { F.ln1p(-p) } else { F.ln(p) }
    let mut low = 0.0
    let mut high = if a > 1.0 { a } else { 1.0 }
    if not upper {
      let estimate = F.exp((target + SpecialKernel.logGammaOnePlus(a)) / a)
      if estimate > 0.0 and estimate < high { high = estimate * 2.0 }
    }
    let mut i = 0
    while SpecialKernel.logRegularizedGammaP(a, high) < F.ln(p) {
      high = high * 2.0
      i = i + 1
      if i > 1024 or not high.isFinite { NumericalRangeError.new("gamma inverse could not bracket a finite result").raise() }
    }
    i = 0
    while i < 1100 {
      let middle = low + (high - low) * 0.5
      if middle == low or middle == high { return high }
      let logTail = if upper { SpecialKernel.logRegularizedGammaQ(a, middle) } else { SpecialKernel.logRegularizedGammaP(a, middle) }
      if (upper and logTail > target) or (not upper and logTail < target) {
        low = middle
      } else {
        high = middle
      }
      i = i + 1
    }
    NumericalEvaluationError.new("incomplete gamma inverse did not converge").raise()
    return 0.0
  }

  @class
  inverseNormal(_ p: Float) -> Float {
    SpecialKernel.unit(p)
    if p == 0.0 { return -1.0 / 0.0 }
    if p == 1.0 { return 1.0 / 0.0 }
    if p == 0.5 { return 0.0 }
    let target = if p < 0.5 { F.ln(p) } else { F.ln1p(-p) }
    let mut low = 0.0
    let mut high = 40.0
    let mut i = 0
    while i < 100 {
      let middle = (low + high) * 0.5
      if middle == low or middle == high {
        if p < 0.5 { return -middle }
        return middle
      }
      let logTail = SpecialKernel.logErfc(middle / 1.4142135623730951) - 0.6931471805599453
      if logTail > target { low = middle } else { high = middle }
      i = i + 1
    }
    NumericalEvaluationError.new("normal inverse did not converge").raise()
    return 0.0
  }
}

export SpecialKernel, NumericalRangeError, NumericalEvaluationError
