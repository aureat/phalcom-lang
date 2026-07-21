# Framework families — experimental design (v0.3): E-1…E-4, W-1…W-3 resolved

- Status: **Experimental (v0.3 track), mandated by PDR-0018 §3.** This file
  turns [frameworks.md](frameworks.md)'s parked open questions into concrete
  experimental rulings so the families are buildable. Canonical naming (the
  Capitalized renames) is frameworks.md's table; drafts remain the prose
  source for per-decorator behavior not restated here.
- Still true and unchanged: these decorators are surfaces over systems
  (`Database`, HTTP server, `EventBus`, `Scheduler`) that must exist first.
  The mandate schedules the *design and the decorator machinery*; each
  ambient system still needs its own admission the way ADR-0058 admitted
  `Reactive` (one at a time, explicitly).

## The shared spine: `T.schema` (resolves E-4 ≡ W-3, jointly, once)

Every Compile-tier framework derive emits, besides its members, one
reflectable **schema object** retained on the class (ordinary passive
`Attribute`-store data, frozen with it under A-5):

```phalcom
Order.schema            // => a Schema: fields (name, type-tag, constraints),
                        //    associations, routes, lifecycle hooks — whatever
                        //    the class's framework decorators declared
```

- `Schema` is a plain `@data`-style value tree (Maps/Lists/Symbols — floor
  types only), *not* live metaobjects: build-time consumers (migration
  diffing, OpenAPI generation, client SDKs) read it via ordinary reflection
  and stay decoupled from compiler internals.
- One schema per class, merged across that class's framework decorators;
  producing it is part of each derive's contract, so derive and tooling can
  never drift — the tooling reads what the derive wrote, not a parallel
  parse.
- This is the whole answer to both E-4 (migrations as build-time consumer)
  and W-3 (OpenAPI/SDK as build-time consumer): **one reflective contract,
  two readers.** Precedent-with-consequence: Rails keeps schema in a
  separate migration DSL and pays permanent schema-drift tooling; ORMs that
  derive schema from the class (Django) get `makemigrations` almost for
  free. Phalcom takes the Django side with the schema reified as a value.

## Persistence rulings

- **E-1 — `find` returns `Option`, `where` returns a `Query` value.**
  `Order.find(id)` ⇒ `Some(order)`/`None` — absence is Option, never a raise
  (ADR-0007 discipline). `Order.where(status: #open)` ⇒ a lazy, immutable
  `Query` object (composable: `.where`, `.orderBy`, `.limit`) that executes
  on iteration/`toList`/`count` — count pushes down to SQL. The query
  builder is a **hand-written companion class the derive targets**, not
  itself derived: deriving a query DSL from field lists produces the
  lowest-common-denominator DSL forever (the Lombok trap); a real class can
  grow.
- **E-2 — identity map is session-scoped and explicit.** A `Session` (unit
  of work) owns the row→instance identity map; `Session.run { s => ... }`
  scopes it; `find`/`where` inside resolve through the ambient current
  session (per-fiber, `Reactive.scope`-style stack — same reified-owner
  pattern as [effect.md §2](effect.md), same fiber-switch discipline).
  Outside any session: fresh instances, no identity guarantee, documented.
  `a.customer == b.customer` for one row holds *within a session* — the
  only place the promise is implementable without a global cache that
  ADR-0052's confinement forbids by spirit (a process-wide receiver-keyed
  strong map is the leak rule's biggest possible violation).
- **E-3 — dirty tracking is implied by `@Entity`, opt-out per column.**
  `@Entity` makes every `@Column` field `Signal`-backed (consuming the one
  `@observable` mechanism); `@Column(tracked: false)` opts out. Rationale:
  partial-update `save` is the family's headline feature and silently wrong
  if a column isn't tracked — default-on puts the failure mode (slightly
  more memory) on the safe side. The Layout cost is per-entity-class, priced
  at design time, and `tracked: false` is the escape.
- Lifecycle hooks (`@BeforeSave` etc.): Install-tier registrations invoked
  by the derived `save`/`delete` inside the transaction scope — order:
  before-hooks (declaration order) → write → after-hooks. A hook that
  throws aborts the write (before) or the transaction (after) — the
  pipeline fixture pins it.

## Web rulings

- **W-1 — no parameter-position attribute grammar.** Binders are
  method-level: `@Body("order")`, `@Param("id")`, `@Query("page")` name the
  *parameter label* they bind. The parameter-position form (`@Body order:`)
  required extending the attribute grammar beyond class-member position —
  a whole grammar axis for one family's ergonomics. Method-level loses
  nothing semantically (labels are already selector identity — the binder
  names a label that must exist, checked at class-definition), and if the
  ergonomics genuinely hurt, the grammar extension can be its own PDR later
  with this design as its fallback. Ship the cheap form first.
- **W-2 — receiver loading is explicit.** Instance routes declare
  `@Post(path: "/:id/cancel", load: Order)` — the framework resolves
  `Order.find(id)` (through the request's session, E-2) and 404s on `None`.
  Inference from an enclosing `@Entity` was the alternative; explicit wins
  because a route file and an entity are different modules more often than
  not, and invisible coupling between two decorator families is how
  framework magic earns its reputation. One extra word per instance route.
- Pipeline (restated as the family's one load-bearing fixture): binder
  decode (400 before any user decorator) → Runtime (`@Traced`) → Install
  (`@Authorize` → `@Validate` → `@RateLimit` → `@Idempotent` →
  `@Transactional`) → woven contracts → body → serialize. Build this as an
  executable golden the day the first route derive exists.

## Build order within the track

1. `Schema` value + retention (needs nothing but the built Compile tier) —
   unblocks migration/OpenAPI tooling prototypes immediately.
2. Persistence derives against an in-memory adapter (no `Database`
   admission needed to prove the derive/schema/hook machinery; SQL adapter
   is a backend behind the same `Query`).
3. Web derives against a stub transport (request-as-Map), pipeline fixture
   first.
4. Real backends (DB driver, HTTP server) — each with its own ambient
   admission PDR, blocked on the reactor/net stack (PDR-0004/0015/0016)
   they obviously ride on.

## What this precludes

- A second schema source (migration DSL, OpenAPI-first annotations) — the
  class's decorators are the single source; tooling reads `T.schema`.
- Global identity maps or receiver-keyed process caches (E-2's session is
  the only identity scope).
- Parameter-position attribute grammar landing silently as part of this
  family — it is its own future PDR or nothing.
