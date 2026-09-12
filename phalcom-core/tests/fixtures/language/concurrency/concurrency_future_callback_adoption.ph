// area: concurrency
// spec: concurrency.md §2
const inner = Future.new()
const result = Future.value(1).then |v| { inner }
System.runScheduled()
System.print(result.isReady)
inner.settleValue(None)
System.runScheduled()
System.print(result.await)
const rejected = Future.new()
const failed = Future.value(1).then |v| { rejected }
System.runScheduled()
rejected.settleError(Error.new("nested"))
System.runScheduled()
try { failed.await } catch e { System.print(e.message) }
let cycle = Future.new()
cycle = Future.value(1).then |v| { cycle }
System.runScheduled()
try { cycle.await } catch e { System.print(e.message) }
// Unit and Error are ordinary successful callback results.
System.print(Future.value(1).map |v| { () }.await)
System.print(Future.error(Error.new("source")).catch |e| { Error.new("data") }.await.message)
