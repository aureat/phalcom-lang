@native
class Number is Object {
  @native +(_ other: Number) -> Number
  @native -(_ other: Number) -> Number
  @native *(_ other: Number) -> Number
  @native /(_ other: Number) -> Number
  @native %(_ other: Number) -> Number
  @native ~/(_ other: Number) -> Number
  @native **(_ other: Number) -> Number
  @native <(_ other: Number) -> Bool
  @native <=(_ other: Number) -> Bool
  @native >(_ other: Number) -> Bool
  @native >=(_ other: Number) -> Bool
  @native compare(_ other: Number) -> Ordering
  @native + -> Number
  @native - -> Number
  @native hash -> Int
  @native toString -> String
  @class
  @native
  new() -> Number
  @class
  @native
  new(_ value: Dynamic) -> Number
}

@native
class Int is Number {
  @native &(_ other: Int) -> Int
  @native |(_ other: Int) -> Int
  @native ^(_ other: Int) -> Int
  @native ~ -> Int
  @native <<(_ shift: Int) -> Int
  @native >>(_ shift: Int) -> Int
  @native bitAt(_ index: Int) -> Int
  @native bitCount -> Int
  @native bitLength -> Int
  @native trailingZeros -> Int
  @class
  @native
  new() -> Int
  @class
  @native
  new(_ value: Dynamic) -> Int
}

@native
class Float is Number {
  @class
  @native
  new() -> Float
  @class
  @native
  new(_ value: Dynamic) -> Float
  @native abs -> Float
  @native sign -> Int
  @native floor -> Int
  @native ceil -> Int
  @native truncated -> Int
  @native rounded -> Int
  @native toIntExact -> Int
  @native isInteger -> Bool
  @native isNaN -> Bool
  @native isFinite -> Bool
  @native isInfinite -> Bool
}
