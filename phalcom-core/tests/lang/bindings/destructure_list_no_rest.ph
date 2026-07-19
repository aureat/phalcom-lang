// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: a rest-less `List` pattern requires an EXACT arity match too (not
// just a `Tuple` pattern) — `[a, b]` against a 2-element `List` binds
// cleanly.
const [a, b] = [1, 2]
System.print(a)
System.print(b)
