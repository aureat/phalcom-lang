// area: bindings
// spec: values-and-absence.md; ADR-0014; ADR-0007
// status: PASS
// U6: a `let` with no initializer reads as `None`. The slot is backed
// internally by the private `Value::Nil` sentinel, surfaced to the shared
// `None` singleton at the read boundary (never the raw sentinel).

let x
System.print(x)
