# U-SCHED — Implementation Spec: native ready-queue + root-drive pump

Companion to [`plan.md`](plan.md) (work order, decisions register, test
strategy). This document is the Rust-level reference: exact `VM` field,
exact primitive signatures, exact hook point in the dispatch loop.

Grounds: [scheduler-unit.md](../../../../design/experimental/v0.2/scheduler-unit.md)
(problem statement, dependency DAG) ·
[concurrency.md §2](../../../../spec/current/concurrency.md) Implementation/Slice B
(design intent this unit realizes) · [system.md §2](../../../../spec/current/system.md)
Scheduler row (`schedule(_)`/`sleep(_)` reserved seam) ·
[ADR-0030](../../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
(no second concurrency primitive — this unit adds a queue, not a mechanism).

**Supersedes scheduler-unit.md's "main runs inside the root scheduler fiber"
model.** [U-FUTURE/plan.md](../../U-FUTURE/plan.md) §6.3/§9 already resolved
**DEC-FUT-SCHED** toward a **root-drive pump** instead: `await` on the root
fiber degrades to "drain the ready-queue, re-check state" rather than
requiring `main` itself to run as a scheduler fiber. That decision is
load-bearing here — this spec builds the pump-drain model, not the
scheduler-fiber-as-main model scheduler-unit.md originally sketched.

---

## 1. Target files (confirmed write-set)

| File | Change |
|---|---|
| `phalcom-core/src/vm/mod.rs` | new `VM` field: `pub(crate) ready_queue: VecDeque<ObjRef>` |
| `phalcom-core/src/vm/bootstrap.rs` | init `ready_queue: VecDeque::new()` in `VM::new` |
| `phalcom-core/src/vm/dispatch.rs` | `VM::run` drains `ready_queue` after `run_until(0)` returns `Ok` (root-drive, §4 below) |
| `phalcom-core/src/primitive/system.rs` | `system_schedule`, `system_next_scheduled` |
| `phalcom-core/src/universe/primitives.rs` | register both class-side on `system_cls`, next to `print`/`new` (L225–227) |
| `phalcom-core/core/core.ph` | `System.runScheduled` — thin `.ph` drain loop over `nextScheduled` |
| `phalcom-core/tests/lang/concurrency/` + `MANIFEST.md` | goldens |
| `docs/spec/current/system.md` | flip `schedule(_)` status note; document the additional `nextScheduled` seam as a floor amendment |
| `docs/spec/current/concurrency.md` | flip Slice B "needs the native ready-queue" note once this lands |

**Deliberately NOT in scope (separate sub-slice, see `plan.md` §9):**
`System.sleep(_)`, any timer completion source, `async`/`await`/`Future`
Slice B itself (U-FUTURE's job, this unit is its precondition).

## 2. `VM` field

```rust
/// FIFO of scheduled-but-not-yet-run fibers ([`System::schedule`]).
///
/// Populated by `System.schedule(_)`; drained by the root-drive pump
/// ([`VM::run`]) and by any `.ph`-level pump loop (`System.runScheduled`,
/// `core.ph`) via [`System::nextScheduled`]. A fiber in this queue has
/// never been resumed (`FiberObject::started == false`) — draining it
/// resumes it as a fresh entry call, exactly like `Fiber#call`'s
/// first-resume path (`primitive/fiber.rs` `fiber_resume`).
pub(crate) ready_queue: VecDeque<ObjRef>,
```

Placement: next to `open_upvalues` in the `VM` struct (`vm/mod.rs`,
current L116). `VecDeque` import added to the existing `std::collections`
use block.

## 3. `System.schedule(_)`

```rust
/// Signature: `System::schedule(_)` — wraps `args[0]` (a `Function`) as a
/// fresh, not-yet-started `Fiber` and enqueues it on [`VM::ready_queue`].
/// Returns the `Fiber`, so a caller (e.g. `Future.async`) can hold a handle
/// to poll it later via the already-landed `isDone`/`error`
/// ([U-FIBER-REFLECT](../../U-FIBER-REFLECT/plan.md)).
///
/// Does **not** run the fiber — it runs at "the next scheduler turn"
/// (system.md §2), i.e. whenever the queue is next drained (§5 below).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `Function`
/// (mirrors `fiber_new`'s own check, `primitive/fiber.rs` L110–118 —
/// reuse that validation rather than duplicating it).
pub fn system_schedule(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let fiber_ref = crate::primitive::fiber::new_fiber_ref(vm, args[0])?;
    vm.ready_queue.push_back(fiber_ref);
    Ok(Value::Obj(fiber_ref))
}
```

`new_fiber_ref` is a small extraction from `fiber_new`'s body (validate +
`FiberObject::new_entry` + `heap.alloc`) so both `Fiber.new` and
`System.schedule` share one construction path — a one-line refactor of
`primitive/fiber.rs`, not a new primitive.

## 4. `System.nextScheduled`

```rust
/// Signature: `System::nextScheduled` — pops and returns the next queued
/// fiber as `Option<Fiber>` (`None` once the queue is empty). The `.ph`-
/// visible drain seam every pump loop (native root-drive, `.ph`
/// `System.runScheduled`, and eventually `Future`'s Slice B await-pump)
/// bottoms out in.
pub fn system_next_scheduled(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match vm.ready_queue.pop_front() {
        Some(fiber_ref) => Ok(crate::primitive::nil::wrap_some(vm, Value::Obj(fiber_ref))),
        None => Ok(vm.none_value()),
    }
}
```

**Not in `system.md`'s Interface table today** — a floor amendment in the
same vein as ADR-0037/0038/0039/0049 (each "amend floor: admit X
primitive"). No new ADR needed: ADR-0030 §Consequences already sanctions
the `Fiber` + queue floor extension for the scheduler seam; this is that
extension. `documentation-and-adrs` should append a short note to
`system.md §2`'s Scheduler row, not draft a new ADR.

## 5. Root-drive: `VM::run`

```rust
pub fn run(&mut self) -> PhResult<Value> {
    let result = self.run_until(0)?;
    self.drain_ready_queue()?;
    Ok(result)
}

/// Runs every fiber in [`Self::ready_queue`] to completion or first
/// suspension, belt-and-suspenders draining any newly-scheduled work a
/// drained fiber itself enqueues (`while let`, not a fixed-size `for`).
/// Mirrors `Fiber#try`'s resume path — failures are captured, not
/// propagated, so one scheduled task's error cannot abort the others or
/// the host program (same posture as `fiber_try`/`FiberResumeMode::Try`).
fn drain_ready_queue(&mut self) -> PhResult<Value> {
    while let Some(fiber_ref) = self.ready_queue.pop_front() {
        let receiver = Value::Obj(fiber_ref);
        crate::primitive::fiber::fiber_try(self, &receiver, &[])?;
        self.run_until(0)?;
    }
    Ok(self.none_value())
}
```

This is the **only** automatic drain point — it fires once, after the root
fiber's own top-level activation ends (`run_until(0)`'s existing
`resumer.is_none()` branch, `vm/dispatch.rs` L238–243, is unchanged; this
wraps *outside* it). A scheduled fiber that itself suspends (e.g. a
`Future` Slice B driver hitting `Fiber.yield` inside `await`) is **not**
re-queued by this loop — re-queueing a suspended fiber is Slice B's job
(a future's settlement enqueues its waiters, `concurrency.md §2` Structure),
not a bare U-SCHED concern. `drain_ready_queue`'s contract is exactly
"run everything currently ready to completion, exactly once" — nothing
about *why* a fiber later becomes ready again.

**Why `fiber_try` and not a raw resume:** `fiber_try` already does exactly
"resume from a fresh/parked state, capture failure instead of propagating"
— identical to what a scheduled fire-and-forget task needs. No new resume
primitive; reuse the landed one via its existing `pub fn` signature
(`primitive/fiber.rs` L158–160).

## 6. `System.runScheduled` (`.ph`, `core.ph`)

```phalcom
class System {
  static runScheduled() {
    while (let Some(f) = System.nextScheduled) {
      f.try()
    }
  }
}
```

The `.ph`-callable counterpart to §5's native belt-and-suspenders drain —
used by any library code (chiefly `Future`'s eventual Slice B await-pump)
that needs to force progress *mid-program*, not just at root-exit. Kept in
`core.ph` rather than native: pure orchestration over two already-native
primitives, no VM state touched directly — same precedent as `Future`'s own
`.ph` state machine (U-FUTURE Slice A).

## 7. Verification

```sh
./scripts/verify.sh
cargo doc --workspace --no-deps
```
