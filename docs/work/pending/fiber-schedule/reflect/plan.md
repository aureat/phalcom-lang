# U-FIBER-REFLECT — Work order: `Fiber#isDone`/`Fiber#error` (Fiber-surface completion)

_Self-contained implementation plan for **one** implementer. Small, surgical native
addition to a landed unit — no new mechanism, no scheduler dependency. **Reviewer ON**
(touches `fiber.rs`/`universe.rs`, floor census). Green gate: `./scripts/verify.sh`
exits 0 + `cargo doc --workspace --no-deps` clean. Grounded in
**[concurrency.md §1](../../../spec/current/concurrency.md)** (Interface table, already
speced — `isDone`/`error` are documented, just unbuilt) and
**[ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)**
(floor-amendment convention, §Consequences). No new ADR needed — same authorization
U-FIBER's own primitives used._

> **Provenance.** Spun out of [U-FUTURE/plan.md](../U-FUTURE/plan.md) §2 gap 1, which
> found `Fiber#isDone`/`error` missing and — **incorrectly** — folded them into its own
> Slice B (gated on **DEC-FUT-SCHED**, the scheduler decision). That coupling was never
> load-bearing: `isDone` is a pure `FiberStatus` read and `error` is a pure `result`-slot
> read, both fields already on `FiberObject` (`heap.rs`) since U-FIBER landed. Neither
> touches `System.schedule`, a ready-queue, or any suspension machinery. This unit
> decouples them so they can land **now**, ahead of U-SCHED, unblocking any consumer
> that needs post-hoc fiber-completion inspection (`Future` Slice B is one, not the
> only one — a user generator wrapper checking `gen.isDone` needs it too).

---

## 1. Mission (one sentence)

Land the two remaining `Fiber` reflective accessors speced in
[concurrency.md §1](../../../spec/current/concurrency.md) — `isDone` (instance,
`true` once `Done`/`Failed`) and `error` (instance, the captured `Error` as
`Option`, if `Failed`) — as plain native reads over `FiberObject`'s existing
`status`/`result` fields, with no new state and no scheduler dependency.

## 2. Design (small — no re-litigation needed)

### 2.1 `isDone` — `FiberObject::status` read
`fiber_is_done(vm, receiver, _args) -> PhResult<Value>`: resolve `receiver` to
its `ObjRef` (mirror `expect_fiber` in `primitive/fiber.rs`), return
`Value::Bool(matches!(status, FiberStatus::Done | FiberStatus::Failed))`.
`Getter` signature, instance-side, mirrors `block_arity`'s shape (`fiber.rs`).

### 2.2 `error` — `FiberObject::result` read, wrapped as `Option`
`fiber_error(vm, receiver, _args) -> PhResult<Value>`: resolve `receiver`,
read `status`; if `Failed`, wrap `result` via the existing
`wrap_some(vm, value) -> Value` helper (`primitive/nil.rs:47`, already used by
`some_new`) and return `Some(error)`; otherwise return `vm.none_value()`
(`None`, matching `Future#value`'s existing `(_state == "fulfilled")
.ifTrue({ Some.new(_value) }, ifFalse: { None })` convention, `core.ph`).
`result` already holds the surface `Error` value for a `Failed` fiber
(`vm.rs` `capture_error_value`, landed) — no new capture path.

### 2.3 No state change, no write-set overlap with U-SCHED
Neither accessor mutates `FiberObject`, reads `VM::native_reentry_depth`, or
touches `switch_pending`/`current` — they are ordinary total reads, callable
from anywhere (including under a native re-entrant frame, unlike
`call`/`try`/`yield`). No restricted-yield-guard interaction. This is exactly
why they don't need U-SCHED: nothing here is a suspension point.

### Rubric — hazards & preclusion
- **Does not preclude U-SCHED / Future Slice B.** `Future`'s pump
  (`concurrency.md §2` Implementation, Slice B) reads `driver.isDone` and
  needs `error`'s captured value for its reject path — this unit ships
  exactly the two reads Slice B's design (`U-FUTURE/plan.md` §6.3) already
  assumes, unchanged.
- **Does not touch the fiber-floor cascade** (`vm.rs` `run_until`) or the
  switch mechanism (`fiber_resume`/`fiber_yield`) — read-only accessors over
  already-landed, already-correct state.
- **`error` on a `Suspended`/`Running`/`Done` fiber is `None`, not an error.**
  `Done` (clean return) has no captured `Error` — `result` holds the return
  value instead, not something `error` should surface as `Some`. Only
  `Failed` produces `Some`.

## 3. Confirmed write-set (tight, disjoint from any live unit)

| File | Why |
|---|---|
| `../../../../../phalcom-core/src/primitive/fiber.rs` | `fiber_is_done`, `fiber_error` — two new `pub fn`s, same shape as `fiber_current`/`fiber_abort` |
| `phalcom-core/src/universe.rs` | register `isDone`/`error` as instance-side `Getter`s on `fiber_cls`; floor-census bump (+2) |
| `../../../../../phalcom-core/tests/lang/concurrency` + `tests/lang/MANIFEST.md` | goldens (below); graduate `pending/concurrency_fiber_wren_is_done_and_error.ph` if present on HEAD (check — a prior session staged this fixture ahead of the unit existing) |
| `../../../../forge/units/U-FUTURE/plan.md` | strike the Slice-B write-set rows this unit now owns; replace with a precondition note (§4 below has the exact edit) |
| `../../../../spec/current/concurrency.md` | flip the `isDone`/`error` implementation-status note (§1 Implementation, already added) from "not yet landed" to landed, citing this unit's commit |

**Deliberately NOT in scope:** `async`/`await`, `System.schedule`, any ready-queue,
any `Future` change beyond what already reads these two accessors once they exist.

### 3.1 Collision risk
`fiber.rs`/`universe.rs` are shared with any concurrently-live Fiber-touching
unit — check `graphify affected "fiber.rs"` before dispatch; low risk (small,
additive, no existing-primitive edits).

## 4. Companion edit to `U-FUTURE/plan.md` (apply alongside this unit)

Replace `U-FUTURE/plan.md` §5's Slice B write-set row:

```
| `phalcom-core/src/primitive/fiber.rs` + `universe.rs` | add `Fiber#isDone` (Done|Failed) and `Fiber#error` (`Option<Error>`); register; floor-census bump |
```

with:

```
| **Precondition, not this unit's write-set:** `Fiber#isDone`/`error` ship via **U-FIBER-REFLECT** (standalone, unblocked, no scheduler dependency) — Slice B depends on it landing but does not implement it. |
```

And update §7 build order step 4 ("Slice B.0 — Fiber-surface completion") to
read: "Slice B.0 — confirm **U-FIBER-REFLECT** has landed (external
precondition); if not, dispatch it first — it has no DEC-FUT-SCHED
dependency and should not block on this ruling." §9's DEC-FUT-SCHED
description should drop `Fiber#isDone` from the "seam" it gates — only
`System.schedule` + the native FIFO + root-drive remain gated by that
decision.

## 5. Build order

1. **`fiber_is_done`.** Add + register + one PASS golden. Green.
2. **`fiber_error`.** Add + register (needs `wrap_some`, already `pub(crate)`
   in `nil.rs` — confirm visibility, widen if `primitive/fiber.rs` can't see
   it). PASS goldens for both `Failed` (`Some`) and `Done`/never-run
   (`None`). Green.
3. **Floor-census bump** (+2) in the same commit as step 2 — `isDone`/`error`
   are the last two rows of `concurrency.md §1`'s Interface table, closing
   U-FIBER's surface completely.

## 6. Test strategy — `concurrency` label

- **`isDone` false while suspended/running (PASS):** a fiber mid-generator
  (yielded, not finished) reports `isDone == false`.
- **`isDone` true once `Done` (PASS):** a fiber whose entry returned reports
  `isDone == true`.
- **`isDone` true once `Failed` (PASS):** a fiber whose entry raised
  uncaught (captured via a resumer's `try`) reports `isDone == true`.
- **`error` is `None` before failure (PASS):** `isDone == false` fiber's
  `.error` is `None`.
- **`error` is `None` on clean `Done` (PASS):** a fiber that returned
  normally (not raised) has `.error == None` — `result` holds the return
  value, not an `Error`; `error` must not conflate the two.
- **`error` is `Some(e)` once `Failed` (PASS):** matches the `Error` a
  concurrent `try()` would have delivered — assert the *same* captured
  instance (`==` identity), not a re-wrapped copy.
- **Cross-check with an existing negative golden:** `fiber_call_finished_uncaught.ph`/
  `fiber_try_finished_uncaught.ph` (landed, `tests/lang/concurrency/negative/`) already
  drive a fiber to `Failed` — reuse that setup rather than inventing a new one.

## 7. Decisions (none open — this unit has no design fork)

No BLOCKED-ON-DECISION register. The only judgment call — whether `error`
returns the raw captured `Error` value or a defensive copy — is settled by
precedent: every other reflective accessor in the codebase (`Method#holder`,
`Class#superclass`, etc.) returns the live handle, never a copy; `error`
follows suit.

## 8. Must-not-preclude check

- **`Future` Slice B / U-SCHED:** *served, not precluded* — ships exactly the
  two reads Slice B's design already assumes (§6.3 of `U-FUTURE/plan.md`),
  with zero coupling to how U-SCHED eventually lands.
- **U-GC:** not precluded — pure reads, no new heap state, no change to
  `FiberObject`'s shape (see `docs/forge/units/U-GC/plan.md` §2 preconditions,
  which already lists `FiberObject`'s current field set as ground truth).

## 9. Traceability

| Claim | Source |
|---|---|
| `isDone`/`error` speced, not landed | `concurrency.md §1` Interface table (lines 46–47); `universe.rs` fiber registration block (no `isDone`/`error` rows) |
| Both are pure reads over existing `FiberObject` fields | `heap.rs` `FiberObject.status`/`result` (landed, U-FIBER) |
| No scheduler/suspension dependency | `primitive/fiber.rs` — neither accessor needs `native_reentry_depth`/`switch_pending`/`current` |
| Coupling to DEC-FUT-SCHED was incidental, not load-bearing | `U-FUTURE/plan.md` §2 gap 1, §5 Slice B write-set, §7 build order step 4 |
| `wrap_some` reuse precedent | `primitive/nil.rs:47` (`some_new`'s own helper) |
| `Future#value`'s `Some`/`None` convention this mirrors | `core.ph` `class Future` `value =>` getter |

## 10. Return contract (report to `phalcom-reviewer`)

`fiber_is_done`/`fiber_error` added + registered · floor-census bump (+2,
with the new total) · the `U-FUTURE/plan.md` companion edit applied (§4) ·
`concurrency.md`'s isDone/error status note flipped to landed · goldens per
§6 all green · confirmation this unit touched **no** scheduler/`vm.rs`
mechanism · `verify.sh` + `cargo doc` tails.
