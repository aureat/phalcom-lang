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

### Track: Concurrency — "Fibers & the restricted loop" — **in progress, 1/4**

Plan: [`CONCURRENCY-PLAN.md`](CONCURRENCY-PLAN.md). Order C1 → C2 → C3, C4 decided last.

| # | Doc | Grip | Commit |
|---|---|---|---|
| C1 | [`concurrency/restricted-loop.md`](concurrency/restricted-loop.md) | a switch is `mem::take` on four VM fields and the loop is never told — which is why it is O(1) *and* why it is illegal under a native frame | *(this pass)* |

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

---

## Owed

Ranked by how many shipped docs point at the gap. A forward pointer is a promise; these are the
unpaid ones.

### 1. Concurrency / fibers — **started; 3 docs left.** *(was 5 pointers, the largest debt)*

C1 is shipped (above) and paid Doc 4's Lie #2 — the `switch_pending` branch. Still owed:

- **C2 — the parked fiber.** The largest remaining debt-payer: Doc 6 quotes `store_live_into`'s
  four-field `mem::take` and ADR-0030 §6's `next_frame_generation` invariant *as given*, and Doc 3
  declared `VM::frames` a "live mirror" as its Lie #2. C1 used the swap as a one-line fact and
  explicitly deferred the mechanism here.
- **C3 — when a fiber fails.** Needs C1 and C2 in the reader's head first.
- **C4 — futures.** Decide at write time; may fold into another doc (plan §3).

One qualification C1 raised and did not own: [`upvalues.md`](vm/upvalues.md) owns the
`GetUpvalue`/`SetUpvalue` fiber-aware branch, which makes Doc 1's "the inner loop is fiber-unaware"
true of the loop's *structure* but not of two of its arms.

### 2. Sacred-selector inliner — 2 pointers

Doc 5 named `GuardBool`/`GuardBlock`, `compile_sacred_call`, and the override-epoch deopt; Doc 6
leaned on it hard for the "the `ifTrue` blocks are not blocks at runtime" trace. Self-contained and
ADR-0018-anchored — the cheapest of the three, and a good palate cleanser between tracks.

### 3. `SuperSend` — 3 partial pointers

Its own opcode (ADR-0040), walk starts *above* the receiver's class, uncached by deliberate decision
(DEC-IC-B). Docs 1, 4 and 5 each defer it. Smallest scope of the three — may be a section rather
than its own doc.

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
