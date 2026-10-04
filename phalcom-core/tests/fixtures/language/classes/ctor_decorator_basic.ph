class Point {
  mut _x
  mut _y
  @constructor
  new(_ x, _ y) {
    _x = x
    _y = y
  }
  x { _x }
  y { _y }
}
let mut p = Point.new(2, 3)
System.print(p.x)
System.print(p.y)
