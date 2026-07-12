// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039; RG-1; RG-2
// status: PASS
// Parity: `1..5` (inclusive, per RG-1's committed literal meaning) has one
// more element than the exclusive `1...5` bound over the same numbers.
// Laziness (RG-2): a huge range (`1..1000000`) is O(1) to construct — `size`/
// `first`/`at(0)` never materialize the sequence, so this stays instant
// rather than allocating a million-element List.

let inc = Range.new(1, 5, true)
let exc = Range.new(1, 5, false)
System.print(inc.size)
System.print(exc.size)
System.print(inc.size - exc.size)
System.print(inc.includes(5))
System.print(exc.includes(5))

let huge = Range.new(1, 1000000, true)
System.print(huge.size)
System.print(huge.first)
System.print(huge.at(0))
System.print(huge.includes(999999))
System.print(huge.includes(1000001))
