// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

class Vec {
  @constructor
  new(x, y) {
    _x = x
    _y = y
  }
  x => _x
  y => _y
  +(other) {
    return Vec.new(_x + other.x, _y + other.y)
  }
}
const a = Vec.new(1, 2)
const b = Vec.new(3, 4)
const c = a + b
System.print(c.x)
System.print(c.y)
