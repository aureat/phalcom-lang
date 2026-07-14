# Persistence / ORM decorators — `@entity`, `@column`, associations

- Status: **Accepted** (ratified 2026-07-14 — banner was stale; ratified under
  [ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md)'s
  broad Install/Dispatch/Runtime ratification, 2026-07-13)
- Date: 2026-07-12
- Depends on:
  [decorators.md](decorators.md) (Compile tier for schema derive; Dispatch tier for lazy load) ·
  [proxy.md](proxy.md) (`Lazy` — associations load on first access) ·
  [decorators-stdlib.md](decorators-stdlib.md) (`@validate` on columns; lifecycle hooks)
- Related:
  [decorators-web.md](decorators-web.md) (a `@resource` maps onto an `@entity`) ·
  [reactivity.md](reactivity.md) (`@observable` columns → dirty tracking) ·
  [values-and-absence.md](../values-and-absence.md) (`Option`)

The ORM tier. Like the web tier, the schema-shaping decorators are **Compile**
(`runtime: false`): the table schema, the migration, and the (de)serializer are a
pure function of the source, so `migrate` and `schema-dump` run at build time. The
one runtime-tier piece is the **lazy association**, which is a
[`Lazy`](proxy.md) proxy — a Dispatch-tier accessor that runs its query on first
access. Compiler-owned decorators are given as **descriptor + derivation**.

## `@entity(table:)` — Compile, class-level

The anchor. From the annotated `@column` fields it derives, in one pass:

- **schema** — `CREATE TABLE` shape and column types;
- **migration** — the diff against the prior schema;
- **query surface** — `Entity.find(id) -> Option`, `Entity.where(…)`,
  `instance.save`, `instance.delete`;
- **mapping** — row ↔ instance (`decode`/`encode`);
- **serialization** — instance ↔ `Map`/JSON (shared with [`@data`](decorators-stdlib.md)).

```phalcom
@entity(table: "orders")
class Order { @id @column var _id: Uuid;  @column var _quantity: Int }
// derives:
//   schema:  orders(id UUID PRIMARY KEY, quantity INT NOT NULL)
//   query:   Order.find(id) / Order.where(...) / order.save / order.delete
//   mapping: Row <-> Order   (decode/encode)
```

## `@column(type:, …)` / `@id` — Compile, field-level

Maps a field to a column and contributes its accessor, schema entry, and
serialization. `@id` marks the primary key (and makes the field the identity used
by `find` and by web instance-routes). `@column` composes with **Install-tier**
[`@validate`](decorators-stdlib.md): Compile *generates* the setter, Install *wraps
the generated setter* with the check — the phase order's "each later tier decorates
what an earlier tier produced."

```phalcom
@id @column          var _id: Uuid          // PK column + serialization
@column @validate(min: 1) var _quantity: Int // column;  setter wrapped with `>= 1` check
@column(type: "money") var _total: Money      // explicit SQL type
```

## `@index` / `@unique` — Compile

Contribute constraints to the derived schema and migration.

```phalcom
@column @unique var _email: String            // -> UNIQUE INDEX in the migration
@index(["_customerId", "_status"]) class Order { … }   // composite index
```

## `@default(v)` — Compile + Layout

A column default; also seeds the slot's initial value.

```phalcom
@column @default("open") var _status: String  // schema DEFAULT 'open' + slot init
```

## `@belongsTo(T)` / `@hasMany(T)` — Compile + Dispatch (lazy)

Declare an association. Compile derives the foreign-key column and the query;
**Dispatch** makes the accessor a [`Lazy`](proxy.md) proxy that runs exactly one
query on first access and caches the result — this is how N+1 loads are avoided
without eager-loading everything.

```phalcom
@belongsTo(Customer) var _customer            // FK: customer_id
// derives a lazy accessor:
customer {
  return _customerLoaded.match(
    some: { c => c },
    none: {
      let c = Customer.find(_customerId).unwrapOr(None)   // one query, first access only
      _customerLoaded = Some.new(c)
      c
    }
  )
}
```

```phalcom
@hasMany(LineItem) var _items                 // derives: LineItem.where(orderId: _id), lazy collection
```

`@hasMany` returns a lazy collection proxy: iterating it triggers the query; slicing
or counting can push down to SQL (`items.count` → `SELECT COUNT(*)`) rather than
loading rows — the proxy is the seam where that optimization lives.

## Lifecycle hooks — Install

`@beforeSave`, `@afterCreate`, `@beforeDelete`, `@afterUpdate` register a method as
a callback that the derived `save`/`delete` invokes at the right point (inside the
same [`@transactional`](decorators-stdlib.md) scope as the write).

```phalcom
@beforeSave  normalizeEmail() { _email = _email.toLower }
@afterCreate sendWelcome()    { Mailer.welcome(self) }
// derived save():
//   runs @beforeSave hooks -> INSERT/UPDATE -> runs @afterCreate hooks (new rows only)
```

## `@observable` columns → dirty tracking

> **Not a separate `@observable`.** This section *consumes* the one reactive
> `@observable` decorator ([decorators-observable.md](decorators-observable.md)); it is
> not a third, ORM-specific sense of the name. A `Signal`-backed column gives dirty
> tracking for free precisely because it is the same reactive field the reactivity
> layer defines — the persistence and reactive substrates meet on one slot. No naming
> collision; E-3 below (opt-in vs `@entity`-implied) is the only open knob.

A column that is also [`@observable`](reactivity.md) (Layout) is `Signal`-backed, so
the ORM gets **dirty tracking for free**: `save` writes only the columns whose
signals fired since load, and an admin/live view can subscribe to changes without
polling. The reactive substrate and the persistence substrate meet on the same slot.

## Open questions

| # | Question |
|---|----------|
| E-1 | Does `find`/`where` return `Option`/a lazy query proxy, and is the query builder itself decorator-derived or a hand-written companion the derive targets? |
| E-2 | Lazy associations need a unit-of-work / identity map so `a.customer == b.customer` for the same row; is that ambient (a session) or per-entity? |
| E-3 | Dirty tracking via `@observable` columns changes slot representation (Layout) — is it opt-in per column or implied by `@entity`? |
| E-4 | Migration derivation is a build-time consumer of Compile output (like OpenAPI in [decorators-web.md](decorators-web.md)) — one reflective `T.schema` contract for both? |
