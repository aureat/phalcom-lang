// area: arithmetic/operators
// spec: values-and-absence.md
// status: PENDING
// ported from wren/test/core/number/clamp.wren: `Number::clamp(_, _)` not on
// the floor yet.
System.print(5.clamp(0, 10))
System.print((-5).clamp(0, 10))
System.print(15.clamp(0, 10))
