// area: booleans
// spec: control-flow.md; values-and-absence.md; next/is-tests.md
// status: PASS
// `not` as a general prefix over an arbitrary boolean expression (not just a
// bare literal) — U-NEG's negation surface: `not (1 == 2)` lowers to
// `(1 == 2).not`, exercising precedence around the parenthesized operand.

System.print(not (1 == 2))
System.print(not (1 == 1))
