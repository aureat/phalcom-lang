class Pair {
  @constructor
  new() { _value = 0 }
  @constructor
  new(_ value) { _value = value }
  value => _value
}
System.print(Pair.new().value)
System.print(Pair.new(4).value)
