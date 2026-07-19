// area: collections
// spec: map-and-set.md §2; iteration.md §1; ADR-0039; ADR-0048
// status: NEGATIVE
// Adapted from wren/test/core/map/iterate_iterator_not_num.wren — post-Route-B
// (ADR-0048) `iterate(_)` does `cursor + 1` directly, so any `String` cursor
// (content irrelevant) hits the same `Expected String, got number` arithmetic
// rejection as `map_wren_iterate_not_int.ph`.

const m = Map.new()
m.at(1, put: 2)
m.at(3, put: 4)
m.iterate("2")
