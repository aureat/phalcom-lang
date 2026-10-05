// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

class Vec {
  mut _x
  mut _y
  @constructor
  new(_ x, _ y) {
    _x = x
    _y = y
  }
  x { _x }
  y { _y }
  +(_ other) {
    return Vec.new(_x + other.x, _y + other.y)
  }
}
let a = Vec.new(1, 2)
let b = Vec.new(3, 4)
let c = a + b
System.print(c.x)
System.print(c.y)
