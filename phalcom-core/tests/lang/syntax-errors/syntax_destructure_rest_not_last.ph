// area: errors
// spec: open-questions.md Q7; ADR-0046 §1; messages-and-selectors.md §5
// status: NEGATIVE
// A rest sub-pattern (`*rest`) must be the last element of a list pattern —
// the same rule U9 enforces for a variadic parameter — a clean parser
// diagnostic, not a panic.

let [*rest, last] = [1, 2, 3]
System.print(last)
