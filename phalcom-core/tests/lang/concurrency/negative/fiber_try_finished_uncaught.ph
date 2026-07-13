// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: NEGATIVE
// Ported from Wren `test/core/fiber/try_done.wren`: `try()` on an already
// `Done` fiber shares the same "cannot resume a finished fiber" guard as
// `call()` (fiber.rs `fiber_resume`), uncaught here.

let fiber = Fiber.new {
  System.print("try")
}

fiber.try()
fiber.try()
