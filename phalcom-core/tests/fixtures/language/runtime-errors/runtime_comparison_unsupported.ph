// area: errors
// spec: values-and-absence.md; messages-and-selectors.md
// status: NEGATIVE
// U5: Number now registers `<(_)` (control-flow.md §1: comparisons are
// ordinary sends), so `3 < 5` is no longer erroneous — see
// arithmetic/arithmetic_comparisons.ph for the now-PASS case. This case
// tests the send still failing to resolve on a receiver that has no `<(_)`.

System.print(true < false)
