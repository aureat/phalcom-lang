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

export Number
