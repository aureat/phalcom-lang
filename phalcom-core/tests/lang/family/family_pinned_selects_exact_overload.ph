// area: family
// spec: selectors.md §3 (Pinned form — exact-overload identity); ADR-0047
// status: PASS
// The whole point of Pinned over Open: a class can define BOTH a labeled
// `move(to:,duration:)` and a positional `move(_,_)` overload (distinct
// selectors, ADR-0012). `obj::#move(to,duration)` pins the labeled one;
// `obj::#move(_,_)` pins the positional one — proven by dispatching each
// through its own Family and observing which method body ran.

class Point {
  construct new(x, y) { _x = x; _y = y }
  move(to:, duration:) {
    return "labeled: to " + to.toString + " over " + duration.toString
  }
  move(a, b) {
    return "positional: " + a.toString + " " + b.toString
  }
}
let p = Point.new(0, 0)
let labeled = p::#move(to,duration)
let positional = p::#move(_,_)
System.print(labeled(5, 2))
System.print(positional(5, 2))
