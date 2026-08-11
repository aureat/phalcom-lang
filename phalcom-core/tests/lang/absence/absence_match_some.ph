// area: absence
// spec: values-and-absence.md §3.2; ADR-0007
// status: PASS
// `Some(x)` uses canonical ordinary class-side call syntax. `match(some:none:)` is the sole
// eliminator that leaves Option-world with a value; here it takes the `some:`
// branch and yields the wrapped value.

const o = Some(42)
System.print(o.match(some: |v| { v }, none: || { 0 }))
