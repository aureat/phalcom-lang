// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// Ported from Wren `test/core/fiber/abort_not_string.wren`: `Fiber.abort(_)`
// accepts any value, not only strings — `try()` hands back the raw `123`.

let fiber = Fiber.new {
  Fiber.abort(123)
}

System.print(fiber.try())
