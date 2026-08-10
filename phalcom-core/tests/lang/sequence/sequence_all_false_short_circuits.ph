// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// all(where:) short-circuits when predicate fails, proven by counter

let counter = 0
let pred = |x| {
  counter = counter + 1
  x < 3
}
System.print([1, 2, 3, 4, 5].all(where: pred))
System.print(counter)
