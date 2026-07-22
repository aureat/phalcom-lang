class Base {
  @class _count = 0

  @constructor
  new() {
    class.bump()
  }

  @class
  bump() {
    if (_count == None) {
      _count = 0
    }
    _count = _count + 1
  }

  @class
  count => _count
}

class Derived extends Base {}

let d1 = Derived.new()
System.print("d1 = \(d1)")
System.print("Base.count = \(Base.count)")
System.print("Derived.count = \(Derived.count)")

let d2 = Derived.new()
System.print("d2 = \(d2)")
System.print("Base.count = \(Base.count)")
System.print("Derived.count = \(Derived.count)")

let b1 = Base.new()
System.print("b1 = \(b1)")
System.print("Base.count = \(Base.count)")
System.print("Derived.count = \(Derived.count)")

let d3 = Derived.new()
System.print("d3 = \(d3)")
System.print("Base.count = \(Base.count)")
System.print("Derived.count = \(Derived.count)")
