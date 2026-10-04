// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/reuse_tombstone.wren (Wren issue #373
// regression) — insert, remove, reinsert at a since-vacated key, remove
// again: the vacated slot must not resurrect a stale entry.

let m = Map.new()
m.at(2, put: "two")
m.at(0, put: "zero")
m.remove(2)
m.at(0, put: "zero again")
m.remove(0)

System.print(m.includes(0))
