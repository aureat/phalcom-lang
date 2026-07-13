// area: collections
// spec: map-and-set.md §2; iteration.md §1; ADR-0039
// status: NEGATIVE
// Adapted from wren/test/core/map/iterate_iterator_not_int.wren. Wren's
// `iterate(_)` takes a raw int cursor and type-checks it directly; Phalcom's
// `iterate(_)` (C-ITER-9) takes an `Option` cursor and calls `.map(_)` on it
// — passing a bare `Number` instead of `None`/`Some(_)` is not a dedicated
// cursor-type check but the same underlying misuse (a non-cursor value where
// the cursor protocol expects one), surfacing as `Number does not understand
// 'map(_:)'`.

let m = Map.new()
m.at(1, put: 2)
m.at(3, put: 4)
m.iterate(1.5)
