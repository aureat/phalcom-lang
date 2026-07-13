// area: collections
// spec: tuple-and-range.md §2; ADR-0035 §1
// status: NEGATIVE
// Wren range/iterate_wrong_type.wren, adapted: Phalcom's `iterate(_)`
// dispatches `cursor.map { c => c + 1 }` (ADR-0035 §1's Option-cursor
// idiom, not a raw type-check), so passing a non-`Option` cursor is a plain
// dNU on `String#map` — never a bespoke "Iterator must be a number"
// diagnostic (Wren has no `Option`, hence its own type-check message).

let r = Range.new(1, 3, true)
r.iterate("")
