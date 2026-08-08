// area: absence
// spec: values-and-absence.md §3.2; ADR-0007
// status: PASS
// U6: `Some` is constructed with the explicit static send `Some.new(_)` (there
// is no bare `Some(x)` call syntax in U6). `match(some:none:)` is the sole
// eliminator that leaves Option-world with a value; here it takes the `some:`
// branch and yields the wrapped value.

const o = Some.new(42)
System.print(o.match(some: |v| { v }, none: { 0 }))
