# C2 recon — the parked fiber

Procedure: [`AUTHORING-LEAN.md`](../AUTHORING-LEAN.md) (experimental variant, first run).
All line numbers are HEAD = `0ce6a9c`.

---

## 1. Architecture vs representation

**Architecture.** Cooperative, single-threaded, one-shot-at-a-time fibers, switched at explicit
`call`/`try`/`yield` sites; ADR-0030 §1/§4 (audit Option A, Lua-5.1 style). A `FiberObject` is one
more arena variant, `Object::Fiber(Box<FiberObject>)`, reached through `Value::Obj(ObjRef)` — no
`Value::Fiber` arm (ADR-0030 §2).

**Representation — the axis that matters.** The VM does **not** hold a pointer, handle, or index
into the running fiber's stack. It holds *the buffers themselves*:

```rust
// phalcom-core/src/heap/fiber.rs:67-78
pub struct FiberObject {
    pub stack: Vec<Value>,                          // "empty while running"
    pub frames: Vec<CallFrame>,                     // "empty while running"
    pub open_upvalues: BTreeMap<usize, ObjRef>,     // "empty while running"
    …
    pub checking: HashSet<ObjRef>,                  // "empty while running"
```

and the switch is four `std::mem::take`s in each direction
(`primitive/fiber.rs::store_live_into` @ L29-43, `::load_live_from` @ L49-59). So a fiber's state is
in exactly one of two places and is **never aliased**: either in `VM::{frames,stack,open_upvalues,
checking}` (it is running) or in its `FiberObject` (it is parked). `vm.current: ObjRef` names *who*
is running; it does not reach the state.

The consequence that decides the doc: **ADR-0030 §3 calls this "an O(1) pointer swap". It is not a
swap and there is no pointer.** It is a move, four times over, and a move abandoned halfway loses a
stack — see finding 4.

Why moving is legal at all: `CallFrame::stack_offset` is window-relative (ADR-0030 §3, D3), so a
per-fiber stack always based at index 0 needs no rebasing when it becomes `vm.stack`.

## 2. The grip, grounded

> **A `FiberObject` is not "a fiber" — it is the set of buffers a fiber is not currently using. The
> design is the partition (four fields move, eight stay resident, one counter must never become
> per-fiber), and because the four *move* rather than swap, a switch is a transaction: abandon it
> halfway and a whole stack is stranded.**

Three-way partition, all grounded:

| Class | Fields | Why |
|---|---|---|
| **Moves** (the four `mem::take`s) | `stack`, `frames`, `open_upvalues`, `checking` | see finding 1 — *two different reasons wear one uniform* |
| **Resident on the `FiberObject`, never mirrored** | `status`, `resumer`, `result`, `entry`, `started`, `resume_slot`, `floor_depth`, `resume_mode` | read/written *about* a fiber by whoever is running, so mirroring them would be wrong, not just wasteful |
| **VM-global, deliberately not per-fiber** | `next_frame_generation` (`vm/mod.rs:109`) | ADR-0030 §6 names relocating it into `FiberObject` as a violated invariant: global monotonicity is the only thing making a cross-fiber return token non-matching |

## 3. What was actually deliberated

ADR-0030 **did** deliberate, at the branch level, and says so in its own *Alternatives considered*
(L152-175): **B — full trampoline** (de-recurse every callback primitive; "not now", additively
reachable), **C — stackful coroutines** (rejected: `unsafe` stack switch, and every parked native
stack becomes a root a future moving collector must scan/relocate — "crown-jewel *stackful-fiber ⊗
moving-GC*"), **preemptive/multithreaded** (rejected: needs a memory model and locks), **resumable
Smalltalk-style suspension** (out of scope per ADR-0008).

So the design-space walk in this doc is **not** a pedagogical reconstruction at the branch level —
the branches were argued, and the GC argument in particular is the ADR's own. What *is*
reconstruction: the finer question of **which fields move**, which no ADR section deliberates. §3
asserts the swap; it never enumerates the set or defends its membership. Label that split honestly.

## 4. Findings that change the doc

**F1 — the four fields are not one decision.** `stack`/`frames`/`open_upvalues` move for a
**representational** reason: they are stack-indexed, and an index is meaningless against another
fiber's stack (`fiber.rs:73-78` — "kept per-fiber because it is stack-index-keyed"). `checking`
moves for a **semantic** one: it is an identity set with no stack dependence at all, moved because
an `@invariant`-guarded call can `yield` mid-body (`fiber.rs:107-117`, ADR-0052 Fix 1,
U-ANNOT-CONTRACTS). It is also the newest of the four and arrived as a *bug fix*, not as part of
§3's design. Two reasons, one `mem::take` block.

**F2 — the failure path clears three of the four.** `vm/dispatch.rs:319-321` clears `frames`,
`stack`, `open_upvalues` on a `Failed` fiber ("clear all three parked fields here") and does **not**
clear `checking`. `checking`'s contents are GC roots (`vm/gc.rs`, `checking` in the live root
enumeration; `heap/trace.rs:18` lists `Fiber.checking` among traced identity sets). A `Failed`
fiber can never resume, so this is retention, not a semantic bug — but it is a real asymmetry
between the swap set and the clear set. **Hand to B to confirm the trace arm.**

**F3 — ADR §7's "fibers are GC roots even when parked" is not what the root set does.**
`vm/gc.rs::collect_roots` pushes `*current` and `ready_queue`, not an enumeration of live fibers;
parked fibers are reached **transitively** (a callee's `resumer` link; a resumer's own parked stack
holding the callee `Value::Obj`). The ADR's invariant is satisfied *by reachability*, not by the
mechanism its wording implies. **Hand to B, adversarially.**

**F4 — "O(1) pointer swap" is falsified by a shipped regression test.** `primitive/fiber.rs:262-268`
validates the first-resume entry arity *before* `store_live_into`, with the comment "Doing this
after `store_live_into` was a real bug — see the regression golden
`fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph`." Under a genuine pointer swap an
early return after the swap is repairable by swapping back; under a move it is a lost stack. This is
the doc's thesis in one commit.

**F5 — the switch is asymmetric.** `store_live_into` runs on *every* park, but `load_live_from` runs
only on the **already-started** path (`fiber.rs:309`). First resume takes the resumer's state out
and then pushes a fresh entry frame into the emptied mirror (`fiber.rs:298-307`). "Swap" describes
one of the two resume paths.

**F6 — fiber pooling is off.** `FiberObject::new_entry_with_buffers`, `VM::fiber_pool`, and the
recycle in `dispatch.rs:277-286` are all `#[cfg(feature = "fiber-pool")]`, **not** default-on, and
`vm/mod.rs:230-233` records it as "measured net negative in whole-process A/B benchmarking
(perf-log, 2026-07-14)". The plan lists "fiber-stack pooling" as content; it must appear as a
*disabled, measured-negative experiment*, never as a feature.

**F7 — three shipped documents and the ADR all say "swap", and none of them is describing a swap.**
ADR-0030 §3 heads its section *"Fiber switch is an O(1) pointer swap"*; [`vm/frames.md`](../vm/frames.md)
Lie #2 (@ L270-277) says *"a fiber switch swaps the live and parked buffers as a unit"* and quotes a
field doc reading *"an O(1) pointer-free copy (a `Vec` swap)"* — **that quoted wording is not at HEAD
any more**; the field now reads *"empty while running — mirrored by `VM::stack`"*
(`heap/fiber.rs:67`). C1 inherited the word too. The operation is `mem::take` in one direction and
assignment in the other, twice — a **move**, and only *both halves together* look like a swap. This
is not pedantry: F4 is the bug that the difference produces. Discharging Doc 3's Lie #2 therefore
means correcting its wording, not just expanding it.

## 5. Forbidden list

> **Read C1 before writing, not after.** C1 §"The swap, in four fields" (L78-132) already spends
> more of C2's plan than the plan realised. It quotes `store_live_into` **in full**, lists the four
> fields, lands the "a running fiber's `FiberObject` is empty" surprise, gives the no-rebasing note,
> and concludes "`vm.current` is bookkeeping, not indirection." C1 also states the handover
> explicitly (L35-37): *"**C2** owns the four fields as a mechanism — what each one is, why
> `next_frame_generation` pointedly stays VM-global, how a parked fiber is a GC root."*
>
> **So the four-`mem::take` block is not C2's reveal — it is C2's premise.** C2 opens where C1
> stopped: *why each* field moves (F1's two reasons), the **eight that do not** (untouched by C1),
> the one counter that must not, and the fact that a move is not a swap (F4). If a draft section
> re-derives "a switch is four takes", cut it.

| Material | Owner | What C2 may do |
|---|---|---|
| `switch_pending`, the `Primitive`-arm reconciliation skip, the restricted-yield guard, `floor_depth`-vs-`native_reentry_depth`, `CannotYieldAcrossNativeFrame`, "the loop is never told" | **C1, [`concurrency/restricted-loop.md`](../concurrency/restricted-loop.md)** (shipped) | name in one line as a consequence; never re-derive. `floor_depth` appears here **only** as a resident (non-moving) field in the partition table |
| The four-`mem::take` block as a *revelation*; the `store_live_into` quote; window-relative `stack_offset` / no-rebasing; "`current` is bookkeeping, not indirection"; the O(1)-regardless-of-depth argument | **C1** §"The swap, in four fields" | cite as **established**, in one or two sentences, with a link. Re-quote `store_live_into` only if the sentence being made is about the *ordering* of the takes (F4), not about their existence |
| `Upvalue::Open { fiber, slot }`, the fiber-aware `GetUpvalue`/`SetUpvalue` branch, the fiber-rooting trace arm, "the second branch on the read path is the feature" | **[`vm/upvalues.md`](../vm/upvalues.md)** (shipped) | `open_upvalues` appears here only as *one of the four buffers that move* and *why a stack-index-keyed map must move*. No upvalue-read mechanism |
| `FrameToken`, generation counters, `DeadFrameError` mechanics | **[`vm/frame-identity.md`](../vm/frame-identity.md)** (shipped) | `next_frame_generation` appears only as *the field that pointedly does not move* — this doc pays back Doc 6's borrowed invariant by explaining the **non**-move, and stops there |
| `VM::frames` as a "live mirror" | **[`vm/frames.md`](../vm/frames.md)** Lie #2 | **this doc's debt to discharge.** Say plainly what the mirror is: not a copy, the buffer itself |
| Fiber failure, the cascade, `fiber_abort`, `Fiber#error`, upvalue-close-on-teardown crash ([`docs/errors/E002`](../../errors/E002-fiber-floor-upvalue-crash.md)) | **C3, future** | the `Failed` clear-set asymmetry (F2) is in scope here **as a fact about the swap set**; the crash and the unwind are not |
| The **execution-model** design space — A restricted / B full trampoline / C stackful coroutines — and the ADR's GC argument against stackful (parked native stacks as roots, conservative scanning vs. precise stack maps), Lua's history, coloring, Go's boundary, the Wren corpus | **C1** §§"The fork that was actually argued", "Lua", "coloring", "Go", "Wren" | **already spent.** C2's design space is a *different axis*: given interpreter-owned buffers, how does the interpreter reach the current fiber's — move / indirect / copy / one shared stack? Native stacks appear only as "C1 killed this branch" plus a link |
| `System.schedule`, the ready-queue, the root-drive pump, `Future`, `async`/`await`, Slice A/B | **C4, future** | one line at most; `ready_queue` may be named as a GC root |

## 6. Open risks

| Assumption | If wrong | Disposition |
|---|---|---|
| `checking` is traced from a *parked* fiber, so F2 is a real retention path | F2 shrinks to a tidiness note and the honesty pass must say so | **B question 2** |
| No live-fiber registry exists; §7's invariant holds only by reachability (F3) | the doc misstates the GC story on a doc-2 whose plan makes §7 explicit content | **B question 3, adversarial** |
| The named regression golden `fiber_first_resume_arity_mismatch_does_not_corrupt_resumer.ph` exists at HEAD and passes | F4 — the thesis — rests on a comment, not a test | **B question 4** |
| A fiber's `resume_slot`/`floor_depth` are never mirrored into the VM | the resident/moves partition (the grip) is wrong | **B question 5** |
| No tracing: `vm-trace` is `LevelFilter::OFF`, `disasm` walks only the top-level chunk | every dynamic claim must be labelled INFERRED or shown via an observable `.ph` program | standing; run programs, do not trace |

## 7. Doc-kind gate (phase 2)

**Kind: mechanism. Agent A is skipped. Phase 3b only — one agent.**

Answered from findings, not from the plan (which happens to agree). The tempting objection is that
ADR-0030 has a real argued *Alternatives considered* — but **C1 already spent that space in full**
(§"The fork that was actually argued": restricted / trampoline / stackful, plus the GC argument,
Lua, coloring, Go, Wren), and §5 of this file forbids C2 from re-walking it. A design space that a
shipped sibling owns is not *this* doc's design space. What is left for C2 — which of a fiber's
twelve fields move, which stay resident, which stays VM-global — is machinery with one implemented
answer and no occupants on other branches. That is the definition of a mechanism doc.

The residual axis (move / indirect / copy / one shared stack) is real but is **representation**, and
§4.1 already assigns representation-vs-assumption checking to the reconciliation pass, which runs
with or without A.

> **Procedure error, recorded rather than hidden.** A *was* dispatched, on the "when in doubt, run
> A" clause, at the same time as recon concluded "mechanism" — holding both answers at once, which
> the gate does not permit. The dispatch was wrong twice over: A's deliverable is the one thing the
> forbidden list bars, and a two-agent C2 is not a test of a variant whose §8 measures agent count.
> Its `draft-concept.md` is **discarded unread** so the run stays a clean single-agent mechanism
> run; its token cost is still reported in §8's log as spent. Reconciliation (§4.1) therefore runs
> in its *recon-assumption vs. B-ground-truth* form, which §4.1 says to run anyway.
