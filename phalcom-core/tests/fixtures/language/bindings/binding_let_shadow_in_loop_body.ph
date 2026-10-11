// area: bindings
// spec: values-and-absence.md; iteration.md §1; ADR-0014
// status: PASS
// A `const` declared inside a `for` loop body is a fresh binding scoped to
// EACH iteration's own body-block — it shadows an outer `const` of the same
// name for the duration of the loop without ever writing through to it,
// and the outer binding is untouched once the loop finishes.

let x = 100
let mut seen: List<Int> = []
let input: Array<Int> = [1, 2]
for i in input {
  let x = i
  seen.append(x)
}
System.print(seen)
System.print(x)
