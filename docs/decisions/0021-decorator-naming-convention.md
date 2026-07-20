# PDR-0021 — Decorator naming: lowercase names are compiler builtins, Capitalized names are `Attribute` classes

- Status: Proposed
- Date: 2026-07-20
- Related: [`docs/spec/decorator/README.md`](../spec/decorator/README.md)
  (naming section this ratifies; collision registry COLL-1…5),
  [`mechanism.md`](../spec/decorator/mechanism.md) (the two resolution paths),
  [PDR-0018](0018-decorator-carry-forward-and-v03-runtime-mandate.md) §5
  (left this open), [PDR-0022](0022-attribute-suffix-resolution.md) (the
  companion lookup rule this convention makes necessary),
  [ADR-0057](../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)
  (decorator/proxy pairs whose spelling this touches).

## Context

The tree already follows a rule nobody ratified: all twelve registered
builtin decorators are lowercase (`@requires` … `@ignore`), and the one
`Attribute` subclass in `core.ph` is Capitalized (`@On`). The library drafts
predate the rule and spell future Install/Runtime decorators lowercase
(`@memoize`, `@traced`); the web draft collides its lowercase `@get`/`@delete`
HTTP verbs with the builtin `@get` accessor derive (COLL-2) and puts `@on`
one character from `@On` (COLL-4). A name in this system is not cosmetic: it
selects a **resolution path** — registry row (compiler-owned, Compile/Layout)
vs `resolves_to_attribute_class` chain-walk (user/stdlib, Install/Dispatch/
Runtime and passive) — and the two paths have different powers, different
tiers, and different failure modes.

## Decision

1. **Lowercase `@name` = compiler-owned builtin**: a row in
   `AttributeRegistry`. Only the compiler may add one; they occupy the
   Compile/Layout tiers plus the subtractive pair. Unknown lowercase names
   fail as `attr.unknown` against the registry's row list.
2. **Capitalized `@Name` = an `Attribute` subclass** (stdlib or user), for
   Install/Dispatch/Runtime tiers and passive metadata, resolved by the
   class chain-walk (per [PDR-0022](0022-attribute-suffix-resolution.md)'s
   suffix-first rule once ratified).
3. **`@On` is the sanctioned exception** — both a registry row and a
   `core.ph` class; it is the bridge between the two worlds and stays
   Capitalized because it *is* a class.
4. **Layout-tier builtins stay lowercase** even though they ship "library"
   behavior: `@lazy`, `@synchronized`, `@observable`, `@computed` are
   compiler-owned by ADR-0052's confinement rule (per-receiver state ⇒
   builtin), and their spelling must say so.
5. **The framework renames in
   [`frameworks.md`](../spec/decorator/frameworks.md) are ratified with this
   record**: `@Get @Post @Put @Patch @Delete @Route @Body @Param @Query
   @Header @Cron @Job`, `@Entity @Column @Id @Index @Unique @Default
   @BelongsTo @HasMany`, lifecycle `@BeforeSave`-family — and `@on` →
   **`@Subscribe`/`@Handles`** (renamed, not merely capitalized: `@on`/`@On`
   differing only by case failed the same-trap test that killed
   `@construct`/`@constructor`). Draft spellings are historical on
   ratification.
6. **Registry precedence is explicit**: if a lowercase name ever gains a
   same-named class in scope, the registry row wins; a user class named
   `data` can never capture `@data`. A test pins this the day any stdlib
   `Attribute` class ships whose name could shadow a builtin.

## Consequences

- Case carries meaning a reader can act on: lowercase = "the compiler is
  doing something here" (derive/weave/drop/layout), Capitalized = "an object
  is instantiated, retained, maybe hooked." That is the C#/.NET attribute
  intuition mapped onto Phalcom's two-path mechanism.
- COLL-2 and COLL-4 dissolve (different paths / renamed); COLL-3 is
  *created* by this convention and resolved by PDR-0022 — the two records
  ratify together or the behavioral family's spelling stays undecidable.
- Drafts and on.md worked examples using lowercase Install decorators
  (`@memoize`, `@retry`) are read as historical; the behavioral family lands
  as `@Memoize`, `@Retry`, `@SynchronizedClassWide`.

## What this precludes

- A lowercase user decorator, ever — lowercase is a closed namespace owned
  by the compiler.
- Case-only disambiguation between two *class-path* names (`@on`/`@On`
  shape): where both sides resolve as classes, distinct words are required.
