# 50. Reclamation is a non-moving precise mark-sweep collector

- Status: Proposed
- Date: 2026-07-13
- Related: [ADR-0009](0009-handle-arena-heap.md) (handle/arena heap — this ADR
  lands the collector it deferred); [ADR-0010](0010-tagged-value-enum.md)
  (`Value` repr; NaN-boxing deferral); [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)
  (inline-cache tags key on handles); [ADR-0013](0013-block-closure-upvalues.md)
  (upvalue cells / non-local return); [ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md)
  (fibers own heap-resident stacks); `docs/spec/v0.2/memory-management.md` (normative);
  `docs/spec/v0.2/system.md` §`gc`; [open question Q4](../spec/v0.2/open-questions.md)
  (`superclass=`); forge finding F5.

> **Provisional number.** `0050` was the next free slot at authoring time on a
> tree with live concurrent sessions. If a concurrent ADR claims `0050`,
> renumber this one — no cross-file index is edited by this ADR.

## Context

[ADR-0009](0009-handle-arena-heap.md) moved the object graph into a central
`Heap` (`SlotMap<ObjRef, Object>`, generational keys) *explicitly so a collector
could drop in later behind the `ObjRef`/`ClassId` surface* — and deferred
reclamation. That deferral has now bound: nothing is ever freed. Every
allocation grows the `SlotMap`; a long-running program or any allocating loop
(the 1M-fiber Skynet benchmark is the extreme) leaks without bound.
`System.gc` is specified (`system.md` §`gc`, returns `None`) but unimplemented.
Smalltalk semantics assume a real collector: cycles are normal, the kernel
*is* a cycle (`Metaclass` is an instance of itself), and `superclass=`
([Q4](../spec/v0.2/open-questions.md)) mutates the graph after construction.

A verification pass over the current tree established the constraints the
collector must honour — these are measured/confirmed facts, not assumptions:

- **Handle stability is load-bearing in three places.** Inline-cache tags key on
  `ClassId`/`ObjRef` ([ADR-0012](0012-selector-signature-encoding-and-dispatch.md)),
  `Value::value_eq` object identity *is* handle equality, and every suspended
  fiber holds live `Value`s across a switch. A collector that reassigned handles
  would break all three.
- **No finalizers exist.** There is zero `impl Drop` on the object graph, so a
  collector introduces no finalizer-ordering or object-resurrection hazard —
  provided it does not add one.
- **Fibers carry no native stack.** A `FiberObject` owns `stack: Vec<Value>`,
  `frames: Vec<CallFrame>`, `open_upvalues: BTreeMap<usize, ObjRef>`, plus
  `resumer`/`result`/`entry` handles ([ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md)) —
  all plain heap-resident Rust fields. The classic "stackful fibers ⊗ moving GC"
  trap (a native stack the collector cannot walk) does not apply; a precise
  tracer walks a fiber's saved state like any other object.
- **Roots are fully reified.** `VM::stack` and `VM::frames` are owned `Vec`s, not
  values stranded in Rust call-frame locals, so root enumeration is *precise* —
  no conservative stack scanning, no false retention, no `unsafe`.
- **`size_of::<Object>()` = 256 bytes (measured).** The `SlotMap` slot is sized
  to the fattest variant (`ClassObject`/`FiberObject`), so every string, range,
  or instance also costs 256 B — a cache-density tax on the hot `heap.get` path
  threaded through all dispatch.

## Decision

Reclamation is a **non-moving, precise, stop-the-world mark-sweep collector**
built directly on the existing `SlotMap`, in safe Rust, behind the current
`ObjRef`/`ClassId`/`Value` surface. No struct on the object graph changes shape.

1. **Non-moving.** Objects keep their `SlotMap` slot for life; sweeping removes
   dead keys (the slot returns to the free list and its generation bumps). This
   preserves the handle stability that inline caches, `==` identity, and fiber
   stacks depend on.
2. **Marks in a side table.** A `slotmap::SecondaryMap<ObjRef, ()>` holds the
   mark set — **no `mark` field is added to any `Object` variant.** Cleared each
   cycle.
3. **Precise tracing.** One exhaustive `match` over `Object` enumerates each
   variant's outgoing handles (an exhaustive match forces every future variant to
   declare its edges). `Value` children are visited through a `Value::as_obj()`
   accessor, **not** by matching `Value`'s arms — so NaN-boxing
   ([ADR-0010](0010-tagged-value-enum.md)) later touches that one accessor and
   leaves the collector untouched. Marking uses an explicit worklist, never Rust
   recursion (a deep list must not overflow the native stack).
4. **Sweep** is a single `SlotMap::retain(|k, _| marked.contains(k))` pass;
   the kernel is pinned and never swept.
5. **Stop-the-world, no write barrier.** The collector runs to completion at a
   safepoint. This needs no barrier, so `superclass=` ([Q4](../spec/v0.2/open-questions.md))
   and every field mutation stay barrier-free for now.
6. **Safepoint-latched.** Collection runs **only** at interpreter-loop safepoints,
   where `VM::stack`/`frames` are the complete root truth — never in the middle of
   a native primitive holding raw `ObjRef`s in Rust locals. `Heap::alloc` latches a
   `gc_pending` flag when the live-byte threshold is crossed; the dispatch loop
   services it at a back-edge.
7. **Temp-root escape hatch.** A primitive that holds a freshly allocated handle
   across a call that re-enters the interpreter (`send_dynamic` / `block_call` /
   `invoke_method_object`) protects it with a `vm.push_temp_root(h)` /
   `pop_temp_root()` scope (the Wren `wrenPushRoot`/`wrenPopRoot` model). The
   temp-root stack is a root.
8. **`System.gc`** forces one full mark-sweep and returns `None` (per
   `system.md` §`gc`). It runs **no finalizers**, performs **no compaction**, and
   **does not** change any handle. Deterministic and safe to call at any
   safepoint.
9. **Companion representation win (same unit).** `Box` the fat `Object` variants
   so `size_of::<Object>()` drops from the measured 256 B toward ~24–32 B,
   shrinking the arena and improving `heap.get` cache density. This is behind the
   `Object` enum and independent of the collector; it ships first.

The self-tuning threshold is Wren's: after a collection, `next_gc = live * grow`
(grow ≈ 1.5, floored). See `docs/spec/v0.2/memory-management.md` for the
normative root set, invariants, and phase contract.

## Consequences

- **Memory is bounded and `System.gc` is real.** The two things the ADR-0009
  deferral left open are closed, with cycles (including the kernel cycle)
  collected correctly — mark-sweep handles cycles by construction.
- **Handles stay stable**, so inline caches ([ADR-0012](0012-selector-signature-encoding-and-dispatch.md)),
  `==` identity, and suspended-fiber `Value`s all survive a collection unchanged.
- **No `unsafe`, no `RefCell`, no write barrier, no finalizers** — the collector
  adds no new panic surface and no new `Drop` hazard; a stale handle after sweep
  still resolves to the existing `dangling ObjRef` diagnostic path, not UB.
- **One standing obligation: the safepoint/temp-root audit.** The *only* way this
  collector drops a live object is a native primitive holding a fresh handle
  across a re-entrant send without a temp-root. That set is bounded and
  enumerable (≈46 re-entrant sites, ≈31 alloc sites in `primitive/`; the
  intersection is smaller) and auditing it is the substantive work of the unit,
  not the mark loop. `verify_invariants()` gains a post-GC kernel-liveness assert.
- **Forward-compatible, nothing foreclosed.** Generational (nursery + remembered
  set), incremental (tri-color the `SecondaryMap`), and even compaction (swap the
  `SlotMap` for a moving arena that keeps the `ObjRef`→slot indirection) can each
  be added later behind the same handle surface. To keep the *generational/
  incremental* retrofit cheap, all field/element mutation should be funnelled
  through a small set of choke-point methods now, so a future write barrier has
  one home.
- **`heap.get` gets denser** from the `Box`-ing win independent of collection
  cadence.

## Alternatives considered

- **Reference counting (`Rc`-style, with a cycle collector).** Rejected. The
  kernel is a cycle, so plain counting cannot free it (this is finding F5, the
  reason [ADR-0009](0009-handle-arena-heap.md) dropped `Rc`); a bolt-on cycle
  collector reintroduces the tracing we would build anyway. Counting would also
  tax every `Copy` of a `Value` with a refcount adjustment, destroying the "`Value`
  moves freely, no clone/refcount" invariant of [ADR-0010](0010-tagged-value-enum.md).
- **Copying / compacting (moving) collector.** Most memory-efficient and
  defragmenting, but it reassigns object addresses — which breaks inline-cache
  tags, `==` identity, and the `Value`s parked in suspended fiber stacks, unless
  a handle-indirection layer is added. The `SlotMap` already gives stable keys for
  free; moving would fight it for a benefit (compaction) not yet needed. Kept
  reversibly open, not taken now.
- **Immediate generational / incremental collector.** The right *eventual* answer
  for allocation-heavy workloads, but both need a write barrier at every mutation
  site — invasive, and premature: the measured bottleneck on the allocation-heavy
  benchmark is **dispatch** (the per-send `IndexMap` probe), not GC pause. Ship
  the simple non-moving collector first; escalate only against a measured pause.
- **Keep deferring reclamation.** Rejected: the heap grows without bound today and
  `System.gc` is a specified no-op stub. The deferral has served its purpose (it
  let the object model land first); it is now the blocker.
