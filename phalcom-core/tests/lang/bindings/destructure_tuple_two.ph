// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: `let (a, b) = point` — irrefutable tuple destructuring, positionally
// through the same `at(_)` List/Tuple already expose.
let (a, b) = (1, 2)
System.print(a)
System.print(b)
