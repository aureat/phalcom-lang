// area: classes
// spec: messages-and-selectors.md; selectors.md
// status: PASS

class Point x => _x
  y => _y
  @constructor
  new(_ x, _ y) {
    _x = x
    _y = y
  }
  move(to, at) {
    return Point.new(to, at)
  }
}
const p = Point.new(0, 0).move(to: 3, at: 4)
System.print(p.x)
System.print(p.y)
