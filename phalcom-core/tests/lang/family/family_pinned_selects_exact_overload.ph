// area: family
// spec: selectors.md §3 (Pinned form — exact-overload identity); ADR-0047
// status: PASS
// The whole point of Pinned over Open: a class can define BOTH a labeled
// `move(to:,duration:)` and a positional `move(_,_)` overload (distinct
// selectors, ADR-0012). `obj::#move(to,duration)` pins the labeled one;
// `obj::#move(_,_)` pins the positional one — proven by dispatching each
// through its own Family and observing which method body ran.

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
System.print("(pinned) " + labeled(5, 2))
System.print("(pinned) " + positional(5, 2))

const f = p::move
System.print("(open) " + f(to: 5, duration: 2))
System.print("(open) " + f(5, 2))
