// area: family
// spec: docs/spec/callables/family.md §3
// status: PASS
// A named structural pattern keeps the receiver bound and selects the
// labeled route at call time.

class Point {
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to target, duration dur) {
    return "moved to \(target) from \(_x), \(_y) in \(dur)"
  }
}
const p = Point.new(1, 0)
const f = p::move(...)
System.print(f(to: 5, duration: 2)) // moved to 5 from 1, 0 in 2
