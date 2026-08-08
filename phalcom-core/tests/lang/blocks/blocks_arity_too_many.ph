// area: blocks
// spec: functions.md §1 (call arity check)
// status: PASS
// Calling a 1-arg block with 2 arguments raises a RuntimeError::Arity before
// the block body ever runs — "before" prints, "after" never does.
const square = { a => a * a }
System.print("before")
try {
  square.call(3, 4)
} catch e {
}
