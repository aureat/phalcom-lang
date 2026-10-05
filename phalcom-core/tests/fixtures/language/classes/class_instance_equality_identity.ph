// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

class Pt {
  mut _x
 x { _x }
  @constructor
  new(_ x) {
    _x = x
  }
  ==(_ other) {
    return true
  }
}
let a = Pt.new(1)
System.print(a == a)
System.print(a == Pt.new(1))
