class Point {
  @constructor
  new(_ x, _ y) {
    _x = x
    _y = y
  }
  x => _x
  y => _y
}
let p = Point.new(2, 3)
System.print(p.x)
System.print(p.y)
