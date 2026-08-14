// area: family
// spec: docs/spec/callables/family.md §1–3
// status: PASS
// Exact references retain selector identity, while a named structural pattern
// routes both overload shapes through the current method table.

class Point {
  @constructor
  new(_ x, _ y) { _x = x; _y = y }
  move(to, duration) {
    return "labeled: to " + to.toString + " over " + duration.toString
  }
  move(_ a, _ b) {
    return "positional: " + a.toString + " " + b.toString
  }
}

const p = Point.new(0, 0)

const labeled = p::#move(to,duration)
const positional = p::#move(_,_)
System.print("(exact) " + labeled(to: 5, duration: 2))
System.print("(exact) " + positional(5, 2))

const f = p::move(...)
System.print("(pattern) " + f(to: 5, duration: 2))
System.print("(pattern) " + f(5, 2))
