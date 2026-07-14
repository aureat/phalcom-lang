# Multithreading feasibility — investigation notes

Status: exploratory, no ADR yet. Not in overlay's open/undecided table — fully open.

## Current committed position

- ADR-0030 (Accepted): cooperative single-threaded; `Fiber` sole concurrency primitive
  (heap object, O(1) pointer-swap switch). "No preemption ⇒ no data races **by
  construction**." This sentence is the entire safety argument — remove single-
  threading and nothing replaces it.
- `VM` owns exactly one `Heap` (`phalcom-core/src/heap/mod.rs:89`), a single
  `SlotMap<ObjRef, Object>`. Not `Sync`, no lock, no `Rc`/`RefCell` (ADR-0009).
- `VM.current`/`frames`/`stack` = live mirror of the running fiber; switching
  fibers is a `Vec` swap, not a real context switch.
- GC in flight (uncommitted at time of writing): non-moving mark-sweep,
  `SecondaryMap` mark table, stop-the-world, no write barrier
  (`heap/trace.rs`, `vm/gc.rs`). ADR-0050 chose non-moving specifically
  because handles must survive collection for parked fiber `Value`s under a
  **single-mutator** assumption.
- ADR-0026/0041: methods reopen at runtime under a coarse pristine-flag /
  override-epoch guard; ADR-0012's inline-cache-ready dispatch slots are
  still unpopulated. Both currently plain reads/bits — correct only because
  there is one mutator.

## Options considered

**A. Shared heap + locks (Java/Go/Rust-style OS threads)**
- `SlotMap` isn't thread-safe — every `Heap::get`/`get_mut` needs a lock or
  the arena needs sharding.
- Stop-the-world mark-sweep needs safepoints on *every* mutator thread;
  today's design has none (never needed them).
- Sacred-inline pristine flags and future ICs need atomics/epochs instead of
  plain bit reads.
- Cost: rewrite of heap, GC, and dispatch-guard layers simultaneously —
  touches ADR-0009/0012/0018/0026/0050 at once. Multi-month.

**B. Isolates / Ractor-style (disjoint heaps, copy-on-send)**
- Fits the existing Fiber-as-object shape: one `VM`+`Heap` per OS thread,
  message passing copies/serializes `Value`s across.
- Kernel bootstrap graph needs duplication per isolate or a read-only shared
  segment — new but *additive* bootstrap-ordering problem; doesn't touch
  existing single-isolate code.
- Precedent: Ruby went GIL → Ractor (isolates), not free-threading, because
  retrofitting shared-mutable safety onto code built single-threaded is
  near-impossible once it's leaked into the API/ecosystem (CPython's
  multi-year nogil effort is the model-A cost paid out).

**C. Keep the de-facto GIL**
- Current model already behaves like one giant single-lock interpreter.
  Gets zero real parallelism. Fine until FFI/native boundaries appear, then
  pays the same retrofit tax as CPython.

## Recommendation

Isolates (B) over shared-memory-with-locks (A):
1. Additive over Fiber/VM-per-isolate vs. A's simultaneous rewrite of
   heap+GC+dispatch.
2. Preserves "no data races by construction" instead of deleting it.
3. Cheaper to retrofit later than A, matching the Ruby precedent above.
4. Phalcom has no FFI/native-thread code yet — the window to choose isolation
   before anything assumes single-mutator access is open now, won't stay
   open once native bindings exist.

## What each choice precludes

- Isolates: no cheap shared-mutable-state API across threads — future
  concurrency must be message-passing shaped, not lock-shaped
  (`Arc<Mutex<T>>`-style primitives don't fit).
- Shared-memory+locks: forecloses ADR-0030's "no data races by construction"
  claim outright — would need a superseding ADR that explicitly walks that
  back, not a perf patch slipped in underneath it.

## Next step

Needs its own ADR before any code moves. Not blocking current GC/perf work
(U-GC); orthogonal axis.
