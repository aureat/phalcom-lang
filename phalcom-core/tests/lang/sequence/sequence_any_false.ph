// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// any(f) returns false when predicate is false for all elements

System.print([1, 2, 3].any { x => x > 5 })
System.print([1, 2, 3].any { x => x == 10 })
