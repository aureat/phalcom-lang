from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One
from .algebra import Basis, I, BasisRealizable, RealEmbeddable
from .rational import Rational
from .float_complex import FloatComplex as FC
from ..math.float_kernel import FloatMathKernel as F

/// Immutable rectangular components. Arithmetic requires only its own capabilities.
data Complex<T>(_ real: T, _ imag: T)

/// Supplies a component-aware complex quotient algorithm.
///
/// This class-side operation lets scalar types preserve their own numerical
/// guarantees: Float uses scaled IEEE-aware arithmetic, while Rational stays
/// exact. Generic `Complex<T>` division dispatches through the active type
/// environment.
trait ComplexDivision {
  divideComplexComponents(
    _ imag: Self, _ otherReal: Self, _ otherImag: Self
  ) -> Tuple<***(Self, Self)>
}

impl ComplexDivision for Float {
  divideComplexComponents(
    _ imag: Float, _ otherReal: Float, _ otherImag: Float
  ) -> Tuple<***(Float, Float)> {
    FC.divide(self, imag, otherReal, otherImag)
  }
}

impl ComplexDivision for Rational {
  divideComplexComponents(
    _ imag: Rational,
    _ otherReal: Rational, _ otherImag: Rational
  ) -> Tuple<***(Rational, Rational)> {
    let denominator = otherReal * otherReal + otherImag * otherImag
    let quotientReal = (self * otherReal + imag * otherImag) / denominator
    let quotientImag = (imag * otherReal - self * otherImag) / denominator
    return (quotientReal, quotientImag)
  }
}

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

impl<T> Zero for Complex<T> where T |= Zero {
  @class zero -> Complex<T> { Complex<T>(T.zero, T.zero) }
}

impl<T> One for Complex<T> where T |= Zero, T |= One {
  @class one -> Complex<T> { Complex<T>(T.one, T.zero) }
}

impl<T> RealEmbeddable for Complex<T> where T |= Zero {
  type Scalar = T

  @class fromReal(_ value: T) -> Complex<T> { Complex<T>(value, T.zero) }
}

impl<T> BasisRealizable<Basis<I>> for Complex<T> where T |= Zero, T |= One {
  type Scalar = T

  @class along(_ coefficient: T, _ basis: Basis<I>) -> Complex<T> {
    Complex<T>(T.zero, coefficient)
  }
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

impl<T> Complex<T> where T |= ComplexDivision {
  /(_ other: Complex<T>) -> Complex<T> {
    let parts = self.real.divideComplexComponents(
      self.imag, other.real, other.imag
    )
    return Complex<T>(parts[0], parts[1])
  }
}

impl<T> Complex<T> where T == Int {
  /// Explicit component conversion; no implicit promotion is performed.
  toFloat -> Complex<Float> { Complex<Float>(Float.new(self.real), Float.new(self.imag)) }
}

impl Complex<Float> {
  abs -> Float { F.hypot(self.real, self.imag) }
  arg -> Float { F.atan2(self.imag, self.real) }
  isFinite -> Bool { self.real.isFinite and self.imag.isFinite }
  isNaN -> Bool { self.real.isNaN or self.imag.isNaN }
  isInfinite -> Bool { self.real.isInfinite or self.imag.isInfinite }
}

impl IntegerPower for Complex<Rational> {
  **(_ exponent: Int) -> Complex<Rational> {
    let one = Rational.fromInt(1)
    let zero = Rational.fromInt(0)
    let mut n = if exponent < 0 { -exponent } else { exponent }
    let mut factor = self
    if exponent < 0 {
      let norm = self.real * self.real + self.imag * self.imag
      factor = Complex<Rational>(self.real / norm, -self.imag / norm)
    }
    let mut result = Complex<Rational>(one, zero)
    while n > 0 {
      if n % 2 == 1 { result = result * factor }
      n = n >> 1
      if n > 0 { factor = factor * factor }
    }
    return result
  }
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

export Complex, ComplexDivision
