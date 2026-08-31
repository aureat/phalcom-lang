// area: absence
// spec: values-and-absence.md §3.1; iteration.md; PDR-0033
// status: PASS
// `Some(None)` is present during iteration; only the cursor's exact `None`
// value terminates the loop.

const values = [None, Some(None)]
values.each |value| { System.print(value) }
System.print(Some(None).isSome)
