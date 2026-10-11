from universe.text.errors import (
  IntParseError,
  FloatParseError
)

native primitive class Number is Object {
  native primitive +(_ other: Number) -> Self
  native primitive -(_ other: Number) -> Self
  native primitive *(_ other: Number) -> Self
  native primitive /(_ other: Number) -> Float
  native primitive %(_ other: Number) -> Self
  native primitive ~/(_ other: Number) -> Self
  native **(_ other: Number) -> Self
  native <(_ other: Number) -> Bool
  native <=(_ other: Number) -> Bool
  native >(_ other: Number) -> Bool
  native >=(_ other: Number) -> Bool
  native primitive compare(_ other: Number) -> Ordering
  native + -> Self
  native - -> Self
  native hash -> Int
  native toString -> String
  @class
  native new() -> Never
  @class
  native new(_ value: Dynamic) -> Never
}

native primitive class Int is Number {
  native &(_ other: Int) -> Int
  native |(_ other: Int) -> Int
  native ^(_ other: Int) -> Int
  native ~ -> Int
  native <<(_ shift: Int) -> Int
  native >>(_ shift: Int) -> Int
  native bitAt(_ index: Int) -> Int
  native bitCount -> Int
  native bitLength -> Int
  native trailingZeros -> Int
  internal native rawToStringRadix(_ radix: Int) -> String
  @class
  internal native rawParse(_ text: String, _ radix: Int) -> Option<Int>
  @class
  native new() -> Int
  @class
  native new(_ value: Dynamic) -> Int

  /// Formats this integer in the given radix (2..36).
  toString(radix: Int) -> String {
    self.rawToStringRadix(radix)
  }

  /// Parses an integer from text in base 10.
  @class
  parse(_ text) -> Result<Int, IntParseError> {
    Int.parse(text, radix: 10)
  }

  /// Parses an integer from text in the specified radix (2..36).
  @class
  parse(_ text, radix: Int) -> Result<Int, IntParseError> {
    let s = text.toString
    let opt = Int.rawParse(s, radix)
    if opt.isSome {
      return Result::Ok(opt.unwrap)
    }
    Result::Error(IntParseError.new("Invalid integer string: " + s))
  }
}

native primitive class Float is Number {
  @class
  native epsilon -> Float
  @class
  native minPositiveSubnormal -> Float
  @class
  native minNormal -> Float
  @class
  native maxFinite -> Float
  @class
  native new() -> Float
  @class
  native new(_ value: Dynamic) -> Float
  @class
  internal native rawParse(_ text: String) -> Option<Float>

  /// Parses a floating-point number from text.
  @class
  parse(_ text) -> Result<Float, FloatParseError> {
    let s = text.toString
    let opt = Float.rawParse(s)
    if opt.isSome {
      return Result::Ok(opt.unwrap)
    }
    Result::Error(FloatParseError.new("Invalid float string: " + s))
  }
  native abs -> Float
  native sign -> Int
  native floor -> Int
  native ceil -> Int
  native truncated -> Int
  native rounded -> Int
  native toIntExact -> Int
  native isInteger -> Bool
  native isNaN -> Bool
  native isFinite -> Bool
  native isInfinite -> Bool
  native isNormal -> Bool
  native isSubnormal -> Bool
  native isPositiveZero -> Bool
  native isNegativeZero -> Bool
  native nextUp -> Float
  native nextDown -> Float
  native ulp -> Float
  native totalCompare(_ other: Float) -> Ordering
  /// Fused multiply-add with a single final rounding.
  native fma(_ multiplier: Float, _ addend: Float) -> Float
  /// Return this value's magnitude with the sign bit copied from `sign`.
  native copySign(_ sign: Float) -> Float
  /// Scale by an integer power of two, following IEEE overflow/underflow rules.
  native scalePow2(_ exponent: Int) -> Float
}
