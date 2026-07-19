// area: concurrency
// spec: concurrency.md §2; ADR-0030
// status: PASS
// C-FUT-3: a `Future` settles exactly once. `Future.value(_)` already
// settles `self` `fulfilled`; a subsequent `settleValue`/`settleError` call
// (of either kind) is a no-op that leaves the first result untouched.

const f = Future.value(1)
f.settleValue(999)
f.settleError(Error.new())
System.print(f.value)

const g = Future.error(Error.new())
g.settleValue(7)
System.print(g.isReady)
System.print(g.value)
