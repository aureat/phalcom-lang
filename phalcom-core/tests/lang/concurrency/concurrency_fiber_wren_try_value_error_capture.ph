// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// Ported from Wren `test/core/fiber/try_value.wren`: `try(_)` both delivers
// the first-resume argument at the entry's parameter and captures an
// uncaught `doesNotUnderstand` failure raised partway through as the
// delivered `Error`, letting the caller resume normally afterward.

const fiber = Fiber.new { v =>
  System.print("before")
  System.print(v)
  true.unknownMethod
  System.print("after")
}

const result = fiber.try("value")
System.print(result.class.name)
System.print(result.message)
System.print("after try")

