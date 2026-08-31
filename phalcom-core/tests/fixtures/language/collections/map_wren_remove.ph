// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Adapted from wren/test/core/map/remove.wren. Wren's `remove(_)` returns the
// removed value (or `null` if absent); Phalcom's `remove(_)` returns `self`
// (chainable, mirrors `List#removeAt`) so the removed value is read via
// `[]` immediately beforehand instead — `remove(_)` itself stays
// idempotent (removing an absent key is a no-op).

const m = Map.new()
m.at("one", put: 1)
m.at("two", put: 2)
m.at("three", put: 3)

System.print(m.size)
System.print(m["two"])
m.remove("two")
System.print(m.size)
System.print(m["three"])
m.remove("three")
System.print(m.size)

// Remove an already-removed entry: no-op, still returns self.
System.print(m.get("two"))
m.remove("two")
System.print(m.size)

System.print(m["one"])
m.remove("one")
System.print(m.size)
