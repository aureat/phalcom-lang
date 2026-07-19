// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: NEGATIVE
// Adapted from wren/test/core/map/iterator_value_iterator_too_small.wren.
// Wren bounds-checks the cursor against the map's live capacity and raises
// "Iterator out of bounds"; Phalcom's `iteratorValue(_)` is a plain
// non-negative-integer index (`expect_index`, shared with List/Tuple), so a
// negative index is rejected as a Type error before any bounds check runs.
// (A too-large *positive* index is a distinct, total case — see
// `map_wren_cursor_roundtrip.ph`'s past-the-end coverage via the cursor
// protocol; `keyAt_`/`valueAt_` return the `None` singleton rather than
// erroring, so there is no NEGATIVE analog for Wren's
// `iterator_value_iterator_too_large.wren`.)

const m = Map.new()
m.at(1, put: "one")
m.iteratorValue(-9999)
