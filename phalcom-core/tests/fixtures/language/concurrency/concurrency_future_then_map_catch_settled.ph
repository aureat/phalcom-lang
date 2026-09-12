// area: concurrency
// spec: concurrency.md §2
// Matching callbacks are asynchronous; unmatched outcomes pass through.
const a = Future.value(10).then |v| { Future.value(v + 1) }
System.runScheduled()
System.print(a.value)
System.print(Future.error(Error.new()).then |v| { Future.value(v + 1) }.isReady)
const b = Future.value(10).map |v| { v * 2 }
System.runScheduled()
System.print(b.value)
System.print(Future.error(Error.new()).map |v| { v * 2 }.isReady)
System.print(Future.value(10).catch |e| { 0 }.value)
const c = Future.error(Error.new()).catch |e| { 99 }
System.runScheduled()
System.print(c.value)
