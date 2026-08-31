// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// any(where:) returns false when predicate is false for all elements

System.print([1, 2, 3].any(where: |x| { x > 5 }))
System.print([1, 2, 3].any(where: |x| { x == 10 }))
