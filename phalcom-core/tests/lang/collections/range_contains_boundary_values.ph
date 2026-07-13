// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: PASS
// Wren range/contains.wren (ordered-range slice only — Phalcom's Range has
// no backwards-range semantics; range_boundary_and_descending.ph already
// pins a `start > end` range as clamped to empty, size 0). `includes(_)` is
// the spec-anchored selector (Wren's `contains`): below-start, at-start,
// at-end (inclusive vs exclusive), and past-end all pinned for both bound
// kinds.

let inc = Range.new(2, 5, true)
System.print(inc.includes(1))
System.print(inc.includes(2))
System.print(inc.includes(5))
System.print(inc.includes(6))

let exc = Range.new(2, 5, false)
System.print(exc.includes(1))
System.print(exc.includes(2))
System.print(exc.includes(5))
System.print(exc.includes(6))
