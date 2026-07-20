# `@effect` — full design (v0.3 experimental)

- Status: **Experimental (v0.3 track), mandated by PDR-0018 §3**, replacing
  this tree's earlier "not before R-5" deferral: this design *supplies* the
  ownership/disposal model for the decorator surface instead of waiting for
  reactivity.md's R-5 to resolve independently. Gates that still hold:
  U-REACTIVE-NATIVE (ADR-0058's `Reactive` module) and the Signal runtime
  must exist first — an effect without signals has nothing to observe.
- The constraint that shapes everything: **no finalizers** (PDR-0005). An
  effect is a subscription — a strong edge from every signal it reads back
  to itself. Nothing can dispose it "when it becomes garbage," because it
  never becomes garbage while subscribed. Disposal must therefore be a
  *designed* event, not a hoped-for one.

## 1. Surface

```phalcom
class Cart {
  @observable _items
  @computed total => _items.fold(0) { s, it => s + it.price }

  @effect
  persistTotal { Storage.save(#total, self.total) }   // re-runs when total changes
}

let cart = Cart.new()
let fx = cart.activateEffects          // explicit start — returns an EffectSet
...
fx.dispose                              // explicit stop — detaches every subscription
```

- `@effect` targets getters/zero-arg methods. It **defines** an effect; it
  never starts one. Marking is Compile-tier work (`runtime: false` in the
  strict sense: the member's own body is untouched); the effect wiring is
  data on the class.
- **Activation is explicit.** `activateEffects` (derived, class-level) runs
  each `@effect` body once inside a tracking context (dependencies
  collected, MobX/Solid model) and returns an `EffectSet` handle.
  `activateEffect(#sel)` is the single-member form returning one `Effect`.
- **No auto-start at construction.** An auto-started effect has no natural
  owner and cannot be reclaimed (the finalizer constraint) — auto-start is
  leak-by-default with extra steps. Solid and Svelte 5 reach the same
  conclusion from the other side: effects live inside an owning scope
  (`createRoot`, `$effect` in component scope), never free-floating.
  Phalcom has no component scope, so the owner must be reified — that is
  what the handle is.

## 2. `Effect` and ownership

- `Effect` is an ordinary object: `dispose()` (idempotent — settle-once
  discipline, same shape as C-FUT-3), `isActive`, `run()` (manual re-run,
  mostly for tests). `dispose` walks the dependency edges and unsubscribes;
  after it, the effect body never re-runs and the object is inert.
- `EffectSet` is a `dispose`-broadcasting collection of `Effect`s — the
  composite handle `activateEffects` returns.
- **Scope ownership (the R-5 answer for this surface):**
  `Reactive.scope { ... }` — effects activated dynamically inside the block
  auto-register to the scope; the scope disposes them at block exit
  (normal or unwind, via the same `ensure` unwind discipline everything
  else uses) unless they were `escape`d (`fx.escapeScope` — explicit,
  loud). Nesting: scopes stack per-fiber (part of the `Reactive` module's
  fiber-switch state, like everything ambient in ADR-0058's design);
  child-scope disposal cascades before parent's. This is Solid's owner tree
  reduced to its load-bearing minimum: registration is dynamic-extent,
  disposal is deterministic, and a leak requires *two* explicit acts
  (activate outside any scope, or escape one) rather than zero.
- The blunt rule for users, printed in the class doc: **an active effect
  keeps its receiver alive.** Dispose what you activate, or activate inside
  a scope that disposes for you.

## 3. Scheduling

Batched, not eager — an effect whose dependencies change is *queued*, and
the queue flushes at `Reactive.flush` / end of `Reactive.batch` (both
already in ADR-0058's ratified module surface, which is why this design can
commit where reactivity.md's R-4 hedged: the module surface already chose
the vocabulary of batching). Consequences fixed here:

- Glitch-free within a batch: an effect reading `total` after two `_items`
  writes in one batch runs once, seeing the final state.
- Effects never run re-entrantly inside a setter — a notifying write only
  enqueues. The write-path cost of `@observable` stays a queue push in the
  worst case.
- An effect that *writes* signals during flush enqueues further work with a
  per-flush iteration ceiling (reusing PDR-0007's bounded-depth posture:
  runaway is a catchable error naming the effect, not a hang). Ceiling
  value measured, not guessed — placeholder 1000, ADR-0051 discipline.

## 4. Failure

An effect body that throws: the error propagates out of `flush` (or the
batch boundary) after the queue drains the *other* pending effects —
one bad effect must not starve its peers; multiple failures aggregate into
one raised error carrying all of them (same aggregation posture the
traceback work already committed to for structured errors, PDR-0010's
cause-chain vocabulary). The throwing effect stays active (retry-on-next-
change, matching `@lazy`'s retryability choice); `dispose` remains the way
to stop a persistently failing effect.

## 5. Implementation notes

- Wiring: `@effect` retention carries the selector list on the class
  (attribute instance, frozen with the store per A-5); `activateEffects` is
  a derived class-side... no — an ordinary `core.ph` method on `Object`?
  Neither: derived per-class member (generate phase) so undeclared classes
  carry nothing. Effect/EffectSet/scope machinery lives in the `Reactive`
  native module's `.ph` companion, not the floor — only the tracking
  context and fiber-switch scope stack are native (ADR-0058's existing
  admission; no new floor rows beyond it).
- Dependency edges: effect → signal registration uses the same
  `dependOn`/notify protocol `@computed` uses; an effect is a `Computed`
  whose "value" is discarded and whose invalidation enqueues instead of
  marking dirty. One mechanism, two policies — implement it that way.

## 6. Test plan (first cut)

Activation runs once and tracks; change → queued → flush runs; batch
coalesces; dispose stops re-runs and is idempotent; scope disposes at exit
and on unwind; escape survives scope exit; effect-writes-signal converges
under the ceiling and errors past it; throwing effect doesn't starve peers
and error aggregates; two fibers' scopes are isolated across yields.

## What this precludes

- Auto-started or finalizer-reclaimed effects — both require machinery
  (component scopes, finalizers) Phalcom has ruled out or doesn't have.
- Eager (per-write) effect execution — batching is the committed policy for
  this surface; an eager mode is a superseding design, not a flag.
- A second effect mechanism in the frameworks (persistence "dirty hooks"
  etc. consume signals/effects; they do not define their own).
