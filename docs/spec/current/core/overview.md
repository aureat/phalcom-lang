# Core Library — Overview

> **Status:** Orientation. A one-page map of Phalcom's core library: the shape,
> the commitments, and where to go next. For the class-by-class definition see
> [`core-classes.md`](./core-classes.md); for the doc-set axes see
> [`README.md`](./README.md). Inherits the baseline pin from `README.md`.

Phalcom's core library is a **hybrid native + self-hosted** library layered over a
**frozen primitive floor**.

## The shape

- **Kernel tower.** `Object → Behavior → {Class, Metaclass}` is the self-describing
  spine, wired by the metaclass *parallel rule*
  ([ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md) /
  [ADR-0003](../../../adr/0003-introduce-behavior-kernel-class.md)). Every other
  core class hangs off `Object`.
- **The floor.** [`floor-census.md`](./floor-census.md) — **80** native
  `(class, selector)` bindings frozen by
  [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md): closed,
  machine-checked (R-INV-0.1), "the default answer to adding a primitive is *no*."
  A capability goes native **only** when it reads representation below the `.ph`
  line (handle bits, an `f64` value, the method map).
- **The surface.** [`core.ph`](../../../../phalcom-core/core/core.ph) — written *in
  Phalcom* over that floor (`Object#isA`, the `Option` combinators, the `List`
  protocol). The language self-hosts above a small native boundary.

## The commitments

- **Absence is `Option`, never `nil`**
  ([ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md) /
  [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md); Invariant 4).
- **Booleans** are abstract `Bool` with `True`/`False` singletons
  ([ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md)).
- **Seven sacred selectors** (`Bool` control-flow + `Block#whileTrue`) are
  compiler-inlined with deopt guards
  ([ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md)).
- **Message-send is the only computational primitive**; everything is an object.

## State (post-U-CORE-1)

16/28 catalog rows exist (5 ✅ fully, 11 ◐ partial, 12 ❌ absent). The tower and
value/callable spine are present but thin; the greenfield is **collections beyond
`List`**, **all of errors** (U-CORE-6), and **all of concurrency** (out of scope).
Track head is **U-CORE-3** (callables/`Method` reflection). Full status matrix in
[`core-classes.md`](./core-classes.md) §10; the per-class delta in
[`catalog-delta.md`](./catalog-delta.md).
