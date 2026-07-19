// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

class Pt {
  x => _x
  construct new(x) {
    _x = x
  }
  ==(other) {
    return true
  }
}
const a = Pt.new(1)
System.print(a == a)
System.print(a == Pt.new(1))
