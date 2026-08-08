// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS (graduated U5: == dispatches through the class hierarchy)

class Pt {
  x => _x
  @constructor
  new(_ x) {
    _x = x
  }
  ==(_ other) {
    return _x == other.x
  }
}
const a = Pt.new(1)
const b = Pt.new(1)
const c = Pt.new(2)
System.print(a == b)
System.print(a == c)
