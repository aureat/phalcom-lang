// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

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
    return true;
  }
}
let a = Pt.new(1)
System.print(a == a)
System.print(a == Pt.new(1))
