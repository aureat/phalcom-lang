// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md
// status: PASS

class Vec {
  x => _x
  y => _y
  static new(x, y) {
    const v = self.new();
    v.init(x, y);
    return v;
  }
  init(x, y) {
    _x = x;
    _y = y;
  }
  +(other) {
    return Vec.new(_x + other.x, _y + other.y);
  }
}
const a = Vec.new(1, 2)
const b = Vec.new(3, 4)
const c = a + b
System.print(c.x)
System.print(c.y)
