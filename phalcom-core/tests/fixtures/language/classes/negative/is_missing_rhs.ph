// area: classes
// spec: next/is-tests.md
// status: NEGATIVE
// `is` requires a class-expression RHS; a bare `x is` with nothing after it
// is a parse error, not e.g. a partial-application or postfix reading.

let x = 3
System.print(x is)
