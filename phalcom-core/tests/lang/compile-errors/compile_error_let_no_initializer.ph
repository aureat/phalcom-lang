// area: compile-errors
// spec: values-and-absence.md; ADR-0014
// status: NEGATIVE
// U6/ADR-0014: `let` is an immutable binding and must be initialized at its
// declaration. A `let` with no initializer is a compile error (use `var x`
// for an uninitialized, `None`-reading binding instead).

let x
System.print(x)
