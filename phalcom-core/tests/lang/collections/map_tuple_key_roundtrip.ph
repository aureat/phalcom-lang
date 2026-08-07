// area: collections
// spec: map-and-set.md §2; tuple-and-range.md §1; DEC-CT-C; ADR-0039
// status: PASS
// A `Tuple` is immutable ⇒ hashable ⇒ a valid Map key (DEC-CT-C) —
// distinguishes the "immutable" bar from "not a List": round-trips a Tuple
// key through put/at/includes/remove.

const m = Map.new()
const k1 = Tuple.__fromList(List.new().add(1).add(2))
m.at(k1, put: "pair")
System.print(m.size)
System.print(m.includes(k1))
const k2 = Tuple.__fromList(List.new().add(1).add(2))
System.print(m.includes(k2))
System.print(m.at(k2))
m.remove(k2)
System.print(m.size)
System.print(m.includes(k1))
