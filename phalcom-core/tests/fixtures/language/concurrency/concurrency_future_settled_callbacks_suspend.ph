// area: concurrency
// spec: concurrency.md §2
// Registration does not execute user code, even for a settled receiver.
const first = Future.new()
const second = Future.new()
let calls = 0
const a = Future.value(1).then |v| {
  calls = calls + 1
  first.await
  second.await
  Future.value(None)
}
const b = Future.value(2).map |v| {
  calls = calls + 1
  first.await
  second.await
  Error.new("data")
}
const c = Future.error(Error.new("source")).catch |e| {
  calls = calls + 1
  first.await
  second.await
  throw Error.new("late")
}
System.print(calls)
System.runScheduled()
System.print(calls)
System.print(a.isReady)
first.settleValue(())
System.runScheduled()
System.print(b.isReady)
second.settleValue(())
System.runScheduled()
System.print(a.await)
System.print(b.await.message)
try { c.await } catch e { System.print(e.message) }
const immediate = Future.value(1).map |v| { throw Error.new("immediate") }
System.print("registered")
System.runScheduled()
try { immediate.await } catch e { System.print(e.message) }
