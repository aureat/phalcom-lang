# C4 recon — "Futures that cannot wait"

Phase 1 of [AUTHORING-LEAN](../AUTHORING-LEAN.md). Written before any drafting. Everything below is
either a cited line or the verbatim output of a program run at HEAD (`05e28b7`).

---

## 1. Architecture vs representation

`Future` is **not a native object**. It is an ordinary Phalcom class in
`phalcom-core/core/core.ph:1346`, and its instances are plain `InstanceObject`s — three fields,
`_state` (a `String`), `_value`, `_waiters` (a `List`). There is no `primitive/future.rs`; the
`phalcom-core/src/primitive/` listing has no `future` entry.

The representational consequence that matters: **`Future` has no privileged access to the VM.**
Everything it does with the scheduler it does through the same two public seams `.ph` user code has —
`System.schedule(_)` and `Fiber.yield`. It is therefore subject to the restricted-yield guard exactly
as user code is, and it does not get to know anything the guard does not tell it. The defect this doc
is about is downstream of precisely that.

Architecturally it is promise-shaped (settle-once, `then`/`map`/`catch`, waiter list). It is *not*
promise-shaped representationally: there is no microtask queue, no job loop, no coloring.
Per [x-style ≠ representation], do not infer either from the other.

## 2. The grip (grounded)

**`Future#await` is the only method in the core library that makes its own precondition fail.** To
discover whether it is allowed to suspend, it attempts a yield inside `{ … }.attempt()` — and
`.attempt()` is two nested native re-entrant frames, so the yield it is testing is a yield the
wrapper has already made illegal. `await` therefore never parks a fiber: on the root it misreads the
refusal and degrades to a busy spin, and off the root it reads the refusal correctly and kills the
awaiting fiber.

Cites: `core.ph:1424-1444` (`await`), `core.ph:627-629` (`attempt`),
`phalcom-core/src/primitive/block.rs:158` (`native_reentry_depth += 1`),
`phalcom-core/src/primitive/fiber.rs:336-339` (the two refusals, in that order),
`phalcom-core/src/primitive/fiber.rs:317` (`floor_depth` written at resume).

## 3. Deliberated vs reconstructed

**Actually deliberated, in the record:**

- The restricted-yield rule itself — ADR-0030 §4, and `block.rs:151-157` argues it in a comment
  naming `.each { Fiber.yield(x) }` as the motivating case. The guard is designed, not incidental.
- The Slice A / Slice B split — `docs/forge/units/U-FUTURE/plan.md` §9, DEC-FUT-SCHED, ruled
  "Option 1 (RECOMMENDED, ADOPTED): U-FUTURE v1 = Slice A only (pure `.ph`, zero native)"
  (`plan.md:240`).
- `Future` as a plain `InstanceObject` with zero new floor — `concurrency.md` §2 "Implementation" ¶1.
- `System.runScheduled`'s bare-statement call form, chosen *specifically* so the pump does not add a
  native frame (`core.ph:1304-1316` argues this in its own comment). Someone understood this hazard
  well enough to design around it one screen above `await`.

**My reconstruction, not in any record:**

- That the `.attempt()` wrapper was chosen to *detect* the root-fiber case, and that the author
  expected the two refusals to be distinguishable by type. The code's shape says this; no comment or
  decision says it. Labelled as inference in the doc.
- That the sync/async split in `then` is Zalgo and that this is a cost. The code is silent; the spec
  does not discuss re-entrancy of continuations at all.

## 4. Findings that change the doc

**F1 — Slice B shipped; three separate records still say it did not.** `async`/`await`/`drain` are
implemented at `core.ph:1409-1461`, landed by `06432bd` ("feat(concurrency): implement Future Slice B
(async/await) and improve native error handling", 2026-07-14). Still contradicted by:
`concurrency.md:187` (`await` status "B"), `plan.md:109-110` (both "**B (DEFERRED → DEC-FUT-SCHED)**"),
and — worst — the class's *own* doc comment at `core.ph:1335-1338`, which says `async(_)`/`await`
"need a native ready-queue … neither of which is landed; that is Slice B … deliberately NOT built
here", sitting eleven lines above `await`'s implementation. The track plan inherited this and
predicted a doc about an unbuilt feature. It is built. The doc is about what it does.

**F2 — `await` can never suspend a non-root fiber.** `.attempt()` (`core.ph:627-629`) expands to
`{ Ok.new(self.call()) }.on(Error) { … }` — `block_on` and `block_call`, each bumping
`native_reentry_depth` (`block.rs:158`). The yield lands at `floor_depth + 2`; the guard fires on
`!=`, not on `> 0` (`fiber.rs:338`). Verified: a scheduled fiber awaiting a pending future ends
`isDone = true`, `error = Some(<CannotYieldAcrossNativeFrame>)`. Control: a **bare** `Fiber.yield(None)`
in the same position parks correctly (`isDone = false`, `error = None`). The wrapper is the whole
difference. Independently reproduced by Agent B with its own program.

**F3 — the root branch is reached by a misread, and it busy-spins.** `fiber_yield` checks root
*first* (`fiber.rs:336`) and returns an untyped `RuntimeError::NotAllowed`, so `isA(CannotYield…)` is
false and control falls to `while (not self.isReady) { System.runScheduled() }` (`core.ph:1435-1437`).
With nothing in the queue that will ever settle the future, this spins forever — no error, no
progress, no diagnosis. Verified: killed by an external alarm at 6s having printed nothing after the
entry line.

**F4 — second-order corruption: the dead waiter is never unregistered.** Only the root branch filters
`Fiber.current` out of `_waiters` (`core.ph:1434`); the `CannotYield…` branch re-raises at
`core.ph:1431` and leaves the now-*failed* fiber in the list. A later `settleValue` drains it into
`System.schedule` (`core.ph:1410-1411`), and the pump then tries to resume a corpse:
`cannot resume a finished fiber`. Found independently by me and by Agent B, from different programs.

**F5 — `then`/`map`/`catch` are conditionally synchronous (Zalgo).** On a settled receiver the
callback runs *bare, in the caller's fiber, right now* (`core.ph:1469`). On a pending receiver the
identical callback is deferred and run inside `Fiber.new(…).try()` (`core.ph:1477-1478`). The
difference is observable in error semantics, not just timing: a throwing callback **kills the caller**
on the settled path and **settles the next future as rejected** on the pending path. Both verified.
Also `then`/`map` on a settled-but-rejected receiver return **`self`**, not a new future — verified
`r.then { … } == r` is `true`.

**F6 — the shipped test suite green-lights all of it.** Four `Future` fixtures. Every single `await`
in `concurrency_future_slice_b.ph` is on the **root** fiber, except one that is deliberately inside an
`ensure` block and *asserts* `CannotYieldAcrossNativeFrame` as the expected result (C-FUT-7). The case
labelled "C-FUT-2: async/await **suspending**" suspends nothing — it awaits at root and takes the pump
branch. The feature's own acceptance test contains no case in which a fiber awaits and resumes.
(`concurrency_future_async_await.ph` also carries a stale `status: PENDING` header while living in the
passing directory.)

## 5. Forbidden list

| Material | Owner |
|---|---|
| Why a switch is `mem::take`, why it is O(1), why the loop is not told | C1 `restricted-loop.md` |
| The guard's *rationale*; `switch_pending`; the primitive-return three-way branch | C1 |
| The twelve/four field partition; park/unpark; `FiberObject` as buffers-not-in-use | C2 `parked-fiber.md` |
| The fiber-floor `Err` arm, `capture_error_value`, the `call`/`try` cascade, E002 | C3 `fiber-failure.md` |
| GC rooting, `temp_roots`, E001 | `upvalue.md` + `docs/errors/` |

C4 may **use** the guard, the floor, and `try`'s error capture as known vocabulary. It may not
re-derive any of them. Its own territory is: `Future`'s state machine, `System.schedule`/`ready_queue`
as *seen from `.ph`*, and the collision between the guard and a `.ph`-level `await`.

## 6. Open risks

| Risk | Disposition |
|---|---|
| Am I sure no path parks? Some resume route might land `floor_depth == depth`. | Posed to Agent B as an explicit REFUTE with instructions to find a counterexample by running programs. None found; guard is `!=` against a floor written at resume, and `.attempt()` adds two unconditionally. |
| Is the root spin real or does something break the loop? | Ran it. Hangs. Confirmed twice, independently. |
| Is `.attempt()` native or `.ph`? Changes the whole story. | `.ph`, `core.ph:627-629`. This was recon question #1 and it was the right one. |
| Does DEC-FUT-SCHED have a `docs/pdr/` entry that supersedes plan.md? | No — `grep -rl DEC-FUT-SCHED docs/pdr/` is empty; the ruling lives only in `err-plan.md` §9. Stated as such. |
| Am I describing a defect or prescribing a fix? | Describe only. Fix directions in E004 are labelled unverified, per the standing rule that a reproduced diagnosis is not a verified fix. |

## 7. Doc-kind gate

**Knot.** Not a fork (nothing is being chosen), not a mechanism (the mechanism is C1's and already
shipped). Two independently correct decisions — the restricted-yield guard, and implementing `await`
in `.ph` with an attempt-and-inspect probe — produce, in combination, a feature that cannot run.
Per AUTHORING-LEAN §3, **knot ⇒ Agent A is skipped**; Agent B ran alone.

[x-style ≠ representation]: ../../../CLAUDE.md
