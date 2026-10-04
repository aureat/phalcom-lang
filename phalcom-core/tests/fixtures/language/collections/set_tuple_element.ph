// area: collections
// spec: map-and-set.md §3; tuple-and-range.md §1; DEC-CT-C; ADR-0039
// status: PASS
// A `Tuple` is immutable ⇒ hashable ⇒ a valid Set element (DEC-CT-C):
// two independently-built value-equal tuples dedup to one member.

let t1 = (1, 2)
let t2 = (1, 2)
let t3 = (3, 4)
let s = Set.new().add(t1).add(t2).add(t3)
System.print(s.size)
System.print(s.includes(t1))
System.print(s.includes(t3))
System.print(s.includes((9, 9)))
