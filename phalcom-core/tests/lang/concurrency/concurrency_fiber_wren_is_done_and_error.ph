// area: concurrency
// spec: concurrency.md §1 (Interface table: `isDone`, `error`)
// status: PASS

// Ported/merged from Wren `test/core/fiber/is_done.wren` and
// `test/core/fiber/error.wren`: `Fiber#isDone` (true once `Done`/`Failed`)
// and `Fiber#error` (the captured `Error` as `Option`, `None` until
// `Failed`) — landed by U-FIBER-REFLECT (`fiber_is_done`/`fiber_error` in
// `primitive/fiber.rs`, registered in `universe/primitives.rs` alongside the
// U-FIBER floor's `call`/`try`/`yield`/`current`/`abort`).

const ok = Fiber.new {
  System.print("1")
  Fiber.yield()
  System.print("2")
}

System.print(ok.isDone)
ok.call()
System.print(ok.isDone)
ok.call()
System.print(ok.isDone)

const failing = Fiber.new {
  "s".unknown
}

System.print(failing.error)
const result = failing.try()
System.print(result.message)
System.print(failing.error.match(some: { e => e.message }, none: { "none" }))
