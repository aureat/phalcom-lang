class Positive {
  @constructor
  @requires(value > 0)
  new(value) { _value = value }
}
Positive.new(0)
