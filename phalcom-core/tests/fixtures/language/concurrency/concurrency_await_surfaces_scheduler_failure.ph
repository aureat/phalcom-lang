// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// Root await must surface the actual scheduled failure that made progress
// impossible, not replace it with the generic scheduler-empty diagnostic.
const pending = Future.new()
System.schedule || {
  Error.new("settler failed").raise()
}

try {
  pending.await
} catch e {
  System.print("caught: " + e.message)
}
