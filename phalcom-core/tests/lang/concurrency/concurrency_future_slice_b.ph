// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS

// C-FUT-1: value/error await
const f1 = Future.value("ok")
System.print(f1.await)

const err = Error.new("rejected")
const f2 = Future.error(err)
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
const f3 = Future.async {
  System.print("async running")
  "async result"
}
System.print(f3.await)

// C-FUT-4: pending then/map/catch settlement.
//
// Re-blessed with E004's fix: the root-fiber `await` pump now stops as soon as
// its OWN future settles, instead of draining the whole ready queue every
// iteration (it used to call `System.runScheduled`, which pumps to exhaustion).
// Same twelve lines, same callbacks, same results — only the interleaving of
// `map run` and `then result` swaps, because `f5.await` no longer runs f6's
// waiter before returning. Nothing is dropped; the remaining waiters run on the
// next pump.
const f4 = Future.new()
const f5 = f4.then |v| {
  System.print("then run: " + v)
  "then result"
}
const f6 = f4.map |v| {
  System.print("map run: " + v)
  "map result"
}
const f7 = f4.catch |e| {
  System.print("catch run: " + e.message)
  "catch result"
}

f4.settleValue("settled")
System.print(f5.await)
System.print(f6.await)
System.print(f7.await) // should pass through fulfilled value without running catch

const f8 = Future.new()
const f9 = f8.catch |e| {
  System.print("catch run error: " + e.message)
  "catch recovered"
}
f8.settleError(Error.new("failed"))
System.print(f9.await)

// C-FUT-8: callbacks returning a Future are assimilated, rather than wrapped
// as a Future value. This exercises fulfilled, pending, and rejected paths.
System.print(Future.value("then").then |v| { Future.value(v + " flattened") }.await)
System.print(Future.value("map").map |v| { Future.value(v + " flattened") }.await)
System.print(Future.error(Error.new("catch")).catch |e| { Future.value(e.message + " flattened") }.await)

// C-FUT-7: await under native frame raises CannotYieldAcrossNativeFrame
const f10 = Future.new()
const helper = Fiber.new {
  try {
    // some body
  } ensure {
    f10.await
  }
}
helper.try()
System.print("caught yield across native frame: " + helper.error.unwrapOr(None).message)
