class Positive {
  mut _value
  @constructor
  @requires(value > 0)
  new(_ value) { _value = value }
  value { _value }
}
System.print(Positive.new(3).value)
