// area: absence
// spec: values-and-absence.md §3.3 (paired `ifTrue(_, ifFalse:_)`, R directly)
// status: PASS
// Mirror of `absence_iftrue_iffalse_paired_true` on a `false` receiver: the
// `ifFalse:` arm's raw value comes back directly, still no `Option` wrapper.

System.print(false.ifTrue({ "yes" }, ifFalse: { "no" }))
System.print(false.ifTrue({ 1 + 1 }, ifFalse: { 0 }))
