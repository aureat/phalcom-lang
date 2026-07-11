# 9. Object graph lives in a handle/arena heap

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/object-model.md` §6; `docs/spec/system.md` (gc); [open question Q4](../spec/open-questions.md); forge finding F5; [ADR-0002](0002-metaclass-tower-parallel-rule.md)

## Context

The kernel is a cyclic object graph: `Metaclass` is an instance of itself, and
the metaclass tower closes at the top ([Object Model §5–6](../spec/object-model.md)).
The current tree owns objects with `Rc<RefCell<T>>` and a `MaybeWeak` cycle-breaker
meant to keep the kernel from leaking. The forge audit found this model fights the
design on two fronts:

- **The cycle-breaker is inert (F5).** Every `set_class_owned` stores a `Strong`
  reference and `Metaclass.class` points at itself, so the weak path is dead code
  and the kernel is never freed.
- **`RefCell` is a panic surface.** A send that borrows a receiver and then
  dispatches into code that re-borrows it double-borrow-panics at runtime — the same
  class of latent hazard as F1.

Beyond correctness, the spec's Smalltalk semantics ultimately need a real
collector: object cycles are normal, `System.gc` is specified ([System §Runtime](../spec/system.md)),
and runtime `superclass=` ([open question Q4](../spec/open-questions.md)) means the
graph mutates after construction. `Rc<RefCell>` cannot host any of that cleanly.

## Decision

Objects live in a central **`Heap`** and are referenced by `Copy` integer
**handles** — `ObjRef` for heap objects, `ClassId` for classes. A handle is an
index into the heap arena, not a pointer; dereferencing goes through the `Heap`.

- No `Rc`, no `RefCell`, no `MaybeWeak`. The kernel cycle is expressed as handles
  that refer to each other with no ownership paradox.
- The bootstrap allocate-then-wire ordering ([Object Model §6](../spec/object-model.md),
  [ADR-0002](0002-metaclass-tower-parallel-rule.md)) becomes "allocate handles,
  then patch their fields" — the circular `Metaclass`/apex wiring is a set of
  handle assignments, not a `new_cyclic` dance.
- The handle API is designed so a tracing collector can later relocate or reclaim
  arena entries behind the same `ObjRef`/`ClassId` surface.

## Consequences

- **No Rc-cycle leak and no `RefCell` double-borrow panic surface** — the two
  concrete failure modes the audit found (F5 and the F1-class hazard) are removed
  by construction, not patched.
- **Inline-cache- and GC-ready.** Handles are stable keys, so they double as inline
  cache tags ([ADR-0012](0012-selector-signature-encoding-and-dispatch.md)); the
  arena is the natural home for a future tracing GC ([System §gc](../spec/system.md)),
  which can compact and relocate without invalidating handles.
- **Runtime hierarchy mutability stays open.** Because links are handles patched in
  place, `superclass=` ([open question Q4](../spec/open-questions.md)) remains
  implementable without unwinding an ownership model.
- Every dereference goes through the `Heap`, so hot paths thread a heap reference;
  this is the deliberate cost of removing pointer aliasing and it keeps object
  access uniform.
- A tracing collector is **not** built now — the arena is designed to host one, but
  scope is kept to ownership/allocation. Reclamation is deferred.

## Alternatives considered

- **`Rc<RefCell<T>>` + intentional process-lifetime kernel cycle** (the current
  substrate, with `Weak` only where needed). Simplest and matches today's code, but
  keeps the `RefCell` borrow-panic surface, still leaks user cycles, and offers no
  path to `System.gc`. Rejected as the design baseline.
- **An immediate tracing GC (`Gc<T>`).** Most Smalltalk-faithful — cycles,
  `System.gc`, mutable `superclass=` all fall out — but it is the heaviest lift and
  front-loads collector complexity onto work that does not yet need it. Rejected as
  too much scope up front; the handle heap is chosen precisely so this can be added
  later without an API break.
