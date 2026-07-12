// area: bindings
// spec: values-and-absence.md; iteration.md §1; ADR-0014
// status: PASS
// A `let` declared inside a `for` loop body is a fresh binding scoped to
// EACH iteration's own body-block — it shadows an outer `let` of the same
// name for the duration of the loop without ever writing through to it,
// and the outer binding is untouched once the loop finishes.

let x = 100
var seen = List.new()
for (i in List.new().add(1).add(2)) {
  let x = i
  seen.add(x)
}
System.print(seen)
System.print(x)
