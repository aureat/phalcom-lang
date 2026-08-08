// E002 regression, cascade case: `outer` captures a local (z) and escapes a
// block over it, then .call()s `inner`, which fails uncaught. The
// fiber-floor cascade walks the Call-mode resumer chain (inner -> outer)
// without ever making `outer` the live/running fiber again, so `outer`'s
// open upvalues live only in its *parked* FiberObject fields, not
// VM::open_upvalues — exercising the fiber-scoped close_fiber_upvalues_from
// path (as opposed to concurrency_fiber_floor_upvalue_close.ph, which
// exercises the originating, still-live fiber's close_upvalues_from(0) path).
let leak = || { 0 }
let inner = Fiber.new || {
  Fiber.abort(Error.new())
}
let outer = Fiber.new || {
  let z = 7
  leak = { z }
  inner.call()
}
outer.try()
System.print(leak.call())
