# Interception decorators — `@delegate`, `@Traced`, `@FeatureFlag` (Dispatch & Runtime tiers)

- Status: **Design-ratified (ADR-0054 broad), nothing built.** Source draft:
  [decorators-dispatch-observability.md](../v0.2/drafts/decorators-dispatch-observability.md)
  (D-1/D-2/D-3 resolved). `Tracer`/`Tracer.stdout` and `OffBehavior` already
  ship in `core.ph` — helper classes landed ahead of their decorators.
- Cost model of record: ADR-0053. Zero machinery exists on HEAD
  (`has_runtime_interceptor`: zero hits) — the ADR is a priced design, not a
  description.

## `@delegate(to:, selectors:)` — verified reclassification, Compile tier

The draft's move of `@delegate` from Dispatch to Compile is correct and worth
restating as the family's design lesson: **forwarding a known selector list
needs no interception at all.** Generated forwarding methods are ordinary,
reflectable, inline-cacheable members; shadowing is statically checkable
(`attr.delegate_shadow`, `attr.delegate_conflict`); and the DNU-collision
hazard (D-4's `attr.dispatch_collision`) never arises. The selector list is a
maintenance surface — visible, greppable, honest. Open whole-protocol
forwarding remains the proxy library's job (`@forwardMissing` recorded for
v0.3 in DEFERRED, not here). Lowercase builtin spelling stands: it is a
compiler derive.

Consequence: **the Dispatch tier currently has zero shipped candidates.** It
stays specified (an attribute installing a `resolveMissing(_)` handler, firing
on lookup miss only, `attr.dispatch_collision` against hand-written
`doesNotUnderstand`) because the *proxy* library and future
`@forwardMissing`-class decorators need the slot — but nothing in this tree
builds it, and no unit should until a concrete decorator forces it.

## `@Traced` — Runtime tier

Per the draft: `aroundSend` interceptor; flags `entry`/`exit` (default on),
`timing` (default **off** — a clock read is real cost), `errors` (on);
pluggable `sink:` implementing `Tracer` (`enter`/`exit`/`threw`); never
swallows — re-raises after logging. Composes with the `Trace` proxy per
ADR-0057 (same substrate, method- vs object-granularity; neither subsumes).
`@Timed` folds in here as `timing: true` ([behavioral.md](behavioral.md)).

The one hazard worth elevating: **sink re-entrancy.** A sink that sends into
the traced object graph recurses through the interceptor. The draft notes it;
the spec rule should be sharper — the interceptor sets a per-fiber
"in-trace" flag and bypasses interception for sends made while it is set
(same per-fiber state discipline as ADR-0052's `checking` set, and it must
live in fiber-switch state for the same reason).

## `@FeatureFlag(name:, whenOff:)` — Runtime tier

Verified sound with one design decision that deserves its rationale kept
loud: **the off-default raises** (`FeatureDisabled`), never silently returns
`None`. A silently-`None` disabled method is a contract violation wearing a
feature flag — the Option discipline (ADR-0007) makes the poisoning worse,
not better, since `None` flows. `OffBehavior.fallback(#selector)` (signature
checked at class-definition) and `OffBehavior.skip(value)` are the explicit
alternatives. `Flags` as one ambient registry is the ratified v0.2 shape
(D-3); per-scope injection is the recorded v0.3 upgrade path.

## Implementation plan — ADR-0053 made real

Ordered; step 1 is shared with nothing and blocks everything:

1. **The guard bit.** `ClassObject.has_runtime_interceptor: bool`, set once
   at class-definition time when any attached attribute declares
   `tier: Runtime`. Dispatch fast path reads it alongside the existing
   class-identity comparison — one bit, only where a comparison already
   happens. Sacred selectors need **no new mechanism**: installing a Runtime
   interceptor on a class in the `Bool`/`Block` families flips the existing
   `*_sacred_pristine` flag (route through `note_method_installed`), and the
   ADR-0018 deopt path does the rest. Fixture: decorated `Bool` still
   evaluates `ifTrue` correctly via the slow path, byte-identical
   observables.
2. **The `Invocation` object** (receiver, selector, args, `proceed`) — one
   allocation per intercepted send, the priced cost. `proceed.call()` runs
   the next interceptor or the real method. Chaining is source-order
   innermost-last (D-3), with the pre-composed-chain caching from
   decorators/README.md §Future optimizations recorded as the follow-up, not
   the first cut.
3. **Dispatch integration:** on `has_runtime_interceptor`, the send path
   builds the Invocation and enters the chain instead of the direct call.
   Undecorated classes: no observable change, and the erasure golden must
   prove it (strip the attribute ⇒ identical bytecode *and* identical trace
   of a probe program).
4. **Then** `@Traced`/`@FeatureFlag` are `core.ph` `Attribute` subclasses
   over the shipped `Tracer`/`OffBehavior` helpers.
5. `@Metered` disposition: park until a metrics sink abstraction exists;
   `@Traced` with a counting sink covers the interim. Rejected as a v0.2
   spec target.

Preclusion check before building: step 1's bit is valid **only** while
attribute retention is frozen (A-5) and hierarchies are sealed — both hold.
If either is ever relaxed, the bit becomes an epoch counter *first* (ADR-0053
names the trigger; this file repeats it so the implementer sees it at the
build site).

## What this precludes

- A Runtime tier that intercepts sacred-selector fast paths without deopt —
  the ADR-0018 guard is the mechanism, reused, not duplicated.
- Silent-`None` feature flagging, in any mode.
- ~~Building the Dispatch tier speculatively, with no shipping decorator.~~
  **Withdrawn by PDR-0018 §3** (user mandate, 2026-07-20): the Dispatch tier
  is built on the v0.3 experimental track with `@ForwardMissing` as its
  first decorator — full design in [dispatch-tier.md](dispatch-tier.md);
  Runtime-tier mechanics deepened in [runtime-tier.md](runtime-tier.md).
