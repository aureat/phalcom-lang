// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Boundary: a freshly-constructed empty Map — size 0, `get` on an absent key
// is total (None, never a panic/error), `includes` false, `remove` on an
// absent key is a no-op that still returns the receiver. `keys` and `values`
// are live views, so their iterable contents are observed through `toList`.

const m = Map.new()
System.print(m.size)
System.print(m.get(1))
System.print(m.get(1) == None)
System.print(m.includes(1))
m.remove(1)
System.print(m.size)
System.print(m.keys.toList)
System.print(m.values.toList)
