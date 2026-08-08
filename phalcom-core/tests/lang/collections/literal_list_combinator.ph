// area: collections
// spec: lexical-structure.md §4; ADR-0029
// status: PASS
// The literal yields a *real* `List`, so `List` combinators (`map`) apply
// directly to it — proving the desugaring produces a genuine kernel `List`,
// not an inert AST shape.

const doubled = [1, 2, 3].map |x| { x + 1 }.toList
System.print(doubled.size)
System.print(doubled.at(0))
System.print(doubled.at(2))
