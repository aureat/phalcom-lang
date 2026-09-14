# 2. Metaclass tower follows the parallel rule

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/object-model.md` §metaclass tower; `phalcom-core/src/universe.rs`, `class.rs`

## Context

Phalcom has a Smalltalk-80 object model: every object is an instance of a class,
and every class is itself an object — an instance of its **metaclass**. For
class-side (`static`) methods to inherit correctly along the superclass chain,
the metaclass hierarchy must run *parallel* to the instance hierarchy.

The current implementation hardwires the superclass of **every** metaclass to
`Class`. That breaks class-side inheritance: a subclass's `static` methods do not
resolve through its parent's class-side methods, because the metaclass chain is
flat instead of parallel.

## Decision

The metaclass tower obeys the **parallel rule**:

```
(X class).superclass  ==  (X.superclass) class
```

That is, the superclass of a class's metaclass is the metaclass of that class's
superclass. `Object class` (the top of the metaclass chain) has superclass
`Class`, closing the tower. `Behavior`/`Class`/`Metaclass` form the shared kernel
(see [ADR-0003](0003-introduce-behavior-kernel-class.md)), and `Metaclass` is an
instance of itself.

This is wired during bootstrap as an explicit ordered step:

1. Allocate kernel classes uninitialized.
2. Wire instance-of relationships.
3. Wire instance-side superclasses.
4. **Wire metaclass-side superclasses by the parallel rule.**
5. Create remaining core classes via the helper.
6. Install primitives.
7. Run `verify_invariants()` as a regression check.

## Consequences

- Class-side (`static`) methods inherit correctly, matching the language's
  intended Smalltalk semantics.
- Bootstrap gains an explicit metaclass-wiring step and a `verify_invariants()`
  routine that asserts the parallel rule holds — cheap, permanent protection
  against reintroducing the flat-chain bug.
- This is a **correctness fix**, not an optional refinement: it is the minimum
  required for static-method inheritance to work at all.
- Requires `Metaclass` to be modeled as an instance of itself, resolved via the
  allocate-then-wire ordering above (`PhRef::new_cyclic`).

> **Superseded (U2, 2026-07-11):** [ADR-0009](0009-handle-arena-heap.md)
> replaced `Rc<RefCell<T>>`/`PhRef::new_cyclic` with a `slotmap`-backed `Heap`
> and `Copy` `ClassId` handles. The allocate-then-wire ordering described above
> is unchanged in spirit but is now implemented as allocate-then-patch over
> `ClassId`s (`Universe::create_core_classes`,
> `phalcom-core/src/universe.rs`) — "instance of itself" is a handle pointing
> at itself, not an `Rc` cycle. This ADR's decision (the parallel rule) is
> implemented as of U2; `verify_invariants()` (`Universe::verify_invariants`)
> is the regression guard referenced above.
