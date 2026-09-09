// area: concurrency
// spec: concurrency.md §2; patch-grade plan C4
// status: PASS
// A late post-await failure rejects the outer Future rather than being
// mistaken for a nonterminal park.

const dependency = Future.new()
const result = Future.async || {
  dependency.await
  throw Error.new("late failure")
}
System.runScheduled()
System.print("pending: " + result.isReady.toString)
dependency.settleValue(None)
System.runScheduled()
System.print("ready: " + result.isReady.toString)
try {
  result.await
} catch e {
  System.print("caught: " + e.message)
}
