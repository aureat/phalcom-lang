// area: family/negative
// spec: docs/spec/callables/family.md §2 — an exact Family rejects a call
// whose incoming shape does not satisfy its retained selector.
// status: NEGATIVE

class Point {
  mut _x
  mut _y
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to target, duration d) {
    return "moved to " + target.toString + " over " + d.toString
  }
}
let p = Point.new(0, 0)
let f = &Point.move(to, duration);
System.print(f(5))
