class Positive {
  @constructor
  @requires(value > 0)
  new(_ value) { _value = value }
}
Positive.new(0)
