// area: blocks
// spec: blocks.md §5; ADR-0013 (open/closed upvalues)
// status: PASS
// Two DIFFERENT closures created in the same scope over the same `var` share
// ONE open upvalue cell — mutating through `inc` must be visible when reading
// through `show`, proving the cell is aliased, not copied per-closure.
var count = 0
let inc = { count = count + 1 }
let show = { count }

inc.call()
inc.call()
System.print(show.call())
inc.call()
System.print(show.call())
