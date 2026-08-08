// area: family
// spec: selectors.md §3 (Open form); ADR-0047
// status: PASS
// `obj::name` produces an Open callable Family bound to `obj`; calling it
// builds the selector from the family's base name plus the call site's
// argument labels, then performs an ordinary send (selectors.md §3).

class Point {
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to target, duration dur) {
    return "moved to \(target) from \(_x), \(_y) in \(dur)"
  }
}
const p = Point.new(1, 0)
const f = p::move
System.print(f(to: 5, duration: 2)) // moved to 5 from 1, 0 in 2
