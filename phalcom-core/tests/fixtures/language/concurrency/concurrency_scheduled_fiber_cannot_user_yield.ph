// area: concurrency
// spec: concurrency.md §2; CONC002.C2.P2
// status: PASS
// A scheduled Fiber with no active coroutine consumer cannot user-yield.

const worker = Fiber.new || {
  Fiber.yield(42)
}

System.schedule(worker)
System.runScheduled()

System.print("worker failed: " + worker.isDone.toString)
System.print("error: " + worker.error.unwrapOr(None).message)
