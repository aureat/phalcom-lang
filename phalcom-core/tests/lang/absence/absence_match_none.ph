// area: absence
// spec: values-and-absence.md §3.2; ADR-0007
// status: PASS
// U6: `match(some:none:)` on the `None` singleton takes the `none:` branch,
// which receives no argument — the substrate eliminator that every U-STD
// combinator (`unwrapOr`/`orElse`/…) is later defined over.

System.print(None.match(some: |v| { v }, none: { 0 }))
