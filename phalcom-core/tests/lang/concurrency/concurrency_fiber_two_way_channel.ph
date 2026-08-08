// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// C-FIB-2 adversarial: `Fiber.yield(_)`'s return value (the argument
// delivered by the *next* `call(_)`) is consumed and folded back into the
// fiber's running total, then re-published on the *following* yield — a
// two-way channel, not a one-shot echo. Each `call(n)` both delivers `n` and
// receives the running total computed *before* `n` was applied.
const doubler = Fiber.new || {
  let total = 0
  let input = Fiber.yield(total)
  while (true) {
    total = total + input
    input = Fiber.yield(total)
  }
}
System.print(doubler.call())
System.print(doubler.call(5))
System.print(doubler.call(10))
System.print(doubler.call(100))
