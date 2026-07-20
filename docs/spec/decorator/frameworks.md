# Framework decorator families — persistence and web

- Status: **Design-ratified (ADR-0054 broad, banners flipped 2026-07-14),
  v0.3+ horizon, open questions live (E-1…E-4, W-1…W-3).** Source drafts:
  [decorators-persistence.md](../v0.2/drafts/decorators-persistence.md),
  [decorators-web.md](../v0.2/drafts/decorators-web.md). This file verifies
  the families against the decorator system and applies the naming
  convention; it does not re-spec the frameworks.
- Nothing here is scheduled. These families presuppose infrastructure that
  does not exist (`Database`, an HTTP server, `Session`, `EventBus`,
  `Scheduler`) — the decorators are the *surface* of those systems, and
  surfaces do not land before their systems.

## Renames under the naming convention (COLL-2, COLL-4 resolved)

All framework decorators are library `Attribute` subclasses ⇒ Capitalized:

| Draft spelling | Canonical | Collision dissolved |
|---|---|---|
| `@entity @column @id @index @unique @default @belongsTo @hasMany` | `@Entity @Column @Id @Index @Unique @Default @BelongsTo @HasMany` | — |
| `@beforeSave @afterCreate @beforeDelete @afterUpdate` | `@BeforeSave @AfterCreate @BeforeDelete @AfterUpdate` | — |
| `@resource / @controller` | `@Resource / @Controller` | — |
| `@get @post @put @patch @delete @route` | `@Get @Post @Put @Patch @Delete @Route` | **COLL-2**: builtin `@get` (field accessor) vs `@Get(path:)` (HTTP) now differ by resolution path, not just case |
| `@body @param @query @header` | `@Body @Param @Query @Header` | also separates web `@Param` from Phaldoc's `@param` tag visually, though they were already namespace-disjoint |
| `@subscribe / @on` | `@Subscribe / @Handles` | **COLL-4**: `@on` was one character from builtin `@On`; renamed outright, not just capitalized |
| `@cron / @job` | `@Cron / @Job` | — |

Case-only distinction (`@get`/`@Get`) is defensible because the two resolve
through *different paths* (registry vs class chain) with registry precedence
— a typo'd case yields `attr.unknown` or an illegal-target error, never the
other decorator silently. `@on`/`@On` failed this test (both would resolve as
classes); hence the rename rather than the capital.

## Verification highlights (what survives scrutiny, what's flagged)

- **Compile-tier derive discipline holds.** `@Entity`/`@Column` deriving
  schema/query/mapping, and route decorators deriving binder + serializer +
  OpenAPI from one annotation, are the `@data` pattern at framework scale —
  one fact, N consumers. The shared open question (E-4 ≡ W-3: a stable
  reflective `T.schema` as the contract between derive and build-time
  tooling) should be answered **once**, jointly — it is the same question.
- **W-1 is a grammar question, not a framework question.** Parameter-position
  binders (`@Body x`) require extending the attribute grammar beyond
  class-member position ([mechanism.md §1](mechanism.md) preclusion). The
  method-level alternative (`@Bind(body: "x")`) needs no grammar change;
  the family should prototype with it before petitioning the grammar.
- **Lazy associations (`@BelongsTo`/`@HasMany` via `Lazy` proxies) are the
  Dispatch tier's first real customer** — noted in
  [interception.md](interception.md), which keeps the tier unbuilt until a
  concrete decorator forces it. E-2 (identity map: `a.customer == b.customer`
  for one row) is the hard question and is an *object-identity* question —
  it must be answered against the `==` ladder, not inside the ORM.
- **The pipeline order is the framework's one load-bearing spec**: binder
  decode (400s) → Runtime (`@Traced`) → Install (`@Authorize` → `@Validate`
  → `@RateLimit` → `@Idempotent` → `@Transactional`) → woven contracts →
  body → serialize. It instantiates the tier model's fixed phase order at
  request scale; any implementation starts by fixture-pinning it.
- **Ambient authority flag.** `Session.current`, `Flags`, `Database`,
  `EventBus` are ambient singletons. ADR-0058's `Reactive` sets the
  precedent that such state is a *native module*, deliberately admitted
  one at a time. Each framework ambient needs the same explicit admission —
  the drafts assume them; the specs must not.

## What this precludes

- Landing any framework decorator before its runtime system, or before the
  Install-tier metaobject gate ([behavioral.md](behavioral.md) plan §1) and —
  for associations — the Dispatch tier.
- Lowercase spellings of framework decorators; the draft spellings are
  historical the moment this file merges.
- A second dirty-tracking mechanism: `@Entity` consumes `@observable`
  ([reactive.md](reactive.md)); E-3 (opt-in per column vs implied) stays the
  only open knob.
