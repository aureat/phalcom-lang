// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// Root await keeps its quiescence diagnostic, but includes the unhandled
// scheduler failures produced while this await itself drove the scheduler.
const pending = Future.new()
System.schedule || {
  Error.new("settler failed").raise()
}

try {
  pending.await
} catch e {
  System.print("caught: " + e.message)
}
