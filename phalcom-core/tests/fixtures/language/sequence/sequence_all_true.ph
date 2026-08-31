// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// all(where:) returns true when predicate is true for all elements

System.print([1, 2, 3].all(where: |x| { x > 0 }))
System.print([1, 2, 3].all(where: |x| { x < 5 }))
