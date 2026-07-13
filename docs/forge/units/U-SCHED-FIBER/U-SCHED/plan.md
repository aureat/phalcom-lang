# U-SCHED — Work order: native ready-queue + root-drive pump

_Self-contained implementation plan for **one** implementer. Foundational,
`vm.rs`-SPINE unit — **serialize**, do not run concurrently with any other
unit touching `vm/dispatch.rs`/`vm/mod.rs`/`vm/bootstrap.rs` or
`universe/primitives.rs`'s `System` block. **Reviewer ON.** Green gate:
`./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps` clean.
Grounded in [scheduler-unit.md](../../../../spec/v0.2/experimental/scheduler-unit.md)
(the proposed decision this unit realizes),
[concurrency.md §2](../../../../spec/v0.2/concurrency.md) Slice B design intent,
[system.md §2](../../../../spec/v0.2/system.md) (`schedule(_)`/`sleep(_)`
reserved seam), and [ADR-0030](../../../../adr/0030-fibers-and-futures-cooperative-concurrency.md)
(no second concurrency primitive — a queue, not a mechanism)._

> **Provenance.** `scheduler-unit.md` names this unit but leaves it
> "Proposed... unowned." [U-FUTURE/plan.md](../../U-FUTURE/plan.md) §9
> (**DEC-FUT-SCHED**) already ruled: U-FUTURE ships Slice A only
> (scheduler-free); async/await wait for **this** unit landing as a
> ratified, owned prerequisite, not for U-FUTURE to absorb the seam itself
> (Option 1, recommended and adopted). This plan is that ratification.

---

## 1. Mission (one sentence)

Give `System.schedule(_)` a real native home — a FIFO of not-yet-started
fibers on the `VM`, a `.ph`-visible drain seam, and a root-drive pump that
runs queued work to completion at program exit and on demand — so
`Future`'s Slice B (`async`/`await`) has an owned, ratified substrate to
build on, without inventing a second concurrency primitive or requiring
`main` to be rewritten as a scheduler fiber.

## 2. Design summary (full detail: [`implementation-spec.md`](implementation-spec.md))

- **`VM::ready_queue: VecDeque<ObjRef>`** — fibers wrapped by
  `System.schedule(_)`, never resumed yet.
- **`System.schedule(_)`** (native, class) — validate + wrap `args[0]` as a
  fresh `Fiber` (reusing `fiber_new`'s validation), push, return the
  `Fiber` handle.
- **`System.nextScheduled`** (native, class getter) — pop-front as
  `Option<Fiber>`. The one drain primitive every pump (native or `.ph`)
  bottoms out in.
- **Root-drive** — `VM::run` drains `ready_queue` (via `fiber_try` +
  `run_until(0)` per entry) once, after the top-level program's own
  activation ends. Belt-and-suspenders: catches anything nothing else
  pumped.
- **`System.runScheduled`** (`.ph`, `core.ph`) — `while (let Some(f) =
  System.nextScheduled) { f.try() }`, the library-level counterpart used
  mid-program (chiefly by `Future`'s eventual await-pump).

### Rubric — hazards & preclusion
- **Does not add a second concurrency primitive.** The queue holds
  `ObjRef`s to ordinary `Fiber`s; every resume goes through the landed
  `fiber_try` (ADR-0030 §1/§Consequences: `Future` — and now the
  scheduler — adds no VM mechanism beyond `Fiber` + a queue).
- **Does not require `main`-as-scheduler-fiber.** Supersedes
  `scheduler-unit.md`'s original sketch ("top-level runs inside the root
  scheduler fiber") with the root-drive pump `U-FUTURE/plan.md §6.3`
  already committed to — `await` degrades to "drain + re-check", it does
  not require the root fiber's own identity to change.
- **A drained fiber's failure cannot abort the host.** `drain_ready_queue`
  resumes via `fiber_try` (capture, not propagate) — one scheduled task
  raising uncaught never brings down the process or other queued work.
- **Re-queuing a suspended (not finished) scheduled fiber is explicitly
  NOT this unit's job.** `drain_ready_queue`'s contract is "run everything
  currently ready, exactly once." A fiber that suspends mid-drain (e.g. a
  future `await`) is not re-enqueued here — that is `Future` Slice B's own
  waiter-list/settlement responsibility (`concurrency.md §2` Structure).
  Keeps queue *policy* out of the scheduler's own surface, matching
  `U-FUTURE/plan.md §10`'s must-not-preclude row on this exact hazard.

## 3. Confirmed write-set

| File | Why |
|---|---|
| `phalcom-core/src/vm/mod.rs` | new `ready_queue: VecDeque<ObjRef>` field |
| `phalcom-core/src/vm/bootstrap.rs` | init the field in `VM::new` |
| `phalcom-core/src/vm/dispatch.rs` | `VM::run` wraps `run_until(0)` with the root-drive (§5 of implementation-spec) |
| `phalcom-core/src/primitive/fiber.rs` | extract `new_fiber_ref` from `fiber_new`'s body (shared by `Fiber.new` and `System.schedule`) |
| `phalcom-core/src/primitive/system.rs` | `system_schedule`, `system_next_scheduled` |
| `phalcom-core/src/universe/primitives.rs` | register both class-side on `system_cls` (L225–227 block); floor-census bump (+2) |
| `phalcom-core/core/core.ph` | `System.runScheduled` |
| `phalcom-core/tests/lang/concurrency/` + `tests/lang/MANIFEST.md` | goldens (§6) |
| `docs/spec/v0.2/system.md` | flip `schedule(_)` status; document `nextScheduled`/`runScheduled` as a floor amendment |
| `docs/spec/v0.2/concurrency.md` | flip Slice B's "needs the native ready-queue" note to landed, citing this unit |

**Deliberately NOT in scope:** `System.sleep(_)`, any timer completion
source, `async`/`await`, `Future` Slice B itself (ships in U-FUTURE, gated
on this unit landing — external precondition, not built here).

### 3.1 Collision risk
`vm/dispatch.rs`/`vm/mod.rs`/`vm/bootstrap.rs` are SPINE files just split
out of the former monolithic `vm.rs` this session — check
`graphify affected "vm/dispatch.rs"` immediately before dispatch; this is
the highest-collision-risk unit in the batch. Serialize against any other
in-flight unit touching those three files or `universe/primitives.rs`.

## 4. Timers/`sleep` — explicitly deferred, not this unit's scope

`scheduler-unit.md`'s dependency DAG bundles "ready-queue + timers" into
one arrow; this plan **splits them**. Reason: `open-questions.md §15`
leaves **fairness** OPEN, and a timer completion source needs a fairness
answer (does a fired timer preempt queued `schedule`d work, or FIFO
alongside it?) this unit cannot unilaterally resolve. The ready-queue +
`schedule` + drain seam is buildable and useful **without** timers —
`Future.async`/`await` (U-FUTURE Slice B) needs exactly this half, not
`sleep`. `System.sleep(_)` is a follow-on unit once fairness is ruled;
flag in `DEFERRED.md`, do not silently fold a timer heap into this unit's
`VM` struct.

## 5. Build order

1. **`VM::ready_queue` field + init.** No behavior change yet. Green.
2. **`new_fiber_ref` extraction** from `fiber_new` (pure refactor,
   `fiber_call`/`try`/goldens must stay byte-identical). Green.
3. **`system_schedule` + `system_next_scheduled` + registration.** PASS
   goldens: schedule doesn't run immediately; `nextScheduled` pops in FIFO
   order; empty queue gives `None`.
4. **`VM::run` root-drive.** PASS golden: a scheduled fiber runs by the
   time the program exits even with no explicit drain call. PASS golden:
   a scheduled fiber that raises does not abort the host or the next
   scheduled fiber.
5. **`System.runScheduled`** (`.ph`, `core.ph`). PASS golden: mid-program
   explicit drain runs everything queued so far, in order.
6. **Floor-census bump** (+2) in the same commit as step 3.

## 6. Test strategy — `concurrency` label

- **Schedule doesn't run synchronously (PASS):** `System.schedule({ System.print("x") })` prints nothing before the next drain point.
- **FIFO order (PASS):** three scheduled fibers print in enqueue order once drained.
- **Root-drive runs at program exit (PASS):** a scheduled fiber's side effect (`System.print`) is observed in the program's stdout even though `main` never calls `runScheduled` — assert via golden stdout, not an internal hook.
- **A raising scheduled fiber does not abort the host (PASS):** schedule two fibers, the first raises uncaught, the second still runs and its side effect is observed; the overall program still exits 0.
- **`nextScheduled` on an empty queue is `None` (PASS).**
- **`System.runScheduled` drains everything queued so far, in order, then returns (PASS):** including work newly scheduled *during* the drain (nested `System.schedule` inside a running scheduled fiber) — assert the nested one also runs before `runScheduled` returns.
- **Regression:** all existing `concurrency` goldens (C-FIB-*) stay byte-identical — this unit adds a field and two primitives, touches no existing `Fiber`/`fiber_resume` behavior.

## 7. Decisions

No BLOCKED-ON-DECISION register for the core slice — every judgment call
above is closed by precedent (ADR-0030 §Consequences authorizes the
`Fiber` + queue floor extension; DEC-FUT-SCHED already ruled U-SCHED is the
right owner). One forward-looking note, not a gate:

- **DEC-SCHED-FAIRNESS (non-blocking, flagged for the `sleep` follow-on).**
  `open-questions.md §15` leaves ready-queue fairness OPEN. This unit's
  plain FIFO is a defensible default for `schedule`-only workloads (no
  priority, no starvation without an infinite producer) but is **not**
  presented as a final fairness ruling — a timer-bearing follow-on must
  re-open this, not inherit it silently.

## 8. Must-not-preclude check

| Hazard | How this plan clears it |
|---|---|
| **A blocking `await` (single-thread deadlock).** | Not this unit's surface (`await` is U-FUTURE Slice B), but the root-drive pump this unit ships is exactly what lets Slice B's `await` degrade to "drain, re-check" instead of blocking. |
| **A second concurrency primitive.** | Queue of `ObjRef`s to ordinary `Fiber`s, resumed via the landed `fiber_try` — no new `Value`/`Object` arm. |
| **Boxing out `Future`'s eventual re-enqueue-on-settle.** | `drain_ready_queue`/`runScheduled` only ever *pop*; nothing here owns *pushing* a resumed-later fiber back on — that stays `Future`'s waiter-list job, layering on additively. |
| **Timers retrofitted awkwardly.** | Deliberately deferred (§4) rather than half-built; the FIFO's shape (`VecDeque<ObjRef>`, no priority field) does not preclude a separate timer-heap sitting alongside it later. |
| **`main`-as-scheduler-fiber retrofit.** | Explicitly rejected in favor of the root-drive pump (§2 rubric) — nothing in this design requires revisiting the root fiber's identity. |

## 9. Traceability

| Claim | Source |
|---|---|
| `U-SCHED` proposed, unowned, sequenced after `Fiber` before `Future` async/await | `scheduler-unit.md` Decision + dependency DAG |
| DEC-FUT-SCHED ruled U-SCHED is the owning unit, not U-FUTURE | `U-FUTURE/plan.md` §9 |
| Root-drive pump (not scheduler-fiber-as-main) is the committed design | `U-FUTURE/plan.md` §6.3 |
| `System.schedule(_)`/`sleep(_)` reserved seam; native-only rationale (no `.ph` class-side mutable state) | `system.md` §2/§3 |
| `Future`'s pump `.try()`s ready drivers | `U-FUTURE/plan.md` §6.3 |
| No second concurrency primitive — `Fiber` + queue only | ADR-0030 §1/§Consequences |
| Fairness OPEN | `open-questions.md` §15 |
| `fiber_try`/`FiberResumeMode::Try` capture-not-propagate semantics reused for drain | `primitive/fiber.rs` L156–160, L285–298 (`vm/dispatch.rs` fiber-floor capture) |
| `VM::run` hook point | `vm/dispatch.rs` L201–203 |

## 10. Return contract (report to `phalcom-reviewer`)

`ready_queue` field + init · `new_fiber_ref` extraction (goldens
byte-identical) · `system_schedule`/`system_next_scheduled` added +
registered · `VM::run` root-drive wired · `System.runScheduled` in
`core.ph` · floor-census bump (+2, new total) · `system.md`/`concurrency.md`
status notes flipped · goldens per §6 all green · confirmation this unit
added **no** new `Value`/`Object` variant and did not touch
`fiber_resume`/`fiber_yield`'s own logic · `verify.sh` + `cargo doc` tails.
