# Proxies & Prototypes — one interception substrate, two libraries

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-12
- Depends on:
  [method-lookup.md](../method-lookup.md) (`doesNotUnderstand`, `Message`, `perform`) ·
  [object-model.md](../object-model.md) (`Object` universal protocol, `Error` root) ·
  [functions.md](../functions.md) (`invokeOn`, `callWith`, `bind`)
- Related:
  [decorators.md](decorators.md) (Dispatch/Runtime tiers; a decorator is a method-granularity proxy) ·
  [reactivity.md](reactivity.md) (auto-tracking is read-interception) ·
  [experimental/annotation-paradigm-bridges.md](../experimental/annotation-paradigm-bridges.md)

## Thesis

`perform` + `doesNotUnderstand` ([method-lookup.md](../method-lookup.md)) is **one
substrate**, and more than one standard library falls out of it. This note specs
two: a **proxy** library (transparent wrappers) and a **prototype** library
(prototype-based objects). Both are the same mechanism — a failed send is reified
as a `Message` and re-dispatched — at different granularities:

| | wrap **once** (install-time) | consult **per send** |
|---|---|---|
| **method** granularity | Install-tier decorator (`@memoize`) | Runtime tier scoped to one selector |
| **object** granularity | **Dispatch / DNU proxy** | **Runtime / around-send proxy** |

A proxy is a decorator at object granularity; an Install-tier decorator is a proxy
for one method. See [decorators.md](decorators.md) for the full tier axis.

## The MOP surface these libraries stand on

All of the below is already in the target spec:

- **`Message`** — the reified failed send ([method-lookup.md §2](../method-lookup.md)):
  `msg.selector` (encoded arity+kind `Symbol`), `msg.name` (base-name `String`),
  `msg.labels` (`List` of `String`), `msg.args` (`List`).
- **`Object.perform(selector, args)`** — reflective send; accepts **only** selector
  symbols ([method-lookup.md §3](../method-lookup.md)). Forwarding a `Message` is
  `target.perform(msg.selector, msg.args)`.
- **`Object.respondsTo(sel)`**, **`Object.doesNotUnderstand(msg)`** —
  [object-model.md](../object-model.md). The default DNU raises `MessageNotUnderstood`.
- **`invokeOn(recv, args)`**, **`callWith(args)`** — apply a callable to an explicit
  receiver / arg `List` ([functions.md](../functions.md)).

Dynamic typing is the enabling condition (see [decorators.md §Interaction with
dynamic typing](decorators.md)): a runtime `isA(T)` check is itself a message a
proxy can intercept and answer as its target's class, which is what makes a proxy
substitutable for the object it wraps. Under static erasure it would not typecheck.

## Library 1 — `Proxy`

A bare object that understands *nothing* of the business protocol: every send
misses and routes through one `doesNotUnderstand`, so a single override intercepts
the target's entire protocol at once.

```phalcom
class Proxy {
  construct on(target:) { _target = target }
  doesNotUnderstand(msg) { return _target.perform(msg.selector, msg.args) }
  respondsTo(sel)        { return _target.respondsTo(sel) }
}
```

**Caveat (identity leak).** `==`, `class`, `hash`, `toString` live on `Object`, so
they never miss and never reach DNU — a `Proxy` leaks its own identity for those.
Full transparency needs either a near-empty root superclass (a `ProtoObject`, not
yet in the model) or the Runtime around-send tier, which intercepts *before*
lookup. This is tracked as [open question P-1](#open-questions).

### `Lazy` — virtual proxy

> **Granularity split ([ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)).**
> This `Lazy` proxy defers building a **whole object**. It is a *different mechanism*
> from the `@lazy` **decorator** ([decorators-behavioral.md](decorators-behavioral.md)),
> which caches one **method result** in a per-receiver slot. Wrap a foreign object you
> can't annotate → `Lazy` proxy; make one getter on your own class compute-once →
> `@lazy`.

Defer building an expensive target until its first message.

```phalcom
class Lazy : Proxy {
  construct from(thunk:) { _thunk = thunk; _built = None }
  doesNotUnderstand(msg) { return self.force.perform(msg.selector, msg.args) }
  force {
    return _built.match(
      some: { it => it },
      none: { let it = _thunk.call(); _built = Some.new(it); it }
    )
  }
}
```

### `Trace` — observability

> **Granularity split ([ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)).**
> This `Trace` proxy observes a **black-box object** from outside (whole protocol,
> with the identity leak of [P-1](#open-questions)). To instrument **your own**
> declaration, use the `@traced` **decorator**
> ([decorators-dispatch-observability.md](decorators-dispatch-observability.md)) —
> Runtime tier, guarded by ADR-0053, no identity leak. Same feature, two granularities;
> both are kept.

Log every message crossing the boundary, for any object.

```phalcom
class Trace : Proxy {
  construct on(target:, tag:) { _target = target; _tag = tag }
  doesNotUnderstand(msg) {
    System.print("[\(_tag)] -> \(msg.name)\(msg.args)")
    let result = _target.perform(msg.selector, msg.args)
    System.print("[\(_tag)] <- \(msg.name) = \(result)")
    return result
  }
}
```

### `Retry` — resilience

> **Granularity split ([ADR-0057](../../../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)).**
> This `Retry` proxy retries **every** method of a wrapped object indiscriminately —
> **unsafe unless every method is idempotent**, which rarely holds for a whole object.
> The **recommended** surface is the `@retry` **decorator**
> ([decorators-behavioral.md](decorators-behavioral.md)), applied per method (the
> author asserts *this* method is retry-safe). Use this proxy only for a black-box
> object you cannot annotate, and only when its whole protocol is idempotent.

One wrapper covers every method.

```phalcom
class Retry : Proxy {
  construct on(target:, times:) { _target = target; _times = times }
  doesNotUnderstand(msg) { return self.attempt(msg, _times) }
  attempt(msg, left) {
    try { return _target.perform(msg.selector, msg.args) }
    catch (e: Error) {                       // scope to a transient type in real code
      if (left == 0) { throw e }
      return self.attempt(msg, left - 1)
    }
  }
}
```

### Composition

Proxies compose by nesting, because each speaks only `perform`/`doesNotUnderstand`:

```phalcom
let raw    = Lazy.from(thunk: openLedger)          // nothing opened yet
let robust = Retry.on(target: raw, times: 3)
let ledger = Trace.on(target: robust, tag: "ledger")

ledger.deposit(100)     // Trace logs -> ; Retry guards ; Lazy forces the target (once) ; deposit runs
```

The connection opens on the first message, once; every send is logged and
retry-guarded with no cooperation from the target.

## Library 2 — `Capability` (object-capability membrane)

A read-only, revocable, transitively-wrapping proxy — the pattern that classes and
decorators cannot express, because it must govern the *whole* protocol of
*arbitrary* objects the host never modified. Built on pure DNU (no unratified
around-send): because the proxy understands nothing, mutators miss too and are
denied in one place.

```phalcom
class AccessDenied  : Error { construct on(selector:) { _selector = selector } }
class AccessRevoked : Error { construct on(selector:) { _selector = selector } }

// Shared revocation cell — one flip disables an entire membrane subgraph at once.
class Revoker {
  construct new() { _live = true }
  revoke { _live = false; return self }
  isLive => _live
}

class Capability {
  construct on(target:, allow:, via:) { _target = target; _allow = allow; _via = via }

  static grant(target:, allow:) {
    return Capability.on(target: target, allow: allow, via: Revoker.new())
  }
  revoke { _via.revoke; return self }

  doesNotUnderstand(msg) {
    if (_via.isLive == false)               { return AccessRevoked.on(selector: msg.name).raise() }
    if (_allow.includes(msg.name) == false) { return AccessDenied.on(selector: msg.name).raise() }
    return self.wrap(_target.perform(msg.selector, msg.args))
  }

  // Results cross the membrane wrapped in the SAME revoker, so the whole reachable
  // graph is governed and dies together. Leaf values pass through untouched.
  wrap(v) {
    if (v.isA(Number) or v.isA(String) or v.isA(Bool)) { return v }
    return Capability.on(target: v, allow: _allow, via: _via)
  }
}
```

```phalcom
let doc  = Document.load(path: "q3-report.md")     // host owns the real, mutable graph
let view = Capability.grant(target: doc, allow: ["title", "sections", "at", "text"])

plugin.render(view)          // reads the graph transitively — each hop is itself a Capability
//   view.setTitle("x")      // -> AccessDenied  (mutators aren't whitelisted)
view.revoke                  // ONE call kills the whole subgraph
//   view.title             // -> AccessRevoked
```

Three properties fall out: **whole-protocol interception** (you can't forget to
guard a mutator, even one added later), **transitive containment** (a child fetched
through the view is itself governed — no escape to a raw mutable object), and
**atomic revocation** (one flip severs the subgraph). This is the E / Caja /
JS-`Proxy` membrane lineage. Its `Revoker` is structurally the same one-cell
teardown as reactive effect disposal ([reactivity.md](reactivity.md)).

## Library 3 — `Prototype`

Prototype-based objects (Self/JS-style differential inheritance) on the class VM,
built entirely on the message substrate: a miss resolves against a slot bag, then
delegates to a parent object.

```phalcom
// Assumes the no-nil Map idiom the core already uses for List/Option:
//   Map.at(k) -> Option ,  Map.at(k, put: v) -> self
class Prototype {
  construct new() {
    _methods = Map.new()      // name -> block; the block's first param is the receiver
    _fields  = Map.new()      // name -> value
    _parent  = None           // Option<Prototype>
  }

  static rooted { return Prototype.new() }
  clone         { let c = Prototype.new(); c.delegateTo(self); return c }
  delegateTo(p) { _parent = Some.new(p); return self }

  field(name:, value:) { _fields.at(name, put: value); return self }
  def(name:, does:)    { _methods.at(name, put: does); return self }
  set(name:, to:)      { _fields.at(name, put: to);    return self }

  // The whole object model is this one method:
  //   1. own method slot?  run it with `self` as receiver (late-bound self)
  //   2. own data slot?    return it
  //   3. else              delegate to parent, or raise MessageNotUnderstood
  doesNotUnderstand(msg) {
    return _methods.at(msg.name).match(
      some: { body => body.callWith(self.receiverArgs(msg)) },
      none: { _fields.at(msg.name).match(
        some: { value => value },
        none: { _parent.match(
          some: { up => up.perform(msg.selector, msg.args) },
          none: { super.doesNotUnderstand(msg) }
        )}
      )}
    )
  }

  receiverArgs(msg) {                     // prepend `self` so slots see their receiver
    let args = List.new()
    args.add(self)
    msg.args.each { x => args.add(x) }
    return args
  }
}
```

**Caveat.** `def`/`set`/`clone`/`field`/`delegateTo`/`receiverArgs` are reserved
meta-protocol names a slot cannot shadow (JS's `__proto__` problem).

### Usage — a multi-tenant configuration cascade

Layered config (platform → plan → tenant → request) *is* differential inheritance:
each layer stores only its deltas, resolution falls through automatically, and
*computed* settings re-derive against whichever layer asks.

```phalcom
let defaults = Prototype.rooted
  .field(name: "theme",       value: "light")
  .field(name: "maxUploadMb", value: 10)
  .field(name: "seats",       value: 3)
  .def(name: "storageGb", does: { me => me.seats * 5 })   // derived from seats

let proPlan = defaults.clone
  .set(name: "maxUploadMb", to: 100)
  .set(name: "seats",       to: 25)

let acme = proPlan.clone
  .set(name: "theme", to: "acme-dark")
  .def(name: "storageGb", does: { me => 500 })            // override the COMPUTATION itself

let request = acme.clone
  .set(name: "theme", to: "high-contrast")                // ephemeral, GC'd after the request

System.print(request.theme)        // high-contrast  (own)
System.print(request.maxUploadMb)  // 100            (from proPlan, 3 hops up)
System.print(request.storageGb)    // 500            (acme's computed override)
System.print(defaults.storageGb)   // 15             (same method, self=defaults -> 3 * 5)
```

`storageGb` is a *method* slot inherited by delegation, so it recomputes against
whichever layer asks (late-bound `self`) — something plain dict-merge cannot do. A
tenant overrides not just an input but the derivation.

## What this precludes / hazards

- **Silent identity leak.** The DNU `Proxy` base is transparent only for *missed*
  sends; `Object`-level protocol leaks. Do not present it as a full membrane — that
  is `Capability`'s job (and even it does not spoof `==` without around-send).
- **Reentrancy.** A DNU handler must never send a *missing* message to `self`;
  forward via `perform` on the target, and keep bookkeeping in real methods (which
  do not miss). The `Membrane` handler must stay a separate object from the proxy
  under any future around-send tier, or its own sends re-trigger interception.
- **`perform` takes only selector symbols.** Forwarding `msg.selector` is faithful;
  never reconstruct a selector from `msg.name` without labels.

## Open questions

| # | Question |
|---|----------|
| P-1 | Commit to a near-empty root (`ProtoObject`) so a cheap DNU proxy can be fully transparent, or declare that transparency always costs the Runtime around-send tier? |
| P-2 | Identity/hash policy is per-proxy: a `Trace`/`Lazy` wants `==`/`hash` transparent to the target; a `Capability` wants them opaque (the boundary *is* a different identity). Offer both as library policies rather than a global default. |
| P-3 | Does `Map` expose the assumed no-nil surface (`at(k) -> Option`, `at(k, put: v) -> self`), and is `isPrimitive` / a leaf-value predicate available for the membrane pass-through? |
| P-4 | `Prototype` reserves meta-protocol selector names; is a sigil/namespace needed so a slot may legitimately be called `clone` or `set`? |
