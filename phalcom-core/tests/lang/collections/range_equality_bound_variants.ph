// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: PASS
// Wren range/equality.wren, adapted to Range.new(_,_,_) (the `..`/`...`
// literal is reserved-inactive per RG-1). Complements
// range_bound_equality_distinct.ph: pins that the inclusive/exclusive flag
// itself is a distinguishing bound field, not just start/end — two ranges
// with identical start/end but opposite inclusivity are unequal, and `!=`
// agrees.

const a = Range.new(2, 5, true)
System.print(a == Range.new(2, 5, true))
System.print(a == Range.new(2, 6, true))
System.print(a == Range.new(2, 5, false))
System.print(a != Range.new(2, 5, false))

const c = Range.new(2, 5, false)
System.print(c == Range.new(2, 5, false))
System.print(c == Range.new(2, 6, false))
System.print(c != Range.new(2, 6, false))
