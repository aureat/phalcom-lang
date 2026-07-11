// area: absence
// spec: values-and-absence.md; control-flow.md; ADR-0007; ADR-0010
// status: PASS
// U6 Invariant 4: a one-armed `ifTrue` whose branch is NOT taken yields the
// `None` singleton, never the raw `nil` sentinel. The result flows straight
// into `print` without crossing a read boundary, so the sacred-inlined
// `Bytecode::Nil` result site (and the `bool_if_true` fallback) must already be
// `None`. Prints `<None instance>`, not `nil`.

System.print(false.ifTrue { 1 })
