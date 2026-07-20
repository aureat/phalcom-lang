# 67. No user-visible shared-memory threads: fibers now, isolates if ever

- Status: Accepted
- Date: 2026-07-20
- Related: [ADR-0009](../adr/accepted/0009-handle-arena-heap.md) (handle/arena heap — the
  single-owner premise this ruling protects),
  [ADR-0030](../adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) (fibers as the
  sole concurrency primitive),
  [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) (precise root enumeration
  from per-VM state), [decision 0068](0068-io-is-future-shaped-reactor-owned.md) (the IO shape
  this ruling makes possible)

## Context

The question was never really open — it was decided implicitly by the heap and never written
down. Writing it down is the point of this decision, because the constraint is invisible from
the surface language and every future contributor will otherwise re-derive it or violate it.

The tree as it stands:

- `VM` owns exactly one `Heap` by value. `SlotMap` is not `Sync`.
- `Value` is `Copy`, but every `Value::Obj` is an index into *that* arena. A `Value` is
  meaningless outside its VM.
- ADR-0009's safety argument is single-owner: "No `Rc`, no `RefCell`." Its claimed consequence
  is that the interpreter carries no borrow-panic surface — true only because there is one
  owner.
- ADR-0050's collector enumerates roots precisely from `VM::stack` / `VM::frames` /
  `VM::temp_roots` — all per-VM. Collection runs at a dispatch-loop safepoint on the thread
  that owns the VM.

Shared-memory threads over that design require one of: a lock around the heap (which forfeits
the representation win the perf work is built on), per-thread heaps with cross-heap references
(which requires a distributed collector), or a rewrite of the heap, the collector, and every
primitive. None of these is a feature increment.

## Decision

### 1. Phalcom exposes no shared-memory threads

There is no `Thread` class, no `Mutex`, no `Atomic`, and no `spawn` that shares objects. In-process
concurrency is **cooperative fibers on a single VM thread** (ADR-0030), and that is the whole
surface.

### 2. Parallelism, if it is ever wanted, is isolates

One `VM` per OS thread, no shared object graph, communication over channels that carry **copied**
values and never handles. This is deliberately left unbuilt and unspecified here; the ruling is
only that isolates are the *door*, so nobody builds a shared-memory door by accident.

### 3. Internal worker threads are permitted, under one condition

The VM being single-threaded does not forbid the *runtime* from using threads invisibly — see
[decision 0068](0068-io-is-future-shaped-reactor-owned.md), which needs a filesystem thread pool.
The condition is absolute:

> **A worker thread must never touch `Value`, `ObjRef`, the heap, or any VM state.** It receives
> owned plain data (`Vec<u8>`, `PathBuf`, scalars) and returns owned plain data over a channel.
> Handles are minted and read only on the VM thread, at a safepoint.

This is [ffi.md](../spec/v0.2/drafts/ffi.md) §4's condition (c) — *the object graph never leaves
the VM* — reused verbatim. It is the same rule, and it earns its keep twice.

## Consequences

- `Value: Copy`, the non-moving arena, precise roots, and zero locking are all **preserved as
  ruled properties** rather than incidental facts.
- The reactor in 0068 need not be thread-safe with respect to the heap; only its completion
  queue crosses threads, and it carries plain data.
- `VM::ready_queue` stays a plain `VecDeque<ObjRef>` and is only ever touched on the VM thread.
- **The cost, named plainly:** no in-process CPU parallelism, ever, without isolates. A
  CPU-bound Phalcom program uses one core. Lua and JavaScript both accepted this trade; Python
  accepted the opposite and has spent over a decade undoing it (PEP 703).
- `native_reentry_depth` (ADR-0030 §4) remains a single-threaded reentrancy guard and needs no
  atomic treatment.

## Alternatives rejected

- **Shared-memory threads with a GIL.** The lock leaks into semantics and, once extensions or
  an ABI exist, into the contract. CPython is the worked example of how expensive that becomes
  to reverse.
- **Real shared-memory threads.** Not a rejected design so much as a different project: it
  invalidates ADR-0009, ADR-0050, and every primitive's `&mut VM` signature.
- **Deferring the ruling.** The failure mode is silent — IO and reactor code would get written
  with unexamined thread-safety assumptions, and the first violation would be found by a data
  race rather than by a reviewer.
