// area: compile-errors
// spec: open-questions.md Q7; ADR-0046 §2
// status: NEGATIVE
// A destructuring pattern has nothing to unpack from an absent value, so
// `const (a, b)` with no `= expr` is a compile error — regardless of `const`/
// `let` (unlike a bare-name `let x`, which is legal and reads `None`).

const (a, b)
System.print(a)
