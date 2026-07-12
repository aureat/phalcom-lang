// area: family
// spec: selectors.md §3 (Pinned form); ADR-0047
// status: PASS
// `obj::#move(to,duration)` pins the full selector identity at the
// reference site — the call site's own labels are ignored, only its
// argument count is checked (selectors.md §3 "Pinned families have their
// selector fully known at compile time").

class Point {
  construct new(x, y) { _x = x; _y = y }
  move(to:, duration:) {
    return "moved to " + to.toString + " over " + duration.toString
  }
}
let p = Point.new(0, 0)
let f = p::#move(to,duration)
System.print(f(5, 2))
