// area: family
// spec: selectors.md §3 (Open form); ADR-0047
// status: PASS
// A call site with no labels builds the positional selector shape
// (`move(_,_)`) from the family's base name — proving the selector is built
// at call time from the *call site's* labels, not fixed at reference time.
// The receiver expression here is itself a `MethodCall` (`Adder.new()`), so
// this also proves `::` composes as an ordinary postfix over any expression.

class Adder {
  move(_ a, _ b) { return a + b }
}
const f = Adder.new()::move
System.print(f(3, 4))
