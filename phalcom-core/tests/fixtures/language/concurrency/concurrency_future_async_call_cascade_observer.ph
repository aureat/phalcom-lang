// area: concurrency
// spec: patch-grade plan C3/C4
// status: PASS
// A Call-mode child failure terminally fails the scheduler-owned action, and
// the durable observer still rejects the outer Future exactly once.

const inner = Fiber.new || { throw Error.new("cascade failure") }
const result = Future.async || { inner.call() }
System.runScheduled()
try {
  result.await
} catch e {
  System.print("caught: " + e.message)
}
