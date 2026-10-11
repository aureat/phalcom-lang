// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: a rest-less `List` pattern requires an EXACT arity match too (not
// just a `Tuple` pattern) — `[a, b]` against a 2-element `List` binds
// cleanly.
let values: List<Int> = [1, 2]
let [a, b] = values
System.print(a)
System.print(b)
