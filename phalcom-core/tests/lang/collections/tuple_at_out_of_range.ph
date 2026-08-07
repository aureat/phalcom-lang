// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039; ADR-0020
// status: PASS
// Boundary: `at(_)` past the arity is total — the `None` singleton, never a
// panic — mirroring List/Map's absence boundary. Empty product construction
// normalizes to Unit, which is intentionally not a Tuple.

const t = Tuple.__fromList(List.new().add(1).add(2))
System.print(t.at(2))
System.print(t.at(2) == None)
System.print(t.at(99))

const e = ()
System.print(e == ())
