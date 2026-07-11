// area: lexical/statements
// spec: lexical-structure.md
// status: PASS
// D3: a statement whose line ends in an operator continues onto the next
// physical line — the trailing-operator newline is suppressed by the lexer, so
// this parses as a single `1 + 2 + 3` expression.
let x = 1 +
        2 +
        3
System.print(x)
