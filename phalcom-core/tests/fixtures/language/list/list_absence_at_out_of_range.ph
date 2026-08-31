// area: list
// spec: U-LIST-plan.md §7; ADR-0020 (absence boundary); ADR-0007
// status: PASS
// `at(_:)` on an out-of-range index (an empty list, index 0) surfaces the
// `None` singleton — never a panic, never the raw `nil` sentinel — reusing
// U6's absence-surfacing boundary (Invariant 4).

const l = []
System.print(l.at(0))
System.print(l.at(0) == None)
