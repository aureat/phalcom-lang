// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: NEGATIVE
// Wren range/no_constructor.wren: `Range` has one constructor,
// `new(_:_:_:)` (start/end/inclusive) — no other arity exists. A 2-arg call
// is a plain dNU on the metaclass, never a silent partial construction.

Range.new(1, 2)
