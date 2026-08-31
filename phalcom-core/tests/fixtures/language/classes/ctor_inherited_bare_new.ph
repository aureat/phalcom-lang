class Base {
  @constructor
  new() { _value = 1 }
}
class Derived is Base {
  @constructor
  new(_ value) { _value = value }
  value { _value }
}
System.print(Derived.new().value) // None
System.print(Derived.new(5).value) // 5
