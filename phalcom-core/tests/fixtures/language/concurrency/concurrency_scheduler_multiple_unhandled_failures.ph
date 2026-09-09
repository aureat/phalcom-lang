// area: concurrency
// spec: concurrency.md; E010
// status: PASS

System.schedule || { Error.new("first detached failure").raise() }
System.schedule || { Error.new("second detached failure").raise() }
System.schedule || { System.print("third sibling ran") }

System.runScheduled()
