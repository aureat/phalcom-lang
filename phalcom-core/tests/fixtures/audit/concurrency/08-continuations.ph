let source = Future.new()
let gate = Future.new()
let t = source.then |v| { gate.await; "then final" }
let m = source.map |v| { gate.await; "map final" }
let rejected = Future.new()
let c = rejected.catch |e| { gate.await; "catch final" }
source.settleValue(1)
rejected.settleError(Error.new("input"))
System.runScheduled()
System.print(t.value)
System.print(m.value)
System.print(c.value)
gate.settleValue(2)
System.runScheduled()
System.print(t.value)
System.print(m.value)
System.print(c.value)
