# Web / transport decorators — `@resource`, routing, param binding

- Status: **Accepted** (ratified 2026-07-14 — banner was stale; ratified under
  [ADR-0054](../../../adr/accepted/0054-two-speed-ratification-annotation-decorator-tiers.md)'s
  broad Install/Dispatch/Runtime ratification, 2026-07-13)
- Date: 2026-07-12
- Depends on:
  [decorators.md](decorators.md) (Compile tier, `runtime: false`, phase order) ·
  [decorators-stdlib.md](decorators-stdlib.md) (the Install/Runtime concerns these compose with)
- Related:
  [decorators-persistence.md](decorators-persistence.md) (the ORM tier a resource maps onto) ·
  [functions.md](../functions.md) (`invokeOn`) ·
  [values-and-absence.md](../values-and-absence.md) (`Option`)

The HTTP framework tier. These are almost all **Compile-tier** (`runtime: false`):
they derive static projections — a route table, request/response schemas, an
OpenAPI document, a typed client SDK — that are a **pure function of the source
AST**. That is the key property: routes and docs can be generated at *build time
without running the service*, and decorators.md's erasure golden-test guarantees
they don't secretly depend on runtime state. They are compiler-owned (per
decorators.md, user code registers only at Install/Runtime), so each is given as
**descriptor + what it derives / expands to**, not as user `wrap` source.

## `@resource(path:)` / `@controller(path:)` — Compile, class-level

Registers a route group and roots every method-level route under `path`. Derives:
the route prefix, the OpenAPI `paths` group, and dependency-injected construction
of the handler class ([`@inject`](decorators-stdlib.md) slots resolved per request).

```phalcom
@resource(path: "/orders")
class Order { … }
// derives:
Router.group("/orders", { … registrations from the @get/@post methods below … })
```

## `@get` / `@post` / `@put` / `@patch` / `@delete` / `@route(method:, path:)` — Compile, method-level

Each derives four things from one annotation: a **route entry**, a **request
binder** (from the parameter binders + their types), a **response serializer**
(from the return type), and an **OpenAPI operation**.

```phalcom
@post
static place(@body draft: OrderDraft) -> Order { <body> }
// derives:
Router.register(
  method: "POST", path: "/orders",
  handler: { req =>
    let draft = OrderDraft.decode(req.body)     // @body + type -> decode + validate (see below)
    let order = Order.place(draft)              // invoke the annotated method
    return Response.json(order.encode)          // -> Order serializer (from @entity/@data)
  },
  openapi: Operation.of(                         // pure build-time schema
    requestBody: OrderDraft.schema,
    response:    Order.schema))
```

Path parameters bind by name: `@get(path: "/:id")` makes `:id` available to a
`@param id:` binder. For an **instance** route (`@post(path: "/:id/cancel")`), the
framework loads the receiver via [`@entity`](decorators-persistence.md)
(`Order.find(id)`) and dispatches the instance method on it — so `:id` resolves the
`self`, not just an argument.

## Parameter binders — Compile, parameter-position

Bind one part of the request into a typed argument. The **type annotation drives
both** the runtime decode/validate (an Install-tier check under dynamic typing) and
the OpenAPI parameter schema (Compile). One declaration, two projections.

| Binder | Binds | Derives (OpenAPI) |
|--------|-------|-------------------|
| `@body param: T` | request body, decoded to `T` | `requestBody` schema from `T` |
| `@param id: T` | a path segment (`:id`) | `path` parameter |
| `@query(name:) q: T` | a query-string field | `query` parameter |
| `@header(name:) h: T` | a request header | `header` parameter |

```phalcom
@get(path: "/:id")
static show(@param id: Uuid, @query(name: "expand") expand: Bool) -> Order { <body> }
// derives the binder preamble:
//   let id     = Uuid.decode(req.path.at("id"))          // typed -> validated
//   let expand = Bool.decode(req.query.at("expand"))     // -> 400 on type mismatch
//   Order.show(id, expand)
```

A binder whose decode fails raises the framework's `BadRequest` (HTTP 400) *before*
any Install-tier decorator (auth, transaction) runs — it is part of the derived
handler shell, outside the method's own decorator stack.

## `@subscribe(topic:)` / `@on(event:)` — Compile + Runtime

Bind a method as a message/event consumer. Compile registers the subscription;
delivery is a Runtime send into the handler (so `@traced`/`@retry` compose over
each message exactly as they do over an HTTP request).

```phalcom
@on(event: "order.placed")
static onPlaced(@body e: OrderPlaced) { … }   // derives: EventBus.subscribe("order.placed", handler)
```

## `@cron(schedule:)` / `@job` — Compile

Register a method as a scheduled task; the schedule string is derived into the
scheduler's table at build time (so the full job inventory is statically known).

```phalcom
@cron(schedule: "0 * * * *")
static sweepExpired() { … }                    // derives: Scheduler.register("0 * * * *", handler)
```

## Composition with the Install/Runtime stack

A route method's *own* decorators (`@authorize`, `@validate`, `@transactional`,
`@traced`, …) sit **inside** the derived handler shell, in the fixed phase order of
decorators.md. So for one endpoint the total pipeline is:

```
request → [derived shell: binder decode → 400 on failure]
        → Runtime  (@traced span, @metered timer)
        → Install  (@authorize → @validate → @rateLimit → @idempotent → @transactional)
        → body (with woven @requires/@ensures)
        → [derived shell: response serialize]
```

The binder shell and serializer are Compile (static, pure); the concern stack is
Install/Runtime (behavior). See [decorators-stdlib.md §Worked example](decorators-stdlib.md)
for the full `Order` resource.

## Open questions

| # | Question |
|---|----------|
| W-1 | Are parameter-position binders (`@body`/`@param`) a distinct grammar position, or sugar for a method-level `@bind(...)` reading a labelled arg? |
| W-2 | Instance-route receiver resolution couples the web tier to `@entity.find`; is that coupling explicit (`@post(path: "/:id/cancel", load: Order)`) or inferred from the enclosing `@entity`? |
| W-3 | Client-SDK / OpenAPI generation is a build-time consumer of the Compile-tier output — ratify a stable reflective schema (`T.schema`) as the contract between derive and tooling? |
