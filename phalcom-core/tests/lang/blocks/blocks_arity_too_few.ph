// area: blocks
// spec: functions.md §1 (call arity check)
// status: PASS
// Calling a 2-arg block with only 1 argument raises a RuntimeError::Arity
// before the block body ever runs — "before" prints, "after" never does.
let add = { a, b => a + b }
System.print("before")
System.print(add.call(3))
System.print("after")
