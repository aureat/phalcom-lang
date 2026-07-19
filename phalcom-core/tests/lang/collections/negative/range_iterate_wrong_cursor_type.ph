// area: collections
// spec: tuple-and-range.md §2; ADR-0035 §1
// status: NEGATIVE
// Wren range/iterate_wrong_type.wren, adapted: post-Route-B (ADR-0048)
// `iterate(_)` does `cursor + 1` directly (not the earlier Option-cursor
// `cursor.map { c => c + 1 }` idiom, and not a raw type-check either), so
// passing a non-numeric cursor is a plain arithmetic rejection — never a
// bespoke "Iterator must be a number" diagnostic (Wren has no `Option`,
// hence its own type-check message).

const r = Range.new(1, 3, true)
r.iterate("")
