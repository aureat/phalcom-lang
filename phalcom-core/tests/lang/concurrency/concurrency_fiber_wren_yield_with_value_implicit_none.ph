// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// Ported from Wren `test/core/fiber/yield_with_value.wren`: a straight-line
// (no loop) fiber yields a value at each of two suspension points, then
// falls off the end without an explicit `return` — the third `call()`
// delivers the entry's implicit completion value, `None` (the last
// statement's value; see `concurrency_fiber_call_resume_value` for the same
// implicit-completion shape).

const fiber = Fiber.new || {
  System.print("fiber 1")
  Fiber.yield("yield 1")
  System.print("fiber 2")
  Fiber.yield("yield 2")
  System.print("fiber 3")
}

System.print(fiber.call())
System.print(fiber.call())
System.print(fiber.call())
