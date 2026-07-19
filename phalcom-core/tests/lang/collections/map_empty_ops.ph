// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Boundary: a freshly-constructed empty Map — size 0, `at` on an absent key
// is total (None, never a panic/error), `includes` false, `remove` on an
// absent key is a no-op that still returns the receiver.

const m = Map.new()
System.print(m.size)
System.print(m.at(1))
System.print(m.at(1) == None)
System.print(m.includes(1))
m.remove(1)
System.print(m.size)
System.print(m.keys)
System.print(m.values)
