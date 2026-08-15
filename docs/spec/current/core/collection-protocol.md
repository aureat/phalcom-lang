# Specification — Collection Protocol (the shared sequence contract)

> **Status:** **Normative contract**, dispatch-ready via
> [U-CORE-5](../../../forge/units/U-CORE-5/ucore5.md) (not yet landed). The
> shared selectors + **laws** every collection must satisfy — with kernel `List`
> as the reference implementation. Adds **zero** floor primitives (no
> [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) amendment).
> Encodes [`decisions.md`](./decisions.md) **Q5** (mutability + equality +
> hashability). Depends on **U-CORE-1** (`isA(_)`, `hash`). Inherits the baseline
> pin from [`README.md`](./README.md).
>
> **Owner:** U-CORE-5.

## 1. Why a contract, not classes

Per [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md) each
collection (`Map`/`Set`/`Tuple`/`Range`) is its own unit. What makes them a
*family* rather than four unrelated classes is a **shared protocol + laws**. This
doc fixes that contract so a new collection is "correct" iff it passes the
conformance harness — not by inspection.

## 2. The sequence protocol

The minimal ordered-collection surface, already satisfied by `List`
([`core.ph`](../../../../phalcom-core/core/core.ph); [`core-classes.md`](./core-classes.md) §6):

| Selector | Meaning | Law |
|---|---|---|
| `size` | element count, a real `Number` | **totality** — always defined, never raises |
| `at(_)` | element at an index → `Option` | in-bounds ⇒ `Some(v)`; out-of-bounds ⇒ `None` (no `nil`, no raise) |
| `add(_)` | append; returns `self` | mutation returns the receiver so calls chain |
| `each(_)` | apply a 1-arg block to each element | **deterministic iteration** — same order every traversal |

Derived combinators (`map`/`filter`/`reduce`/`includes`/`isEmpty`) are `.ph` over
these four (U-STD) and inherit their laws — they are **not** part of the minimal
contract, but a conformant collection gets them for free once the four hold.

## 3. Laws

1. **Totality.** `size` and `each` are defined for every valid receiver; `at(_)`
   is total via `Option` (out-of-bounds → `None`, never a raise or `nil`,
   [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)).
2. **Deterministic iteration.** `each` visits elements in a fixed order; two
   traversals of an unmutated collection agree. (For unordered collections —
   `Set`/`Map` — "fixed order" means *stable within a run*, not sorted.)
3. **Structural equality.** `a == b` iff same size and pairwise-`==` elements in
   iteration order; `==` is reflexive/symmetric/transitive. `!=` is its negation.
4. **Hashability-iff-immutable.** A collection is hashable **by value** iff it is
   immutable. Mutable collections (`List`) are **not** value-hashable — they
   inherit identity `Object#hash` (so `a == b` may hold while `a.hash != b.hash`
   for two structurally-equal mutable lists; this is intended). Immutable
   collections (`Tuple`) hash by value, consistent with structural `==`.

> **`List` is mutable** ⇒ identity hash, structural `==`. It is the reference
> implementation for laws 1–3; law 4's immutable branch is exercised by `Tuple`.

## 4. `List#==` / `!=` (the reference `.ph`)

U-CORE-5's only code artifact beyond the harness — structural equality on `List`,
`.ph` over the floor (guarded by `isA` so a non-`List` argument is `!=`, not a
type error):

```phalcom
class List {
  ==(other) {
    other.isA(List).ifFalse { return false }
    (self.size == other.size).ifFalse { return false }
    var i = 0
    while (i < self.size) {
      (self.at(i) == other.at(i)).ifFalse { return false }
      i = i + 1
    }
    return true
  }
  !=(other) => (self == other).not
}
```

## 5. Conformance harness

A reusable test harness **keyed by "the collection under test"**: given a factory
and sample elements, it asserts laws 1–4 mechanically. Every future collection unit
(`Map`/`Set`/`Tuple`/`Range`) instantiates it instead of hand-writing law tests —
the harness *is* the definition of "conformant."

## 6. Traceability

| Claim | Source |
|---|---|
| Contract, harness, `List#==`, Q5 | [U-CORE-5 as-built](../../../forge/units/U-CORE-5/ucore5.md); [`decisions.md`](./decisions.md) Q5 |
| Each collection its own unit | [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md) |
| Family representation + literals ratified | [ADR-0032](../../../adr/0032-collections-representation-and-literals.md) |
| `at(_)`→`Option`, no `nil` | [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md); [values-and-absence.md](../values-and-absence.md) |
| `hash`/`isA` dependency | [U-CORE-1 as-built](../../../forge/units/U-CORE-1/ucore1.md) |
