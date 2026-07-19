// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/count.wren (Wren's `count` is Phalcom's
// `size`): grows as new keys land, stays put when an existing key is
// overwritten with a new value.

const m = Map.new()
System.print(m.size)
m.at("one", put: "value")
System.print(m.size)
m.at("two", put: "value")
System.print(m.size)
m.at("three", put: "value")
System.print(m.size)

// Overwriting an existing key does not grow size.
m.at("two", put: "new value")
System.print(m.size)
