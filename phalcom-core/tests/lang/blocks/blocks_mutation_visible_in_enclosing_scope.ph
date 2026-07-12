// area: blocks
// spec: blocks.md §5 (open upvalue aliasing)
// status: PASS
// A block mutating a captured `var` mutates the SAME stack slot the
// enclosing scope reads — the upvalue is still open (block never escapes),
// so the enclosing scope observes the mutation immediately after `call()`.
var total = 10
let addFive = { total = total + 5 }

addFive.call()
System.print(total)
addFive.call()
System.print(total)
