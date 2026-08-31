// area: runtime-errors
// spec: open-questions.md Q7; ADR-0046 §2
// status: NEGATIVE
// U14: a tuple pattern requires an EXACT-arity scrutinee — `(a, b)` against a
// 3-tuple is a clean runtime error, not a panic or a silent truncation.

const (a, b) = (1, 2, 3)
System.print(a)
