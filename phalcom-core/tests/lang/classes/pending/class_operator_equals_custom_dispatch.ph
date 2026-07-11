// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PENDING

class Pt {
  x => _x
  static new(x) {
    let p = self.new();
    p.init(x);
    return p;
  }
  init(x) {
    _x = x;
  }
  ==(other) {
    return _x == other.x;
  }
}
let a = Pt.new(1)
let b = Pt.new(1)
let c = Pt.new(2)
System.print(a == b)
System.print(a == c)
