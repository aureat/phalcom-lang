// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// count(where:) counts only elements matching predicate

System.print([1, 2, 3, 4, 5].count(where: |x| { x > 2 }))
System.print([1, 2, 3].count(where: |x| { x == 10 }))
