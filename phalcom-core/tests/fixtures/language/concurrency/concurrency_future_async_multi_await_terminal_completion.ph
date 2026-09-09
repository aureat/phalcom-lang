// area: concurrency
// spec: concurrency.md §2; patch-grade plan C4
// status: PASS
// Future.async settles only after an action completes all of its awaits.

const first = Future.new()
const second = Future.new()
const result = Future.async || {
  System.print("async-start")
  first.await
  System.print("async-mid")
  second.await
  "async-done"
}
System.print("ready-0: " + result.isReady.toString)
System.runScheduled()
System.print("ready-1: " + result.isReady.toString)
first.settleValue(1)
System.runScheduled()
System.print("ready-2: " + result.isReady.toString)
second.settleValue(2)
System.runScheduled()
System.print("ready-3: " + result.isReady.toString)
System.print(result.await)
