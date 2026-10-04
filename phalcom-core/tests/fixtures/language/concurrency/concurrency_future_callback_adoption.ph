// area: concurrency
// spec: concurrency.md §2
let inner = Future.new()
let result = Future.value(1).then |v| { inner }
System.runScheduled()
System.print(result.isReady)
inner.settleValue(None)
System.runScheduled()
System.print(result.await)
let rejected = Future.new()
let failed = Future.value(1).then |v| { rejected }
System.runScheduled()
rejected.settleError(Error.new("nested"))
System.runScheduled()
try { failed.await } catch e { System.print(e.message) }
let mut cycle = Future.new()
cycle = Future.value(1).then |v| { cycle }
System.runScheduled()
try { cycle.await } catch e { System.print(e.message) }
// Unit and Error are ordinary successful callback results.
System.print(Future.value(1).map |v| { () }.await)
System.print(Future.error(Error.new("source")).catch |e| { Error.new("data") }.await.message)
