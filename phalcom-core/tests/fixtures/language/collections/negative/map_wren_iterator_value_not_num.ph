// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: NEGATIVE
// Adapted from wren/test/core/map/iterator_value_iterator_not_num.wren — a
// `String` index hits the same `expect_index` guard as a fractional
// `Number` (see `map_wren_iterator_value_not_int.ph`).

let m = Map.new()
m.at(1, put: "one")
m.iteratorValue("2")
