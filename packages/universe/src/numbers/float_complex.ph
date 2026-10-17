from ..math.float_kernel import FloatMathKernel as F

/// Scalar-pair helpers; independent of Complex to keep the package graph acyclic.
class FloatComplex {
  @class
  negative(_ x: Float) -> Bool {
    if x < 0.0 { return true }
    if x == 0.0 { return 1.0 / x < 0.0 }
    return false
  }

  @class
  normalize(_ value: Float) -> (Float, Int) {
    if value == 0.0 { return (value, 0) }
    let mut x = value
    let mut n = 0
    while x.abs >= 1.3407807929942597e154 { x = x * 7.458340731200207e-155; n = n + 512 }
    while x.abs < 7.458340731200207e-155 { x = x * 1.3407807929942597e154; n = n - 512 }
    while x.abs >= 4294967296.0 { x = x / 4294967296.0; n = n + 32 }
    while x.abs < 2.3283064365386963e-10 { x = x * 4294967296.0; n = n - 32 }
    while x.abs >= 1.0 { x = x * 0.5; n = n + 1 }
    while x.abs < 0.5 { x = x * 2.0; n = n - 1 }
    return (x, n)
  }

  // Separate product exponents retain small components beside huge partners.
  @class
  dot(_ a: Float, _ c: Float, _ b: Float, _ d: Float) -> (Float, Int) {
    let aa = FloatComplex.normalize(a)
    let cc = FloatComplex.normalize(c)
    let bb = FloatComplex.normalize(b)
    let dd = FloatComplex.normalize(d)
    let x = aa[0] * cc[0]
    let y = bb[0] * dd[0]
    let xe = aa[1] + cc[1]
    let ye = bb[1] + dd[1]
    if x == 0.0 { return (x + y, ye) }
    if y == 0.0 { return (x + y, xe) }
    if xe >= ye { return (x + y.scalePow2(ye - xe), xe) }
    return (x.scalePow2(xe - ye) + y, ye)
  }

  @class
  divide(_ a: Float, _ b: Float, _ c: Float, _ d: Float) -> (Float, Float) {
    let infinity = 1.0 / 0.0
    let nan = 0.0 / 0.0
    if c == 0.0 and d == 0.0 {
      let pole = infinity.copySign(c)
      return (pole * a, pole * b)
    }
    if a.isFinite and b.isFinite and c.isFinite and d.isFinite {
      let denominator = FloatComplex.dot(c, c, d, d)
      let real = FloatComplex.dot(a, c, b, d)
      let imag = FloatComplex.dot(b, c, -a, d)
      return ((real[0] / denominator[0]).scalePow2(real[1] - denominator[1]),
              (imag[0] / denominator[0]).scalePow2(imag[1] - denominator[1]))
    }
    if (c.isInfinite or d.isInfinite) and a.isFinite and b.isFinite {
      let crMagnitude = if c.isInfinite { 1.0 } else { 0.0 }
      let diMagnitude = if d.isInfinite { 1.0 } else { 0.0 }
      let cr = crMagnitude.copySign(c)
      let di = diMagnitude.copySign(d)
      return (0.0 * (a * cr + b * di), 0.0 * (b * cr - a * di))
    }
    if (a.isInfinite or b.isInfinite) and c.isFinite and d.isFinite {
      let arMagnitude = if a.isInfinite { 1.0 } else { 0.0 }
      let biMagnitude = if b.isInfinite { 1.0 } else { 0.0 }
      let ar = arMagnitude.copySign(a)
      let bi = biMagnitude.copySign(b)
      return (infinity * (ar * c + bi * d), infinity * (bi * c - ar * d))
    }
    return (nan, nan)
  }

  @class
  polar(_ radius: Float, _ angle: Float) -> (Float, Float) {
    let nan = 0.0 / 0.0
    if angle == 0.0 {
      if radius.isInfinite {
        let zeroSign = if FloatComplex.negative(radius) { -angle } else { angle }
        return (radius, 0.0.copySign(zeroSign))
      }
      if radius.isNaN { return (radius, angle) }
      return (radius, radius * angle)
    }
    if radius == 0.0 and not angle.isFinite { return (0.0, 0.0) }
    if not angle.isFinite { return (nan, nan) }
    if radius.isInfinite {
      return (radius.copySign(F.cos(angle)) * (if radius < 0.0 { -1.0 } else { 1.0 }),
              radius.copySign(F.sin(angle)) * (if radius < 0.0 { -1.0 } else { 1.0 }))
    }
    return (radius * F.cos(angle), radius * F.sin(angle))
  }
}

export FloatComplex
