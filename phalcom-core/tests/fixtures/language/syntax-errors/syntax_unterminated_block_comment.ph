// area: errors
// spec: lexical-structure.md
// status: NEGATIVE
// D1: a `/*` with no closing `*/` is an unterminated-comment lexical error.
System.print(1) /* oops, never closed
