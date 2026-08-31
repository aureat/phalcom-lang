// area: lexical/literals
// spec: lexical-structure.md §5; ADR-0022
// status: PASS
// D4: multiple `\(expr)` interpolations in one string, including an arithmetic
// expression, plus the `\\(` escape for a literal `\(`.
const a = 3
const b = 4
System.print("\(a) plus \(b) is \(a + b)")
System.print("literal: \\(not interp)")
