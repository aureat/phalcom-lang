class Positive {
  @constructor
  @requires(value > 0)
  new(_ value) { _value = value }
  value { _value }
}
System.print(Positive.new(3).value)
