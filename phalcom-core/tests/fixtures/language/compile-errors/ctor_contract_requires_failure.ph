class Positive {
  mut _value
  @constructor
  @requires(value > 0)
  new(_ value) { _value = value }
}
Positive.new(0)
