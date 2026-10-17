from .number import Number
from universe.text.errors import IntParseError

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

export Int
