// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// A scheduler failure already owned by a durable Future completion observer
// must become a rejected Future only; it must not also leak into the pump's
// unhandled-failure channel.
const owned = Future.async || {
  Error.new("owned failure").raise()
}

try {
  owned.await
} catch e {
  System.print("future: " + e.message)
}

System.runScheduled()
System.print("after pump")
