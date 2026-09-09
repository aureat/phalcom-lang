// area: concurrency
// spec: concurrency.md §2; patch-grade plan C2
// status: PASS
// The same scheduler-owned Fiber may park on two Futures in sequence. Each
// await receives a new generation and the old Future cannot resume the later
// suspension.

const first = Future.new()
const second = Future.new()
const worker = Fiber.new || {
  System.print("park-a")
  first.await
  System.print("park-b")
  second.await
  System.print("done")
}
System.schedule(worker)
System.runScheduled()
first.settleValue(1)
System.runScheduled()
second.settleValue(2)
System.runScheduled()
