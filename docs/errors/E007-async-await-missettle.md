# E007 · `Future.async` settles prematurely when its action `await`s — wrong value, silently

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`, isolated by control)
- **Severity:** **blocker** — the canonical `async { … await … }` composition returns garbage (`Some(None)`) with no error; wrong data propagates downstream
- **Subsystem:** core library (`Future.async`) × fiber resume semantics
- **Related:** [E004](E004-await-cannot-suspend.md) (this is the defect *behind* the one E004 fixed: awaiting now suspends, but the suspension is misread); `docs/spec/impl/reactor` completion-machinery spec (the structural fix's home)

## Defect

`Future.async(action)`'s driver (`phalcom-core/core/core.ph:1642-1655`) runs the action on a
fresh fiber and treats the **first return** of `fib.try()` as the fiber's **completion**:

```phalcom
const fib = Fiber.new(action)
const res = fib.try()          // returns on the fiber's FIRST yield, not on completion
if (fib.error.isSome) { … } else { f.settleValue(res) }
```

But `Fiber#try` has resume semantics (`concurrency.md` §interface: "returns the next
*yielded or returned* value"). When `action` awaits a pending future, `await`'s non-root
branch (`core.ph:1629-1632`) registers the fiber as a waiter and executes a bare
`Fiber.yield(None)` — so `fib.try()` returns `None` while `fib` is merely *parked*. The
driver settles the outer future with `None`; the settle-once guard (`core.ph:1544-1557`)
then permanently blocks the real result.

The real result is not merely late — it is **delivered to the wrong fiber and discarded**.
When the inner future settles, `drain()` (`core.ph:1579-1587`) re-enqueues `fib` on the
ready queue; whichever fiber pumps the queue becomes `fib`'s *new* resumer, because
`fiber_resume` reassigns `resumer` on every resume
(`phalcom-core/src/primitive/fiber.rs:326`). `fib`'s completion value lands at that pump
site's `try()` expression and is thrown away.

## Repro (observed 2026-07-20)

```phalcom
const inner = Future.new()
const outer = Future.async {
  const v = inner.await
  v + 100
}
System.runScheduled()
System.print("outer settled while inner pending: \(outer.isReady)")
inner.settleValue(1)
System.runScheduled()
System.print("outer.value: \(outer.value)")
```

Output:

```
outer settled while inner pending: true
outer.value: Some(None)
```

Expected: `false`, then `Some(101)`.

## Control

A non-suspending action settles correctly — `Future.async { 42 }` yields `Some(42)`
(covered by `tests/lang/concurrency/concurrency_future_async_await.ph`). The defect is
strictly in the *suspending* composition, which no fixture exercises.

## Fix direction (unverified — reproduce-then-re-derive applies)

The driver's premise ("one `try()` = one lifetime") is false whenever the action can
suspend. Candidate repairs, none verified:

1. **Completion machinery** (the reactor spec's direction): fiber completion notifies a
   registered continuation instead of "whoever resumed me last". Makes the driver loop
   unnecessary and fixes the resumer-reassignment leak for every future composition, not
   just `async`.
2. Driver loops `while (not fib.isDone)` re-resuming — but this alone is wrong: after
   `drain()` re-enqueues `fib`, both the driver *and* the pump would race to resume it,
   and the driver has no way to park until `fib` is ready without itself awaiting.

Option 2's inadequacy is the instructive part: any fix that keeps completion delivery
keyed to the *dynamic resumer* re-creates the leak somewhere else.
