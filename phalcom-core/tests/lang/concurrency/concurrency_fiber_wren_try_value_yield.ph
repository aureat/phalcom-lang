// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// Ported from Wren `test/core/fiber/try_value_yield.wren`: a first `try(_)`
// resume delivers its argument at the entry parameter and suspends at
// `Fiber.yield`; a second `try(_)` delivers its argument as the `yield`'s
// return value and then captures the subsequent uncaught failure.

let fiber = Fiber.new { v =>
  System.print("before")
  System.print(v)
  var w = Fiber.yield()
  System.print(w)
  true.unknownMethod
  System.print("after")
}

fiber.try("value1")
let result = fiber.try("value2")
System.print(result.class.name)
System.print(result.message)
System.print("after try")

