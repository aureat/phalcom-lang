/*@navigation.point.definition*/
class Point {
  mut _x: Int = 0
  mut _y: Int = 0

  @constructor
  new(_ x: Int, y: Int) {
    _x = x
    _y = y
  }

  x -> Int { _x }
  y -> Int { _y }
}

export Point
