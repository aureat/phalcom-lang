// area: concurrency
// spec: concurrency.md; ADR-0030 §3
// status: PASS
// C-FIB-2: the argument passed to a resuming `call(_)` is delivered as the
// value `Fiber.yield(_)` returns inside the fiber. `System.print(_)` returns
// `Unit` after emitting its argument.

const echo = Fiber.new || {
  const a = Fiber.yield("ready")
  System.print(a)
  const b = Fiber.yield("ready again")
  System.print(b)
}
System.print(echo.call())
System.print(echo.call(10))
System.print(echo.call(20))
