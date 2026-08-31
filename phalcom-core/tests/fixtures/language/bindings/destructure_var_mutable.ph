// area: bindings
// spec: open-questions.md Q7; ADR-0046 §2
// status: PASS
// U14: a destructuring `let` binds mutable slots, inheriting ADR-0014's
// `const`/`let` rule unchanged.
let (a, b) = (1, 2)
a = a + 1
System.print(a)
System.print(b)
