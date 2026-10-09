// area: collections
// spec: lexical-structure.md §4; ADR-0029
// status: PASS
// An explicitly contextualized literal yields a real `List`, so `List`
// combinators (`map`) apply directly to it — proving construction is not an
// inert AST shape.

let values: List<Int> = [1, 2, 3]
let doubled = values.map |x| { x + 1 }.toList
System.print(doubled.size)
System.print(doubled.at(0))
System.print(doubled.at(2))
