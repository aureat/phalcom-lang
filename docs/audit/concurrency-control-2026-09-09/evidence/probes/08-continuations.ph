const source = Future.new()
const gate = Future.new()
const t = source.then |v| { gate.await; "then final" }
const m = source.map |v| { gate.await; "map final" }
const rejected = Future.new()
const c = rejected.catch |e| { gate.await; "catch final" }
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
