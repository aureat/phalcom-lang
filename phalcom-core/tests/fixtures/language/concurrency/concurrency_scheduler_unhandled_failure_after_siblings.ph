// area: concurrency
// spec: concurrency.md; E010
// status: PASS

// An unowned scheduled failure is isolated until the current pump has drained
// unrelated ready work, then reported without turning the pump call into an
// exception. A second drain proves the report is consumed exactly once.
System.schedule || {
  System.print("failing task")
  Error.new("scheduled boom").raise()
}

System.schedule || {
  System.print("sibling survived")
}

System.runScheduled()
System.print("after first drain")
System.runScheduled()
System.print("after second drain")
