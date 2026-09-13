// area: concurrency
// spec: CONC002.C4.P1
// status: PASS

const p = Future.new()
System.print(p.isReady)
System.print(p.value)
System.print(p.outcome.isSome)

const ok = Future.value(42)
System.print(ok.isReady)
System.print(ok.value)
System.print(ok.outcome.unwrapOr(None).isOk)
System.print(ok.settleValue(999).value)

const err = Future.error(Error.new("boom"))
System.print(err.isReady)
System.print(err.value)
System.print(err.outcome.unwrapOr(None).isErr)
System.print(err.settleError(Error.new("other")).value)
