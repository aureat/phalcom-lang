class Point {
  @constructor
  new(x, y) {
    _x = x
    _y = y
  }
  x => _x
  y => _y
}
let p = Point.new(2, 3)
System.print(p.x)
System.print(p.y)
