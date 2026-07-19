// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039; RG-1
// status: PASS
// Boundary: an exclusive range whose bounds coincide (`1...1`) is empty
// (size 0, `toList` = `[]`); an inclusive single-element range (`1..1`) has
// size 1. Adversarial: a *descending* range (`start > end`) — pins whatever
// the runtime actually does (clamped to empty, per the `size` formula's
// max-with-0), never a negative size or an infinite/backwards walk.

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
System.print(desc.toList)
System.print(desc.includes(3))

const descExclusive = Range.new(5, 1, false)
System.print(descExclusive.size)
System.print(descExclusive.toList)
