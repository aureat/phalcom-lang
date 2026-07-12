// area: family
// spec: selectors.md §3 (Open form); ADR-0047
// status: PASS
// `obj::name` produces an Open callable Family bound to `obj`; calling it
// builds the selector from the family's base name plus the call site's
// argument labels, then performs an ordinary send (selectors.md §3).

class Point {
  construct new(x, y) { _x = x; _y = y }
  move(to:, duration:) {
    return "moved to " + to.toString + " over " + duration.toString
  }
}
let p = Point.new(0, 0)
let f = p::move
System.print(f(to: 5, duration: 2))
