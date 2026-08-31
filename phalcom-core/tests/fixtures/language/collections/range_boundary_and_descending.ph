// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039; RG-1
// status: PASS
// Boundary: an exclusive range whose bounds coincide is empty; an inclusive
// single-element range has size 1. Descending ranges remain constructible and
// retain their non-iterative size/includes behavior; E.2 rejects traversal.

const empty = Range.new(1, 1, false)
System.print(empty.size)
System.print(empty.toList)
System.print(empty.includes(1))

const single = Range.new(1, 1, true)
System.print(single.size)
System.print(single.first)
System.print(single.last)
System.print(single.toList)

const desc = Range.new(5, 1, true)
System.print(desc.size)
System.print(desc.includes(3))

const descExclusive = Range.new(5, 1, false)
System.print(descExclusive.size)
