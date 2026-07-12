# Collections

Every collection in Phalcom — ordered or not, mutable or not — answers the same
two-message cursor protocol, and every loop and combinator you'll ever write is
built on top of it.

For the exhaustive grammar and laws, see
[Iteration](../spec/v0.2/iteration.md) and
[Collection Protocol](../spec/v0.2/core/collection-protocol.md). This page is
the tour.

## The literals

```phalcom
[1, 2, 3]        // List  — ordered, mutable
{ a: 1, b: 2 }   // Map   — hash map, bare-identifier keys are symbols
(3, 4)           // Tuple — fixed-arity, immutable
Set(1, 2, 3)     // Set   — via a constructor send, not a literal
1..5             // Range — inclusive of 5 (reserved, not yet active)
1...5            // Range — exclusive of 5 (reserved, not yet active)
```

`List`, `Map`, and `Tuple` literals are live today — the parser desugars each to
an ordinary construction send, so there's no new runtime behavior to reason
about:

```phalcom
[1, 2, 3]        ≡  List.new().add(1).add(2).add(3)
{ a: 1, b: 2 }    ≡  Map.new().at(#a, put: 1).at(#b, put: 2)
(3, 4)            ≡  a Tuple construction over the immutable arm
```

`Set` and `Range` are further along the design than the syntax: their classes
and semantics are committed, but the *sigils* are reserved-inactive —
`#{1, 2, 3}` and the bare `a..b`/`a...b` operators parse to nothing yet. That's
deliberate, not an oversight: a bare `{1, 2, 3}` would be indistinguishable
from a block, so a set literal needs its own sigil (`#{...}`), and it isn't
wired up. Until it is, build a `Set` the same way you'd build anything else —
a constructor send: `Set.new()` or `Set(1, 2, 3)`. `Range`'s `a..b`/`a...b`
sigils are reserved with their meaning already fixed (inclusive / exclusive —
see below) so activating them later is a parser change, not a fresh design
question.

| Literal | Status | Notes |
|---|---|---|
| `List` `[a, b, c]` | ships | mutable, identity-hash |
| `Map` `{a: 1, b: 2}` | ships | `{}` stays the empty block; empty map is `Map.new()` |
| `Tuple` `(a, b)` | ships | comma disambiguates from `(a)` grouping; `()` is empty |
| `Set` `Set(1, 2, 3)` | send, not a literal | `#{...}` sigil reserved-inactive |
| `Range` `1..5` / `1...5` | reserved-inactive | construct via `Range`'s constructor for now |

`Tuple` is immutable, which under the collection-protocol's
hashability-iff-immutable law makes it value-hashable — two structurally equal
tuples hash the same, so they work as `Map`/`Set` keys. `List`, `Map`, and
`Set` are mutable and keep identity hash instead: `a == b` can hold for two
lists while `a.hash != b.hash`. This is intended, not a gap — see
[Collection Protocol §3](../spec/v0.2/core/collection-protocol.md#3-laws).

`Range` is lazy once it lands: `a..b` won't allocate a million elements any
more than a `while` loop does. `each` generates values on the fly; `toList` is
the explicit materialize-it escape hatch.

Full detail on each literal — evaluation order, key rules, edge cases — lives
in [List Literal Syntax](../spec/v0.2/core/list-literal-syntax.md),
[Map and Set](../spec/v0.2/core/map-and-set.md), and
[Tuple and Range](../spec/v0.2/core/tuple-and-range.md), ratified by
[ADR-0029](../adr/0029-list-literal-syntax.md) and the collections umbrella
[ADR-0032](../adr/0032-collections-representation-and-literals.md).

## One protocol behind every loop

A value is iterable if it answers exactly two selectors:

| Selector | Given | Returns |
|---|---|---|
| `iterate(_)` | the previous cursor, or `None` to start | `Some(nextCursor)`, or `None` when exhausted |
| `iteratorValue(_)` | a cursor | the element at that cursor |

The cursor is an ordinary value — for `List` it's just an integer index — not
an allocated iterator object. There's no separate "iterator" type the way
there is in Python; `Option` alone carries the "is there more?" signal
([Values](values.md) — there is no `nil` to smuggle it through instead).

That's the entire contract. Define these two methods and every combinator —
`each`, `map`, `filter`, `reduce`, and `for` itself — works on your type for
free:

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

`iterate`/`iteratorValue` are ordinary sends, dispatched like any other
method — they aren't sacred selectors and the compiler doesn't inline them.
`List` is the reference iterable; `Map`, `Set`, `Tuple`, `Range`, and your own
classes all conform the same way.

## `for` is a `while` loop in disguise

`for (x in coll) { body }` desugars straight to a `while` loop over the cursor
protocol — not to `coll.each { ... }`:

```phalcom
var _c = coll.iterate(None)
while (_c.isSome) {
  let x = coll.iteratorValue(_c.unwrap)
  body
  _c = coll.iterate(_c)
}
```

That choice of desugar is why `break` and `continue` work inside `for`: they're
loop-control on the `while`, and a block handed to `.each` has no way to
express "stop the caller's loop." See
[Control Flow](control-flow.md) for how the inliner turns this into cheap
jumps without ever touching `iterate`/`iteratorValue` themselves.

## Combinators are just `.ph` over the protocol

`each`, `map`, `filter`, and `reduce` aren't VM primitives — they're
core-library methods written against `iterate`/`iteratorValue`, the same two
selectors your own types implement:

```phalcom
[1, 2, 3].each { n => System.print(n) }        // full traversal, no break/continue
[1, 2, 3].map { n => n * 2 }                   // [2, 4, 6]
[1, 2, 3].filter { n => n > 1 }                // [2, 3]
[1, 2, 3].reduce(0) { acc, n => acc + n }       // 6
```

The distinction to keep straight: `each` always runs to completion — it's the
full-traversal form. `for` is the one with loop control. Reach for `each` when
you're just doing something with every element; reach for `for` the moment you
need `break` or `continue`.

---

Next: [Errors](errors.md) — `throw`/`try`/`on`/`catch`/`ensure` for the
exceptional case, and `Result`/`Ok`/`Err` for the expected one.
