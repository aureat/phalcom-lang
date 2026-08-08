// area: absence
// spec: values-and-absence.md §3.3
// status: PASS
// Adversarial: the paired `ifTrue(_, ifFalse:_)` send used inline as an
// operand of `+` — since it returns R directly (not an Option), it composes
// into an ordinary arithmetic expression with no extraction step at all.

const a = 3
const b = 4
System.print((a > b).ifTrue(|| { a }, ifFalse: || { b }) + 10)
