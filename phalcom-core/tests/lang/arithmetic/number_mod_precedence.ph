// area: arithmetic/operators
// spec: values-and-absence.md; messages-and-selectors.md
// status: PASS
// ported from wren/test/core/number/mod.wren: `%` is left-associative and
// binds tighter than `+`.
System.print(13 % 7 % 4)
System.print(13 + 1 % 7)
