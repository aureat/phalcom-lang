// area: list
// spec: U-LIST-plan.md §7; ADR-0019; ADR-0020
// status: PASS
// `[]` allocates empty, `add(_:)` pushes and returns self for
// chaining, `size` and `at(_:)` round-trip the pushed values.

const l = []
l.append(10)
l.append(20)
l.append(30)
System.print(l.size)
System.print(l.at(0))
System.print(l.at(1))
System.print(l.at(2))
