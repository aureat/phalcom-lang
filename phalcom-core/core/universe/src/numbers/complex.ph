from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower
from .float_complex import FloatComplex as FC
from ..math.float_kernel import FloatMathKernel as F

/// Immutable rectangular components. Arithmetic requires only its own capabilities.
data Complex<T>(_ real: T, _ imag: T)

impl<T> Complex<T> {
  @class
  new<C>(_ real: C, _ imag: C) -> Complex<C> { Complex<C>(real, imag) }

  @class
  fromPolar(radius: Float, angle: Float) -> Complex<Float> {
    let parts = FC.polar(radius, angle)
    return Complex<Float>(parts[0], parts[1])
  }

  /// Both components remain visible, including zero signs.
  toString -> String { "\(real) + \(imag)i" }
}

impl<T> Complex<T> where T |= Addable {
  +(_ other: Complex<T>) -> Complex<T> {
    Complex<T>(self.real + other.real, self.imag + other.imag)
  }
}

impl<T> Complex<T> where T |= Subtractable {
  -(_ other: Complex<T>) -> Complex<T> {
    Complex<T>(self.real - other.real, self.imag - other.imag)
  }
}

impl<T> Complex<T> where T |= Negatable {
  - -> Complex<T> { Complex<T>(-self.real, -self.imag) }
  conjugate -> Complex<T> { Complex<T>(self.real, -self.imag) }
}

impl<T> Complex<T> where T |= Addable, T |= Multipliable {
  /// Algebraic squared norm; Float component products can overflow.
  normSquared -> T { self.real * self.real + self.imag * self.imag }
}

impl<T> Complex<T> where T |= Addable, T |= Subtractable, T |= Multipliable {
  *(_ other: Complex<T>) -> Complex<T> {
    Complex<T>(self.real * other.real - self.imag * other.imag,
               self.real * other.imag + self.imag * other.real)
  }
}

impl<T> Complex<T> where T == Int {
  /// Explicit component conversion; no implicit promotion is performed.
  toFloat -> Complex<Float> { Complex<Float>(Float.new(self.real), Float.new(self.imag)) }
}

impl<T> Complex<T> where T == Float {
  /(_ other: Complex<Float>) -> Complex<Float> {
    let parts = FC.divide(self.real, self.imag, other.real, other.imag)
    return Complex<Float>(parts[0], parts[1])
  }
  abs -> Float { F.hypot(self.real, self.imag) }
  arg -> Float { F.atan2(self.imag, self.real) }
  isFinite -> Bool { self.real.isFinite and self.imag.isFinite }
  isNaN -> Bool { self.real.isNaN or self.imag.isNaN }
  isInfinite -> Bool { self.real.isInfinite or self.imag.isInfinite }
}

impl IntegerPower for Complex<Int> {
  **(_ exponent: Int) -> Complex<Int> {
    if exponent < 0 { ArgumentError.new("Complex<Int> power requires a non-negative exponent").raise() }
    let mut n = exponent
    let mut result = Complex<Int>(1, 0)
    let mut factor = self
    while n > 0 {
      if n % 2 == 1 {
        result = Complex<Int>(result.real * factor.real - result.imag * factor.imag,
                              result.real * factor.imag + result.imag * factor.real)
      }
      n = n >> 1
      if n > 0 {
        factor = Complex<Int>(factor.real * factor.real - factor.imag * factor.imag,
                              factor.real * factor.imag + factor.imag * factor.real)
      }
    }
    return result
  }
}

impl IntegerPower for Complex<Float> {
  **(_ exponent: Int) -> Complex<Float> {
    let mut n = if exponent < 0 { -exponent } else { exponent }
    let mut result = Complex<Float>(1.0, 0.0)
    let mut factor = if exponent < 0 { Complex<Float>(1.0, 0.0) / self } else { self }
    while n > 0 {
      if n % 2 == 1 {
        result = Complex<Float>(result.real * factor.real - result.imag * factor.imag,
                                result.real * factor.imag + result.imag * factor.real)
      }
      n = n >> 1
      if n > 0 {
        factor = Complex<Float>(factor.real * factor.real - factor.imag * factor.imag,
                                factor.real * factor.imag + factor.imag * factor.real)
      }
    }
    return result
  }
}

export Complex
