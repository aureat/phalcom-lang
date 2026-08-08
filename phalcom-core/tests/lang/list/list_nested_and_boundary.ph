// area: list
// spec: U-LIST-plan.md; ADR-0007; ADR-0020 (absence boundary)
// status: PASS
// Adversarial: a list of lists (a genuinely nested, mutable-in-mutable
// structure — mutating an inner list is visible through the outer list's
// reference); empty-list boundary for `each`/`map`/`filter`/`reduce`
// (all must be no-ops/identity, never a panic on a zero-length receiver).

const outer = []
outer.append([1, 2])
outer.append([3])
System.print(outer.size)
System.print(outer.at(0).size)
outer.at(0).append(99)
System.print(outer.at(0))
System.print(outer.at(0).size)
System.print(outer)

const empty = []
System.print(empty.isEmpty)
let sum = 0
empty.each |x| { sum = sum + x }
System.print(sum)
System.print(empty.map |x| { x * 2 }.toList)
System.print(empty.filter |x| { true })
System.print(empty.reduce(0) |acc, x| { acc + x })
System.print(empty.includes(1))
