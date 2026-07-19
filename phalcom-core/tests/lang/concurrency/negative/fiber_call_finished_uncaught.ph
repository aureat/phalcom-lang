// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE
// Ported from Wren `test/core/fiber/call_done.wren`: resuming a `Done`
// fiber a second time is illegal, uncaught here (contrast the caught form
// in `../concurrency_fiber_abort_then_resume_fails.ph`).

const fiber = Fiber.new {
  System.print("call")
}

fiber.call()
fiber.call()
