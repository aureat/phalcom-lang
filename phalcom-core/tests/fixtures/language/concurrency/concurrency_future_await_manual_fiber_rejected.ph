// spec: concurrency.md §2; CONC002.C2.P2
// status: PASS
// A manually resumed Fiber may await a pending Future. When the Future is pending
// and the scheduler is empty with no progress, quiescence detection raises a catchable error.

const pending = Future.new()
const manual = Fiber.new || { pending.await }
manual.try()
System.print(manual.error.unwrapOr(None).message)
