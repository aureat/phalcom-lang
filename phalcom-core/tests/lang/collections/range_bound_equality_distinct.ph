// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: PASS
// `Range#==` compares the normalized bound fields (start/end/inclusive), NOT
// the generated sequence (documented tradeoff — comparing sequences would
// defeat RG-2 laziness for a huge range). Two bound-distinct ranges that
// happen to generate the same elements are `!=`, even though their
// materialized `toList`s are equal.

const a = Range.new(1, 5, true)
const b = Range.new(1, 6, false)
System.print(a.toList)
System.print(b.toList)
System.print(a.toList == b.toList)
System.print(a == b)
System.print(a != b)

const c = Range.new(1, 5, true)
System.print(a == c)
