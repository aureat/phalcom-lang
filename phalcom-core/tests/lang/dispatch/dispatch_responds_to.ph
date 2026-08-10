// area: dispatch
// spec: method-lookup.md §2; ADR-0012
// status: PASS
// `respondsTo(_:)` is a pure exact-selector probe: true for a defined
// selector, false for an unknown one — and it NEVER triggers
// `doesNotUnderstand(_:)` (the false branch would otherwise raise).

class Widget { render { "drawn" }
}
const w = Widget.new()
System.print(w.respondsTo(Symbol.new("render")))
System.print(w.respondsTo(Symbol.new("nonexistent")))
