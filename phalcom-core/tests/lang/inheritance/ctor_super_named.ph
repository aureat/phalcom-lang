class Base {
  @constructor
  make(_ value) { _value = value }
  value { _value }
}
class Derived is Base {
  @constructor
  make(_ value) { super.make(value) }
}
System.print(Derived.make(11).value)
