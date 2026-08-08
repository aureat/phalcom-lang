// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS
// Ported from Wren `test/core/fiber/call.wren`: a `Fiber#call` with no
// resumed value runs the entry to its first suspension point (here, the
// whole body, since there is no `Fiber.yield`) before control returns to
// the caller.

const fiber = Fiber.new || {
  System.print("fiber")
}

System.print("before")
fiber.call()
System.print("after")
