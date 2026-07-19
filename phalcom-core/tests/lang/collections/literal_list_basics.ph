// area: collections
// spec: lexical-structure.md §4; ADR-0029; ADR-0032 §1
// status: PASS
// The list literal `[…]` desugars in the parser to a `List.new().add(…)`
// construction chain (zero new primitives). Round-trips size/at and renders
// via the landed `List.toString`; `[]` is the bare `List.new()`.

const l = [1, 2, 3]
System.print(l.size)
System.print(l.at(0))
System.print(l.at(2))
System.print(l.toString)
System.print([].size)
