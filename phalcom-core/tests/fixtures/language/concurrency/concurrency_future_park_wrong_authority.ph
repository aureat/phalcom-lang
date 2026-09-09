// area: concurrency
// spec: concurrency.md §2; patch-grade plan C2
// status: PASS
// A Future park is not a public coroutine suspension: call, try, and schedule
// cannot steal it, and the authorized Future wake still resumes the waiter.

const future = Future.new()
const waiter = Fiber.new || {
  System.print("waiter-start")
  future.await
  System.print("waiter-resumed")
}
System.schedule(waiter)
System.runScheduled()

const callProbe = Fiber.new || { waiter.call() }
System.schedule(callProbe)
System.runScheduled()
const tryProbe = Fiber.new || { waiter.try() }
System.schedule(tryProbe)
System.runScheduled()

let scheduleError = None
try {
  System.schedule(waiter)
} catch e {
  scheduleError = e.message
}
System.print("schedule: " + scheduleError)
System.print("call: " + callProbe.error.unwrapOr(None).message)
System.print("try: " + tryProbe.error.unwrapOr(None).message)

future.settleValue(7)
System.runScheduled()
