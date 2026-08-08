// E002 regression (docs/errors/E002-fiber-floor-upvalue-crash.md): a block
// captures a fiber-local and escapes into a module global; the fiber then
// fails uncaught (Fiber.abort) and is .try()-ed by its resumer. Before the
// fix the fiber-floor failure capture cleared the failing fiber's stack
// without closing its open upvalues first, so the escaped block's
// Upvalue::Open dangled and calling it later panicked (index out of bounds,
// dispatch.rs GetUpvalue). Printing 42 proves the upvalue was closed
// (promoted to a heap cell holding x) before the fiber's stack was
// discarded.
let leak = || { 0 }
let b = Fiber.new || {
  let x = 42
  leak = || { x }
  Fiber.abort(Error.new())
}
b.try()
System.print(leak.call())
