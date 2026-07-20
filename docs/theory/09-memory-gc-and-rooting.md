# 09 — Memory, collection, and the root set

> **Thesis:** a garbage collector is not primarily an algorithm. It is a *claim about
> reachability*, and every bug in a collector is a place where the claim was wrong — a live
> object nobody could name, or a name nobody thought to look at. The algorithm is the easy
> half. Enumerating the roots, exhaustively and durably, is the hard half, and it is the half
> that cannot be tested into correctness.

---

## 1. The ownership decision, made before the collector existed

**`[V]`** Objects live in a central arena `Heap` and are referenced by `Copy` integer handles
(`ObjRef` = 32-bit index + 32-bit generation; `ClassId`). No `Rc`, no `RefCell`, no weak
references, and **no `unsafe` for the object graph**.

**`[V]`** The alternatives and their stated costs:

- **`Rc<RefCell<T>>` with a process-lifetime kernel cycle.** Keeps the borrow-panic surface, still
  leaks user cycles, and offers no path to a real `System.gc`.
- **An immediate tracing collector.** Most faithful to the semantics, heaviest lift, front-loads
  collector complexity before the VM is correct. Deferred *behind the same handle API* so it could
  be added later without touching call sites.

The second point is the transferable one. Choosing handles was not choosing a collector; it was
**choosing to keep the collector decision open**. Two hazards were removed by construction rather
than patched: reference-cycle leaks (the kernel *is* a cycle) and double-borrow panics.

**`[V]`** The cost, stated: dereferencing is a bounds-checked table lookup rather than a pointer
chase, and the arena must be threaded through every function that touches an object. For a
language runtime — where the object graph is arbitrary and cyclic by definition — this is close to
a free win, and it is the same move that made the metaclass tower constructible (see
[`03`](03-object-model-and-the-metaclass-tower.md) §3).

---

## 2. Why the collector is non-moving, and what that later bought

**`[V]`** The shipped design: stop-the-world, **non-moving**, precise mark-sweep over the existing
slot map, in safe Rust, behind the current handle surface. Marks live in a side table; tracing uses
an exhaustive `match` with an explicit worklist; the sweep is a `retain`; collection is
safepoint-latched; a temp-root escape hatch exists; `System.gc` runs no finalizers, performs no
compaction, and changes no handle.

**`[V]`** Rejected, with reasons worth keeping:

- **Reference counting plus a cycle collector** — the kernel *is* a cycle, so a bolt-on cycle
  collector reintroduces tracing anyway, and refcounting would tax every `Copy` of a value.
- **Copying/compacting** — reassigns addresses, breaking inline-cache tags, identity equality, and
  values held in suspended fiber stacks.
- **Generational or incremental now** — both need a write barrier at every mutation site, and the
  measured bottleneck was dispatch, not pause time.
- **Continue deferring reclamation** — the heap grows without bound.

**`[V]`** Non-moving keeps handles stable, which is what lets caches, identity comparison, and
parked fiber values survive collection untouched. And a much later decision turned that choice
into a **security** property: a native byte-buffer type with a `zeroize` contract observes that a
moving collector "copies live objects and scatters stale secret images no `zeroize` can reach."
Any future record reopening the moving-collector door now has to address that coupling.

**A decision's consequences keep arriving.** The GC choice was argued on cache stability and
implementation cost; it turned out to also be the precondition for a memory-hygiene guarantee
nobody had contemplated. This is the positive-valence twin of the deferral-decay effect in
[`04`](04-values-absence-and-representation.md) §2.

---

## 3. The real hazard: fresh handles across allocation chains

**`[V]`** This is the headline correction of the entire garbage-collection audit, and it overturned
the team's own prior belief.

The belief was that the hazard is **re-entrancy** — user code running inside a primitive. The audit
found seven hazard sites sharing a completely different shape: `stack.pop()` or `split_off()`
removes a value from the root set, and then **several allocations occur while the result is held
only in a Rust local**. One message-reification path chains six allocations that way. The Rust
local is invisible to the collector; any of those allocations can trigger a collection; the value
is freed while still in use.

**`[V]`** The fix is structural rather than site-by-site. **Invariant L**: `Heap::alloc` may only
*latch* a "collection pending" flag — it may **never collect**. Collection happens exclusively at
the dispatch loop's back-edge safepoint, which is the one point where the root set is coherent.

This latched-collector pattern is the single most portable idea in this file. It converts an
open-ended obligation ("every allocation site must have rooted everything reachable") into a
bounded one ("the root set must be correct at one known program point"), and it makes the
correctness argument a property of the *loop* rather than of every author who ever writes a
primitive.

**`[V]`** The escape hatch for the residual cases is a `temp_roots` stack with a
**depth-and-truncate** API: save the depth, push what must survive, do the dangerous work,
truncate back. Depth-and-truncate rather than push/pop pairs, because truncation is
exception-safe — an early return or a raise cannot leave the stack unbalanced.

---

## 4. Root enumeration, and why auditing does not scale

**`[V]`** An audit found **two untraced edges and three missed roots** in a system that had already
been reviewed:

- `Object::Block.closure` — the sole retainer of closures passed around as values, and **entirely
  absent from the specification's own trace table**.
- `Upvalue::Open`'s fiber handle — the spec claimed it aliases a live stack slot that is already
  traced. False: the slot lives on that fiber's stack *only while that fiber is current*.
  Otherwise it is parked inside the fiber object.
- Missed roots: the sealed-class map, the invariant-checking set, and the scheduler's ready queue.

**`[V]`** The most instructive detail is how the worst one escaped. The ready queue — which holds
fibers scheduled but never resumed, reachable from nowhere else — survived a regex audit **because
the field was `pub(crate)` and the search pattern required `pub`**.

**`[V]`** The fix was not a better audit. It was **making the compiler the auditor**: exhaustive
destructures in the root-collection and edge-enumeration routines, so that adding a field without
classifying it *fails the build*.

And a subtlety that generalizes to any exhaustiveness-based safety argument:

> A wildcard-free `match` catches new **variants**. Only destructuring catches new **fields on
> existing variants**.

**The rule:** prefer a construct that fails the build over a discipline that fails silently. A
review process catches what a reviewer thought to look for; a type error catches what nobody
thought about at all. This is the same principle behind the exhaustive opcode `index()` match that
prevents silent histogram corruption, and behind selector encoding funneling through one helper.

---

## 5. A passing test that passed for the wrong reason

**`[V]`** The `ensure` use-after-free is worth studying not for the bug but for its diagnostics.

`block_ensure` held the protected block's outcome only in a Rust local while running the cleanup
block. Cleanup re-enters the interpreter and can therefore hit the back-edge safepoint. Fix: save
the temp-root depth, root the outcome value or the error handle, run cleanup, truncate.

The instructive part: **the error-path test passed while the ok-path test crashed** — because the
exception machinery incidentally kept the error value reachable through the frames. The passing
test was not evidence of correctness on its own path; it was evidence that a *different* mechanism
happened to be holding the value.

> A passing negative control can be passing for the wrong reason.

This is the same epistemics as
[`00-provenance-and-citation-discipline.md`](00-provenance-and-citation-discipline.md) §R4: ask
what would look different if the claim were false. Here, the ok-path and error-path tests looked
equally green right up until one of them didn't, and nothing in the test *design* distinguished
"passes because rooted" from "passes because incidentally reachable."

---

## 6. Concurrency changes the root set

**`[V]`** Parked fibers are roots. The scheduler's ready queue holds fibers reachable from nowhere
else. The invariant-checking set holds receivers under an `@invariant` guard and must be rooted
too.

**`[V]`** And this is the load-bearing argument for keeping per-fiber stacks *inside the arena
object* rather than in native memory: it is what makes them discoverable by a tracing collector at
all. The concurrency design and the collector design are the same design viewed from two angles —
see [`01`](01-coroutines-and-the-suspension-problem.md) §6.

**`[V]`** A related honesty note recorded in the concurrency documentation: a decision record's
section on this reads as a *specification of mechanism* ("fibers are GC roots"), but the
implementation has **no registry of live fibers**. It knows about the current fiber, about a queue,
and about edges. Reachability is achieved transitively rather than by enumeration — which is
correct, and is not what the prose describes.

---

## 7. Tuning, measured

**`[M]`** One heuristic that paid, and it is a nice example of a cheap, well-targeted idea. Rather
than a uniform "next collection at 1.5× live," compute a **yield ratio**
`(before − live) / before`. If it is under 10% — meaning the heap is dense with live objects and
the collection was largely wasted — grow the threshold by 4.0× instead. Measured: −11.7% on the
concurrency benchmark, −7.4% on fiber churn, −0.6% on a garbage-heavy tree benchmark, +0.8%
(noise) on a loop benchmark, resident memory stable.

The shape of the idea: **back off exactly on the workloads where the collector cannot pay**, and
leave the rest alone.

**`[X]`** Two that did not pay, both already catalogued in
[`08-performance-epistemology.md`](08-performance-epistemology.md) §5 — fiber-buffer pooling and
pre-sizing — and both failing by the same mechanism: capacity retained in shells that outlive their
run. Worth repeating here because it is a genuinely non-obvious memory result: **a bounded pool can
have an unbounded, linear-in-workload cost**, if what it retains is held by objects whose lifetime
it does not control. The pool was capped at 100 entries and cost ~450 bytes *per fiber created*.

**`[M]`** And the diagnosis that reframed the whole area: resident memory on the concurrency
benchmark is dominated by roughly a million **immortal fiber shells** in the slot map, because no
collector was running at the time. Buffers were second-order.

> **A local optimization cannot move a global constraint.** Before optimizing an allocation, check
> whether the thing you are optimizing is what is actually resident.

---

## 8. Layout arithmetic

**`[V]`** A slot map sizes every slot to its **fattest variant**. The class object at 280 bytes was
the fattest, so leaving it inline taxed every string, tuple, and instance on the hot path. Hence
selective boxing of the fat arms — an explicit companion to the collector work, targeting
280 B → ~40 B.

**`[M]`** And the comparative numbers that explain a 2.0× resident-memory ratio against the
reference implementation: 16-byte values versus 8, and 96-byte call frames versus 24. Fiber stacks
are stacks of values, so both multipliers apply to every live frame in the system.

**The rule:** in any uniform-slot arena, the largest variant sets the price for every allocation of
every type. The diagnosis is hard (a slow lookup on unrelated types) and the fix is easy (box the
outlier), which is exactly the ordering that makes it worth knowing in advance.
