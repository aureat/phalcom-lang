class Base {
  @constructor
  new(value) { _base = value }
  base => _base
}
class Derived is Base {
  @constructor
  new(value) { super.new(value); _derived = value + 1 }
  derived => _derived
}
let d = Derived.new(2)
System.print(d.base)
System.print(d.derived)
