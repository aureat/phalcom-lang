// area: collections
// spec: lexical-structure.md §7; ADR-0032 §3.2
// status: PASS (regression)
// The tuple `(…)` arm must fall through to *exactly* the pre-U-COLL grouping
// when there is no top-level comma: `(x)` is `x` (never a one-tuple), and
// parenthesised precedence is unperturbed.

let x = 5
System.print((x))
System.print((1 + 2) * 3)
