// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// find(where:) returns Some(x) when element is found

let result = [1, 2, 3, 4].find(where: |x| { x == 3 })
System.print(result.class.name)
result.match(some: |x| { System.print(x) }, none: || { System.print("not found") })
