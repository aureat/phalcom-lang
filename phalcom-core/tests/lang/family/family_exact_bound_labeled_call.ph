// area: family
// spec: docs/spec/callables/family.md §1 and §2
// status: PASS
// Exact Family retains labeled selector identity, so the call must use the
// same labeled shape. `::` owns selector-spec context; no `#` is used here.

class Point {
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to, duration) {
    return "moved to " + to.toString + " over " + duration.toString
  }
}
const p = Point.new(0, 0)
const f = p::move(to,duration)
System.print(f(to: 5, duration: 2))
