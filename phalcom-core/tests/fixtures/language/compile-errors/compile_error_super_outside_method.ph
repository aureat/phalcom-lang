// area: compile-errors
// spec: method-lookup.md §1.14; U-INH §3.4
// status: NEGATIVE
// U-INH: `super` has no defining class to anchor its lookup at top level.
super.foo()
