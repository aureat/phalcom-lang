// area: collections
// spec: map-and-set.md §3; tuple-and-range.md §1; DEC-CT-C; ADR-0039
// status: PASS
// A `Tuple` is immutable ⇒ hashable ⇒ a valid Set element (DEC-CT-C):
// two independently-built value-equal tuples dedup to one member.

const s = Set.new()
const t1 = Tuple.__fromList(List.new().add(1).add(2))
const t2 = Tuple.__fromList(List.new().add(1).add(2))
const t3 = Tuple.__fromList(List.new().add(3).add(4))
s.add(t1).add(t2).add(t3)
System.print(s.size)
System.print(s.includes(t1))
System.print(s.includes(t2))
System.print(s.includes(Tuple.__fromList(List.new().add(9).add(9))))
