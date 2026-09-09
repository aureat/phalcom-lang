// area: concurrency
// spec: patch-grade plan C3/C4
// status: PASS
// A parked action and its completion observer survive a forced GC before the
// dependency wakes the action.

const dependency = Future.new()
const result = Future.async || {
  dependency.await
  "survived-gc"
}
System.runScheduled()
System.gc
dependency.settleValue(None)
System.runScheduled()
System.print(result.await)
