# `docs/learn` — tracker

Index of the learner's course: what is shipped, what is owed, and who owes it.

Procedure for writing one is [`AUTHORING.md`](AUTHORING.md). Each doc is a full five-phase run;
its scratch (`recon.md`, `REQUIREMENTS.md`, `draft-concept.md`, `source-map.md`) lives in a working
folder named for the concept, and the shipped doc lands in its part folder.

> **Staleness rule.** The *shipped* column is durable — it names commits. The *owed* section is a
> record of intent and **rots**. Before picking anything up, verify against the tree rather than
> trusting a row here; this repo has burned two sessions on notes that claimed work was unmerged
> when it was merged, and vice versa. Update this file in the same pass that ships a doc, never later.

---

## Shipped

### Track: VM — "Execution & Dispatch" — **complete, 6/6**

A deliberate spiral. Ordering rationale: loop first (it supplies the grip), artifact before frame
(a frame points at a `Callable`), send before caches, identity last (it is a knot needing all five).

| # | Doc | Grip | Commit |
|---|---|---|---|
| — | [`vm/upvalues.md`](vm/upvalues.md) | capture names a slot, not an address | (first module; predates the track) |
| 1 | [`vm/execution-loop.md`](vm/execution-loop.md) | running a program is one `while` over a `match`; everything is one arm | `24f867a` |
| 2 | [`vm/compiled-artifact.md`](vm/compiled-artifact.md) | four layers, not three: `Chunk` ⊂ `Callable` ← `ClosureObject` ← `BlockObject` | `edfacf8` |
| 3 | [`vm/frames.md`](vm/frames.md) | a frame is a **value**, not an object — `Copy`, in a `Vec`, no parent pointer | `39e8e49` |
| 4 | [`vm/message-send.md`](vm/message-send.md) | a call site names a **selector**, not a method | `de49d3a` |
| 5 | [`vm/caches-and-fusion.md`](vm/caches-and-fusion.md) | resolve once per call **site**, not per call | `79e5a3e` |
| 6 | [`vm/frame-identity.md`](vm/frame-identity.md) | a `FrameToken` is a pointer split in two: *where to look* vs *who it was* | `603ff18` |

### Track: Concurrency — "Fibers & the restricted loop" — **COMPLETE, 4/4**

Plan: [`CONCURRENCY-PLAN.md`](CONCURRENCY-PLAN.md). Order C1 → C2 → C3, C4 decided last.

| # | Doc | Grip | Commit |
|---|---|---|---|
| C1 | [`concurrency/restricted-loop.md`](concurrency/restricted-loop.md) | a switch is `mem::take` on four VM fields and the loop is never told — which is why it is O(1) *and* why it is illegal under a native frame | `a457904` |
| C2 | [`concurrency/parked-fiber.md`](concurrency/parked-fiber.md) | a `FiberObject` is the set of buffers a fiber is *not* using; four of twelve fields move, and because they **move** rather than swap, every bug here is a move that did not finish | `66c1db5` |
| C3 | [`concurrency/fiber-failure.md`](concurrency/fiber-failure.md) | an error leaves a fiber twice over and the two exits are different machines — inside, a Rust `Err` that closes escaped capture cells before reclaiming their slots; at the floor, a value in a slot with the frames dropped in bulk. Containment is implemented as **deletion** | `9ebd67e` |
| C4 | [`concurrency/future-await.md`](concurrency/future-await.md) | `Future#await` is the only method in the core library that makes its own precondition fail — it probes "may I yield?" inside `.attempt()`, which is two native re-entrant frames, so the probe *is* the obstruction. It never parks a fiber on any path | `8bee47a` |

**C4 found its own plan wrong in the largest way yet.** The plan (and `CONCURRENCY-PLAN.md` §3, and
the doc's own candidate grip) assumed `async`/`await` were **unbuilt** Slice B. They landed
2026-07-14 in `06432bd`. Three records still say otherwise — `concurrency.md:187`,
`U-FUTURE/plan.md:109-110`, and `core.ph:1335-1338`, the last being a class doc comment asserting
the methods are "deliberately NOT built here" eleven lines above their implementations. The doc
became the opposite of what was planned: not a doc about a feature that was not built, but about one
that was built, is green in CI, and cannot run. Filed as
[E004](../errors/E004-await-cannot-suspend.md). Recon in [`future-await/`](future-await/).
Doc-kind gate: **knot** ⇒ Agent A skipped, B alone.

**C2 was the first run of the experimental [`AUTHORING-LEAN.md`](AUTHORING-LEAN.md)** — three phases,
one agent, scratch in [`parked-fiber/`](parked-fiber/) (`recon.md` + `source-map.md`, no
`REQUIREMENTS.md`, no `draft-concept.md`). Outcome logged in `AUTHORING-LEAN.md` §8. It also
**corrected two things it inherited**: C1/Doc 3/ADR-0030 all call the switch a "swap" when it is a
`mem::take` move, and recon's own retention finding was cut down by the source map (see §4.1 there).

**C1 corrected two things its own plan asserted**, both recorded in
[`restricted-loop/recon.md`](restricted-loop/recon.md) and the doc:
- The plan attributed the `each`-vs-`while` restriction to ADR-0018's inliner. Wrong as the general
  rule — `each` is **written in Phalcom** (`core.ph::Iterable#each`), and its `for` is the
  compiler's own frameless lowering, not the inliner. The line is drawn at **block invocation**
  (`Block#call` → `block_call`), not at native-vs-Phalcom code.
- Plan §6's headline question is answered: the typed switch signal **shipped**, so C1 is a fork
  doc — but as a `bool` field, not the ADR's `ControlFlow` return (deliberate; D-FIB-5).

### Track: Object model — **1 doc, no declared plan**

| Doc | Grip | Note |
|---|---|---|
| [`object-model/metaclass-tower.md`](object-model/metaclass-tower.md) | the tower is a finite cyclic graph, not an infinite regress | Ships with **no forward-pointer section**, so nothing declares what the rest of this track is. Deciding that is itself an open task. |

### Standalone

| Doc | Grip | Commit |
|---|---|---|
| [`vm/sacred-inliner.md`](vm/sacred-inliner.md) | every `if` is compiled **twice** — inlined fast path and real-send fallback, side by side in one chunk — and the guard is a forward jump between them. Deopt is free because there is no deopt; the bill is code size, a 2^depth compile blowup no gate measured, and two copies that do not agree | `92244a6` |
| [`vm/supersend.md`](vm/supersend.md) | `super` is the least dynamic thing a program can write, and is resolved dynamically **by name** on every call — three decisions each traded a static fact the compiler already had for a lookup the VM redoes forever | `72fadbb` |

**The Owed list is now empty of ranked gaps.** All three (concurrency 4/4, inliner, `SuperSend`) are
paid. What remains is unranked: the object-model track has no declared plan, and memory management is
named once by `execution-loop.md` and promised by nothing.

---

## Owed

Ranked by how many shipped docs point at the gap. A forward pointer is a promise; these are the
unpaid ones.

### 1. Concurrency / fibers — **PAID IN FULL, 4/4.** *(was 5 pointers, the largest debt)*

C1 paid Doc 4's Lie #2 — the `switch_pending` branch. C2 paid Doc 3's Lie #2 (`VM::frames` as "live
mirror") and Doc 6's borrowed `next_frame_generation` invariant. C3 paid C1's
`CannotYieldAcrossNativeFrame`-as-a-value pointer and C2's two-scars handoff. C4 paid C1's
`System.schedule`/ready-queue pointer and C3's "how a scheduled fiber's failure reaches the host."
No forward pointer into this track is now unpaid.

Three items the track raised and did not own, none assigned to a unit:

- **The `call`-mode cascade has no test coverage past its first hop.** Sixteen concurrency fixtures
  combine `Fiber.new` with `.call()`; none makes a fiber fail while resumed by a fiber that was
  itself resumed. The cascade loop's second-and-later iterations are untested at HEAD.
- **[E002](../errors/E002-fiber-floor-upvalue-crash.md) is open with an unverified fix direction**,
  and C3's finding is that the direction is larger than it looks: the failure path has no unwind to
  hook a per-cell step onto, so a fix must *introduce* the walk. C1's open question (narrowing the
  resume-side over-restriction) now carries this as a second cost — the guard is what keeps E002's
  family mostly unreachable.
- ~~**E004**~~ — **fixed in `f479189`**, immediately after C4 shipped. Three independent defects
  wearing one name: `await`'s self-defeating probe (fixed with a new floor binding, `Fiber#isRoot`,
  136 → 137 — the question had no answer in the language), a pump loop with no quiescence check, and
  a dead waiter left registered. C4's coverage finding — **no test in the corpus had a fiber await a
  pending future and later resume** — is closed by `concurrency_future_await_suspends.ph`. Note that
  E004's own recorded fix direction for the third defect was **wrong and unimplementable**; the
  working repair guards at `drain`. See [E004](../errors/E004-await-cannot-suspend.md) §The fix.

One qualification C1 raised and did not own: [`upvalues.md`](vm/upvalues.md) owns the
`GetUpvalue`/`SetUpvalue` fiber-aware branch, which makes Doc 1's "the inner loop is fiber-unaware"
true of the loop's *structure* but not of two of its arms.

### 2. Sacred-selector inliner — **PAID.** *(was 2 pointers)*

[`vm/sacred-inliner.md`](vm/sacred-inliner.md) — *every `if` is compiled twice and a guard picks
which copy runs; deopt is free because there is no deopt, and the bill is paid in code size, in a
compile-time blowup nothing measured, and in two copies that do not actually agree.* Doc 5's
`compile_sacred_call`/deopt handoff and Doc 6's "not blocks at runtime" lean are both discharged.

Raised and not owned:

- **[E005](../errors/E005-nonlocal-return-some-wrapped.md) — new, open.** A non-local `return`
  through the *non-inlined* `ifTrue` comes back `Some`-wrapped. Reachable with no override, from
  ordinary code, by hoisting a block into a `let`. The fix is a primitive-ABI signalling change, not
  a local patch, and every `block_call` post-processor is unaudited.
- **The inliner's runtime value has never been measured.** Every number attached to it in
  `perf-log/` is compile time. The one cut nobody ran.
- **Decision 0065 closed the threat the guard defends against** — kernel classes can no longer be
  reopened from surface Phalcom, so no program can make `GuardBool`'s override question answer "yes."
  The guard is still exercised in-crate. Nothing reconciles the two decisions.

### 3. `SuperSend` — **PAID.** *(was 3 partial pointers)*

[`vm/supersend.md`](vm/supersend.md) — *`super` is the least dynamic thing a program can write and is
resolved dynamically, by name, on every call.* TRACKER guessed it "may be a section rather than its
own doc"; it is doc-length, and the reason is that its three most interesting facts are all things
the plan did not know about. Docs 1, 4 and 5's pointers are discharged.

Raised and not owned:

- **[E006](../errors/E006-inherited-field-diagnostic-shadowing.md) — new, open.** Reading an
  inherited field reports `Read-before-write`; following that advice silently yields two slots
  (`Base sees: 7` / `Derived sees: 999`). Behaviour is spec-correct at every step — the defect is
  the diagnostic and the path it steers down. `ReadBeforeWrite` has zero tests anywhere.
- **The classic `super` correctness case has no fixture.** Lookup must start above the *lexically
  defining* class, not the receiver's; the wrong rule infinite-loops on a three-level chain with an
  inherited-but-not-overridden method. Phalcom is correct and nothing holds it correct — the four
  `super` fixtures all test cases where both rules agree. Four lines to close.
- **`SuperSend` caching** — DEC-IC-B, open. The honest sequencing argument is that it needs the
  general hierarchy-invalidation machinery anyway; build that first, then piggyback.
- **ADR-0040's option space is narrower than the real one** — its four alternatives debate *which
  opcode* and *what walk*, never *what to bake*. A forward cell or an install-time `home_class`
  field on the method are unweighed.

### Unowned but named in passing

- **Memory management / GC** — `execution-loop.md` gestures at it once ("the memory-management doc's
  subject"). No other doc promises it.
- **The rest of the object-model track** — undeclared, see above.

---

## Proposed: concurrency track

Grounded 2026-07-19 against HEAD. Two premises this proposal *started* with turned out to be wrong,
which is why the recon is recorded here rather than assumed:

- **`Future` is not native.** There is no `primitive/future.rs`. `Future` is written in Phalcom, in
  `core/core.ph` (~L1327-1469), as a plain `InstanceObject` — "zero new floor, zero native code, no
  `Fiber` dependency."
- **And it never suspends.** Only **U-FUTURE Slice A** shipped: `value(_)`/`error(_)` construct an
  *already-settled* future, so `then`/`map`/`catch` fire synchronously and `_waiters` is *always
  empty*. `async(_)`/`await` and the pending→settle drain are Slice B, gated on DEC-FUT-SCHED and
  deliberately not built.

Other load-bearing facts, verified: `Fiber` **is** native — nine primitives in `primitive/fiber.rs`
(`new`, `current`, `isDone`, `error`, `abort`, `call`, `try`, `resume`, `yield`). `VM::ready_queue`
exists (`vm/mod.rs` ~L134) and is drained in the dispatch loop (~L261). ADR-0030 is unusually rich —
seven numbered decisions **plus** an Alternatives section, so much of this track is a real fork
rather than the pedagogical reconstruction the VM docs kept having to admit to.

### The shape

**C1 — The restricted loop** *(fork)*. Cooperative vs preemptive, and then the interesting part:
ADR-0030 §4's "restricted (Option A)" re-entrancy model, and §5's **typed** switch signal — the ADR
explicitly rejected inferring a switch from a frame-length delta, which is a genuine deliberated
alternative with a named reason. Pays `message-send.md`'s Lie #2 (`switch_pending` firing inside a
primitive). Candidate grip: *a fiber switch is not a jump — it is a swap of which buffers the one
loop is looking at.*

**C2 — The parked fiber** *(mechanism)*. `store_live_into`/`load_live_from`: the four fields that
move (`frames`, `stack`, `open_upvalues`, `checking`) and, just as important, the ones that don't —
`next_frame_generation` stays behind, which is the invariant `frame-identity.md` had to borrow.
Fiber-stack pooling, and §7 (parked fibers are still GC roots). Pays Doc 3's Lie #2 and the fact Doc
6 leaned on. This doc discharges the most debt.

**C3 — When a fiber fails** *(tension — and the strongest of the three)*. Two committed features
collide: fiber teardown and upvalue closing. **The scars here are our own, reproducible, and unfixed
at HEAD** — an uncaught fiber failure drops a live stack without closing open upvalues, so an
escaped capturing block then indexes an empty stack; and `block_ensure` holds a pending result
unrooted across cleanup, where a GC during cleanup frees it. Also `fiber_abort`, `Fiber#error`, and
how `DeadFrameError` crosses a fiber boundary (which `frame-identity.md` set up).

> Most docs borrow their scar from another language. This one can show a live bug in its own
> subject, with a repro. That is rarer and worth more — **but it obligates a fresh reproduction at
> write time**: these are recorded in session memory as confirmed, and a grep of
> `docs/forge/DEFERRED.md` at HEAD does **not** find CB-7/CB-8/CB-9, so the intended write-up was
> either never committed or lives elsewhere. Re-verify before citing. See
> [`verified-diagnosis-is-not-a-verified-fix`] — on that backlog, 4 of 6 prescriptions were wrong
> even though all 6 diagnoses reproduced.

**C4 — Futures that cannot wait** *(honesty doc — write last, may collapse)*. The landed-vs-planned
spine Doc 5 already proved works: a `Future` in library Phalcom whose entire asynchronous half is
absent, whose `_waiters` list exists solely so Slice B is an additive change, and which is therefore
a *state machine* rather than a concurrency primitive. Genuinely interesting — "what does a future
even mean when it is always already settled" is a real question — but **thin**, and its scope
depends on whether Slice B lands first. If it does not, fold it into C1 as a section.

### Order, and why

**C1 → C2 → C3**, with C4 decided last. C1 first because the fork supplies the grip and the other
two are downstream of it. C2 second because it discharges the most inherited debt and C3 needs its
vocabulary. C3 last of the three because a tension doc requires both prior mechanisms in the
reader's head.

### Risks to settle in recon, not in the draft

- **Overlap with `upvalues.md`.** That doc already spent `Upvalue::Open { fiber, slot }`, the
  fiber-rooting trace arm, and the "second branch on the read path is the feature" line. C2 and C3
  both graze it. Same failure mode Doc 6 had to design around; handle it the same way — an explicit
  forbidden-list in REQUIREMENTS.
- **How much of ADR-0030 is shipped.** The ADR covers Futures at length and Futures are a quarter
  built. Assume nothing; the ADR-vs-HEAD gap is likely to be this track's recurring honesty note,
  exactly as `U-IC`'s plan was Doc 5's.
- **No live tracing.** `vm-trace` emits nothing (the CLI hardcodes `LevelFilter::OFF`) and `disasm`
  only walks the top-level chunk. A fibers track wants to *show* a switch happening and currently
  cannot. Worth fixing before C1 rather than working around it three times.
