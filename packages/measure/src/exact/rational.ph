import universe.math as math

/// Exact rational number backed by arbitrary-precision `Int`.
///
/// ## Invariants
///
/// - denominator is strictly positive;
/// - numerator and denominator are relatively prime;
/// - zero is represented uniquely as `0/1`.
///
/// Public arithmetic never routes through Float.
data Rational private(
  _ numerator: Int,
  _ denominator: Int
)

// Canonical static decimal powers. These are direct transparent-data
// constructions, so they do not require arbitrary method execution during
// static evaluation.
const rationalTenth = Rational(1, 10)
const rationalHundredth = Rational(1, 100)
const rationalThousandth = Rational(1, 1_000)
const rationalMillionth = Rational(1, 1_000_000)
const rationalBillionth = Rational(1, 1_000_000_000)
const rationalTrillionth = Rational(1, 1_000_000_000_000)
const rationalQuadrillionth = Rational(1, 1_000_000_000_000_000)
const rationalQuintillionth = Rational(1, 1_000_000_000_000_000_000)
const rationalSextillionth = Rational(1, 1_000_000_000_000_000_000_000)
const rationalSeptillionth = Rational(1, 1_000_000_000_000_000_000_000_000)
const rationalOctillionth = Rational(1, 1_000_000_000_000_000_000_000_000_000)
const rationalNonillionth = Rational(1, 1_000_000_000_000_000_000_000_000_000_000)

const rationalTen = Rational(10, 1)
const rationalHundred = Rational(100, 1)
const rationalThousand = Rational(1_000, 1)
const rationalMillion = Rational(1_000_000, 1)
const rationalBillion = Rational(1_000_000_000, 1)
const rationalTrillion = Rational(1_000_000_000_000, 1)
const rationalQuadrillion = Rational(1_000_000_000_000_000, 1)
const rationalQuintillion = Rational(1_000_000_000_000_000_000, 1)
const rationalSextillion = Rational(1_000_000_000_000_000_000_000, 1)
const rationalSeptillion = Rational(1_000_000_000_000_000_000_000_000, 1)
const rationalOctillion = Rational(1_000_000_000_000_000_000_000_000_000, 1)
const rationalNonillion = Rational(1_000_000_000_000_000_000_000_000_000_000, 1)

impl Rational {
  /// Constructs and canonicalizes `numerator / denominator`.
  @class
  of(_ numerator: Int, over denominator: Int) -> Rational {
    if denominator == 0 {
      ArgumentError.new("Rational denominator cannot be zero").raise()
    }
    if numerator == 0 { return Rational(0, 1) }

    let mut n = numerator
    let mut d = denominator
    if d < 0 {
      n = -n
      d = -d
    }

    let divisor = math.gcd(n, d)
    Rational(n ~/ divisor, d ~/ divisor)
  }

  /// Constructs an integral rational.
  @class whole(_ value: Int) -> Rational { Rational(value, 1) }

  /// Constructs an exact decimal `significand × 10^-places`.
  @class
  decimal(_ significand: Int, places: Int) -> Rational {
    if places < 0 {
      ArgumentError.new("Rational.decimal places must be non-negative").raise()
    }
    Rational.of(significand, over: Rational.powerOfTen(places))
  }

  /// Exact 10^exponent for non-negative integral exponent.
  @class
  powerOfTen(_ exponent: Int) -> Int {
    if exponent < 0 {
      ArgumentError.new("Rational.powerOfTen exponent must be non-negative").raise()
    }
    let mut result = 1
    let mut base = 10
    let mut power = exponent
    while power > 0 {
      if power % 2 == 1 { result = result * base }
      power = power ~/ 2
      if power > 0 { base = base * base }
    }
    result
  }

  @class zero -> Rational { Rational(0, 1) }
  @class one -> Rational { Rational(1, 1) }

  numerator -> Int { self.numerator }
  denominator -> Int { self.denominator }
  isZero -> Bool { self.numerator == 0 }
  isWhole -> Bool { self.denominator == 1 }

  sign -> Int {
    if self.numerator < 0 { return -1 }
    if self.numerator > 0 { return 1 }
    0
  }

  abs -> Rational {
    if self.numerator < 0 {
      Rational(-self.numerator, self.denominator)
    } else {
      self
    }
  }

  reciprocal -> Rational {
    if self.numerator == 0 {
      ArgumentError.new("zero has no reciprocal").raise()
    }
    Rational.of(self.denominator, over: self.numerator)
  }

  +(_ other: Rational) -> Rational {
    // Reduce the denominator cross-factor first to avoid unnecessarily large
    // arbitrary-precision intermediate values.
    let g = math.gcd(self.denominator, other.denominator)
    let leftFactor = other.denominator ~/ g
    let rightFactor = self.denominator ~/ g

    Rational.of(
      self.numerator * leftFactor + other.numerator * rightFactor,
      over: self.denominator * leftFactor
    )
  }

  -(_ other: Rational) -> Rational {
    self + (-other)
  }

  *(_ other: Rational) -> Rational {
    let g1 = math.gcd(self.numerator, other.denominator)
    let g2 = math.gcd(other.numerator, self.denominator)
    Rational.of(
      (self.numerator ~/ g1) * (other.numerator ~/ g2),
      over: (self.denominator ~/ g2) * (other.denominator ~/ g1)
    )
  }

  /(_ other: Rational) -> Rational {
    if other.numerator == 0 {
      ArgumentError.new("cannot divide Rational by zero").raise()
    }
    self * other.reciprocal
  }

  - -> Rational { Rational(-self.numerator, self.denominator) }

  /// Integral exponentiation by squaring.
  **(_ exponent: Int) -> Rational {
    if exponent == 0 { return Rational.one }
    if exponent < 0 { return self.reciprocal ** (-exponent) }

    let mut base = self
    let mut power = exponent
    let mut result = Rational.one

    while power > 0 {
      if power % 2 == 1 { result = result * base }
      power = power ~/ 2
      if power > 0 { base = base * base }
    }

    result
  }

  compare(_ other: Rational) -> Ordering {
    (self.numerator * other.denominator).compare(
      other.numerator * self.denominator
    )
  }

  <(_ other: Rational) -> Bool { self.compare(other) === Ordering::Less }
  <=(_ other: Rational) -> Bool {
    let order = self.compare(other)
    order === Ordering::Less or order === Ordering::Equal
  }
  >(_ other: Rational) -> Bool { self.compare(other) === Ordering::Greater }
  >=(_ other: Rational) -> Bool {
    let order = self.compare(other)
    order === Ordering::Greater or order === Ordering::Equal
  }

  ==(_ other: Dynamic) -> Bool {
    if not other.is(Rational) { return false }
    self.numerator == other.numerator and self.denominator == other.denominator
  }

  toIntExact -> Int {
    if self.denominator != 1 {
      ArgumentError.new("Rational is not an integer").raise()
    }
    self.numerator
  }

  toFloat -> Float {
    Float.new(self.numerator) / Float.new(self.denominator)
  }

  toString -> String {
    if self.denominator == 1 { "\(self.numerator)" }
    else { "\(self.numerator)/\(self.denominator)" }
  }

  // Named decimal powers used by standard prefixes.
  @class tenth -> Rational { rationalTenth }
  @class hundredth -> Rational { rationalHundredth }
  @class thousandth -> Rational { rationalThousandth }
  @class millionth -> Rational { rationalMillionth }
  @class billionth -> Rational { rationalBillionth }
  @class trillionth -> Rational { rationalTrillionth }
  @class quadrillionth -> Rational { rationalQuadrillionth }
  @class quintillionth -> Rational { rationalQuintillionth }
  @class sextillionth -> Rational { rationalSextillionth }
  @class septillionth -> Rational { rationalSeptillionth }
  @class octillionth -> Rational { rationalOctillionth }
  @class nonillionth -> Rational { rationalNonillionth }

  @class ten -> Rational { rationalTen }
  @class hundred -> Rational { rationalHundred }
  @class thousand -> Rational { rationalThousand }
  @class million -> Rational { rationalMillion }
  @class billion -> Rational { rationalBillion }
  @class trillion -> Rational { rationalTrillion }
  @class quadrillion -> Rational { rationalQuadrillion }
  @class quintillion -> Rational { rationalQuintillion }
  @class sextillion -> Rational { rationalSextillion }
  @class septillion -> Rational { rationalSeptillion }
  @class octillion -> Rational { rationalOctillion }
  @class nonillion -> Rational { rationalNonillion }
}

export (
  Rational,
  rationalTenth, rationalHundredth, rationalThousandth,
  rationalMillionth, rationalBillionth, rationalTrillionth,
  rationalQuadrillionth, rationalQuintillionth, rationalSextillionth,
  rationalSeptillionth, rationalOctillionth, rationalNonillionth,
  rationalTen, rationalHundred, rationalThousand, rationalMillion,
  rationalBillion, rationalTrillion, rationalQuadrillion, rationalQuintillion,
  rationalSextillion, rationalSeptillion, rationalOctillion, rationalNonillion
)
