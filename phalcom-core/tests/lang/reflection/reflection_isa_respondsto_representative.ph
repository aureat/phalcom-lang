// area: reflection
// spec: object-model.md; method-lookup.md §2; ADR-0012
// status: PASS
// Single representative pairing of `isA(_)` (type-membership probe, walks
// the superclass chain) and `respondsTo(_:)` (exact-selector probe, no
// chain walk semantics of its own) on the SAME instance — deep-chain and
// arity-family variants are already exercised in metaclass/ and dispatch/,
// not duplicated here.

class Widget { render => "drawn"
}
const w = Widget.new()
System.print(w.isA(Widget))
System.print(w.isA(Object))
System.print(w.isA(String))
System.print(w.respondsTo(Symbol.new("render")))
System.print(w.respondsTo(Symbol.new("nonexistent")))
