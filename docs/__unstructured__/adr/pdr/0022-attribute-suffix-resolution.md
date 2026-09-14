# PDR-0022 — Attribute-class resolution is suffix-first: `@Name` resolves `NameAttribute`, then `Name`

- Status: Proposed
- Date: 2026-07-20
- Related: [PDR-0021](0021-decorator-naming-convention.md) (the convention
  that creates the collision this resolves; ratify together),
  [`mechanism.md`](../design/decorators/canonical/mechanism.md) §Naming resolution
  (options table this record decides),
  [ADR-0057](../adr/accepted/0057-decorator-granularity-vs-proxy-granularity-split.md)
  (kept intact by this rule), `resolves_to_attribute_class`
  (`compiler/attributes.rs` — the lookup this extends).

## Context — COLL-3

ADR-0057 deliberately keeps decorator/proxy pairs under their natural names:
`@retry`/`Retry`, `@traced`/`Trace`, `@lazy`/`Lazy` — sigil vs class as the
disambiguator. PDR-0021 makes user-authorable Install decorators Capitalized
`Attribute` *classes* — so the decorator behind `@Retry` must be a class,
and a class named `Retry` already exists (the proxy). Two classes cannot
share one name in one module. Three options were analyzed in mechanism.md:
lowercase attribute classes (breaks the class-naming convention), rename one
side (re-opens ADR-0057), or suffix resolution (.NET's rule).

## Decision

1. **`@Name` resolution order**: try `NameAttribute` first, then `Name`.
   Both lookups use the existing `Attribute`-subclass chain-walk; the first
   name in scope that resolves to an `Attribute` subclass wins. If neither
   does, `attr.unknown` (registry rows having been consulted before either —
   [PDR-0021](0021-decorator-naming-convention.md) ruling 6).
2. **Suffix-first is load-bearing, not stylistic.** With the reverse order,
   a class named `Retry` that is *not* an `Attribute` subclass would shadow
   nothing (the chain-walk filters), but a future `Attribute`-subclassed
   `Retry` in scope would silently capture `@Retry` away from
   `RetryAttribute`. Suffix-first means the ceremonial name — the one that
   can only be an attribute — always wins, and adding a non-attribute class
   named like a decorator can never change decoration behavior.
3. **The suffixed class is the real name; the sigil strips it.** stdlib
   decorators colliding with proxies are classes `RetryAttribute`,
   `TracedAttribute`, `LazyAttribute`(builtin-`@lazy` excepted — it is a
   registry row, PDR-0021 ruling 4); used as `@Retry`, `@Traced`. A
   non-colliding decorator (e.g. `Memoize`) may skip the suffix; the suffix
   is available, not mandatory.
4. **`@NameAttribute` written out is legal** and resolves the same class
   (the sigil does not *forbid* the suffix; it merely makes it optional) —
   matching C#, where `[FooAttribute]` and `[Foo]` are interchangeable, so
   there is no spelling that works in a class-position but not in
   decorator-position.
5. **Diagnostics name both probes**: an `attr.unknown` for `@Retry` reports
   "no `RetryAttribute` or `Retry` attribute class in scope," so the rule
   teaches itself at the failure site.

## Consequences

- ADR-0057's surface survives verbatim: `@Retry(times: 3)` on a method,
  `Retry.on(target:)` for the proxy — the user-visible pair is untouched;
  only the decorator's *defining* class carries the suffix.
- Cost: one extra scope probe per decorator use, compile-time only, and only
  when the unsuffixed probe would have been tried anyway.
- Precedent-with-consequence: .NET has run exactly this rule since 2002; the
  known cost is the one this record accepts explicitly — a class genuinely
  named `FooAttribute` that is not an attribute becomes confusing to use as
  one word. The chain-walk filter (must extend `Attribute`) removes even
  that: a non-attribute `FooAttribute` is simply invisible to the sigil.

## What this precludes

- A third resolution namespace (decorator-specific registries for user
  code, import-qualified attribute references) — resolution stays "the
  registry, then classes in scope," with this one ordering rule inside the
  class half.
- Un-suffixing a colliding decorator later (moving `RetryAttribute` to
  `Retry`) without retiring the proxy name first — the pair is stable by
  construction.
