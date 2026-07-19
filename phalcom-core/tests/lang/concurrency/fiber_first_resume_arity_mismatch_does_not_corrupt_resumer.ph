// area: concurrency
// spec: concurrency.md; ADR-0030 §3/§4
// status: PASS
// Regression: `fiber_resume` (fiber.rs) used to steal the calling fiber's
// live stacks (`store_live_into`) *before* validating a not-yet-started
// callee's entry arity, and (independently) the arity error always named
// the signature "call" even when raised from `try()`. Neither defect was
// externally observable in this exact shape — `outer` is unconditionally
// marked `Failed` by the same fiber-floor cascade either way, discarding
// its state before anything reads it — but the ordering is still the wrong
// invariant (validate before mutating shared VM/heap state, not after) and
// the message text was a genuine, user-visible bug. Locks both: `root`
// (which resumed `outer` via `try()`) recovers the captured `Error` and
// keeps running, and the message correctly says "try" as the signature.

const inner = Fiber.new { x => x }
const outer = Fiber.new {
  inner.call()
  System.print("unreachable: outer body continues past inner.call")
}
const r = outer.try()
System.print(r.class.name)
System.print(r.message)
System.print("root continues")
