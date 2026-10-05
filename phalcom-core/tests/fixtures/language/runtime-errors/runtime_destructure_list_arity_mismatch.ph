// area: runtime-errors
// spec: open-questions.md Q7; ADR-0046 §2
// status: NEGATIVE
// U14: a rest-less list pattern requires an exact-arity scrutinee too —
// `[a, b]` against a 1-element `List` is a clean runtime error.

let [a, b] = [1]
System.print(a)
