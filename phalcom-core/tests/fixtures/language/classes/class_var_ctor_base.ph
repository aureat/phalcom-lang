class Base {
  @class mut _count = 0

  @constructor
  new() {
    class.bump()
  }

  @class
  bump() {
    _count = _count + 1
  }

  @class
  count { _count }
}

let mut b1 = Base.new()
System.print("b1 = \(b1)")
System.print("Base.count = \(Base.count)")
let mut b2 = Base.new()
System.print("b2 = \(b2)")
System.print("Base.count = \(Base.count)")
