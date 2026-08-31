// area: absence
// spec: values-and-absence.md; object-model.md; ADR-0007; ADR-0010
// status: PASS
// U6 Invariant 4: the root class has no superclass; `Object.superclass` returns
// the `None` singleton (surface absence value), never the raw `nil` sentinel.
// The primitive result flows straight into `print`. Prints `None`.

System.print(Object.superclass)
