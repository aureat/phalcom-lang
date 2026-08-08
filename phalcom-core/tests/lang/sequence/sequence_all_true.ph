// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// all(f) returns true when predicate is true for all elements

System.print([1, 2, 3].all |x| { x > 0 })
System.print([1, 2, 3].all |x| { x < 5 })
