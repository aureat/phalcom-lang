// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// any(where:) short-circuits when predicate succeeds, proven by counter

let mut counter = 0
let mut pred = |x| {
  counter = counter + 1
  x > 3
}
System.print([1, 2, 3, 4, 5].any(where: pred))
System.print(counter)
