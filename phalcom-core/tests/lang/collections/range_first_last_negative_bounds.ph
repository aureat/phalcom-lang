// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: PASS
// Wren range/from.wren + range/to.wren, ordered-range slice only (Phalcom
// has no backwards-range walk; see range_boundary_and_descending.ph).
// `first`/`last` are the spec-anchored endpoint selectors (Wren's
// `from`/`to`); pinned across negative bounds and a single-element range.

System.print(Range.new(-5, 3, true).first)
System.print(Range.new(-5, 3, true).last)
System.print(Range.new(-5, -2, true).first)
System.print(Range.new(-5, -2, true).last)
System.print(Range.new(3, 3, true).first)
System.print(Range.new(3, 3, true).last)
System.print(Range.new(-5, 3, false).last)
