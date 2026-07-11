// area: absence
// spec: values-and-absence.md; system.md; ADR-0007; ADR-0010
// status: PASS
// U6 Invariant 4: `System.print(_)` is a surface-reachable send whose result
// flows straight into the outer `print` argument without crossing a read
// boundary, so it must yield the `None` singleton, not the raw `nil` sentinel.
// The inner call prints `1`; the outer prints the inner's `None` result.

System.print(System.print(1))
