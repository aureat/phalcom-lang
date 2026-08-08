// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS
// Ported from Wren `test/core/fiber/try_without_error.wren`: `Fiber#try`
// behaves exactly like `Fiber#call` when the entry never raises — it is
// only the failure path that differs (captured `Error` value instead of
// propagation).

const fiber = Fiber.new || {
  System.print("fiber")
}

System.print("before")
System.print(fiber.try())
System.print("after")
