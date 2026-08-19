# Decorator library — definitions, by tier and owner

- Status: **Draft** (exploration only — not implemented; see [decorators/](../decorators/) for the built surface)
- Ratification note: ratified 2026-07-14 under [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md)'s broad Install/Dispatch/Runtime ratification (2026-07-13). **None of it is built**, and this file additionally uses a superseded surface — see the relocation table below. `@data`/`@get`/`@set`/`@construct`/`@requires`/`@ensures`/`@invariant` **are** built; their as-built specs are in [decorators/](../decorators/), and the sketches of them below are historical.
- Date: 2026-07-12
- Depends on:
  [decorators/README.md](../decorators/README.md) (the five-tier model, descriptor, phase order) ·
  [functions.md](../functions.md) (`invokeOn`, `signature`, `bind`) ·
  [method-lookup.md](../method-lookup.md) (Dispatch tier: `doesNotUnderstand`, `perform`) ·
  [error-handling.md](../error-handling.md) (`try`/`catch`, `throw`, `raise`)
- Related:
  [decorators-web.md](decorators-web.md) (HTTP/transport framework tier) ·
  [decorators-persistence.md](decorators-persistence.md) (ORM tier) ·
  [reactivity.md](reactivity.md) (`@observable`, `@computed` are its ergonomic layer) ·
  [proxy.md](proxy.md) (an Install decorator is a method-granularity proxy)

> **Superseded surface + relocations (2026-07-13).** This doc uses the pre-[A-1](../decorators/on.md)
> registration surface (`@install`/`@dispatch`/`@runtime` markers, lower-case class
> names, `wrap(method) → { recv, args => … }`). The ratified surface is
> `@On(target…, tier: …)` on a capitalized `Attribute` subclass whose hook returns a
> `Method.fromBlock` ([attribute-classes.md A-1](../decorators/on.md)). The following
> decorators now have **authoritative dedicated specs** at ratification depth; treat the
> subsections below as historical sketches, superseded on both surface and semantics:
>
> | Decorator(s) here | Authoritative spec | Key correction |
> |---|---|---|
> | `@memoize`, `@synchronized`, `@retry` | [decorators-behavioral.md](decorators-behavioral.md) | `@synchronized` is a **cooperative monitor**, not an OS `Mutex`; `@synchronized` default is **Layout/per-receiver** |
> | `@traced`, `@featureFlag`, `@delegate` | [decorators-dispatch-observability.md](decorators-dispatch-observability.md) | `@delegate` is **Compile** (explicit selectors), not Dispatch; `@featureFlag` off-default is **raise**, not silent `None` |
> | `@observable` | [decorators-observable.md](decorators-observable.md) | tier is **Layout + generate**, not "Layout + Install" |
>
> `@computed` (Install → **Layout** per [ADR-0052](../../../adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md))
> and `@timed`/`@authorize`/`@transactional`/`@rateLimit` remain sketched here pending
> their own dedicated specs.

This is the concrete standard library implied by [decorators/README.md](../decorators/README.md). It
defines every core decorator, split by **owner**: the two static tiers are
compiler-owned (given as descriptor + expansion, since they are not user-writable
per decorators.md); Install / Dispatch / Runtime are user-definable and given in
full source. The framework tiers (`@resource`, `@entity`, …) live in
[decorators-web.md](decorators-web.md) and
[decorators-persistence.md](decorators-persistence.md); the
[worked example](#worked-example-a-vertical-slice-resource) below ties all of them
together.

## The definition surface

A decorator is a class named for its `@sigil`, tagged with its tier, implementing
that tier's one hook. Args in `@name(args)` are captured in `@constructor
new(...)`;
the hook closes over them. Inside a hook `self` is the decorator instance and
`recv` is the domain receiver.

| Tier | Hook the class implements | Owner | `runtime` |
|------|---------------------------|-------|-----------|
| Compile | `expand(node, target)` → members | compiler | `false` |
| Layout | `reserve(node, target)` → slots | compiler | `false` |
| Install | `wrap(method)` → `{ recv, args => … }` | user | `true` |
| Dispatch | `onMiss(recv, msg)` → value | user | `true` |
| Runtime | `aroundSend(recv, msg, proceed)` → value | user | `true` |

> **Unratified surface.** The `@install`/`@dispatch`/`@runtime` registration
> markers, the hook names (`wrap`/`onMiss`/`aroundSend`), and `proceed` are a
> concrete rendering of decorators.md's Rust descriptor — its open **D-2**. The
> tier determines the `runtime` flag; no separate declaration is needed.

Composition is the one fixed total order from
[decorators.md §Composition](../decorators/README.md): generate → weave → finalize → install
→ dispatch → runtime. On a member wearing several, weave bakes into the body,
Install wrappers nest source-order (innermost-last), Dispatch fills the miss path,
and the Runtime around-hook fires first per send and `proceed`s inward.

## Compile / Layout tier — moved

This section previously sketched `@data`, `@get`/`@set`, `@construct`,
`@requires`/`@ensures`, `@invariant` and `@observable`. **All of those except
`@observable` are now built**, and their as-built specs live in
[decorators/](../decorators/) — one file each:

| Decorator | As-built spec |
|---|---|
| `@data` | [decorators/data.md](../decorators/data.md) |
| `@get` / `@set` | [decorators/accessors.md](../decorators/accessors.md) |
| `@construct` | [decorators/construct.md](../decorators/construct.md) |
| `@requires` | [decorators/requires.md](../decorators/requires.md) |
| `@ensures` | [decorators/ensures.md](../decorators/ensures.md) |
| `@invariant` | [decorators/invariant.md](../decorators/invariant.md) |
| `@sealed` / `@variant` | [decorators/sealed.md](../decorators/sealed.md) |

The sketches are not reproduced here — several of them diverged from what shipped
(notably `@data`'s `hash`/`toString`/`with` pseudocode and `@ensures`'s `result`
binding), and the as-built files are authoritative. `@observable` remains unbuilt;
see [decorators-observable.md](decorators-observable.md).

## Install tier (`wrap(method)` → callable)

### `@memoize`

Class-wide cache, keyed on `(recv, args)` — **not** `args` alone. Keying on
`args` only would share one cached result across every instance of the
decorated class, silently wrong for any method whose result depends on
receiver state (correct only by accident for a pure function of its
arguments, e.g. `Fib.fib`). The `(recv, args)` tuple is itself the key into
one shared `Map`; this is a class-wide cache (not per-receiver storage — see
`attribute-classes.md`'s Install/Layout line), so it does not need a
reserved slot, but it does retain every `(recv, args)` pair it has ever seen
for the life of the attribute instance — same retention shape as `@computed`
before its Layout-tier fix (ADR-0052), acceptable here only because the *key*
holding `recv` is a tuple inside a `Map`, still a strong reference. Treat this
as a known, documented cost of the pattern, not a leak to silently fix: a
`@memoize`d method on a long-lived, high-cardinality receiver set is not
free, and callers should prefer `@computed`'s Layout-tier, per-receiver
approach when the receiver itself should own the cache lifetime.

```phalcom
@install
class memoize {
  wrap(method) {
    let cache = Map.new()                       // keyed by (recv, args)
    return { recv, args => let key = Tuple.new(recv, args);
      cache.at(key).match(
        some: { hit => hit },
        none: { let v = method.invokeOn(recv, args); cache.at(key, put: v); v }
      )
    }
  }
}
```

### `@timed`

```phalcom
@install
class timed {
  wrap(method) {
    return { recv, args =>
      let start = Clock.now
      let result = method.invokeOn(recv, args)
      System.print("\(method.signature) took \(Clock.now - start)ms")
      result
    }
  }
}
```

### `@synchronized`

```phalcom
@install
class synchronized {
  @constructor
  new() { _lock = Mutex.new() }       // one lock per decorated method; lock on `recv` for a monitor
  wrap(method) {
    let lock = _lock
    return { recv, args => lock.hold { method.invokeOn(recv, args) } }
  }
}
```

### `@authorize(role:)`

```phalcom
@install
class authorize {
  @constructor
  new(role:) { _role = role }
  wrap(method) {
    let role = _role
    return { recv, args =>
      if (Session.current.hasRole(role)) { method.invokeOn(recv, args) }
      else { AccessDenied.on(need: role).raise() }
    }
  }
}
```

### `@retry(times:, on:)`

```phalcom
@install
class retry {
  @constructor
  new(times:, on:) { _times = times; _type = on }
  wrap(method) {
    let max = _times; let type = _type
    return { recv, args => self.attempt(method, recv, args, max, type) }
  }
  attempt(method, recv, args, left, type) {
    try { return method.invokeOn(recv, args) }
    catch (e: Error) {
      if (e.is(type) == false or left == 0) { throw e }   // rethrow non-matching / exhausted
      return self.attempt(method, recv, args, left - 1, type)
    }
  }
}
```

### `@transactional`

```phalcom
@install
class transactional {
  wrap(method) {
    return { recv, args => Database.transaction { method.invokeOn(recv, args) } }  // commit on return, rollback on throw
  }
}
```

### `@rateLimit(perMinute:)`

```phalcom
@install
class rateLimit {
  @constructor
  new(perMinute:) { _max = perMinute; _window = Window.minutes(1) }
  wrap(method) {
    let max = _max; let window = _window
    return { recv, args =>
      window.tick
      if (window.count > max) { RateLimitExceeded.new().raise() }
      method.invokeOn(recv, args)
    }
  }
}
```

### `@computed`

Wraps a getter as a reactive [`Computed`](reactivity.md), one per receiver.

```phalcom
@install
class computed {
  wrap(method) {
    let memos = Map.new()                        // recv -> Computed (strong ref: dispose with the owner)
    return { recv, args => memos.at(recv).match(
      some: { c => c.value },
      none: {
        let c = Computed.new(compute: { method.invokeOn(recv, args) })
        memos.at(recv, put: c)
        c.value
      }
    )}
  }
}
```

## Dispatch tier (`onMiss(recv, msg)`, wired into the DNU chain)

### `@delegate(to:)`

Forward a sub-protocol to a component field (composition over inheritance). Runs
*before* a hand-written `doesNotUnderstand` (decorators.md **D-4**).

```phalcom
@dispatch
class delegate {
  @constructor
  new(to:) { _slot = to }              // name of the field holding the delegate
  onMiss(recv, msg) {
    let target = recv.perform(_slot, List.new()) // read recv's delegate field
    return target.perform(msg.selector, msg.args)
  }
}
```

```phalcom
class Car { @delegate(to: "_engine") var _engine }   // car.rpm, car.start(...) -> _engine
```

*(`@method_missing` is not a separate decorator — it names a hand-written method as
the class's `doesNotUnderstand`; the plain hook, not a wrapper.)*

## Runtime tier (`aroundSend(recv, msg, proceed)`)

`proceed` is the 0-arg continuation that runs the real method — the "proceed"
mechanism of decorators.md **D-3**.

### `@traced`

```phalcom
@runtime
class traced {
  aroundSend(recv, msg, proceed) {
    System.print("-> \(msg.name)\(msg.args)")
    let result = proceed.call()
    System.print("<- \(msg.name) = \(result)")
    return result
  }
}
```

### `@featureFlag(name:)`

```phalcom
@runtime
class featureFlag {
  @constructor
  new(name:) { _flag = name }
  aroundSend(recv, msg, proceed) {
    if (Flags.enabled(_flag)) { return proceed.call() }
    return None                                  // gated off -> no-op fallback
  }
}
```

## Worked example: a vertical-slice `Resource`

The payoff of the five-tier model at scale. In a normal stack the concept "an
Order" is re-declared in eight places that must be kept in sync by hand — a
migration, an ORM model, request/response DTOs, a validation schema, a serializer,
a router with per-route middleware, an OpenAPI doc, and a client SDK. They drift,
and every endpoint re-wires the same cross-cutting stack in a hand-chosen order.
Decorators collapse the eight declarations into **one source of truth** from which
the rest are *derived* ([decorators-web.md](decorators-web.md),
[decorators-persistence.md](decorators-persistence.md)), and collapse the
middleware into a **tier-ordered stack** fixed by semantics.

```phalcom
@resource(path: "/orders")               // Compile: derive REST routes + OpenAPI          (decorators-web.md)
@entity(table: "orders")                 // Compile: derive schema, migration, query builder (decorators-persistence.md)
class Order {

  @id @column
  var _id: Uuid                          // Compile: PK column + (de)serialization

  @column @validate(min: 1)              // Compile: column;  Install: wrap the generated setter with a check
  var _quantity: Int

  @column(type: "money")
  @observable                            // Layout: reactive slot -> dirty-tracking + live admin view
  var _total: Money

  @column @default("open")
  var _status: String

  @belongsTo(Customer) var _customer     // Compile + Dispatch: association loaded on first access (a Lazy proxy)
  @hasMany(LineItem)   var _items        // Compile: lazy collection

  @inject var _payments: PaymentGateway  // Layout: dependency-injected slot

  isCancellable => _status == "open"

  // ---- create : POST /orders ----
  @post                                        // Compile: route + request/response OpenAPI
  @authorize(role: "customer")                 // Install: auth gate
  @validate                                    // Install: validate @body against the derived schema
  @rateLimit(perMinute: 60)                    // Install: throttle
  @idempotent(key: header("Idempotency-Key"))  // Install: dedupe retried POSTs
  @transactional                               // Install: one DB transaction
  @traced                                      // Runtime: distributed span around everything
  @metered("orders.place")                     // Runtime: latency + count
  static place(@body draft: OrderDraft) -> Order {
    let order = Order.new(quantity: draft.quantity, customer: draft.customerId)
    order._payments.charge(draft.payment, for: order)     // the @inject'd gateway
    return order.save                                     // @entity persistence
  }

  // ---- instance action : POST /orders/:id/cancel ----
  @post(path: "/:id/cancel")                   // :id resolves the RECEIVER via @entity, then dispatches cancel()
  @authorize(role: "customer")
  @requires(self.isCancellable)                // Weave: precondition baked into the body
  @transactional
  @traced
  cancel() -> Order {
    _status = "cancelled"
    return self.save
  }
}
```

### What fires, in what order — the determinism payoff

For `POST /orders`, the fixed phase order resolves the entire stack with **no
user-visible ordering choice**:

1. **[Web-tier shell](decorators-web.md)** (Compile-derived): decode `@body` to
   `OrderDraft`, `400` on failure — *outside* the method's own stack.
2. **Runtime** first, per send: `@traced` opens a span, `@metered` starts the
   timer — they wrap everything below.
3. **Install**, source-order outermost-first: `@authorize` → `@validate` →
   `@rateLimit` → `@idempotent` → `@transactional`. Authz is checked *before*
   validation, and the transaction opens *innermost* — you physically cannot open a
   DB transaction outside the auth check, because the tiers, not the programmer,
   decide the nesting.
4. The **body** runs (with `@requires` woven in on `cancel`), then the web shell
   serializes the `Order` result.

An Express/Koa chain gets this ordering right only by discipline; here it is a
property of the model.

### The four problems it solves

- **Technical** — routing, (de)serialization, caching, transactions, lazy
  association loading (a Dispatch proxy), and dirty-tracking (`@observable` →
  reactivity) are handled by the substrate, not per endpoint.
- **Logistical (drift elimination)** — the migration, DTO schema, validator,
  serializer, OpenAPI doc, and typed client are all **derived from the same
  `@column`/`@validate`/`@post` declarations**. Change `@validate(min: 1)` once and
  every projection updates in lockstep; there is no second place to forget.
- **Architectural** — cross-cutting concerns are declarative and land in the tier
  correct for their timing; layers (persistence / transport / policy /
  observability) are orthogonal and independently pluggable; composition is
  deterministic by construction.
- **Rapid (DX)** — one class is a complete vertical slice (database → API →
  validation → docs → client). A day of wiring across eight files becomes one
  reviewable declaration.

### Why the tier model is what makes this work

Java and Python both have decorators and neither gets this. The lever is the
**tier + `runtime` flag**:

- `@resource`/`@entity`/`@get`/`@column` are **Compile** (`runtime: false`), so the
  migration SQL, OpenAPI JSON, and typed client are a **pure function of the source
  AST** — generated at *build time without running the service*, with decorators.md's
  erasure golden-test guaranteeing they don't secretly depend on runtime state.
- `@authorize`/`@validate`/`@transactional` (Install) and `@traced`/`@metered`
  (Runtime) *are* runtime behavior.

Java annotations are inert metadata needing a separate, disconnected processor;
Python decorators are runtime-only and invisible to build tooling. Phalcom's model
is the **union**: the same annotation set is simultaneously executable behavior and
build-time codegen, and the tier flag tells the toolchain honestly which is which —
which is what lets one declaration be the single source of truth for both the
running service and every generated artifact.

## Assumptions

Licensed by decorators.md's "assume full specification" working mode: the
`@install`/`@dispatch`/`@runtime` registration surface + hook names (**D-2**);
`Map.at(k) -> Option` and `Map.at(k, put: v) -> self`; `method.invokeOn` /
`method.signature` ([functions.md](../functions.md), U-CORE-3); ambient `Session`,
`Database`, `Clock`, `Mutex`, `Flags`, `Window`, and `Contract` stand for the
application + concurrency standard library.
