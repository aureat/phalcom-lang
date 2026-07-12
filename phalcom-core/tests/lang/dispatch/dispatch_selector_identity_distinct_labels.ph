// area: dispatch
// spec: messages-and-selectors.md §5; method-lookup.md
// status: PASS
// Selector identity by LABEL, not just arity: `foo(a:)` and `foo(b:)` are
// both 1-positional-arg keyword methods on the SAME class with the SAME bare
// name (`foo`) and the SAME arity (1), differing only in their keyword
// label. Each is its own selector (`foo(a:)` vs `foo(b:)`) and dispatches to
// its own body.

class Multi {
  foo(a:) { return "a=" + a.toString }
  foo(b:) { return "b=" + b.toString }
}
let m = Multi.new()
System.print(m.foo(a: 1))
System.print(m.foo(b: 2))
