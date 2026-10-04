// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS (graduated U5: == dispatches through the class hierarchy)

class Pt {
  mut _x
 x { _x }
  @constructor
  new(_ x) {
    _x = x
  }
  ==(_ other) {
    return _x == other.x
  }
}
let a = Pt.new(1)
let b = Pt.new(1)
let c = Pt.new(2)
System.print(a == b)
System.print(a == c)
