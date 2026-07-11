# Annotations — paradigm bridges (`@data`, contracts, `@observable`)

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md)
- Related: object-model.md §8 (value types override `==`), ADR-0011, concurrency §1 (cooperative single-thread), selectors.md §5

## Context

Annotations let Phalcom's Smalltalk core reach *other paradigms* as pure sugar.
Three bridges, each a different annotation facet: **generate**, **weave**,
**intercept**.

## Bridge A — `@data`/`@sealed`/`@variant` → algebraic/functional (generate)

```phalcom
@data @sealed
class Shape { @variant Circle(radius:); @variant Rect(w:, h:) }

shape.match { Circle(r) => 3.14 * r * r ; Rect(w, h) => w * h }
```

`@data` derives structural `==` **and** consistent `hash` (both or neither — a
lone `==` breaks the equality ladder), plus functional-update `with(...)` (cheap:
ADR-0011 fixed layout ⇒ copy-with is memcpy-shaped). `@sealed` freezes the
subclass set at finalization → the closed-world fact that makes `match`
exhaustiveness checkable (Phalcom has no type checker, so `@sealed` is the *sole*
route). `match` desugars to a generated visitor (double-dispatch) — the textbook
OO encoding of ADTs (Expression Problem). Precludes open extension of a sealed
family (the point). Downstream of [annotations-construct.md](annotations-construct.md)
(needs field decls).

## Bridge B — Design by Contract → formal-methods (weave)

Full spec in [annotations-contracts.md](annotations-contracts.md). The one
pure method-table-macro bridge; contracts are the runtime semantics of gradual
types (reserved direction).

## Bridge C — `@observable`/`@computed` → reactive dataflow (intercept)

```phalcom
class Cart {
  @observable var _items
  @computed total => _items.fold(0) { s, it => s + it.price }
}
c::total.subscribe { t => System.print("total → {t}") }
```

`@observable var _x` is `@get @set` **plus** a `self.__notify(#x)` in the setter;
`@computed` derives a memoized getter that collects the observables it read and
recomputes when they go dirty (MobX/Vue auto-tracking). Prior commitments
*enable* it: cooperative single-thread (concurrency §1) makes the "current
subscriber" dynamic variable race-free; `obj.x` already being a send
(selectors.md §5) means reactive reads cost nothing extra (Solid-signals model).

Hazards: **glitches** (recompute once after all inputs settle — pick eager-glitchy
vs batched-glitch-free on purpose); reactive state (dep map + dirty flags) isn't
a declared field, so it needs reserved hidden slots (layout tier) or an
identity-keyed side table — the same `Map<Symbol,…>` shape a
`doesNotUnderstand`-delegation prototype object uses, so one substrate serves both.

## Unifying finding

Two tiers, drawn by whether an attribute touches instance layout:

| Tier | Attributes | selectors.md §4 "no new machinery" holds? |
|------|-----------|-------------------------------------------|
| method-table macro | `@get`/`@set`, `@requires`/`@ensures`/`@invariant` | **Yes** |
| layout derive | `@construct`, `@data`/`@variant`, `@observable` | No — needs layout ADR |

The tier line is the seam the annotation work should be cut along.
