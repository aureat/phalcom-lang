class Base {
  @constructor
  new() { _value = 1 }
}
class Derived extends Base {
  @constructor
  new(value) { _value = value }
  value => _value
}
System.print(Derived.new().value)
System.print(Derived.new(5).value)
