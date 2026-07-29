class Base {
  @constructor
  make(value) { _value = value }
  value => _value
}
class Derived is Base {
  @constructor
  make(value) { super.make(value) }
}
System.print(Derived.make(11).value)
