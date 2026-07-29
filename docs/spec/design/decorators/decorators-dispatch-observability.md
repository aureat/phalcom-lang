# Dispatch & observability decorators — `@delegate`/`@traced`/`@featureFlag`

- Status: **Draft** (exploration only — not implemented; see [decorators/](../decorators/) for the built surface)
- Ratification note: the *design* was ratified 2026-07-13 under [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md) (D-1/D-2/D-3 resolved the same day; DEFERRED.md cites those IDs against this file — do not renumber). **None of it is built** — `@delegate`/`@traced`/`@featureFlag` are unregistered names that raise `attr.unknown`; the Compile-tier `@delegate` derive does not exist and no Runtime interceptor is ever consulted. `Tracer`/`Tracer.stdout` and `OffBehavior` **are** shipped in core.ph (D-2/D-3's helper classes, ahead of the decorators that would use them).
- Date: 2026-07-13
- Depends on:
  [decorators/on.md](../decorators/on.md) (the `Attribute` root, `@On(target…, tier:)`,
  the `expand(_)`/`resolveMissing(_)`/`aroundSend(_)` hook protocol) ·
  [decorators/README.md](../decorators/README.md) (the five-tier axis, phase order, the `runtime`
  flag; **D-4** dispatch-collision resolution) ·
  [method-lookup.md](../method-lookup.md) (`doesNotUnderstand`, `Message`, `perform`
  — the substrate `@delegate`/`@traced` interception stands on) ·
  [system.md](../system.md) (`System.print` — the default trace sink)
- Related:
  [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
  (Runtime interceptor guard bit — the cost model for `@traced`/`@featureFlag`) ·
  [ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)
  (decorator-vs-proxy granularity — why `@traced` the decorator and `Trace` the
  proxy coexist) ·
  [proxy.md](proxy.md) (`Trace` proxy — the object-granularity sibling of `@traced`;
  the DNU/`Prototype` substrate `@delegate` shares) ·
  [decorators-stdlib.md](decorators-stdlib.md) (the earlier scattered sketch this doc
  supersedes for these three) ·
  [decorators-persistence.md](decorators-persistence.md) (associations are a
  `@delegate`-adjacent Dispatch pattern)

## Context

Three decorators sit on the **control-flow / observability** side of the tier axis:
`@delegate` forwards a sub-protocol to a component, `@traced` logs the calls
crossing a boundary, `@featureFlag` gates a call on a runtime flag. Each has either
a competing proxy-library rendering ([proxy.md](proxy.md)'s `Trace`) or a
name-only mention with no worked semantics (`@delegate`, `@featureFlag`) — resolved
and specified here at ratification depth.

Two resolutions frame the doc:

1. **`@delegate` is Compile, not Dispatch.** [decorators-stdlib.md](decorators-stdlib.md)
   sketches `@delegate(to:)` as a **Dispatch**-tier `onMiss` forwarder (every miss
   routes through `doesNotUnderstand`). This doc argues for and specifies the
   **Compile** form instead — forwarding methods generated statically from an
   explicit selector list, with zero runtime indirection — and confines the
   open-ended whole-protocol forward to an Open question, not the default.

2. **`@traced` decorator and `Trace` proxy are the same feature at two
   granularities, both retained.** Per
   [ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md):
   `@traced` is per-declaration, author-applied observability; `Trace`
   ([proxy.md](proxy.md)) wraps any black-box object from outside. Non-conflicting
   split — argued below, not merged.

## Decision

### `@delegate` — Compile (builtin), explicit-selector forwarding methods

`@delegate(to:, selectors:)` on a field generates, at compile time, one forwarding
method per named selector, each forwarding to that field (composition over
inheritance). It is a **Compile**-tier derive — pure AST→AST, the same class as
`@get`/`@set`/`@data` — so it is builtin-owned
([decorators/README.md](../decorators/README.md)'s user/compiler tier line; `tier: Compile` is
compiler-reserved, [attribute-classes.md A-3](../decorators/on.md)).

```phalcom
class Car {
  @delegate(to: _engine, selectors: [#rpm, #start(_), #stop])
  var _engine
  // generate phase expands to three ordinary methods:
  //   rpm       => _engine.rpm
  //   start(x)  => _engine.start(x)
  //   stop      => _engine.stop
}
```

**Why Compile over Dispatch — the argument.** A Dispatch-tier `@delegate` (the
stdlib sketch) installs a `doesNotUnderstand` handler that forwards *every* missed
selector to the field. Compile-tier `@delegate` with an explicit selector list is
strictly better for the common case (delegate a *known* sub-protocol) on four axes:

- **No runtime indirection.** The generated methods are ordinary `Method` objects
  in the class dictionary — monomorphic, inline-cacheable
  ([ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)),
  no lookup-miss round-trip, no `Message` reification per call. The Dispatch form
  pays a full DNU miss + `perform` on every forwarded send.
- **Statically shadow-checkable.** Because the forwards are real generated members,
  a collision with a hand-written method of the same selector is caught at compile
  time as `attr.accessor_collision` (the same diagnostic `@get`/`@set`/`@construct`
  already raise), with a precise span — see shadowing rules below. The Dispatch form
  can only discover a shadow at runtime (and DNU never even fires for a
  hand-written method, so the delegation silently loses).
- **Reflectable and documentable.** `Car.methods` lists `rpm`/`start`/`stop`; a
  Dispatch forwarder leaves the class's surface protocol empty and answers
  everything at runtime — invisible to tooling, docs, and `respondsTo`.
- **No DNU-collision hazard.** The Compile form never touches the DNU chain, so it
  cannot collide with a hand-written `doesNotUnderstand`
  ([decorators.md D-4](../decorators/README.md)'s `attr.dispatch_collision`). The Dispatch
  form must reserve the DNU slot and conflict-check it.

The cost is that the delegated selectors must be **enumerated** — you cannot say
"forward everything `_engine` understands" at compile time, because Phalcom is
dynamically typed and the field's protocol is unknown until runtime. That
open-ended whole-protocol forward is exactly what the [proxy.md](proxy.md) `Proxy`
DNU library and a hand-written `doesNotUnderstand` already express; `@delegate`
deliberately does **not** duplicate it (see Open question D-1).

**Shadowing rules.** Two cases:

- **Class defines the same selector the delegate list names.** The class's own
  method wins, and listing an already-defined selector is a **compile error**
  (`attr.delegate_shadow`, a sibling of `attr.accessor_collision`) — silent
  shadowing (delegate quietly loses) is exactly the footgun this spec family rejects
  everywhere. Remove the selector from the list to keep the hand-written method, or
  remove the hand-written method to delegate.
- **Two `@delegate`s name the same selector (different fields).** Ambiguous forward
  → `attr.delegate_conflict` at compile time. The author picks one, or writes an
  explicit disambiguating method. No source-order "last wins".

`selectors:` entries are **selector literals** (`#rpm`, `#start(_)`) so arity and
kind are pinned — `#start` (getter) and `#start(_)` (unary) are distinct forwards,
consistent with Phalcom's selector-identity model
([ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)).

### `@traced` — Runtime (user), configurable observability around a send

`@traced` logs a method's (or class's) invocations — entry, exit, and, optionally,
timing and exceptions. It is a **Runtime**-tier user decorator: an `aroundSend`
interceptor consulted per send, which is what lets it observe *every* send to the
receiver (including inherited and dynamically-dispatched ones), not just one
statically-known method body. Runtime, not Install, because whole-object tracing is
the headline use, and [ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)
already priced the exact guard bit a Runtime interceptor needs (an undecorated
class pays nothing).

```phalcom
@On(Method, tier: Runtime)
class Traced is Attribute {
  var _entry   var _exit   var _timing   var _errors   var _sink
  @constructor
  new(entry: true, exit: true, timing: false, errors: true, sink: Tracer.stdout) {
    _entry = entry; _exit = exit; _timing = timing; _errors = errors; _sink = sink
  }

  aroundSend(inv) {                                    // inv: selector, name, args, proceed
    _entry.ifTrue { _sink.enter(inv.name, inv.args) }
    let start = _timing.ifTrue { Some.new(Clock.now) }.orElse { None }
    return { inv.proceed }.on(Error) { e =>
      _errors.ifTrue { _sink.threw(inv.name, e) }
      throw e                                          // never swallow — observability, not handling
    }.andThen { result =>
      _exit.ifTrue { _sink.exit(inv.name, result, start.map { s => Clock.now - s }) }
      result
    }
  }
}
```

- **What gets traced — default set + configurable.** Default: **entry** (name +
  args), **exit** (name + return value), **exceptions** (name + error). **Timing**
  is opt-in (`timing: true`) because a clock read per send is a real cost most
  traces don't want. Each is an independent flag, so `@traced(entry: false)` logs
  only exits, `@traced(timing: true)` adds latency, etc.
- **Output sink — pluggable `Tracer` protocol, default stdout.** Phalcom has **no
  dedicated logging primitive** ([system.md](../system.md) exposes `System.print`
  and no logger), so the default sink `Tracer.stdout` writes via `System.print`. The
  `sink:` argument accepts any object answering the `Tracer` protocol
  (`enter(name, args)`, `exit(name, result, elapsed)`, `threw(name, error)`), so a
  structured logger, a span collector, or a test double drops in without changing the
  decorator. This is the seam the ratified `@traced` needs; a concrete `Tracer` core
  class is Open question D-2.
- **Never swallows.** The interceptor observes and re-raises; it must not turn a
  thrown method into a silent success (that would make `@traced` behavior-changing,
  breaking the observability contract that stripping `@traced` leaves results
  identical).

**Relationship to the `Trace` proxy ([proxy.md](proxy.md)) — two granularities,
both kept.** Argued *for* the split
([ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)):

| | `@traced` (this doc) | `Trace` proxy ([proxy.md](proxy.md)) |
|---|---|---|
| Applied by | the **author** of the class, at a declaration | a **third party**, wrapping any object |
| Scope | one method / one class you own | the whole protocol of a black-box object |
| Cost model | Runtime interceptor bit ([ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)) | DNU miss + `perform` per crossing |
| Identity | the real object (interception before lookup) | leaks `==`/`class`/`hash` (proxy caveat P-1) |

The split is non-conflicting because the two answer different questions: "instrument
*my* code" (`@traced`) vs "observe an object I *don't own* and can't annotate"
(`Trace`). Neither subsumes the other — a decorator cannot instrument a third-party
object, a proxy cannot be part of a class's own declaration — so both survive.
[proxy.md](proxy.md)'s `Trace` subsection is annotated to point here for the
owned-code case.

### `@featureFlag` — Runtime (user), gate a call on a runtime flag

`@featureFlag(name:, whenOff:)` consults a runtime flag registry on each send; if the
flag is on, the method runs; if off, the configured off-behavior applies. It is
**Runtime** because the flag can flip at runtime and must be consulted *per call* —
an Install-tier wrapper would bake the flag state in at class-definition time, which
is wrong (the flag would be frozen at boot). Runtime consultation + ADR-0053's guard
bit + the "interceptor-declared bypass" optimization
([decorators.md Future optimizations](../decorators/README.md)) make an *off* flag cheap.

```phalcom
@On(Method, tier: Runtime)
class FeatureFlag is Attribute {
  var _flag   var _whenOff
  @constructor
  new(name:, whenOff: OffBehavior.raise) { _flag = name; _whenOff = whenOff }

  aroundSend(inv) {
    return Flags.enabled(_flag)
      .ifTrue  { inv.proceed }
      .ifFalse { _whenOff.applyTo(inv) }       // raise | fallback selector | skip-with value
  }
}
```

- **What gates it — an ambient `Flags` service, queried, not baked.** The decorator
  reads `Flags.enabled(name)` from a well-known ambient registry (a module singleton,
  same shape as `System`), rather than closing over a boolean at class-def. Querying
  (not injecting a captured value) is what makes runtime flips take effect. Whether
  `Flags` is a ratified core module or a user-supplied service the decorator resolves
  by name is Open question D-3.
- **Behavior when off — default `raise`, three configurable modes.** The default is
  **`OffBehavior.raise`** — raise `FeatureDisabled(name)` — chosen over the stdlib
  sketch's silent `return None` because a silently-`None`-returning method violates
  its own contract/return shape and hides the disablement (exactly the silent-wrong
  this spec family rejects: truthiness ban, `dispatch_collision`, `delegate_shadow`).
  The three modes:
  - `OffBehavior.raise` (default) — raise `FeatureDisabled`; the caller decides.
  - `OffBehavior.fallback(#selector)` — call a named fallback method on the receiver
    with the same args (the graceful-degradation path; the fallback's signature must
    match, checked at class-def against the receiver's protocol).
  - `OffBehavior.skip(value)` — return a fixed value (the *explicit* no-op, distinct
    from silent-None because the author names the value and thereby the intent).
- **Tier justification, restated.** Runtime, not Install: the flag is dynamic. Not
  Dispatch: the method exists and is understood (no lookup miss) — gating an
  *understood* selector is per-send interception, which is Runtime by definition.

## Composition

Phase order `generate → weave → finalize → install → dispatch → runtime` places
`@delegate` (Compile/generate) first and `@traced`/`@featureFlag` (Runtime) last, so
they never fight:

- **`@delegate` (Compile) ⊗ any later tier.** `@delegate` *generates* the forwarding
  methods in the generate phase; a later `@traced`/`@featureFlag`/`@memoize` on the
  *same selector* then decorates the generated forwarder exactly as it would a
  hand-written method (a forwarder is an ordinary `Method`). So `@traced rpm` on a
  class that `@delegate`s `#rpm` traces the forwarding call — expected and useful.
- **`@traced` (Runtime) ⊗ `@featureFlag` (Runtime)** — same tier, **source order,
  innermost-last**, and Runtime hooks **chain** ([decorators.md D-3](../decorators/README.md)):
  the outer interceptor's `proceed` runs the inner one. `@traced @featureFlag method`:
  trace outermost — you see the entry log, then the flag check; if the flag is off
  and `whenOff` raises, `@traced`'s error branch logs the `FeatureDisabled` and
  re-raises (you observe the gating). `@featureFlag @traced`: flag outermost — an off
  flag short-circuits *before* any trace entry is logged (you see nothing when
  disabled). Prefer `@traced @featureFlag` if you want disablement visible in the
  trace; `@featureFlag @traced` if a disabled feature should be invisible. Document
  at the call site.
- **`@traced`/`@featureFlag` (Runtime) ⊗ `@synchronized` (Layout,
  [decorators-behavioral.md](decorators-behavioral.md)).** Runtime interceptors wrap
  outside the Layout-baked monitor, so the trace/flag check runs *before* the
  monitor is entered — a flag-off `@featureFlag` never enters the critical section,
  and `@traced` logs entry before the (possibly suspending) monitor acquire. Correct.
- **`@featureFlag` ⊗ `@traced` on a Runtime-decorated `Bool`/`Block`
  ([ADR-0053](../../../adr/accepted/0053-runtime-decorator-interception-reuses-override-epoch-guard.md)).**
  Decorating a sacred-selector family flips the family's `*_sacred_pristine` flag and
  deopts the inliner — vanishingly rare and already covered by ADR-0053; noted so an
  implementer knows `@traced`/`@featureFlag` on `Bool` is not free.

## Hazards

- **`@delegate` selector list drifts from the delegate's real protocol.** If
  `_engine` gains a `reset` method, `car.reset` still misses until the author adds
  `#reset` to the list — the explicit list is a maintenance surface. Accepted: this
  is the cost of static forwarding, and it is *visible* (the class's protocol is what
  it declares), unlike a Dispatch forwarder that silently tracks the field's protocol
  and thereby couples the two classes invisibly. The whole-protocol escape hatch is
  Open question D-1.
- **`@traced`/`@featureFlag` per-send cost.** Both are Runtime interceptors — a
  genuine per-call branch on the decorated receiver
  ([decorators/README.md](../decorators/README.md)'s "Inline-cache invalidation" hazard,
  guarded by ADR-0053). Undecorated classes pay one bit-check; decorated ones route
  through the interceptor. `@featureFlag`'s off case can expose the "would I do
  anything" bypass probe ([decorators.md Future optimizations](../decorators/README.md)) so a
  hot, usually-off flag skips the full `aroundSend` body.
- **`@featureFlag(whenOff: fallback(#sel))` fallback-signature mismatch.** The
  fallback method must accept the same args as the gated method; a mismatch is caught
  at class-def (the receiver's protocol is known), not deferred to a runtime dispatch
  failure — same fail-early stance as `@delegate`'s shadow check.
- **`@traced` sink that itself sends to a `@traced` receiver.** A `Tracer` whose
  `enter`/`exit` re-enters a traced object recurses. The default `Tracer.stdout` is
  a leaf (`System.print` only); a custom sink must not send to the object graph it
  traces — the same reentrancy caveat proxy.md states for DNU handlers.

## Test strategy

Golden `.ph` cases (positive stdout-exact unless noted):

- **`@delegate`**: (1) forwards each listed selector to the field (assert result +
  that the field's method ran); (2) `#start` vs `#start(_)` forward independently
  (selector identity); (3) *negative-lane* — listing a selector the class already
  defines → `attr.delegate_shadow` compile error with a span; (4) *negative-lane* —
  two `@delegate`s naming the same selector → `attr.delegate_conflict`; (5)
  reflection — `Car.methods` includes the generated forwarders (visible protocol);
  (6) erasure — the generated forwarders are ordinary methods (stripping `@delegate`
  removes exactly those members, `runtime: false`).
- **`@traced`**: (1) default set — entry/exit/error logged, timing absent; (2)
  `timing: true` adds elapsed; (3) a thrown method logs `threw` and re-raises (never
  swallowed — assert the error propagates); (4) custom `sink:` receives structured
  calls (assert via a recording double); (5) erasure — stripping `@traced` leaves the
  method's result identical (observability-only, `runtime: true` but result-preserving).
- **`@featureFlag`**: (1) flag on → method runs; (2) flag off, default → raises
  `FeatureDisabled`; (3) `whenOff: fallback(#sel)` off → fallback runs with the same
  args; (4) `whenOff: skip(v)` off → returns `v`; (5) flag flipped at runtime between
  two calls → second call observes the new state (proves per-send consultation, not
  baked); (6) *negative-lane* — `fallback(#sel)` with a mismatched signature →
  compile error.
- **Composition lane**: (1) `@traced @featureFlag` — disablement visible in trace;
  `@featureFlag @traced` — disabled feature invisible; (2) `@traced` on a
  `@delegate`-generated forwarder traces the forward; (3) `@featureFlag @synchronized`
  — off flag never enters the monitor.

## What this precludes

- **A silently-lossy `@featureFlag`.** The default raises; the only "no-op" modes are
  the *explicit* `skip(value)` and `fallback(#sel)`, where the author names the
  intent. There is no mode that returns `None` and hides the disablement — foreclosed
  by design.
- **`@delegate` as an open whole-protocol forwarder (by default).** The default is
  the explicit, statically-checked, IC-friendly selector list. Open-ended forwarding
  is not built into `@delegate`; it lives in the DNU/`Proxy` library
  ([proxy.md](proxy.md)) or a hand-written `doesNotUnderstand`, and whether a
  dedicated `@forwardMissing` Dispatch decorator should exist is Open question D-1 —
  not silently folded into `@delegate`.
- **`@traced` subsuming the `Trace` proxy (or vice-versa).** The two granularities
  are kept distinct; neither is deleted, and neither is presented as a replacement for
  the other's use case.

## Open questions — resolved

| # | Decision |
|---|----------|
| D-1 | **(a) — `@delegate` stays Compile-only, explicit-selector.** No separate Dispatch-tier `@forwardMissing(to:)`. Open whole-protocol forwarding remains the hand-written `Proxy`/DNU library's job ([proxy.md](proxy.md)). The `@forwardMissing` alternative is recorded for v0.3 reconsideration in [DEFERRED.md](../../../forge/DEFERRED.md), to be revisited only if whole-interface delegation proves common enough to earn a second decorator. |
| D-2 | **(a) — `Tracer` is a ratified core protocol** (`enter`/`exit`/`threw`) with a `Tracer.stdout` default, as specified above. The raw-three-blocks alternative is recorded for v0.3 in [DEFERRED.md](../../../forge/DEFERRED.md). |
| D-3 | **(a) — `Flags` is a ratified ambient core module** (`Flags.enabled(name)`), one global registry, as specified above. The injected/per-scope `FeatureFlags` service alternative is recorded for v0.3 in [DEFERRED.md](../../../forge/DEFERRED.md), as the natural upgrade once dependency injection (`@inject`, [decorators-stdlib.md](decorators-stdlib.md)) is specified. |
