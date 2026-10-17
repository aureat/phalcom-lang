from .number import Number
from universe.text.errors import FloatParseError

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

export Float
