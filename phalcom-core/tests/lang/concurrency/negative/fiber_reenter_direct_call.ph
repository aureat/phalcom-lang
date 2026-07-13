// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE
// Ported from Wren `test/core/fiber/call_direct_reenter.wren`: a fiber
// resuming itself from inside its own entry is `Running`, not `Suspended`
// — rejected by the same guard as calling a finished fiber, distinct
// message ("fiber is already running").

var fiber = None

fiber = Fiber.new {
  fiber.call()
}

fiber.call()
