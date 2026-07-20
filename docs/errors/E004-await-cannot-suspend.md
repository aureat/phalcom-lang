# E004 · `Future#await` can never suspend a fiber; its own wrapper trips the restricted-yield guard

- **Status:** **FIXED** at `f479189` — verified 2026-07-19: all three repros below now behave
  correctly, both controls still hold, the full suite is green from a clean checkout, and the
  previously-missing coverage landed as
  `phalcom-core/tests/lang/concurrency/concurrency_future_await_suspends.ph`. See
  [The fix](#the-fix) for what changed and why the obvious repair was not the one taken.
- *Originally:* OPEN — confirmed 2026-07-19 (reproduced under `target/debug/phalcom`, isolated by control)
- **Severity:** blocker — the feature's central operation could not execute; two secondary failure modes (silent hang, cross-waiter corruption)
- **Subsystem:** core library (`Future`) × fibers / restricted-yield guard
- **Related:** [E002](E002-fiber-floor-upvalue-crash.md), [E001](E001-gc-ensure-temp-root-uaf.md) — same recurring shape: a participant removed from the machinery on one exit path and not the other. Narrative: [`docs/learn/concurrency/future-await.md`](../learn/concurrency/future-await.md).

## Defect

`Future#await` (`phalcom-core/core/core.ph:1424-1444`) probes whether it may suspend by running

```phalcom
const res = { Fiber.yield(None) }.attempt()
```

`.attempt()` is **not** native — it is a Phalcom method (`core.ph:627-629`) expanding to
`{ Ok.new(self.call()) }.on(Error) { e => Err.new(e) }`. Both `.on(_)(_)` (`block_on`) and
`self.call()` (`block_call`) re-enter the interpreter through `phalcom-core/src/primitive/block.rs:158-160`,
each incrementing `native_reentry_depth`.

The guard (`phalcom-core/src/primitive/fiber.rs:338`) refuses a yield when
`native_reentry_depth != fiber.floor_depth`, where `floor_depth` is recorded at resume
(`fiber.rs:317`). The probe therefore runs at `floor_depth + 2` and **fails unconditionally**, for
every fiber, in every program. There is no depth at which `floor_depth + 2 == floor_depth`.

Three consequences:

**(a) Non-root `await` kills the awaiting fiber.** The guard error is typed, so `await` takes its
`isA(CannotYieldAcrossNativeFrame)` branch (`core.ph:1430-1432`) and re-raises. Per the fiber-floor
capture, the fiber ends `Failed`.

**(b) Root `await` busy-spins with no diagnosis.** `fiber_yield` checks root-ness *first*
(`fiber.rs:336`) and returns an **untyped** `RuntimeError::NotAllowed`, so `isA(…)` is false and
control reaches `while (not self.isReady) { System.runScheduled() }` (`core.ph:1435-1437`). If nothing
in the ready queue will settle the future, this loops forever over an empty queue — no error, no
quiescence check. Note this branch is selected by the *ordering* of two guard clauses in `fiber_yield`,
not by any positive test for root-ness.

**(c) The dead fiber stays registered as a waiter.** Only the root branch filters `Fiber.current` out
of `_waiters` (`core.ph:1434`); branch (a) re-raises without unregistering. A later `settleValue`
drains the failed fiber into `System.schedule` (`core.ph:1410-1411`), and the pump then attempts to
resume it — killing the whole run, including healthy waiters registered on the same future.

## Reproduction

All three under `target/debug/phalcom`.

```phalcom
// (a) non-root await on a pending future — fiber fails instead of parking
const f = Future.new()
const w = Fiber.new { f.await }
System.schedule(w)
System.runScheduled()
System.print("w isDone = " + w.isDone.toString)   // -> true
System.print("w error  = " + w.error.toString)    // -> Some(<CannotYieldAcrossNativeFrame>)
```

```phalcom
// (b) root await on a future nothing will settle — hangs, no output, no error
const f = Future.new()
System.print("about to await a future with no settler")
System.print(f.await.toString)                    // never reached; spins at 100% CPU
```

```phalcom
// (c) the corpse in the waiter list takes down an unrelated waiter
const f = Future.new()
const w = Fiber.new { f.await }
System.schedule(w)
System.runScheduled()                             // w fails, stays in _waiters
f.then { v => System.print("block waiter ran with " + v.toString) }
System.print("waiters registered; settling now")
f.settleValue(9)
System.runScheduled()
// -> waiters registered; settling now
// -> cannot resume a finished fiber          (the block waiter never runs)
```

**Controls** (isolate the wrapper, not the yield, as the cause):

```phalcom
// bare yield in the same position parks correctly
const w = Fiber.new { Fiber.yield(None); System.print("resumed") }
System.schedule(w)
System.runScheduled()
System.print(w.isDone.toString)                   // -> false   (parked, healthy)
```

```phalcom
// the same yield under .attempt() does not
const w = Fiber.new { System.print({ Fiber.yield(None) }.attempt().toString) }
System.schedule(w)
System.runScheduled()                             // -> Err(<CannotYieldAcrossNativeFrame>)
```

```phalcom
// the root refusal is untyped — the sole basis for branch (b)
System.print({ Fiber.yield(None) }.attempt().toString)   // root -> Err(<Error>)
```

## Why the suite was green

`phalcom-core/tests/lang/concurrency/concurrency_future_slice_b.ph` is the feature's acceptance test
and calls `await` twelve times. Every call is on the **root** fiber (branch (b), with a queue that does
settle the future) except one, which is deliberately inside an `ensure` block and *asserts*
`CannotYieldAcrossNativeFrame` as the expected result (C-FUT-7). That assertion is correct on its own
terms — an `await` inside `ensure` really does cross a native frame — but it fixes the observation to a
cause that is not the only cause, so the identical failure with no user-supplied native frame reads as
already-specified behaviour.

The case labelled `C-FUT-2: async/await suspending` awaits at root and suspends nothing.
**No test in the corpus has a fiber await a pending future and later resume.**

Also stale, found in the same pass:
`phalcom-core/tests/lang/concurrency/concurrency_future_async_await.ph` carries a `status: PENDING`
header while living in the passing directory, so it runs as a green test.

## Doc debt (fix in the same change, per the ADR/STATUS two-way-sync rule)

Slice B landed in `06432bd` (2026-07-14). Three records still describe it as unbuilt:

- `docs/spec/v0.2/concurrency.md:187` — `await` status `B`, not landed.
- `docs/forge/units/U-FUTURE/plan.md:109-110` — `async(_)`/`await` "**B (DEFERRED → DEC-FUT-SCHED)**".
- `phalcom-core/core/core.ph:1335-1338` — the `Future` class doc comment says `async(_)`/`await` are
  "deliberately NOT built here", eleven lines above their implementations.

## The fix

Landed `f479189`. The three consequences were independent and each needed its own repair.

**(a) — a predicate, not a probe.** Added `Fiber#isRoot`
(`phalcom-core/src/primitive/fiber.rs`), the predicate form of `fiber_yield`'s root refusal:
`vm.heap.fiber(fiber_ref).resumer.is_none()`. `await` now *asks* which branch it is on instead of
attempting a yield and reading the wreckage, and the `Fiber.yield` is **bare** — no wrapper, so
nothing between the fiber floor and the switch. A comment at the call site says why, because the
failure mode is invisible: any future edit that wraps that yield in `.attempt()`, `.on(_)`, or
`ensure` silently reinstates the whole bug.

This is a **floor amendment** (136 → 137 bindings), recorded with its rationale in
`phalcom-core/tests/invariants.rs`. Justified against the ADR-0019 freeze on the ground that no
arrangement of library code can observe root-ness without it: attempt-and-inspect is unfixable in
`.ph` when the attempt changes the answer.

**(b) — quiescence check.** The root pump now pops one entry at a time via `System.nextScheduled`
and raises when the queue drains while the receiver is still pending, instead of calling
`System.runScheduled` in a loop that spins on an empty queue. Side effect: `await` now returns as
soon as *its own* future settles rather than draining the whole queue first, which reordered two
lines of `concurrency_future_slice_b`'s golden — same twelve lines, re-blessed with the reason
recorded in the fixture.

**(c) — guarded at `drain`, not at `await`.** The §Defect text and the original fix sketch both said
to unregister `Fiber.current` from `_waiters` on `await`'s raising branch. **That was the wrong
place.** With a bare yield there is no way to catch the raise without reintroducing a native frame —
the same catch-22 as (a). `drain` instead skips waiters that are finished fibers. This is also
strictly more robust: it covers a waiter that dies for reasons having nothing to do with `await`,
which the unregister-on-error-path repair would have missed.

Point (c) is why the standing rule exists. The diagnosis was reproduced and correct; the prescription
derived from it was not implementable, and following it would have re-broken (a).

**Coverage.** `concurrency_future_await_suspends.ph` asserts all three fixes plus the two controls,
including that the restricted-yield guard still refuses a wrapped yield — the fix removed `await`'s
self-inflicted native frame, not the rule. The three stale records listed above were corrected in the
same change.
