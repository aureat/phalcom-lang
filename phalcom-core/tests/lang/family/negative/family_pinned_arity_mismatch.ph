// area: family/negative
// spec: selectors.md §3 (Pinned form, U16-Pinned) — a Pinned family's call
//   ignores the call site's own labels but still validates its argument
//   *count* against the pinned selector's arity; a mismatch is a hard error,
//   never a silent wrong dispatch.
// status: NEGATIVE

class Point {
  @constructor
  new(x, y) { _x = x; _y = y }
  move(to:, duration:) {
    return "moved to " + to.toString + " over " + duration.toString
  }
}
const p = Point.new(0, 0)
const f = p::#move(to,duration)
System.print(f(5))
