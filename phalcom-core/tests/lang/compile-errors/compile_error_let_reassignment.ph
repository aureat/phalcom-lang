// area: compile-errors
// spec: ADR-0064
// status: NEGATIVE
// ADR-0064: `const` is immutable — reassigning it is a compile error. Declare
// the binding with `let` to allow mutation. (Filename kept as the
// pre-registered golden exception across the U-BINDINGS codemod — see
// implementation-spec.md §1.3 — even though the fixture body now uses
// `const`.)

const x = 1
System.print(x)
x = 2
System.print(x)
