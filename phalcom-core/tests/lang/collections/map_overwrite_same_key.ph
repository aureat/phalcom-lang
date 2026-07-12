// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Boundary: re-putting the same key overwrites the value in place — size
// does not grow, and the key's iteration slot is not duplicated.

let m = Map.new()
m.at(1, put: "a")
System.print(m.size)
m.at(1, put: "b")
System.print(m.size)
System.print(m.at(1))
m.at(1, put: "c")
System.print(m.size)
System.print(m.at(1))
System.print(m.keys)
System.print(m.values)
