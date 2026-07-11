// area: absence
// spec: values-and-absence.md; control-flow.md; ADR-0007; ADR-0010
// status: PASS
// U6 Invariant 4: a taken `ifTrue` with an empty body produces the block's
// absent result, which surfaces to the `None` singleton rather than the raw
// `nil` sentinel. Prints `<None instance>`, not `nil`.

System.print(true.ifTrue { })
