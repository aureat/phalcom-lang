// Numerical algorithms adapted from CPython v3.13.9, cmathmodule.c.
// Copyright (c) 2001-2024 Python Software Foundation; All Rights Reserved.
// See CPYTHON-LICENSE.txt and README.md for the adaptation and changes.
from ...numbers.complex import Complex
from ...numbers.float_complex import FloatComplex as H
from ..float_kernel import FloatMathKernel as F

/// Principal binary64 complex functions; nonfinite cases return IEEE components.
class ComplexMathKernel {
  @class powInt(_ z: Complex<Float>, _ exponent: Int) -> Complex<Float> { z ** exponent }

  @class
  pow(_ z: Complex<Float>, _ exponent: Complex<Float>) -> Complex<Float> {
    if exponent.real == 0.0 and exponent.imag == 0.0 { return Complex<Float>(1.0, 0.0) }
    if z.real == 0.0 and z.imag == 0.0 {
      if exponent.imag != 0.0 or exponent.real.isNaN { return Complex<Float>(0.0 / 0.0, 0.0 / 0.0) }
      if exponent.real > 0.0 { return Complex<Float>(0.0, 0.0) }
      return Complex<Float>(1.0 / 0.0, 0.0)
    }
    return ComplexMathKernel.exp(exponent * ComplexMathKernel.log(z))
  }

  @class
  sqrt(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if y.isInfinite { return Complex<Float>(inf, y) }
    if x.isInfinite {
      if x > 0.0 { return Complex<Float>(inf, if y.isNaN { nan } else { H.copySign(0.0, y) }) }
      return Complex<Float>(if y.isNaN { nan } else { 0.0 }, H.copySign(inf, y))
    }
    if x.isNaN or y.isNaN { return Complex<Float>(nan, nan) }
    if x == 0.0 and y == 0.0 { return Complex<Float>(0.0, y) }
    let ax = x.abs
    let ay = y.abs
    let s = if ax < 2.2250738585072014e-308 and ay < 2.2250738585072014e-308 {
      H.scale(F.sqrt(H.scale(ax, 53) + F.hypot(H.scale(ax, 53), H.scale(ay, 53))), -27)
    } else {
      2.0 * F.sqrt(ax / 8.0 + F.hypot(ax / 8.0, ay / 8.0))
    }
    let d = ay / (2.0 * s)
    if x >= 0.0 { return Complex<Float>(s, H.copySign(d, y)) }
    return Complex<Float>(d, H.copySign(s, y))
  }

  @class
  log(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite or y.isInfinite { return Complex<Float>(inf, F.atan2(y, x)) }
    if x.isNaN or y.isNaN { return Complex<Float>(nan, nan) }
    let ax = x.abs
    let ay = y.abs
    let real = if ax > 4.4942328371557893e307 or ay > 4.4942328371557893e307 {
      F.ln(F.hypot(ax / 2.0, ay / 2.0)) + 0.6931471805599453
    } else if ax < 2.2250738585072014e-308 and ay < 2.2250738585072014e-308 {
      if ax == 0.0 and ay == 0.0 { -inf } else {
        F.ln(F.hypot(H.scale(ax, 53), H.scale(ay, 53))) - 36.7368005696771
      }
    } else {
      let h = F.hypot(ax, ay)
      if h >= 0.71 and h <= 1.73 {
        let a = if ax >= ay { ax } else { ay }
        let b = if ax >= ay { ay } else { ax }
        F.ln1p((a - 1.0) * (a + 1.0) + b * b) / 2.0
      } else { F.ln(h) }
    }
    return Complex<Float>(real, F.atan2(y, x))
  }

  @class
  log1p(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    if x == 0.0 and y == 0.0 { return z }
    if x.abs <= 0.5 and y.abs <= 0.5 {
      return Complex<Float>(F.ln1p(x * (2.0 + x) + y * y) / 2.0, F.atan2(y, 1.0 + x))
    }
    return ComplexMathKernel.log(Complex<Float>(1.0 + x, y))
  }

  @class
  expTimes(_ x: Float, _ factor: Float) -> Float {
    if factor == 0.0 { return factor }
    if x <= 700.0 { return F.exp(x) * factor }
    if x < 1400.0 { return (F.exp(x - 700.0) * factor) * 1.0142320547350045e304 }
    return H.copySign(F.exp(x + F.ln(factor.abs)), factor)
  }

  @class
  exp(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x == -inf {
      if y.isFinite { return Complex<Float>(H.copySign(0.0, F.cos(y)), H.copySign(0.0, F.sin(y))) }
      return Complex<Float>(0.0, 0.0)
    }
    if x == inf {
      if y == 0.0 { return Complex<Float>(inf, y) }
      if y.isFinite { return Complex<Float>(H.copySign(inf, F.cos(y)), H.copySign(inf, F.sin(y))) }
      return Complex<Float>(inf, nan)
    }
    if x.isNaN and y == 0.0 { return Complex<Float>(nan, y) }
    if not x.isFinite or not y.isFinite { return Complex<Float>(nan, nan) }
    return Complex<Float>(ComplexMathKernel.expTimes(x, F.cos(y)), ComplexMathKernel.expTimes(x, F.sin(y)))
  }

  @class
  cosh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite {
      if y == 0.0 { return Complex<Float>(inf, H.copySign(0.0, if x < 0.0 { -y } else { y })) }
      if y.isFinite { return Complex<Float>(H.copySign(inf, F.cos(y)), H.copySign(inf, F.sin(y)) * H.copySign(1.0, x)) }
      return Complex<Float>(inf, nan)
    }
    if not x.isFinite or not y.isFinite {
      if x == 0.0 { return Complex<Float>(nan, 0.0) }
      if y == 0.0 { return Complex<Float>(nan, 0.0) }
      return Complex<Float>(nan, nan)
    }
    if x.abs > 700.0 {
      return Complex<Float>(ComplexMathKernel.expTimes(x.abs, 0.5 * F.cos(y)),
                            ComplexMathKernel.expTimes(x.abs, 0.5 * F.sin(y)) * H.copySign(1.0, x))
    }
    return Complex<Float>(F.cosh(x) * F.cos(y), F.sinh(x) * F.sin(y))
  }

  @class
  sinh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite {
      if y == 0.0 { return Complex<Float>(x, y) }
      if y.isFinite { return Complex<Float>(H.copySign(inf, F.cos(y)) * H.copySign(1.0, x), H.copySign(inf, F.sin(y))) }
      return Complex<Float>(inf, nan)
    }
    if not x.isFinite or not y.isFinite {
      if x == 0.0 { return Complex<Float>(0.0, nan) }
      if y == 0.0 { return Complex<Float>(nan, y) }
      return Complex<Float>(nan, nan)
    }
    if x.abs > 700.0 {
      return Complex<Float>(ComplexMathKernel.expTimes(x.abs, 0.5 * F.cos(y)) * H.copySign(1.0, x),
                            ComplexMathKernel.expTimes(x.abs, 0.5 * F.sin(y)))
    }
    return Complex<Float>(F.sinh(x) * F.cos(y), F.cosh(x) * F.sin(y))
  }

  @class
  tanh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let nan = 0.0 / 0.0
    if x.isInfinite {
      if y == 0.0 { return Complex<Float>(H.copySign(1.0, x), y) }
      if y.isFinite { return Complex<Float>(H.copySign(1.0, x), H.copySign(0.0, F.sin(y) * F.cos(y))) }
      return Complex<Float>(H.copySign(1.0, x), 0.0)
    }
    if not x.isFinite or not y.isFinite {
      if y == 0.0 { return Complex<Float>(nan, y) }
      if x == 0.0 and y.isNaN { return Complex<Float>(x, nan) }
      return Complex<Float>(nan, nan)
    }
    if x.abs > 350.0 { return Complex<Float>(H.copySign(1.0, x), 4.0 * F.sin(y) * F.cos(y) * F.exp(-2.0 * x.abs)) }
    let tx = F.tanh(x)
    let ty = F.tan(y)
    let cx = 1.0 / F.cosh(x)
    let product = tx * ty
    let denominator = 1.0 + product * product
    return Complex<Float>(tx * (1.0 + ty * ty) / denominator, ((ty / denominator) * cx) * cx)
  }

  @class cos(_ z: Complex<Float>) -> Complex<Float> { ComplexMathKernel.cosh(Complex<Float>(-z.imag, z.real)) }
  @class sin(_ z: Complex<Float>) -> Complex<Float> {
    let s = ComplexMathKernel.sinh(Complex<Float>(-z.imag, z.real))
    return Complex<Float>(s.imag, -s.real)
  }
  @class tan(_ z: Complex<Float>) -> Complex<Float> {
    let s = ComplexMathKernel.tanh(Complex<Float>(-z.imag, z.real))
    return Complex<Float>(s.imag, -s.real)
  }

  @class
  acos(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if y.isInfinite { return Complex<Float>(if x.isNaN { nan } else { F.atan2(y.abs, x) }, -H.copySign(inf, y)) }
    if x.isInfinite {
      if y.isNaN { return Complex<Float>(nan, inf) }
      return Complex<Float>(if x > 0.0 { 0.0 } else { 3.141592653589793 }, -H.copySign(inf, y))
    }
    if x.isNaN or y.isNaN { return Complex<Float>(if x == 0.0 { 1.5707963267948966 } else { nan }, nan) }
    if x.abs > 4.4942328371557893e307 or y.abs > 4.4942328371557893e307 {
      return Complex<Float>(F.atan2(y.abs, x), -H.copySign(F.ln(F.hypot(x / 2.0, y / 2.0)) + 1.3862943611198906, y))
    }
    let s1 = ComplexMathKernel.sqrt(Complex<Float>(1.0 - x, -y))
    let s2 = ComplexMathKernel.sqrt(Complex<Float>(1.0 + x, y))
    return Complex<Float>(2.0 * F.atan2(s1.real, s2.real), F.asinh(s2.real * s1.imag - s2.imag * s1.real))
  }

  @class
  acosh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite or y.isInfinite { return Complex<Float>(inf, F.atan2(y, x)) }
    if x == 0.0 and y.isNaN { return Complex<Float>(nan, 1.5707963267948966) }
    if x.isNaN or y.isNaN { return Complex<Float>(nan, nan) }
    if x.abs > 4.4942328371557893e307 or y.abs > 4.4942328371557893e307 {
      return Complex<Float>(F.ln(F.hypot(x / 2.0, y / 2.0)) + 1.3862943611198906, F.atan2(y, x))
    }
    let s1 = ComplexMathKernel.sqrt(Complex<Float>(x - 1.0, y))
    let s2 = ComplexMathKernel.sqrt(Complex<Float>(x + 1.0, y))
    return Complex<Float>(F.asinh(s1.real * s2.real + s1.imag * s2.imag), 2.0 * F.atan2(s1.imag, s2.real))
  }

  @class
  asinh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite { return Complex<Float>(x, F.atan2(y, x.abs)) }
    if y.isInfinite { return Complex<Float>(if x.isNaN { inf } else { H.copySign(inf, x) }, if x.isNaN { nan } else { H.copySign(1.5707963267948966, y) }) }
    if x.isNaN and y == 0.0 { return Complex<Float>(nan, y) }
    if x.isNaN or y.isNaN { return Complex<Float>(nan, nan) }
    if x.abs > 4.4942328371557893e307 or y.abs > 4.4942328371557893e307 {
      return Complex<Float>(H.copySign(F.ln(F.hypot(x / 2.0, y / 2.0)) + 1.3862943611198906, x), F.atan2(y, x.abs))
    }
    let s1 = ComplexMathKernel.sqrt(Complex<Float>(1.0 + y, -x))
    let s2 = ComplexMathKernel.sqrt(Complex<Float>(1.0 - y, x))
    return Complex<Float>(F.asinh(s1.real * s2.imag - s2.real * s1.imag), F.atan2(y, s1.real * s2.real - s1.imag * s2.imag))
  }

  @class
  atanh(_ z: Complex<Float>) -> Complex<Float> {
    let x = z.real
    let y = z.imag
    let inf = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if x.isInfinite { return Complex<Float>(H.copySign(0.0, x), if y.isNaN { nan } else { H.copySign(1.5707963267948966, y) }) }
    if y.isInfinite { return Complex<Float>(if x.isNaN { 0.0 } else { H.copySign(0.0, x) }, H.copySign(1.5707963267948966, y)) }
    if x == 0.0 and y.isNaN { return Complex<Float>(x, nan) }
    if x.isNaN or y.isNaN { return Complex<Float>(nan, nan) }
    if x < 0.0 {
      let s = ComplexMathKernel.atanh(Complex<Float>(-x, -y))
      return Complex<Float>(-s.real, -s.imag)
    }
    let ay = y.abs
    if x > 6.703903964971298e153 or ay > 6.703903964971298e153 {
      let h = F.hypot(x / 2.0, y / 2.0)
      return Complex<Float>(x / 4.0 / h / h, H.copySign(1.5707963267948966, y))
    }
    if x == 1.0 and ay < 1.4916681462400413e-154 {
      if ay == 0.0 { return Complex<Float>(inf, y) }
      return Complex<Float>(-F.ln(F.sqrt(ay) / F.sqrt(F.hypot(ay, 2.0))), H.copySign(F.atan2(2.0, -ay) / 2.0, y))
    }
    return Complex<Float>(F.ln1p(4.0 * x / ((1.0 - x) * (1.0 - x) + ay * ay)) / 4.0,
                          -F.atan2(-2.0 * y, (1.0 - x) * (1.0 + x) - ay * ay) / 2.0)
  }

  @class asin(_ z: Complex<Float>) -> Complex<Float> {
    let s = ComplexMathKernel.asinh(Complex<Float>(-z.imag, z.real))
    return Complex<Float>(s.imag, -s.real)
  }
  @class atan(_ z: Complex<Float>) -> Complex<Float> {
    let s = ComplexMathKernel.atanh(Complex<Float>(-z.imag, z.real))
    return Complex<Float>(s.imag, -s.real)
  }
}

export ComplexMathKernel
