class Point {
  mut _x
  mut _y
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  x { _x }
  y { _y }
}
let mut C = Point
let mut ps = [Point]
System.print(C.new(4, 5).x)
System.print(ps[0].new(6, 7).y)
