// area: concurrency
// spec: concurrency.md §2
// Callback failure rejects the result, even when the source is already ready.
let mapped = Future.value(1).map |v| { v.frobnicate() }
let chained = Future.value(1).then |v| { v.frobnicate() }
System.runScheduled()
try { mapped.await } catch e { System.print(e.class.name) }
try { chained.await } catch e { System.print(e.class.name) }
