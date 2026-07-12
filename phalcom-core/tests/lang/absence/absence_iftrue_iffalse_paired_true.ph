// area: absence
// spec: values-and-absence.md §3.3 (the "sacred `ifTrue(_, ifFalse:_)` selector" —
// atomic, both arms in one send, returns R directly, NOT an Option)
// status: PASS
// Adversarial: the paired two-armed form is locked distinct from the
// one-armed `ifTrue { }` (which wraps in `Some`/`None`, U-CORE-2). On a
// `true` receiver, `ifTrue(_, ifFalse:_)` returns the TAKEN arm's raw value
// directly — no `Some` wrapper, no `.unwrapOr`/`.match` needed to extract it.

System.print(true.ifTrue({ "yes" }, ifFalse: { "no" }))
System.print(true.ifTrue({ 1 + 1 }, ifFalse: { 0 }))
