// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// Ported from Wren `test/core/fiber/yield.wren`: three no-argument
// `Fiber.yield()` suspension points interleave with the caller's own prints
// across three `call()` resumes — a bare `yield()` still hands control back
// even though no value is threaded.

const fiber = Fiber.new {
  System.print("fiber 1")
  Fiber.yield()
  System.print("fiber 2")
  Fiber.yield()
  System.print("fiber 3")
}

fiber.call()
System.print("main 1")
fiber.call()
System.print("main 2")
fiber.call()
System.print("main 3")
