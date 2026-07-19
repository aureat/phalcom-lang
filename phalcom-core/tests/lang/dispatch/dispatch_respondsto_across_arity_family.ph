// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md
// status: PASS
// Selector identity across an arity family: `m()`, `m(a)`, and `m(a, b)` on
// ONE class are three DISTINCT selectors (`m()`, `m(_)`, `m(_,_)`), each
// independently probeable via `respondsTo(_)`. A 3-arg `m` was never
// defined, so `m(_,_,_)` correctly reports false — `respondsTo` is an exact
// selector match, not an arity-family membership test.

class Overload {
  m() { return 0; }
  m(a) { return a; }
  m(a, b) { return a + b; }
}
const o = Overload.new()
System.print(o.respondsTo(Symbol.new("m()")))
System.print(o.respondsTo(Symbol.new("m(_)")))
System.print(o.respondsTo(Symbol.new("m(_,_)")))
System.print(o.respondsTo(Symbol.new("m(_,_,_)")))
