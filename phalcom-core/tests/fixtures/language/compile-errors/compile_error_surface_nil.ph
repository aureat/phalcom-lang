// area: compile-errors
// spec: values-and-absence.md §3; ADR-0007; ADR-0010
// status: NEGATIVE
// U6 removes surface `nil`: absence is the `Option` type, never a `nil`
// literal, and the private `Value::Nil` sentinel has no surface syntax
// (Invariant 4). With the `nil` keyword gone, `nil` is just an undefined
// identifier, so this fails to compile rather than printing the sentinel.

System.print(nil)
