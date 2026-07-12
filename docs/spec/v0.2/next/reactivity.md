# Reactivity & Signals — the fourth capability from the substrate

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-12
- Depends on:
  [method-lookup.md](../method-lookup.md) (read/write interception) ·
  [functions.md](../functions.md) (blocks, `call`) ·
  [values-and-absence.md](../values-and-absence.md) (`Option`, `match`)
- Related:
  [decorators.md](decorators.md) (`@observable` is the Layout tier this fleshes out) ·
  [proxy.md](proxy.md) (auto-tracking is read-interception; disposal is revocation) ·
  [object-model.md](../object-model.md) (`Set`, metaclass class-side state)

## Thesis

Reactivity is the **fourth** capability that falls out of the message substrate,
alongside proxies, prototypes, and decorators. Each is one facet of message
interception:

- **Proxies** intercept *sends* → control / security.
- **Prototypes** resolve *misses* → delegation.
- **Decorators** wrap *methods* by tier → policy.
- **Signals** intercept *reads and writes* → **a read registers a dependency, a
  write notifies dependents.**

Automatic dependency tracking — what makes SolidJS / Vue / Signals feel automatic —
*is* message interception, which Phalcom already has. So reactivity is not a new
mechanism; it is the runtime that [decorators.md](decorators.md)'s `@observable`
(Layout tier, "reserves reactive slots") already points at, and it realizes the
[paradigm-bridges](../experimental/annotation-paradigm-bridges.md) claim literally:
a signal's observer `Set` *is* the dependency edge, tracked by read-interception.

## Model: push-pull, glitch-free

The design is **push-pull** (the modern consensus — Solid, Preact Signals,
Reactively):

- a **write** *pushes* a cheap staleness mark transitively through the graph
  (flag-setting, no user code);
- **recomputation** is *pull* — lazy, on read;
- **effects** are the roots, scheduled on change.

This is what avoids the two failure modes: pure push glitches (the diamond
problem — a node recomputes with a stale operand), and pure pull cannot schedule
effects.

## The runtime

```phalcom
// Tracking-context stack + effect scheduler.
// (Class-side state — the metaclass tower gives classes their own slots.)
class Reactive {
  static boot { _current = None; _batching = false; _pending = Set.new() }
  static current { return _current }

  static trackedBy(computation, run) {          // run `run` with `computation` as current reader
    let prev = _current
    _current = Some.new(computation)
    let result = run.call()                     // every reactive read inside registers a dep
    _current = prev
    return result
  }
  static untracked(run) {                        // read WITHOUT subscribing
    let prev = _current;  _current = None
    let result = run.call();  _current = prev
    return result
  }

  static schedule(effect) { _pending.add(effect); if (_batching == false) { self.flush } }
  static batch(run) {
    let was = _batching;  _batching = true
    run.call()
    _batching = was;  if (was == false) { self.flush }
  }
  static flush { let due = _pending; _pending = Set.new(); due.each { e => e.run } }
}
```

## The three primitives

### `Signal` — a reactive cell

```phalcom
class Signal {
  construct new(value:) { _value = value; _observers = Set.new() }

  value {                                        // tracked READ
    Reactive.current.ifSome { c => c.dependOn(self) }
    return _value
  }
  value=(next) {                                 // WRITE
    if (_value != next) {                        // equality bail: no-op writes don't propagate
      _value = next
      _observers.each { c => c.markStale(self) } // push staleness to dependents
    }
  }
  subscribe(c)   { _observers.add(c);    return self }
  unsubscribe(c) { _observers.remove(c); return self }
}
```

### `Computed` — a cached derived value (observer *and* observable)

```phalcom
class Computed {
  construct new(compute:) {
    _compute = compute;  _sources = Set.new();  _observers = Set.new()
    _stale = true;  _value = None
  }
  dependOn(src)  { _sources.add(src); src.subscribe(self); return self }
  subscribe(c)   { _observers.add(c); return self }
  unsubscribe(c) { _observers.remove(c); return self }

  value {                                        // pull: recompute if stale, then track ME upward
    if (_stale) { self.recompute }
    Reactive.current.ifSome { c => c.dependOn(self) }
    return _value.unwrapOr(None)
  }
  markStale(src) {
    if (_stale == false) {                       // already stale -> don't re-propagate
      _stale = true
      _observers.each { c => c.markStale(self) } // transitively mark dependents
    }
    return self
  }
  recompute {
    _sources.each { s => s.unsubscribe(self) }   // drop old deps...
    _sources = Set.new()
    let next = Reactive.trackedBy(self) { _compute.call() }   // ...re-collect fresh (dynamic deps)
    _stale = false;  _value = Some.new(next)
    return next
  }
}
```

### `Effect` — the scheduled root

```phalcom
class Effect {
  construct new(run:) { _run = run; _sources = Set.new(); self.run }
  dependOn(src)  { _sources.add(src); src.subscribe(self); return self }
  markStale(src) { Reactive.schedule(self); return self }      // effects are scheduled, not lazy
  run {
    _sources.each { s => s.unsubscribe(self) }; _sources = Set.new()
    Reactive.trackedBy(self) { _run.call() }
    return self
  }
  dispose { _sources.each { s => s.unsubscribe(self) }; _sources = Set.new(); return self }
}
```

### Why the diamond stays glitch-free

Given `A→B`, `A→C`, `B→D`, `C→D`, and an effect `E` on `D`: writing `A` marks `B`
and `C` stale, which mark `D` stale *once* (the second mark bails), which schedules
`E`. When `E` runs it *pulls* `D`, which recomputes once against freshly-pulled `B`
and `C`. `D` recomputes exactly once, never with a stale operand.

The skeleton above is glitch-free but slightly **over-eager**: it propagates
staleness before checking a recomputed value. The production refinement is the
**clean / check / dirty** three-color marking (Reactively, Signals proposal): a
write marks direct dependents *dirty* and transitive dependents *check*; a *check*
node, when pulled, recomputes only if a source actually changed, and bails (stays
clean) if its own value is `==` its previous — stopping the cascade.

## The ergonomic layer: `@observable` / `@computed`

Nobody writes `cart.totalSignal.value`. The decorators hide the boxing (see
[decorators-stdlib.md](decorators-stdlib.md) for their definitions): `@observable`
(Layout+Install) makes a field signal-backed; `@computed` (Install) wraps a getter
as a `Computed`.

```phalcom
class Cart {
  @observable var _items                         // field -> Signal; reads tracked, writes notify
  @observable var _taxRate

  @computed subtotal => _items.reduce(0) { sum, it => sum + it.price }
  @computed total    => self.subtotal * (1 + _taxRate)
}
```

```phalcom
let cart = Cart.new(items: goods, taxRate: 0.08)

Effect.new(run: {
  System.print("Total: \(cart.total)")           // reads total -> subtotal -> items, taxRate
})                                                // prints once, immediately

cart.taxRate = 0.10                               // one write -> total stale -> effect reruns ONCE

Reactive.batch({
  cart.taxRate = 0.12
  cart.applyDiscount(promo)                       // several writes...
})                                                // ...effect reruns ONCE at batch close
```

The view (`\(cart.total)`) declares its dependencies just by reading them — no
`subscribe`, no dependency arrays, no manual invalidation — and the model just
assigns fields.

## Design calls

- **Equality is a message.** The write-time bail uses `!=`, which is
  user-overridable, so a [`@data`](decorators-stdlib.md) value object with a custom
  `==` controls its own propagation — and drives the three-color bailout above.
- **Shallow vs deep — the classic trap.** `@observable var _items` tracks the
  *reference*. `_items.add(x)` mutates in place, so the signal never fires. Either
  update immutably (new reference) or make the collection a **reactive proxy** — a
  [`Proxy`](proxy.md) over a `List` that notifies on `add`/`at:put:`. Deep
  reactivity = signals + proxies, one substrate again. Ship shallow-by-default
  (predictable) with an opt-in reactive collection.
- **Disposal is revocation.** A reactive graph leaks without an owner: effects hold
  sources, sources hold effects. The disposal scope is structurally the same
  one-cell teardown as the [`Capability`](proxy.md) membrane's `Revoker`.
- **Scheduling policy.** Sync flush (shown) suits a server; a UI wants microtask or
  animation-frame batching. `Reactive.flush` is the seam.
- **Untracked reads & cycles.** `Reactive.untracked { }` covers reads that must not
  subscribe; a `Computed` that reads itself needs a cycle guard (raise, not hang).

## Ties to the model & open questions

`@observable`'s Layout tier reserves the `Signal` slot; auto-tracking is the same
read-interception [proxies](proxy.md) use — so `@traced` (a Runtime decorator) and
reactive tracking are the same interception seam. That is exactly
[decorators.md](decorators.md)'s **D-3** (do multiple Runtime around-send hooks
chain?), now with a concrete stake: a reactive object that is also `@traced` must
compose both interceptors.

| # | Question |
|---|----------|
| R-1 | Adopt the three-color (clean/check/dirty) algorithm as the ratified propagation semantics, or the simpler over-eager mark-and-pull? |
| R-2 | Does `Reactive`'s ambient tracking context live in class-side (metaclass) state, or a well-known global singleton? |
| R-3 | Default reactivity granularity: shallow (reference) with opt-in reactive collections, vs deep (auto-wrap nested collections in reactive proxies)? |
| R-4 | Effect scheduling policy — synchronous, microtask, or frame — and is it per-`Reactive`-scope configurable? |
| R-5 | Ownership: is there a `Reactive.root { }` owner tree (Solid-style) that ties effect disposal to a lexical scope, unifying with the membrane `Revoker`? |
