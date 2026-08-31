// area: concurrency
// spec: concurrency.md; ADR-0030
// status: PASS

// E004 regression. Before the fix, NO test in the corpus had a fiber await a
// pending future and later resume — every `await` in the suite ran on the root
// fiber (which takes the pump branch and never yields) or deliberately under a
// native frame (which is expected to raise). The feature's central operation
// was uncovered, and it did not work.

// E004(a): a non-root fiber awaiting a pending future must PARK, not fail.
const f = Future.new()
const worker = Fiber.new || {
  System.print("worker: awaiting")
  System.print("worker: got " + f.await.toString)
  "worker done"
}
System.schedule(worker)
System.runScheduled()
System.print("parked, not failed: isDone = " + worker.isDone.toString)
System.print("no error: " + worker.error.toString)

// ...and must resume with the settled value once someone settles it.
f.settleValue(42)
System.runScheduled()
System.print("after settle: isDone = " + worker.isDone.toString)

// E004(c): a waiter that died between registering and settling must not take
// the future's other waiters down with it. `doomed` awaits from inside an
// `ensure` block — a real native frame, so it correctly raises and dies with
// its registration still in `_waiters`. The healthy block waiter must still run.
const g = Future.new()
const doomed = Fiber.new || {
  try { } ensure { g.await }
}
doomed.try()
System.print("doomed died: " + doomed.isDone.toString)
g.then |v| { System.print("healthy waiter still ran with " + v.toString) }
g.settleValue(7)
System.runScheduled()

// E004(b): awaiting a future nothing can settle reports it instead of spinning
// forever in silence. On the root fiber, an empty ready queue plus a pending
// receiver means no progress is possible.
const stuck = Future.new()
try {
  stuck.await
} catch e {
  System.print("caught: " + e.message)
}

// The guard itself must remain intact: a yield under a native frame is still
// refused. The fix removed `await`'s self-inflicted frame, not the rule.
const guarded = Fiber.new || { || { Fiber.yield(None) }.attempt() }
System.schedule(guarded)
System.runScheduled()
System.print("guard intact: " + guarded.isDone.toString)
