# 26. Methods are open; superclass reparenting is sealed

> **RETIRED 2026-07-19 by [PDR-0001](../../pdr/0001-classes-are-closed.md).**
> **Axis 1 is reversed** — classes are closed after definition; class reopening is removed
> from the language. **Axis 2 is unchanged and strengthened** — the superclass link stays
> sealed. This ADR's own rejection of "fully sealed (Wren)" reasoned that *"Axis 1 is free
> here, so there is no reason to forbid it"*; that priced dispatch cost only, and the
> namespace cost (two modules' same-named classes silently collapsing into one `ClassId`)
> turned out to be larger. ADR-0018's override-epoch guard, cited here as the thing that
> made Axis 1 free, is **retained** — it backs bootstrap and the deferred reflection layer,
> just no longer this argument. See 0065 for the verified mechanics.
>
> This file stays in `accepted/` pending the `docs/adr` → `docs/pdr` migration; its
> status line above is authoritative, not its path.

- Status: Retired (superseded by PDR-0001, 2026-07-19)
- Date: 2026-07-12
- Related: [ADR-0011](0011-static-instance-slot-layout.md) (static per-class slot layout),
  [ADR-0017](0017-class-side-stored-static-fields.md) (class-side field offsets up the tower),
  [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) (override-epoch deopt guard),
  [ADR-0009](0009-handle-arena-heap.md) (handle heap — makes a future reshape implementable),
  `docs/spec/current/object-model.md`, `docs/spec/current/open-questions.md` Q4

## Context

Open-question Q4 asks whether the class hierarchy is mutable at runtime
(Smalltalk: `Circle.superclass = Shape` is legal) or sealed after definition
(Wren: no). The question actually bundles **two axes** with very different costs in
Phalcom's VM:

- **Axis 1 — reopening a class to add or replace methods** (monkeypatching).
- **Axis 2 — reparenting: changing a class's `superclass` after definition.**

Two shipped invariants set the costs:

- [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) already ships an
  **override-epoch deopt guard**: overriding a method bumps an epoch and the inliner's
  caches re-check. So **Axis 1 is already cheap and supported** by construction.
- [ADR-0011](0011-static-instance-slot-layout.md) /
  [ADR-0017](0017-class-side-stored-static-fields.md) compute field offsets from the
  **ancestor chain** at definition time, with a proven offset-stability-up-the-tower
  invariant. **Reparenting (Axis 2) shifts every slot offset** in the subtree,
  invalidating live instances' memory layout — that is instance migration, not a cache
  bump.

## Decision

**Split the axes: methods are open, the superclass link is sealed.**

- **Axis 1 — open.** A class may have methods added or replaced after definition. This
  reuses the ADR-0018 epoch mechanism: the change bumps the override epoch and inline
  caches re-validate. No new machinery.
  ```phalcom
  class Circle { area => 0 }
  Circle.define("area") { 3.14 * _r * _r }   // legal; bumps epoch, caches re-check
  ```
- **Axis 2 — sealed.** A class's `superclass` is fixed at definition. Reassigning it at
  runtime is an error.
  ```phalcom
  class Circle < Shape { }
  Circle.superclass = Rectangle              // ERROR: reparenting is sealed
  ```
- **Door left open (explicitly non-foreclosed).** Reparenting is sealed *by policy*,
  not by impossibility — [ADR-0009](0009-handle-arena-heap.md)'s handle heap keeps it
  implementable. A future ADR may add an **opt-in `reshape` primitive** that reparents
  *and* migrates live instances (recompute offsets, rebuild affected objects, bump the
  epoch). Sealing now is forward-compatible: *adding* mutability later breaks nothing.

## Consequences

- **Slot layout stays provably stable.** Because the ancestor chain never moves, the
  ADR-0011/0017 offset-stability proof holds unconditionally; inline caches key on
  method epoch (already there), not on a mutable hierarchy.
- **The live, message-y feel is kept where it is cheap.** REPL-driven "patch this
  method and retry" works (Axis 1) — the part of the Smalltalk experience that matters
  most in practice — without paying for the part that is dangerous and rare.
- **Reparenting requires a deliberate, migrating primitive.** You cannot silently
  corrupt live instances by reassigning a superclass; the only path to reparenting is a
  future op that *knows* it must migrate. This is the honest reflection of the cost, not
  a limitation forced by laziness.
- **Roughly the Ruby model.** Open classes (add/replace methods), but a class's
  superclass cannot be changed — a well-trodden, understood point in the design space.

## Alternatives considered

- **Fully sealed (Wren).** Fastest invariants, but loses live method redefinition — a
  large part of the Smalltalk-lineage feel Phalcom targets. Rejected: Axis 1 is free
  here, so there is no reason to forbid it.
- **Fully mutable (Smalltalk).** Maximal reflective power, but reparenting directly
  contradicts the ADR-0011/0017 fixed-offset design (every reparent walks and rebuilds
  live instances) and forces inline caches to guard the ancestor chain. Rejected as the
  *default*; preserved as a future opt-in `reshape` primitive.
- **Seal methods too, allow only construction-time definition.** Removes the epoch
  mechanism's whole reason to exist and kills REPL patching. Rejected.
