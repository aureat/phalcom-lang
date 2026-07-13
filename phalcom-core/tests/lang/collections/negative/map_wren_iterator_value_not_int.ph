// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: NEGATIVE
// Adapted from wren/test/core/map/iterator_value_iterator_not_int.wren.
// `iteratorValue(_)` (backing `rawKeyAt`/`rawValueAt`) requires a
// non-negative integer index — a fractional `Number` is rejected by the
// same `expect_index` guard List/Tuple/Map share.

let m = Map.new()
m.at(1, put: "one")
m.iteratorValue(1.5)
