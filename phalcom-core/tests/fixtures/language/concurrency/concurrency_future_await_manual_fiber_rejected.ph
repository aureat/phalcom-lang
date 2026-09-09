// area: concurrency
// spec: concurrency.md §2; patch-grade plan C2
// status: PASS
// Await is a scheduler-owned operation. A manually resumed Fiber must fail
// before it registers a Future waiter.

const pending = Future.new()
const manual = Fiber.new || { pending.await }
manual.try()
System.print(manual.error.unwrapOr(None).message)
