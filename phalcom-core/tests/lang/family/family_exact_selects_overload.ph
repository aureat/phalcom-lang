// area: family
// spec: docs/spec/callables/family.md §1–3
// status: PASS

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

const labeled = (Point >> #move(to, duration)).bind(p)
const positional = (Point >> #move(_,_)).bind(p)
System.print("(exact) " + labeled(to: 5, duration: 2))
System.print("(exact) " + positional(5, 2))

const f = (Point >> #move(...)).bind(p)
System.print("(pattern) " + f(to: 5, duration: 2))
System.print("(pattern) " + f(5, 2))
