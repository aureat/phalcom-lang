// area: runtime-errors
// spec: open-questions.md Q7; ADR-0046 §2
// status: NEGATIVE
// U14: `[first, *rest]` requires the scrutinee to have AT LEAST the fixed
// element count — a 0-element List is one short.

const [first, *rest] = []
System.print(first)
