// area: family
// spec: docs/spec/callables/family.md §3
// status: PASS

class Point {
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to target, duration dur) {
    return "moved to \(target) from \(_x), \(_y) in \(dur)"
  }
}
const p = Point.new(1, 0)
const f = (Point >> #move(...)).bind(p)
System.print(f(to: 5, duration: 2))
