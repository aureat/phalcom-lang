// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// An unowned scheduled failure is isolated until the current pump has drained
// unrelated ready work, then surfaced to the pump caller.
System.schedule || {
  System.print("failing task")
  Error.new("scheduled boom").raise()
}

System.schedule || {
  System.print("sibling survived")
}

try {
  System.runScheduled()
} catch e {
  System.print("caught: " + e.message)
}
