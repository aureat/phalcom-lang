// area: lexical/literals
// spec: lexical-structure.md §5; ADR-0022
// status: PASS
// D4: `\(expr)` string interpolation (ADR-0022). The lexer splits the string
// into literal/expression segments and the parser desugars them to a `+`-chain
// of `String.new(_)`-stringified parts.
let name = "Ada"
System.print("\(name) is great")
