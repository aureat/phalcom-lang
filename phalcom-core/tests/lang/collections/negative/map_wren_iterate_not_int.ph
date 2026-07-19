// area: collections
// spec: map-and-set.md §2; iteration.md §1; ADR-0039; ADR-0048
// status: NEGATIVE
// Adapted from wren/test/core/map/iterate_iterator_not_int.wren. Wren's
// `iterate(_)` takes a raw int cursor and type-checks it directly; Phalcom's
// post-Route-B `iterate(_)` (C-ITER-9, ADR-0048) does `cursor + 1` directly —
// not a dedicated cursor-type check — so a value that arithmetic itself
// rejects (a `String`, matching `range_iterate_wrong_cursor_type.ph`'s
// pattern) is the misuse this pins, surfacing as `Expected String, got
// number` (a bare `Number` like `1.5` no longer errors here: `1.5 + 1`
// succeeds and just walks off the end into `None`).

const m = Map.new()
m.at(1, put: 2)
m.at(3, put: 4)
m.iterate("")
