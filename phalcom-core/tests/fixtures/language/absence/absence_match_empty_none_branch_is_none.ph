// area: absence
// spec: values-and-absence.md §3.2; ADR-0007; U6-plan.md §4
// status: PASS
// U6: `match(some:none:)` on `None` takes the `none:` branch; an *empty*
// `none:` block falls off its end and surfaces to the `None` singleton. This
// exercises the non-inlined `block_call` fall-off-end path, mirroring the
// inliner's `Nil` placeholder.

System.print(None.match(some: |v| { v }, none: || { }))
