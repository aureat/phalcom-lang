// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PENDING
// Ported from wren/test/core/map/clear.wren. There is no `clear_`
// primitive (primitive/map.rs) and no `.ph` `clear` wrapper — `Map` has no
// bulk-empty operation today. Pinning the intended surface: `clear()`
// empties the map in place and returns the receiver (chainable, mirroring
// `remove(_)`/`at(_, put:)`'s self-return convention rather than Wren's
// `null`).

const a = Map.new()
a.at(1, put: 1)
a.at(2, put: 2)
a.at(3, put: 3)
a.clear()
System.print(a)
System.print(a.size)
