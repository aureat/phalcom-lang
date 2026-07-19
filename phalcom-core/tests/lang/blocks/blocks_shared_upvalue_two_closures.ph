// area: blocks
// spec: blocks.md §5; ADR-0013 (open/closed upvalues)
// status: PASS
// Two DIFFERENT closures created in the same scope over the same `let` share
// ONE open upvalue cell — mutating through `inc` must be visible when reading
// through `show`, proving the cell is aliased, not copied per-closure.
let count = 0
const inc = { count = count + 1 }
const show = { count }

inc.call()
inc.call()
System.print(show.call())
inc.call()
System.print(show.call())
