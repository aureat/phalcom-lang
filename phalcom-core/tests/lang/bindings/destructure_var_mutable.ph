// area: bindings
// spec: open-questions.md Q7; ADR-0046 §2
// status: PASS
// U14: a destructuring `var` binds mutable slots, inheriting ADR-0014's
// `let`/`var` rule unchanged.
var (a, b) = (1, 2)
a = a + 1
System.print(a)
System.print(b)
