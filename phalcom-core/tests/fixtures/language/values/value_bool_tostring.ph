// area: values
// spec: values-and-absence.md; U-CORE-4 (R-INV-4.1)
// status: PASS
// `Bool#toString` is `.ph`-derived over the sacred `ifTrue(_, ifFalse)`
// selector (`core.ph`'s `Bool` reopen); non-sacred itself, so no inliner
// deopt (floor-census §5).

System.print(true.toString)
System.print(false.toString)
