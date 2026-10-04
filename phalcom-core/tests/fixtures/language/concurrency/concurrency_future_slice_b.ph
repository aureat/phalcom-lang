// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS

// C-FUT-1: value/error await
let f1 = Future.value("ok")
System.print(f1.await)

let err = Error.new("rejected")
let f2 = Future.error(err)
try {
  f2.await
} catch e {
  System.print("caught: " + e.message)
}

// C-FUT-2: async/await on the ROOT fiber, which cannot yield and so drives the
// scheduler itself rather than suspending. The genuinely-suspending case — a
// non-root fiber that parks on a pending future and resumes — is
// `concurrency_future_await_suspends.ph` (E004); until that landed, this case
// was labelled "suspending" while exercising only the fallback.
let f3 = Future.async || {
  System.print("async running")
  "async result"
}
System.print(f3.await)

// C-FUT-4: pending then/map/catch settlement.
//
// Re-blessed with E004's fix: the root-fiber `await` pump now stops as soon as
// its OWN future settles, instead of draining the whole ready queue every
// iteration (it used to call `System.runScheduled`, which pumps to exhaustion).
// Same callbacks and results. Terminal observer handoffs are scheduler work,
// so both callback actions may begin before either derived Future is settled.
let f4 = Future.new()
let f5 = f4.then |v| {
  System.print("then run: " + v)
  Future.value("then result")
}
let f6 = f4.map |v| {
  System.print("map run: " + v)
  "map result"
}
let f7 = f4.catch |e| {
  System.print("catch run: " + e.message)
  "catch result"
}

f4.settleValue("settled")
System.print(f5.await)
System.print(f6.await)
System.print(f7.await) // should pass through fulfilled value without running catch

let f8 = Future.new()
let f9 = f8.catch |e| {
  System.print("catch run error: " + e.message)
  "catch recovered"
}
f8.settleError(Error.new("failed"))
System.print(f9.await)

// C-FUT-8: callbacks returning a Future are assimilated, rather than wrapped
// as a Future value. This exercises fulfilled, pending, and rejected paths.
System.print(Future.value("then").then |v| { Future.value(v + " flattened") }.await)
System.print(Future.value("map").then |v| { Future.value(v + " flattened") }.await)
System.print(Future.error(Error.new("catch")).recoverWith |e| { Future.value(e.message + " flattened") }.await)

// C-FUT-7: await under native frame raises CannotYieldAcrossNativeFrame
class Probe {
  hash {
    f10.await
    return 0
  }
}
let f10 = Future.new()
let helper = Fiber.new || {
  let mut s = Set.new()
  s.add(Probe.new())
}
helper.try()
System.print("caught yield across native frame: " + helper.error.unwrapOr(None).message)
