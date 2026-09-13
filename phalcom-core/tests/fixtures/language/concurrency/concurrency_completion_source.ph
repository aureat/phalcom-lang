// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

const cs = CompletionSource.new()
System.print(cs.future == cs.future)
System.print(cs.future.isReady)

System.print(cs.tryResolve("first"))
System.print(cs.tryResolve("second"))
System.print(cs.tryReject(Error.new("error")))
System.print(cs.future.await)

const cs2 = CompletionSource.new()
System.print(cs2.tryReject(Error.new("failed")))
System.print(cs2.tryReject(Error.new("second")))
System.print(cs2.tryResolve("ok"))
System.print(cs2.future.isReady)
System.print(cs2.future.outcome.unwrapOr(None).isErr)
