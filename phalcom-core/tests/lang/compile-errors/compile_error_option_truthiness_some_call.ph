// area: compile-errors
// spec: values-and-absence.md §3.5; PDR-0033
// status: NEGATIVE
// `Some(value)` is an Option literal for the syntax-only truthiness check.

if (Some(1)) {
  System.print(1)
}
