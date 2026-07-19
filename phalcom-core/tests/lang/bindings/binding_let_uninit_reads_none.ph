// area: bindings
// spec: ADR-0064; ADR-0007
// status: PASS
// `let x` with no initializer is legal and reads `None` (carried over from
// ADR-0007 verbatim; ADR-0064 only flips which keyword is mutable).

let x
System.print(x)
