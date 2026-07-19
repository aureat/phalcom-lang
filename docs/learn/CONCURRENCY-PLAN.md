# Track plan — Concurrency

Plan only. No doc is written. Each doc below still gets its own full five-phase
[AUTHORING](AUTHORING.md) run; this file decides **the split, the order, and what each doc owes**,
so that four separate recon passes don't each re-derive the track's shape.

Status in [TRACKER.md](TRACKER.md). Grounded against HEAD 2026-07-19; see [§7](#7-open-risks) for
what that grounding did *not* settle.

---

## 1. Why this track, and why it is the largest debt

Five shipped VM docs point here — Docs 1, 2, 3, 4 and 6. That is not a coincidence of scheduling; it
is structural. **The VM docs currently cannot explain their own guards:**

- Doc 1 hoists a `Rc<Callable>` and keys the guard on `closure_id` rather than `ip`. Why that is
  sound depends on what a fiber switch can and cannot change mid-loop.
- Doc 3 declared `VM::frames` a "live mirror" as **Lie #2** and deferred the whole notion.
- Doc 6 quoted `store_live_into`'s four-field `mem::take` and ADR-0030 §6's invariant *as given* —
  it borrowed the fact it needed and left an IOU in its forward pointers.
- `upvalues.md` has an `Upvalue::Open { fiber, slot }` whose `fiber` field it explains only locally.
- Doc 4 marked the primitive-return `switch_pending` branch as Lie #2's other half.

So the track's obligation is unusually concrete: **it must pay five named debts, by name.** A reader
finishing it should be able to go back to Doc 1 and say why the hoist is safe.

## 2. What makes this track different from the VM track

**ADR-0030 is genuinely deliberated, and richly so.** The VM docs kept having to confess that their
design-space walks were pedagogical reconstruction — no bake-off ever happened for stack-vs-register,
for frame representation, for the dispatch fork. That confession was honest but repetitive.

Here the opposite holds. ADR-0030 carries **seven numbered decisions** and **three named rejected
alternatives** (B: full trampoline; C: stackful coroutines; and a third, unread at plan time). It
records a *foreclosed capability* with a concrete failing program. It names its own lineage
("audit Option A / **Lua-5.1 style**"). It states five pre-fiber invariants that bound other units.

Consequence for the plan: **these docs are mostly forks, not mechanisms.** Weight the design space
heavily; the usual "the space is a reconstruction" caveat will be *false* here and must not be
copy-pasted from the VM docs.

**Second difference: the scars are ours.** Most docs borrow a scar from Go or JS. This track can
show live, reproducible bugs in its own subject (§5, C3).

## 3. The four docs

### C1 — The restricted loop *(fork)*

**Subject.** Why cooperative, why single-threaded, and — the real content — ADR-0030 §4's
**restricted (Option A)** execution model and §5's **typed** switch signal.

**Candidate grip.** *A fiber switch is not a jump. It is a swap of which buffers the one loop is
looking at — which is why a switch is O(1), and why it cannot happen while a native Rust frame is
holding the loop's place.*

**Design space (real, from the ADR).** A = restricted re-entrant loop (taken); B = full trampoline,
de-recurse every callback primitive, yield anywhere; C = stackful coroutines with real native
stacks; plus preemption/OS threads as the outer boundary. The ADR argues A→B is *purely additive*
and A→C is not — that asymmetry is the decision's spine and should be the doc's.

**The teaching moment is a foreclosure, and it is unusually clean.** ADR-0030 §4 gives both sides:

```phalcom
Fiber.new { let n = 0; while (true) { Fiber.yield(n); n = n + 1 } }   // works
Fiber.new { list.each { x => Fiber.yield(x) } }                        // CannotYieldAcrossNativeFrame
```

Two programs that look equally reasonable; one is legal and one is not, and **the reason is Doc 5's
inliner** — `while` lowers to `Jump`/`Loop` inside one chunk with no frame and no native re-entry,
while `each` goes through `block_call`, which calls `run_until` on the Rust stack. A user-visible
language restriction that falls directly out of an optimizer decision. That is the best
predict-then-check candidate in the whole track: *show both programs, ask which one cannot work.*

**Pays.** Doc 4's Lie #2 (`switch_pending` in the primitive-return path).
**Fixture.** `concurrency_fiber_restricted_yield_guard.ph` exists.
**Must not restate.** Doc 1's loop structure; Doc 5's inliner mechanics (cite, don't re-teach).

### C2 — The parked fiber *(mechanism)*

**Subject.** What a `FiberObject` owns, what a switch actually moves, and what deliberately stays
behind.

**Candidate grip.** *A fiber is not a thread of execution — it is a set of buffers. Switching is
`mem::take` on four fields, and the design is entirely in which four.*

**Content.** `store_live_into`/`load_live_from`; the four fields that move (`frames`, `stack`,
`open_upvalues`, `checking`) and the ones that pointedly do not — `next_frame_generation` above all,
which is ADR-0030 §6's named invariant and the fact Doc 6 had to borrow. Fiber-stack pooling.
ADR-0030 §2 (heap object, no new `Value` arm) and §3 (`stack_offset` stays frame-relative so
per-fiber stacks starting at 0 need no rebasing — a small decision with large consequences). §7:
parked fibers are GC roots, and a collector scanning only `current` would free live objects.

**Pays.** Doc 3's Lie #2, and Doc 6's borrowed invariant — this doc discharges the most debt.
**Must not restate.** `upvalues.md` already spent `Upvalue::Open { fiber, slot }`, the fiber-rooting
trace arm, and "the second branch on the read path is the feature." Highest overlap risk in the
track; handle it as Doc 6 handled `upvalues.md` — an explicit forbidden-list in REQUIREMENTS.

### C3 — When a fiber fails *(tension — the strongest doc)*

**Subject.** Fiber teardown colliding with upvalue closing, and error unwinding stopping at the
**fiber floor** instead of terminating the host (ADR-0030 §6, second half).

**Candidate grip.** *A fiber's failure is contained by design — the unwind stops at its floor and the
error lands in its result slot. The bugs are all in what the floor forgets to do on the way down.*

**Content.** `fiber_abort`, `Fiber#error`, `Fiber#isDone`, `try` vs `call`. How `DeadFrameError`
crosses a fiber boundary (Doc 6 set this up and can be paid back here).
`CannotYieldAcrossNativeFrame` as a *catchable* error.

**The scars are our own, reproducible, and unfixed at HEAD** — an uncaught fiber failure drops a
live stack without closing open upvalues, so an escaped capturing block then indexes an empty stack;
and `block_ensure` holds a pending result unrooted across cleanup, where a GC during cleanup frees
it. Same family: *a value held live across a re-entrant interpreter call that the root/unwind scan
does not see.*

> **Obligation.** These are recorded in session memory as confirmed with repros, but a grep at HEAD
> does **not** find CB-7/CB-8/CB-9 in `docs/forge/DEFERRED.md` — the write-up was either never
> committed or lives elsewhere. **Reproduce from scratch at write time; do not cite from memory.**
> On that same backlog 4 of 6 *prescriptions* were wrong even though all 6 *diagnoses* reproduced —
> so the doc may describe the bug, and must not prescribe the fix.

**Needs.** C1 and C2 both, in the reader's head. Genuinely last of the three.

### C4 — Futures that cannot wait *(honesty doc — write last, may collapse)*

**Subject.** A `Future` that is always already settled.

**Grounding that changes everything about this doc.** `Future` is **not native** — no
`primitive/future.rs` exists. It is written in Phalcom in `core/core.ph` (~L1327-1469) as a plain
`InstanceObject`: "zero new floor, zero native code, no `Fiber` dependency." And only **U-FUTURE
Slice A** shipped — `value(_)`/`error(_)` construct an already-settled future, so `then`/`map`/`catch`
fire **synchronously** and `_waiters` is *always empty*, kept as a field only so Slice B's
pending→settle drain is an additive layout change. `async(_)`/`await` need a native ready-queue and
are gated on DEC-FUT-SCHED.

**Candidate grip.** *Phalcom's `Future` contains no concurrency. It is a settle-once state machine
that happens to share a name with one — and the half that would make it asynchronous is a library
change plus one scheduler hook, not a VM feature.*

**Why it might be good.** "What does a future mean when it is always already settled" is a real
question, and the answer — that `then` is just `map` on a one-slot container — is the kind of reframe
this course exists for. It is also the landed-vs-planned spine Doc 5 proved works.

**Why it might collapse.** Thin. If Slice B lands before this is written, the doc changes completely
and should be re-planned. If it does not, consider folding it into C1 as a closing section rather
than padding it to doc length. **Decide at write time, not now.**

## 4. Order

**C1 → C2 → C3**, with C4 decided last.

- C1 first: the fork supplies the grip, and the other two are downstream of the execution model.
- C2 second: discharges the most inherited debt, and C3 needs its vocabulary (floor, park, swap).
- C3 third: a tension doc requires both prior mechanisms already in the reader's head.
- C4 whenever its subject stops moving.

## 5. Comparison cast (provisional — each doc re-runs the filter)

| Language | Filter test | Why |
|---|---|---|
| **Lua 5.1** | ancestor (4) | Named *in the ADR itself* as the lineage. Its restricted-yield rule is the same rule, and Lua's later lift of it is the A→B path argued here. |
| **Wren** | took the same branch, bill attached (1) | The fixture corpus contains a whole `concurrency_fiber_wren_*` family — Phalcom validated its fiber semantics against Wren's directly. Rare and strong evidence. |
| **Go** | other branch + scar (1, 2) | Goroutines: preemptive-ish, real stacks, the branch C/thread boundary. Bill: data races, `sync`, a whole race detector. |
| **JS** | names it (3) | `async`/`await` and function coloring — the vocabulary for exactly what C4's Slice-A future is missing. |

Likely cut: Erlang (processes are a different unit of isolation — a whole other argument), Ruby
`Fiber` (same branch as Lua, adds nothing Lua doesn't), Python `asyncio` (JS already makes the
coloring point). Name the cuts in the docs, per the filter.

## 6. What must be true before C1 starts

- **Per-doc recon is still mandatory.** This plan decides the split, not the content. Every doc runs
  its own Phase 1 — the last three docs each found a wrong premise in their own plan.
- **Settle the shipped-vs-specified question first.** ADR-0030 §5 says the typed switch signal
  replaces "the `frames.len()` heuristic that the primitive arm **currently** uses." That "currently"
  was written pre-implementation. Whether the typed signal actually shipped, or the heuristic is
  still there, decides whether C1 is a fork doc or a landed-vs-planned doc. **This is the single
  highest-value thing to verify, and it is C1's recon question #1.**

## 7. Open risks

| Risk | If wrong, the track… |
|---|---|
| **ADR-0030 covers Futures at length; Futures are a quarter built.** | …repeats spec text as if shipped. Expect the ADR-vs-HEAD gap to be this track's recurring honesty note, exactly as `U-IC`'s plan was Doc 5's. Assume nothing from §1's surface list. |
| **Overlap with `upvalues.md`** (open cells name a fiber; fiber rooting; teardown clearing parked state). | …C2 and C3 restate a shipped doc. Forbidden-list in each REQUIREMENTS, as Doc 6 did. |
| **The CB-7/8/9 record is not at HEAD.** | …C3 cites a backlog that does not exist. Reproduce first; the doc's best material is also its least verified. |
| **No live tracing.** `vm-trace` is hardcoded `LevelFilter::OFF`; `disasm` walks only the top-level chunk. | …a track whose subject is *switching* cannot show a switch. Recorded separately by the user, to be done later. Until then every dynamic claim needs the hoist-to-module-level disassembly workaround, and frame-level claims must be labelled INFERRED — as `frame-identity.md` §hard-trace already had to be. |
| **C4's subject may move.** | …the doc is written against a surface that changes under it. Re-plan C4 if Slice B lands. |
