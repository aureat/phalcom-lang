// area: collections
// spec: map-and-set.md §2; iteration.md §1; ADR-0039
// status: NEGATIVE
// Adapted from wren/test/core/map/iterate_iterator_not_num.wren — a `String`
// cursor hits the same non-`Option` misuse as a `Number` cursor (see
// `map_wren_iterate_not_int.ph`), just with a different `does not
// understand 'map(_:)'` receiver.

let m = Map.new()
m.at(1, put: 2)
m.at(3, put: 4)
m.iterate("2")
