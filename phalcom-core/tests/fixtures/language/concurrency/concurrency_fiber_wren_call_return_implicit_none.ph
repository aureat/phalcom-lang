// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// Ported from Wren `test/core/fiber/call_return_implicit_null.wren`: a
// fiber entry with no explicit `return` and no `Fiber.yield` completes on
// its first `call()`, delivering the last statement's value — `Unit`, since
// `System.print` answers `Unit` — as the call's result.

const fiber = Fiber.new || {
  System.print("fiber")
}

System.print(fiber.call())
