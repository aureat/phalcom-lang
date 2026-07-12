// area: compile-errors
// spec: open-questions.md Q7; ADR-0046 §2
// status: NEGATIVE
// A destructuring pattern has nothing to unpack from an absent value, so
// `let (a, b)` with no `= expr` is a compile error — regardless of `let`/
// `var` (unlike a bare-name `var x`, which is legal and reads `None`).

let (a, b)
System.print(a)
