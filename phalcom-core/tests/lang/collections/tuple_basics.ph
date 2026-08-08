// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039
// status: PASS
// Tuple surface protocol: literal construction, at/size, structural ==
// across independently-built tuples, value-hash equality, immutability (no
// mutation selector — dNU).

const a = (1, 2)
const b = (1, 2)
System.print(a.size)
System.print(a.at(0))
System.print(a.at(1))
System.print(a == b)
System.print(a.hash == b.hash)
System.print(a == [1, 2])
