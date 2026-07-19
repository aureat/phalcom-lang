// area: concurrency
// spec: concurrency.md; ADR-0030 §6
// status: PASS
// Ported from Wren `test/core/fiber/abort.wren` (the `try()`-return half —
// `isDone`/`error` accessors are not yet implemented, see
// `concurrency/pending/concurrency_fiber_wren_is_done_and_error.ph`):
// `Fiber.abort(_)` raises its argument at the fiber floor; `try()` captures
// it and hands back the exact value passed to `abort`, unwrapped, not
// wrapped in an `Error` instance.

const fiber = Fiber.new {
  Fiber.abort("Error message.")
}

System.print(fiber.try())
