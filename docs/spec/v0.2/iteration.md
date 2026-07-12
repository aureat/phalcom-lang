# Iteration

Part of the [Phalcom Language Specification](README.md). Status: Normative
(promoted from `experimental/iteration-protocol.md`, ratified by
[ADR-0035](../../adr/0035-iteration-protocol-cursor.md)).

Every form of iteration in Phalcom — `for`, `.each`, `.map`, `.filter`,
`.reduce` — bottoms out in **one** two-selector cursor protocol. A value is
iterable iff it implements that protocol; a user type opts in by defining two
methods and inherits every combinator for free.

## 1. The protocol

A value is **iterable** iff it answers:

| Selector | Given | Returns |
|---|---|---|
| `iterate(_)` | the previous cursor, or `None` to start | `Some(nextCursor)`, or `None` when exhausted |
| `iteratorValue(_)` | a cursor | the element at that cursor |

The cursor is an ordinary value — for `List` it is an integer index; for a tree it
might be a node. **No iterator object is allocated.** `Option` carries the "is
there more?" signal, so no surface `nil` ever appears
([Invariant 4](README.md); [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md)).

These two selectors are the only "magic methods" of iteration — the analogue of
Python's `__iter__`/`__next__`, but cursor-based rather than object-based. They are
**ordinary sends**, dispatched normally (§4).

```phalcom
class Countdown {
  construct from(n:) { _n = n }
  iterate(cursor) {
    let next = cursor.map { c => c - 1 }.unwrapOr(_n)
    return (next >= 0).ifTrue { Some(next) }.ifFalse { None }
  }
  iteratorValue(cursor) => cursor
}
for (x in Countdown.from(n: 3)) { System.print(x) }   // 3 2 1 0
```

## 2. `for` desugars to the cursor loop

`for (x in coll) { body }` lowers to a `while` loop over the protocol — **not** to
`coll.each { … }`. This is the authoritative desugar (it supersedes the
[control-flow.md](control-flow.md) §1 sketch):

```phalcom
var _c = coll.iterate(None)
while (_c.isSome) {
  let x = coll.iteratorValue(_c.unwrap)
  body
  _c = coll.iterate(_c)
}
```

`for` lowers to `while` (rather than `.each`) so that **`break`/`continue` work**
(§3) — they are loop-control on the `while`, which a block handed to `.each` cannot
express.

## 3. `break` and `continue`

`break` and `continue` are loop-control keywords for `for` and `while`, lowering to
jumps in the enclosing desugared loop — no block send, no floor primitive:

- `break` — leave the loop immediately.
- `continue` — skip to the next `iterate(_)` step.

```phalcom
for (x in xs) {
  if (x < 0) { continue }
  if (x > 100) { break }
  process(x)
}
```

Because `for` is `while` underneath, both compose with the cursor protocol without
any special handler.

## 4. Dispatch and the inliner

`iterate(_)`/`iteratorValue(_)` are **not** sacred selectors and are **not**
inlined ([ADR-0018](../../adr/0018-sacred-selector-inliner-and-override-guard.md)).
Only the loop *scaffold* — the `whileTrue(_)` skeleton and the `Option` `isSome`
test — is inlined to jumps. A `for` loop is therefore an inlined `while` skeleton
driving two regular protocol sends per step: cheap control flow, fully generic
iteration.

## 5. Combinators are `.ph` over the protocol

`.each(_)`, `.map(_)`, `.filter(_)`, `.reduce(_)` are `core.ph` defaults written on
top of `iterate`/`iteratorValue`, so one contract covers all iteration. `.each` is
the **full-traversal** form (no `break`/`continue`); `for` is the loop-control form.
`List` is the reference iterable ([collection-protocol.md](core/collection-protocol.md));
`Map`/`Set`/`Tuple`/`Range` and user types conform by implementing the two
selectors.

## 6. Relationship to `Fiber`

The cursor protocol needs **no** `Fiber`. A lazy or infinite sequence is produced
by a `Fiber`-backed generator, subject to the restricted-yield model
([ADR-0030](../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4): a
generator that `yield`s **under a non-inlined native callback** (a `block_call`
inside a combinator) raises `CannotYieldAcrossNativeFrame`, while a cursor-based or
inlined producer suspends freely. An external pull-iterator / `Stream` layer, if ever wanted, builds
on `Fiber` — not on this protocol.

## 7. Non-goals

- **External (pull) iterators** as the primitive — this protocol is
  internal/cursor-based.
- **Mutation-during-iteration safety** is not guaranteed by the protocol; a
  collection may add fail-fast via a modification counter (a separate decision).
