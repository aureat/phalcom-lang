# Iteration protocol (proposed — untracked gap)

> **Promoted → [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)** + normative
> spec [`iteration.md`](../iteration.md) (2026-07-12). This draft is retained for
> history; the ratified version adds the `for` desugar, `break`/`continue`, the
> inliner interaction, and the `Fiber` (ADR-0030) relationship.

- Status: Superseded by ADR-0035 · no open-Q covers this; **blocks `for` implementation**
- Axis: closures-control (iteration/generators)

## Problem

`for x in coll`, `.each`, and `.map` desugar to block sends, but no selector
contract defines what makes a value *iterable*. `for` cannot be implemented until
that contract exists. Precedent: Wren uses `iterate(_)`/`iteratorValue(_)` (null
seed → advance → fetch); Smalltalk uses `do:`; Python exposes an explicit
`Iterator` object with `__next__`.

## Decision

**A two-selector cursor protocol, Wren-style — no allocated iterator object.**

- `iterate(_)` — given the previous cursor (or `None` to start), return the next
  cursor as `Some(cursor)`, or `None` when exhausted.
- `iteratorValue(_)` — given a cursor, return the element at it.

`for x in coll { … }` desugars to:

```phalcom
var _c = coll.iterate(None)
while (_c.isSome) {
  let x = coll.iteratorValue(_c.unwrap)
  { … }.call()
  _c = coll.iterate(_c)
}
```

`.each(_)`/`.map(_)` are `core.ph` defaults written *on top of* this protocol, so
one contract covers all iteration. Uses `Option` for the cursor — no surface
`nil`, consistent with Invariant 4.

## Precludes

- **External (pull) iterators** as the primitive — this is internal/cursor-based.
  A `Stream`/generator layer, if wanted later, builds on `Fiber`, not on this.
- Infinite lazy sequences without a `Fiber`-backed producer.
- Mutation-during-iteration safety is **not** guaranteed by the protocol; a
  collection may define fail-fast via a modification counter (separate decision).
