// area: compile-errors
// spec: values-and-absence.md; ADR-0014
// status: NEGATIVE
// U6/ADR-0014: `let` is immutable — reassigning it is a compile error. Declare
// the binding with `var` to allow mutation. (Was a PASS fixture before U6, when
// all bindings were mutable; U6 legalizes the distinction.)

let x = 1
System.print(x)
x = 2
System.print(x)
