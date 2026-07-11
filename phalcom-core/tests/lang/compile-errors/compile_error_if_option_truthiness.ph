// area: compile-errors
// spec: values-and-absence.md §3.5; ADR-0007
// status: NEGATIVE
// U6/BD-U6-1 (Option A): `Option` has no truth value. Branching on a literal
// Option is rejected at compile time (the statically detectable class); a
// non-literal Option condition is a hard runtime type error — there is no
// silent boolean coercion anywhere.

if (None) {
  System.print(1)
}
