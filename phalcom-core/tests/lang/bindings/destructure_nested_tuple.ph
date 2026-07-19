// area: bindings
// spec: open-questions.md Q7; ADR-0046
// status: PASS
// U14: patterns nest recursively — `const ((a, b), c) = …` binds all three.
const ((a, b), c) = ((1, 2), 3)
System.print(a)
System.print(b)
System.print(c)
