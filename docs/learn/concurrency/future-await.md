# C4 — Futures that cannot wait

*Concurrency track, doc 4 of 4 — the last one. Read [C1 — the restricted loop](restricted-loop.md),
[C2 — the parked fiber](parked-fiber.md), and [C3 — when a fiber fails](fiber-failure.md) first. This
doc uses their vocabulary — the floor, the park, the native-frame guard, the failure capture — and
re-derives none of it.*

---

## The grip

> **`Future#await` is the only method in the core library that makes its own precondition fail.**
> To find out whether it is allowed to suspend, it attempts a yield inside `{ … }.attempt()`. But
> `.attempt()` is two nested native re-entrant frames — so the yield it is testing is a yield the
> wrapper has already made illegal. The probe *is* the obstruction.
>
> The two ways that failure is read are both wrong. On the root fiber the refusal arrives untyped, is
> misread as "nothing to wait on yet," and degrades to a busy spin. Off the root fiber it arrives
> typed, is read correctly as "you are under a native frame," and kills the awaiting fiber — never
> mentioning that the native frame is `await`'s own.
>
> `await` therefore never parks a fiber. Not on any path. The half of `Future` that would make it a
> concurrency primitive has shipped, is green in CI, and has never run.

C1 established that a fiber switch is cheap *because* it is illegal under a native frame — one fact
read from two directions. This doc is what happens when a library method built in Phalcom forgets
which direction it is standing in.

---

## The debt this pays, and the borders

C1 handed over `System.schedule(_)` and the ready queue as "named, not explained." C2 handed over the
seam where a fiber's `floor_depth` is written. C3 closed with the pump, the root-drive, and the
question of *how a scheduled fiber's failure reaches the host* — and answered the fiber half of it.
This doc answers the library half.

What stays with its owner: the guard's rationale, `switch_pending`, and the primitive-return branch
are [C1's](restricted-loop.md). The four-of-twelve field partition is [C2's](parked-fiber.md). The
fiber-floor `Err` arm, `capture_error_value`, and the `call`/`try` cascade are
[C3's](fiber-failure.md). All four are used here as known words.

---

## First: the plan for this document was wrong, and so is the code's own comment

The track plan describes C4 as a doc about an *unbuilt* feature — `Future` reduced to its
"scheduler-free half," `async(_)`/`await` deferred to a Slice B "gated on DEC-FUT-SCHED," `_waiters`
"always empty." It even warned the doc might be too thin to exist.

Slice B landed on 2026-07-14 in `06432bd`. `await` is `core.ph:1424`. `static async(_)` is
`core.ph:1448`. `drain` is `core.ph:1409`. Three records still say otherwise:

| Record | What it says | Line |
|---|---|---|
| `docs/spec/v0.2/concurrency.md` | `await` status **B**, not landed | `:187` |
| `docs/forge/units/U-FUTURE/plan.md` | `async(_)`, `await` — **B (DEFERRED → DEC-FUT-SCHED)** | `:109-110` |
| **`core.ph` itself** | "`async(_)`/`await` … neither of which is landed; that is Slice B … **deliberately NOT built here**" | `:1335-1338` |

The third is the one to sit with. That sentence is a class doc comment, and the class it documents
implements both methods eleven lines further down. Nothing here is unusual or careless: a comment
written to describe a *plan* was correct on the day it was written, the plan changed, and the comment
did not. This is the ordinary way a record goes stale — not by being wrong, but by being outlived.
The rule that catches it is the one this course keeps arriving at: **the tree is the only witness.**

So this is not a doc about a feature that was not built. It is a doc about a feature that was built,
passes its tests, and does not work.

---

## Predict before you read

Two programs. They differ in one thing: *who* awaits.

```phalcom
// A — the root fiber awaits
const f = Future.async { 40 + 2 }
System.print("isReady = " + f.isReady.toString)
System.print(f.await.toString)
```

```phalcom
// B — a scheduled fiber awaits the same kind of future
const f = Future.new()
const w = Fiber.new { f.await }
System.schedule(w)
System.runScheduled()
System.print("w isDone = " + w.isDone.toString)
System.print("w error  = " + w.error.toString)
```

Program A prints what you expect:

```
isReady = false
42
```

The future is genuinely pending when `await` is called — `isReady` says so — and `await` returns the
settled value. That is a future working.

Now B. The natural prediction is *the worker parks and stays parked* — nothing settles `f`, so
`isDone` is `false` and `error` is `None`. That prediction is worth making, because it is the one the
method's name licenses, and because it is wrong in a direction you would not guess:

```
w isDone = true
w error  = Some(<CannotYieldAcrossNativeFrame>)
```

The worker did not park. It **failed**, with C1's guard error — the error you get for yielding out of
`.each { … }`. There is no `.each` here. There is no user-supplied native frame anywhere in program B.

---

## What `Future` actually is

Strip the concurrency vocabulary and `Future` is small. It is a plain Phalcom class
(`core.ph:1346`) — not a native object, no `primitive/future.rs`, no VM support of any kind. Three
fields:

| Field | Holds | Line |
|---|---|---|
| `_state` | one of the **strings** `"pending"` / `"fulfilled"` / `"rejected"` | `:1349` |
| `_value` | the settled value, or the captured `Error` | `:1350` |
| `_waiters` | a `List` of things to schedule on settle | `:1351` |

`settleValue`/`settleError` (`:1383`, `:1397`) are a settle-once latch: if `isReady`, return `self`
unchanged; otherwise write the state, write the value, and `drain`. `drain` (`:1409`) is three lines —
push every waiter into `System.schedule(_)`, then empty the list.

That `Future` has no privileged access is the load-bearing fact of this whole document. It reaches the
scheduler through the same two seams `.ph` user code has — `System.schedule(_)` and `Fiber.yield` —
and it is subject to the guard exactly as user code is. **A library written in the language does not
get an exemption from the language's rules.** That is normally the point of writing the library in the
language. Here it is the bug.

One nice consequence, before the bug. `_waiters` holds two different kinds of thing: `await` registers
a **`Fiber`** (`:1426`), while `then`/`map`/`catch` register a **`Block`** (`:1475`, `:1502`, `:1530`).
`drain` treats them identically, and that works because `System.schedule(_)` type-switches on arrival:

```rust
// phalcom-core/src/primitive/system.rs — system_schedule
let fiber_ref = match args[0] {
    Value::Obj(id) if matches!(vm.heap.get(id), crate::heap::Object::Fiber(_)) => id,
    _ => crate::primitive::fiber::new_fiber_ref(vm, args[0])?,
};
vm.ready_queue.push_back(fiber_ref);
```

Already a fiber? Enqueue it. Anything else? Wrap it. The queue holds exactly one kind of thing, so the
waiter list is allowed to hold two. This is a genuinely good seam, and it is why `then` works at all.

---

## The wrapper that defeats itself

Here is `await`, edited only for length:

```phalcom
await {
  if (not self.isReady) {
    _waiters.add(Fiber.current)
    const res = { Fiber.yield(None) }.attempt()      // <- the probe
    if (res.isErr) {
      const err = res.unwrapErr
      if (err.isA(CannotYieldAcrossNativeFrame)) {
        return err.raise()                            // <- branch 1
      }
      // Yield failed because we are on the root fiber!
      _waiters = _waiters.filter { w => w != Fiber.current }
      while (not self.isReady) { System.runScheduled() }   // <- branch 2
    }
  }
  if (_state == "rejected") { return _value.raise() }
  return _value
}
```

The design is legible and, read on its own, sensible: *try to yield; if the yield is refused because
we are the root fiber, fall back to pumping the scheduler ourselves.* The root fiber genuinely cannot
yield — it has no resumer — so some fallback is required, and a pump is the obvious one.

The problem is the word `.attempt()`. It is not a native primitive. It is a two-line Phalcom method on
`Function`:

```phalcom
// core.ph:627-629
attempt() {
  return { Ok.new(self.call()) }.on(Error) { e => Err.new(e) }
}
```

`.on(_)(_)` is `block_on`; `self.call()` is `block_call`. Both are native, and both re-enter the
interpreter through the same three lines:

```rust
// phalcom-core/src/primitive/block.rs:158-160
vm.native_reentry_depth += 1;
let result = vm.run_until(base_frames);
vm.native_reentry_depth -= 1;
```

So by the time `Fiber.yield` is reached, `native_reentry_depth` has been incremented **twice** — once
by `.on`'s driving call, once by the explicit `self.call()`. And the guard is not a test against zero:

```rust
// phalcom-core/src/primitive/fiber.rs:336-339
let Some(resumer) = vm.heap.fiber(me).resumer else {
    return Err(RuntimeError::NotAllowed("cannot yield the root fiber".to_string()).into());
};
if vm.native_reentry_depth != vm.heap.fiber(me).floor_depth {
    return Err(cannot_yield_across_native_frame(vm));
}
```

It is a test against `floor_depth` — the depth recorded on the fiber *at the moment it was resumed*
(`fiber.rs:317`). C2 explained why: a fiber may legitimately be resumed from inside a native frame, so
"legal to yield" means *back at the depth you were resumed at*, not *at zero*. That relative rule is
correct, and it is what makes the wrapper fatal. There is no depth at which `floor_depth + 2` equals
`floor_depth`. The probe fails **unconditionally**, for every fiber, in every program, forever.

The control makes it a one-line difference. Same fiber, same position, same scheduler:

```phalcom
Fiber.new { Fiber.yield(None); … }              // parks:  isDone = false, error = None
Fiber.new { { Fiber.yield(None) }.attempt() }   // dies:   Err(<CannotYieldAcrossNativeFrame>)
```

The yield is fine. The wrapper around the yield is the whole defect.

---

## Two refusals, two misreadings

`fiber_yield` refuses in two places, and — this is the detail everything turns on — **in this order**:
root first, guard second. The two refusals do not have the same type.

| Awaiter | Which refusal fires | Type | `isA(CannotYield…)` | Branch taken | Result |
|---|---|---|---|---|---|
| **root fiber** | `:336` root check | untyped `RuntimeError::NotAllowed` | `false` | 2 — pump | busy spin |
| **any other fiber** | `:339` depth guard | typed `CannotYieldAcrossNativeFrame` | `true` | 1 — re-raise | fiber dies |

The root fiber never reaches the depth guard, because the root check precedes it. So `await`'s comment
— *"Yield failed because we are on the root fiber!"* — is accidentally correct: branch 2 really is the
root-fiber branch. But it is correct for the wrong reason. It is not that the root case was
distinguished; it is that the root check happens to come first and happens to return a different type.
Reorder those two `if`s in `fiber_yield` — a change with no other observable effect, the kind of
tidying a reviewer would wave through — and `await`'s root path silently switches to branch 1. The
library's control flow depends on the ordering of two guard clauses in a Rust function that has no
idea it is being read as a signal.

Verify the types directly. Same expression, two receivers:

```phalcom
// root fiber
const r = { Fiber.yield(None) }.attempt()
System.print("root attempt gave " + r.toString)     // -> Err(<Error>)
```

```phalcom
// scheduled fiber
System.print("attempt gave " + r.toString)          // -> Err(<CannotYieldAcrossNativeFrame>)
```

`Err(<Error>)` — untyped, generic. That is the whole basis on which `await` decides it is the root
fiber: **not a check, an absence.** Anything that fails to be a `CannotYieldAcrossNativeFrame` is
treated as proof of root-ness.

### Branch 2: the spin

The root path "works" in the demo and hangs in the real case, for the same reason. Pump until settled:

```phalcom
while (not self.isReady) { System.runScheduled() }
```

If something in the queue will eventually settle the future, this terminates and `await` returns the
value — that is program A, and every `await` in the test suite. If nothing will, `System.runScheduled`
drains an empty queue, returns, and is called again:

```phalcom
const f = Future.new()             // nobody will ever settle this
System.print("about to await a future with no settler")
System.print(f.await.toString)
```

```
about to await a future with no settler
```

…and then nothing. No error, no deadlock detection, no progress — a hot loop over an empty queue. The
program above was killed by an external alarm after six seconds. A future nobody settles is the most
ordinary bug in asynchronous code, and Phalcom's response to it is to spin a core at 100% in silence.

Worth naming precisely: this is **not** the guard's fault and not the wrapper's. Branch 2 would still
spin if the root fiber's refusal were detected properly. It is a third, independent gap — no
quiescence check. The pump has all the information it needs (the queue is empty and `self` is still
pending, so no progress is possible) and does not look at it.

### Branch 1: the fiber dies, and the corpse stays subscribed

Off the root, `await` re-raises. C3 tells us exactly what happens next: an uncaught error at a fiber's
floor is captured into that fiber, which is marked `Failed`, and its frames are dropped in bulk. So
the awaiting fiber ends `isDone = true` with `Some(<CannotYieldAcrossNativeFrame>)` — which is what
program B printed.

But look at which branch does the cleanup:

```phalcom
if (err.isA(CannotYieldAcrossNativeFrame)) {
  return err.raise()                                       // no filter
}
_waiters = _waiters.filter { w => w != Fiber.current }      // only branch 2 unregisters
```

Branch 2 removes itself from `_waiters`. Branch 1 does not. The failed fiber stays in the waiter list
of a future it is no longer waiting for — and cannot ever wait for, being dead. Then someone settles
the future, `drain` runs, and `System.schedule` faithfully enqueues a corpse:

```phalcom
const f = Future.new()
const w = Fiber.new { f.await }
System.schedule(w)
System.runScheduled()        // w reaches await, fails, stays in _waiters
f.then { v => System.print("block waiter ran with " + v.toString) }
System.print("waiters registered; settling now")
f.settleValue(9)
System.runScheduled()
```

```
waiters registered; settling now
cannot resume a finished fiber
```

The block waiter never ran. `drain` scheduled the dead fiber first, the pump called `try()` on it, and
the run died on C3's finished-fiber check — taking the *healthy* waiter down with it. One fiber's
failed `await` corrupts the settle path of a future shared by everyone else.

This is the same shape as C3's E002 and the `upvalue` doc's E001, one level up the stack: **a
participant is removed from the machinery on one exit path and not the other.** In C3 it was cells
closed by `unwind_to` and not by the floor's bulk drop. Here it is waiters unregistered by branch 2
and not by branch 1. The recurring lesson of this track is that the cleanup which lives *inside* one
arm of a conditional is cleanup the other arm does not get.

---

## The other half: `then` is conditionally synchronous

`await` is the broken half. `then`/`map`/`catch` work — but they are worth a section, because they
have a property the spec never discusses and that most future libraries explicitly forbid.

```phalcom
then(f) {
  if (self.isReady) {
    if (_state == "fulfilled") { return Future.value(f.call(_value)) }   // NOW, in this fiber
    else { return self }
  } else {
    const f_next = Future.new()
    _waiters.add({ … Fiber.new({ f.call(_value) }).try() … })           // LATER, in a new fiber
    return f_next
  }
}
```

Two execution models behind one selector, chosen by a *timing* property of the receiver:

```phalcom
const root = Fiber.current
Future.value(1).then { v => System.print("settled cb on root? " + (Fiber.current == root).toString) }
Future.async { 1 }.then { v => System.print("pending cb on root? " + (Fiber.current == root).toString) }
System.runScheduled()
```

```
settled cb on root? true
pending cb on root? false
```

Same call, same callback shape. In one case it ran immediately in the caller's fiber; in the other it
ran later, in a fiber it can never name. JavaScript's promise spec has a name for the hazard of a
callback that *might* be synchronous — it is why `then` is defined to always defer, even on an
already-resolved promise. Phalcom does the thing that spec exists to prevent.

And the difference is not merely stylistic, because the two paths have different error semantics. The
settled path calls `f.call(_value)` bare; the pending path calls it inside `Fiber.new(…).try()`. So a
throwing callback:

```phalcom
Future.value(1).then { v => throw Error.new() }
System.print("survived")            // never prints — the error propagates, process exits 1
```

```phalcom
const b = Future.async { 1 }.then { v => throw Error.new() }
System.runScheduled()
System.print("survived, b.isReady = " + b.isReady.toString)   // survived, b.isReady = true
System.print(b.await.toString)                                // re-raises here instead
```

**The same callback throwing the same error either kills your program or quietly rejects the next
future, depending on whether the receiver had settled by the time you attached.** That is not a
timing difference; it is a difference in who is responsible for an error.

One more, smaller: on a settled receiver whose state is the *wrong* one, `then` and `map` return
`self` rather than a fresh future.

```phalcom
const r = Future.error(Error.new())
System.print((r.then { v => v } == r).toString)     // true
```

Chaining off a rejected future hands you back the same object. Harmless today — futures are settle-once,
so an aliased settled future cannot diverge — but it means future identity is not a reliable proxy for
"a distinct step in the chain," and any later code that keys on a future's identity inherits that.

---

## Why the tests are green

All of the above is in a repository whose concurrency suite passes. That is worth explaining, because
the explanation is more interesting than "nobody tested it."

There are four `Future` fixtures. Two cover settled futures only. One has a stale `status: PENDING`
header while sitting in the passing directory. The fourth is
`concurrency_future_slice_b.ph` — the feature's own acceptance test, which does exercise
`Future.new`, `Future.async`, and `await`, twelve times.

Every one of those `await` calls is on the **root fiber** — where branch 2 pumps, the queue is
non-empty, and the future settles. Except one, which is deliberately placed inside an `ensure` block
and *asserts* the failure:

```phalcom
// C-FUT-7: await under native frame raises CannotYieldAcrossNativeFrame
const helper = Fiber.new {
  try { } ensure { f10.await }
}
helper.try()
System.print("caught yield across native frame: " + helper.error.unwrapOr(None).message)
```

Read that as its author did and it is a correct, valuable test: awaiting inside an `ensure` block
really does cross a native frame, and raising is really the right behaviour. Nothing about it is
wrong. What it does is fix the observation to a cause — the user's `ensure` — that is not the only
cause. The identical output appears with no `ensure`, no `try`, no native frame of any kind. The test
locks in the symptom while attributing it to the wrong thing, so the day the bug appears without a
native frame in sight, the suite has already agreed that this is how it behaves.

And the case labelled **"C-FUT-2: async/await suspending"** does not suspend anything. It awaits at
root, takes the pump branch, and never yields. The label names the feature; the code exercises the
fallback.

So the suite's coverage of "a fiber awaits a pending future and later resumes" is: **zero cases.** Not
one test in the corpus performs the operation the feature exists for. Everything around it is tested
well — settle-once, `then` chaining, error passthrough, the `ensure` interaction. The hole is exactly
the shape of the feature.

The general form, which is the part worth keeping: **a fallback path that works is the most effective
way to hide that the primary path does not.** Branch 2 is a good fallback. It made every root-fiber
demo produce the right answer, which is why nobody wrote the non-root test, which is why branch 1 was
never run outside the one case where its failure was the expected result.

---

## The design space

Two things were decided here, and both are defensible in isolation.

**The guard is right.** C1 argued this and it is not reopened: a switch under a live native frame
corrupts a parked fiber's position. Making it depth-relative rather than depth-zero (`!= floor_depth`)
is *more* permissive than the naive rule, and deliberately so.

**Writing `Future` in Phalcom is right.** Zero new floor, zero native code, no VM surface to keep in
sync — the same reasoning that puts `List`'s protocol in `core.ph` over native primitives. The class
is 200 readable lines that a user can subclass.

What was not decided — because nobody appears to have noticed a decision was needed — is that a
library written in the language is subject to the language's restrictions **including the ones its own
call frames create.** `await` needed to know a fact about the VM (may I yield?) and the only tool it
had was to try it and look at the wreckage. That is attempt-and-inspect where the language offers no
predicate, and it is the sort of thing that reads as pragmatic right up until the attempt itself
changes the answer.

The neighbours are instructive, and each takes a different exit:

- **Lua 5.1** — the ancestor named in ADR-0030, with the same restriction on yielding across a C call
  boundary. It has no futures at all; `coroutine.resume` returns a `(status, value)` **pair** rather
  than raising. So the "may I?" question is answered by a return value, not by a thrown error that
  something else might have caused. Lua 5.2+ later lifted the restriction with continuation functions
  — the A-to-B path this design deferred.
- **JavaScript** — `then` is *always* asynchronous, even on a settled promise, precisely to eliminate
  the conditional-synchrony above. Its `await` is a syntactic form the engine compiles, not a library
  method: it cannot be defeated by a wrapper because there is no wrapper. Function coloring is the bill
  it pays; a uniform, unforgeable suspension point is what it buys.
- **Go** — deletes the question. A goroutine blocking on a channel receive *is* the wait, and the
  runtime — which has real stacks and preemption — does the parking. The bill is data races and a
  race detector, which C1 already covered.

Cut from this doc: Erlang (message-passing isolation is a different unit of argument), Python
`asyncio` (JS already makes the coloring point), Rust's own futures (a poll-based state machine is a
third model and would need its own doc to be fair to).

---

## What you can now re-derive

If this landed, you can reconstruct without looking:

1. Why `await` fails for *every* non-root fiber — not some — from `!=` in the guard plus two
   `native_reentry_depth += 1`s in `.attempt()`'s expansion.
2. Why the root fiber takes a different branch, and why that difference is the *ordering* of two
   `if`s in `fiber_yield` rather than anything anyone designed.
3. Why a failed `await` corrupts a future for its other waiters, from which branch runs the
   `_waiters` filter.
4. Why the same throwing callback either kills the process or rejects a future, from where
   `Fiber.new(…).try()` appears in `then` and where it doesn't.
5. Why a suite can be green over a feature that has never once performed its central operation — and
   what a fallback path costs when it is good enough to hide the absence of the primary one.

---

## Anchors

| Claim | Where | Verified by |
|---|---|---|
| `Future` is a plain `.ph` class, no native support | `core.ph:1346`; no `primitive/future.rs` | file listing |
| Slice B shipped 2026-07-14 | `06432bd`; `core.ph:1409,1424,1448` | `git log -S` |
| Three records still say it did not | `concurrency.md:187`, `plan.md:109-110`, `core.ph:1335-1338` | quoted |
| `.attempt()` is `.ph`, expands to two native re-entries | `core.ph:627-629`; `block.rs:158` | quoted |
| Guard is `!= floor_depth`, root check precedes it | `fiber.rs:336-339`; `floor_depth` written `:317` | quoted |
| Non-root `await` on a pending future fails | — | program run: `isDone = true`, `Some(<CannotYieldAcrossNativeFrame>)` |
| Bare `Fiber.yield` in the same position parks | — | control run: `isDone = false`, `error = None` |
| Root refusal is untyped | `fiber.rs:336` | program run: `Err(<Error>)` |
| Root `await` on an unsettleable future spins forever | `core.ph:1435-1437` | run; killed by alarm at 6s, no output |
| Branch 1 leaves the dead fiber in `_waiters` | `core.ph:1428-1434` | program run: `cannot resume a finished fiber` |
| `then` is sync-if-settled, async-if-pending | `core.ph:1466-1490` | program run: `true` / `false` |
| Throwing callback: kills caller vs rejects next | `core.ph:1469` vs `:1477-1478` | two program runs |
| `then` on a rejected future returns `self` | `core.ph:1471` | program run: `true` |
| No test covers a non-root park-and-resume | `concurrency_future_slice_b.ph` | read; all 12 awaits root or `ensure` |

Defect record: [E004](../../errors/E004-await-cannot-suspend.md).

---

## Where the track ends

That is the concurrency track: the [switch](restricted-loop.md), the [park](parked-fiber.md), the
[failure](fiber-failure.md), and the [wait](future-await.md) — and the wait is the one that does not
work.

The shape the four docs share is worth stating once, at the end, because none of them could state it
alone. Phalcom's fibers are a *restricted* mechanism, and every doc in this track found the same thing
at a different altitude: the restriction is well-designed, well-documented, correctly implemented — and
the code that has to live *with* it keeps failing at the boundary. C1's guard is right. C2's move is
right. C3's two teardown machines are each right. C4's `await` calls the right primitive. Every
individual decision here would survive review. Four of them in a row produced a busy spin, a dropped
upvalue, and a feature that cannot run.

Left open, and not this doc's to close:

- **E004** — `await` cannot suspend; the waiter leak; the quiescent spin. Reproduced, not fixed.
- **E002** — still open from C3: the fiber floor drops the stack without closing upvalues.
- Three stale records naming Slice B as unbuilt, listed above.
- `concurrency_future_async_await.ph`'s `status: PENDING` header, in the passing directory.
