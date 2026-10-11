from .capabilities import Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One
from ..math.integer import IntegerMath

/// Canonical exact ratio of two arbitrary-precision integers.
///
/// The private data constructor ensures every public value is normalized:
/// the denominator is positive, the components are coprime, and zero is 0/1.
data Rational private(_ numerator: Int, _ denominator: Int)

impl Rational {
  @class
  private failDivisionByZero(_ denominator: Int) -> Int { 1 ~/ denominator }

  @class
  new(_ numerator: Int, _ denominator: Int) -> Rational {
    if denominator == 0 { Rational.failDivisionByZero(denominator) }
    if numerator == 0 { return Rational(0, 1) }

    let sign = if denominator < 0 { -1 } else { 1 }
    let signedNumerator = numerator * sign
    let positiveDenominator = denominator * sign
    let absoluteNumerator = if signedNumerator < 0 { -signedNumerator } else { signedNumerator }
    let divisor = IntegerMath.gcd(absoluteNumerator, positiveDenominator)
    Rational(signedNumerator ~/ divisor, positiveDenominator ~/ divisor)
  }

  @class
  fromInt(_ value: Int) -> Rational { Rational(value, 1) }

  /// Converts an integer exactly, rejecting a fractional value.
  toIntExact -> Int {
    if self.numerator % self.denominator != 0 {
      ArgumentError.new("Rational.toIntExact: value is not an integer").raise()
    }
    self.numerator ~/ self.denominator
  }

  /// Explicitly approximates the exact ratio as binary64. Scaling the integer
  /// operands before conversion avoids `infinity / infinity` for large ratios
  /// whose quotient is still finite.
  toFloat -> Float {
    if self.numerator == 0 { return 0.0 }
    let absoluteNumerator = if self.numerator < 0 { -self.numerator } else { self.numerator }
    let numeratorShift = if absoluteNumerator.bitLength > 54 { absoluteNumerator.bitLength - 54 } else { 0 }
    let denominatorShift = if self.denominator.bitLength > 54 { self.denominator.bitLength - 54 } else { 0 }
    let numeratorHead = absoluteNumerator >> numeratorShift
    let denominatorHead = self.denominator >> denominatorShift
    let exponent = numeratorShift - denominatorShift
    let magnitude = (Float.new(numeratorHead) / Float.new(denominatorHead)) * (2.0 ** exponent)
    if self.numerator < 0 { -magnitude } else { magnitude }
  }

  /// Rounds toward negative infinity.
  floor -> Int { self.numerator ~/ self.denominator }

  /// Rounds toward zero, matching the explicit narrowing operation.
  truncated -> Int {
    let floorQuotient = self.numerator ~/ self.denominator
    if self.numerator < 0 and self.numerator % self.denominator != 0 { floorQuotient + 1 } else { floorQuotient }
  }

  ceil -> Int {
    let quotient = self.numerator ~/ self.denominator
    if self.numerator % self.denominator != 0 { quotient + 1 } else { quotient }
  }

  abs -> Rational {
    if self.numerator < 0 { Rational(-self.numerator, self.denominator) } else { self }
  }

  sign -> Int {
    if self.numerator < 0 { -1 } else if self.numerator > 0 { 1 } else { 0 }
  }

  inverse -> Rational {
    if self.numerator == 0 { Rational.failDivisionByZero(self.numerator) }
    Rational.new(self.denominator, self.numerator)
  }

  +(_ other: Rational) -> Rational {
    let common = IntegerMath.gcd(self.denominator, other.denominator)
    let leftScale = other.denominator ~/ common
    let rightScale = self.denominator ~/ common
    Rational.new(
      (self.numerator * leftScale) + (other.numerator * rightScale),
      self.denominator * leftScale)
  }

  -(_ other: Rational) -> Rational {
    let common = IntegerMath.gcd(self.denominator, other.denominator)
    let leftScale = other.denominator ~/ common
    let rightScale = self.denominator ~/ common
    Rational.new(
      (self.numerator * leftScale) - (other.numerator * rightScale),
      self.denominator * leftScale)
  }

  *(_ other: Rational) -> Rational {
    let absoluteLeft = if self.numerator < 0 { -self.numerator } else { self.numerator }
    let absoluteRight = if other.numerator < 0 { -other.numerator } else { other.numerator }
    let leftCancel = IntegerMath.gcd(absoluteLeft, other.denominator)
    let rightCancel = IntegerMath.gcd(absoluteRight, self.denominator)
    Rational.new(
      (self.numerator ~/ leftCancel) * (other.numerator ~/ rightCancel),
      (self.denominator ~/ rightCancel) * (other.denominator ~/ leftCancel))
  }

  /(_ other: Rational) -> Rational {
    if other.numerator == 0 { Rational.failDivisionByZero(other.numerator) }
    Rational.new(self.numerator * other.denominator, self.denominator * other.numerator)
  }

  - -> Rational { Rational(-self.numerator, self.denominator) }

  **(_ exponent: Int) -> Rational {
    if exponent == 0 { return Rational(1, 1) }
    if exponent < 0 {
      if self.numerator == 0 { ArgumentError.new("Rational power: zero cannot have a negative exponent").raise() }
      return Rational.new(self.denominator ** (-exponent), self.numerator ** (-exponent))
    }
    Rational.new(self.numerator ** exponent, self.denominator ** exponent)
  }

  compare(_ other: Rational) -> Ordering {
    (self.numerator * other.denominator).compare(other.numerator * self.denominator)
  }

  <(_ other: Rational) -> Bool { self.compare(other) === Ordering::Less }
  <=(_ other: Rational) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Less { return true }
    return order === Ordering::Equal
  }
  >(_ other: Rational) -> Bool { self.compare(other) === Ordering::Greater }
  >=(_ other: Rational) -> Bool {
    let order = self.compare(other)
    if order === Ordering::Greater { return true }
    return order === Ordering::Equal
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Rational) { return false }
    if self.numerator == other.numerator {
      return self.denominator == other.denominator
    }
    return false
  }

  hash -> Int { (self.numerator.hash * 31) + self.denominator.hash }

  toString -> String {
    if self.denominator == 1 { "\(self.numerator)" } else { "\(self.numerator)/\(self.denominator)" }
  }
}

impl Addable for Rational {}
impl Subtractable for Rational {}
impl Multipliable for Rational {}
impl Negatable for Rational {}
impl IntegerPower for Rational {}

impl Zero for Rational {
  @class zero -> Rational { Rational.new(0, 1) }
}

impl One for Rational {
  @class one -> Rational { Rational.new(1, 1) }
}

export Rational
