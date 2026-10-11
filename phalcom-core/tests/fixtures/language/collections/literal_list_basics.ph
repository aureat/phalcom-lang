// area: collections
// spec: lexical-structure.md §4; ADR-0029; ADR-0032 §1
// status: PASS
// The list literal `[…]` desugars in the parser to a `[…]`
// construction chain (zero new primitives). Round-trips size/at and renders
// via the landed `List.toString`; both values select List explicitly.

let l: List<Int> = [1, 2, 3]
System.print(l.size)
System.print(l.at(0))
System.print(l.at(2))
System.print(l.toString)
let empty: List<Int> = []
System.print(empty.size)
