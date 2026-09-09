// area: concurrency
// spec: concurrency.md §2; patch-grade plan C4
// status: PASS
// Pending then/map/catch callbacks may park, return None or Error as data,
// adopt a nested Future, and fail after waking.

const source = Future.new()
const gate = Future.new()
const thenFuture = source.then |value| {
  System.print("then-start")
  gate.await
  "then-done"
}
const mapFuture = source.map |value| {
  System.print("map-start")
  gate.await
  None
}
const rejected = Future.new()
const catchFuture = rejected.catch |error| {
  System.print("catch-start")
  gate.await
  Error.new("data")
}
source.settleValue(3)
rejected.settleError(Error.new("source"))
System.runScheduled()
System.print("pending: " + thenFuture.isReady.toString + "," + mapFuture.isReady.toString + "," + catchFuture.isReady.toString)
gate.settleValue(1)
System.runScheduled()
System.print(thenFuture.await)
System.print(mapFuture.await)
System.print("catch-data: " + catchFuture.await.message)

const nestedSource = Future.new()
const nestedGate = Future.new()
const nested = nestedSource.then |value| {
  nestedGate.await
  Future.value("nested-done")
}
nestedSource.settleValue(4)
System.runScheduled()
System.print("nested-pending: " + nested.isReady.toString)
nestedGate.settleValue(5)
System.runScheduled()
System.print(nested.await)

const failSource = Future.new()
const failGate = Future.new()
const failed = failSource.then |value| {
  failGate.await
  throw Error.new("callback-failure")
}
failSource.settleValue(6)
System.runScheduled()
failGate.settleValue(7)
System.runScheduled()
try {
  failed.await
} catch e {
  System.print("callback-error: " + e.message)
}
