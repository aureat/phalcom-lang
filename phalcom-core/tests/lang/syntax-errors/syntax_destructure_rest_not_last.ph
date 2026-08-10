// area: errors
// spec: open-questions.md Q7; ADR-0046 §1; messages-and-selectors.md §5
// status: NEGATIVE
// A rest sub-pattern (`*rest`) must be the last element of a list pattern —
// the same terminal-rest rule used for declaration parameters — a clean
// parser diagnostic, not a panic.

const [*rest, last] = [1, 2, 3]
System.print(last)
