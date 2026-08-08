// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039
// status: NEGATIVE
// Adversarial: unlike a whole-number negative coordinate (which is relative
// to the end), a fractional Number is not an index shape. `at(_)` raises a
// Type error rather than rounding or truncating it.

const t = (1, 2)
t.at(1.5)
