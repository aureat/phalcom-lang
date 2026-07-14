// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// all(f) short-circuits when predicate fails, proven by counter

var counter = 0
var pred = { x =>
  counter = counter + 1
  x < 3
}
System.print([1, 2, 3, 4, 5].all(pred))
System.print(counter)
