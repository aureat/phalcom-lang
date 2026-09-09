// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// An older detached failure is reported by the ordinary scheduler boundary and
// must not be reused as causal context for a later await window.
System.schedule || { Error.new("old detached failure").raise() }
System.runScheduled()

const pending = Future.new()
try {
  pending.await
} catch e {
  System.print("caught: " + e.message)
}
