# Dispatch tier — full design (v0.3 experimental)

- Status: **Experimental (v0.3 track), mandated by PDR-0018 §3** (withdrawing
  the earlier "not until a decorator forces it" posture). First shipped
  decorator: `@ForwardMissing(to:)` — the D-1 alternative recorded in
  DEFERRED, promoted.
- Position in the model: fires on **lookup miss only** — strictly between the
  failed dictionary walk and `doesNotUnderstand` reification. The hit path is
  untouched by construction; this tier can never tax a successful send.

## 1. The `resolveMissing(_)` protocol

An attribute declaring `@On(Class, tier: Dispatch)` implements:

```phalcom
resolveMissing(msg)   // msg: the reified Message (selector, args) — returns
                      // Some(aMethod) to handle, None to decline
```

- Receives the same `Message` object the dNU path already reifies (selector
  symbol + args List) — no new reification machinery; the miss path builds it
  once and threads it through resolvers and, if all decline, into
  `doesNotUnderstand` unchanged.
- Returns `Option<Method>`: `Some(m)` ⇒ `m.invokeOn(receiver, msg.args)` is
  the send's result; `None` ⇒ next resolver, then the ordinary dNU path
  (user-defined dNU cannot coexist — see §3 — so "ordinary" means the default
  raise).

## 2. Resolution is transient — no installation

The resolved `Method` is **invoked, never installed** into the class
dictionary. This is the design's load-bearing choice:

- **Soundness for free.** No dictionary mutation ⇒ no `world_version` bump,
  no IC invalidation, no interaction with PDR-0001's closure or the kernel
  install choke point. The miss path is already the slow path; staying off
  the dictionary keeps every warm site's assumptions intact.
- **Semantic honesty.** `respondsTo(sel)` stays dictionary-truthful and
  therefore *lies about resolvable selectors* — a forwarded selector answers
  `false`. Recorded as the tier's documented lie rather than papered over:
  extending `respondsTo` to consult resolvers would make a reflection
  primitive run user code, which Smalltalk's `respondsTo:` experience says
  to refuse (introspection must not have effects). A future optional
  `canResolve(_)` hook (pure-by-contract, floor-not-proof) is the recorded
  extension if tooling demands it; not in the first cut.
- **Cost model.** Every missed send re-runs the resolver chain. A resolver
  *may* memoize internally (attribute-instance state, class-wide — the
  on.md state-scope table's row 2), e.g. `@ForwardMissing` caching
  `selector → forwarder` in a Map on the attribute instance. That cache is
  selector-keyed, not receiver-keyed — ADR-0052's confinement rule is
  satisfied; a receiver-keyed resolver cache remains forbidden.
- **Materialization deferred.** Install-on-first-miss (Ruby's
  `define_method`-from-`method_missing` pattern) is the recorded follow-up
  *if* miss-path profiles demand it; it must route through the single
  install choke point and bump epochs like any install. Not in v0.3's first
  cut — it trades the soundness-for-free property for speed nobody has
  measured a need for.

## 3. Collision and chaining rules

- **Hand-written `doesNotUnderstand(_)` + any Dispatch attribute = compile
  error** (`attr.dispatch_collision`, D-4's ruling, already specified) — no
  ordering question exists between the two mechanisms.
- Multiple Dispatch attributes on one class: chained in **source order,
  first `Some` wins**. Order is user-controlled and must be documented at
  the declaration site — same posture as the Install/Runtime stacking rule.
- Inheritance: resolvers are **per-declaring-class and consulted along the
  superclass chain, subclass-first**, after the *entire* dictionary walk
  fails. A subclass resolver therefore shadows a superclass resolver the
  same way a subclass method shadows one — the rule a Smalltalk-shaped
  language must pick to keep the two lookup ladders congruent.

## 4. `@ForwardMissing(to:)` — the first Dispatch decorator

```phalcom
@On(Class, tier: Dispatch)
class ForwardMissing is Attribute { ... }

@ForwardMissing(to: #_inner)
class LoggingWrapper {
  _inner
  constructor new(inner:) { _inner = inner }
  log(msg) { ... }              // own members win — they hit the dictionary
}
```

- `resolveMissing(msg)` reads the named field off the receiver and returns
  `Some` of a forwarder invoking `msg.selector` on it with `msg.args`
  (`perform`-based; selector-symbols only, per the existing `perform`
  contract). Field absent/`None` ⇒ `None` (decline, ordinary dNU raise).
- Whole-protocol forwarding is exactly what the Compile-tier
  `@delegate(to:, selectors:)` refuses to be — the pair now covers both ends
  deliberately: enumerated forwarding = Compile, zero runtime cost,
  reflectable; open forwarding = Dispatch, slow-path-only, honest about
  `respondsTo`. The choice between them is the user naming their trade.
- One `to:` per class in the first cut; two `@ForwardMissing` chain by §3 if
  declared, and the doc says prefer one.

## 5. Interaction with the Runtime tier

Order per the total phase order: a Runtime chain wraps the *send*, Dispatch
resolves the *miss* — an intercepted send that misses runs interceptors
first, resolver inside (`proceed` reaches the miss path). The per-fiber
interceptor bypass ([runtime-tier.md §4](runtime-tier.md)) applies to sends
a *resolver* makes, same as any interceptor-adjacent code — resolvers run
under the depth guard while invoked from an intercepted send, and outside it
otherwise. No special case.

## 6. Test plan (first cut)

Positive: forward hit (missing selector reaches inner), own-member
precedence, decline falls to dNU raise with the *original* selector in the
message, chain first-Some-wins, subclass-shadows-superclass resolver,
resolver memoization observable only as speed (behavior-identical).
Negative: `attr.dispatch_collision` fixture; `respondsTo` documented-lie
fixture (asserts `false`, pinning the honesty rule); resolver returning a
non-Option ⇒ `RuntimeError::Type` naming the attribute.

## What this precludes

- Dispatch resolvers on the hit path, under any future optimization — the
  tier's contract *is* miss-only.
- A resolver installing into the dictionary outside the §2 materialization
  follow-up (single choke point, epochs, its own review).
- `respondsTo` running resolvers.
