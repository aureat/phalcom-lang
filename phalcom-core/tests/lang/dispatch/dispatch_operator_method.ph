// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md
// status: PASS

class Vec x => _x
  y => _y
  @class
  new(_ x, _ y) {
    const v = self.new();
    v.init(x, y);
    return v;
  }
  init(_ x, _ y) {
    _x = x;
    _y = y;
  }
  +(_ other) {
    return Vec.new(_x + other.x, _y + other.y);
  }
}
const a = Vec.new(1, 2)
const b = Vec.new(3, 4)
const c = a + b
System.print(c.x)
System.print(c.y)
