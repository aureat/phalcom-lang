# PDR-0028 — `@class` placement and `@constructor` method canon

- Status: Accepted
- Date: 2026-07-21
- Supersedes: ADR-0063's target-polymorphic `@constructor` surface
- Amends: ADR-0054's `@construct` target scope and PDR-0021's constructor-name collision rationale
- Related: [current Classes spec](../spec/current/classes.md), [current decorator specs](../spec/current/decorators/README.md), [pending ctor unit](../work/pending/ctor/plan.md)

> **Implemented and compatibility policy amended (2026-08-08).** PDR-0032 keeps
> the placement/constructor split but makes retired `static` a targeted error,
> not a non-fatal lowering alias.

## Context

The current documentation has two competing constructor models. ADR-0063 and the
experimental design tree make `@constructor` target-polymorphic and schedule
`@construct` for deletion. The current implementation-facing decorator spec still
describes `@construct` as a class derive, while the language guides teach the old
`construct` and `static` keywords.

The language needs one stable surface before implementation migration begins:
placement must be explicit, constructor marking must be method-local, and old
spellings must teach their replacements without turning migration into a hard
failure.

## Decision

1. **`@class` is target-polymorphic for class-side placement.** It is legal on
   fields, methods, getters, and setters. It places the declaration on the class
   object/metaclass side.
2. **`@constructor` is method-only.** It marks a method as a constructor. A
   constructor allocates, runs its body against the fresh instance, and returns
   that instance.
3. **`@construct` is class-only.** It derives a constructor method from declared
   fields, in declaration order, using the field-derive rules in the current
   decorator spec.
4. The `static` and `construct` declaration keywords are retired. New source uses
   `@class` and `@constructor`.
5. `construct` and `constructor` are reserved names. They cannot be introduced as
   user-defined declaration names, selector families, or attribute classes.
6. Legacy forms are rejected with targeted migration diagnostics:

   ```text
   @constructor
   new(...) { ... }  → did you mean @constructor?
   static foo(...) { ... }      → did you mean @class?
   static _field = ...          → did you mean @class?
   class foo(...) { ... }       → did you mean @class?
   ```

   Diagnostics point at the legacy spelling and name the canonical decorator.
   No compatibility lowering preserves the declaration's old meaning.

7. `docs/spec/current` is authoritative. The former forge unit is pending
   implementation material, and experimental constructor notes are retained under
   `docs/work/pending/ctor/notes/`.

## Consequences

- `@construct` and `@constructor` remain distinct names with distinct legal targets.
- `@class` owns both class-side fields and class-side behavior; no `@classField`
  or `static` spelling is needed.
- Parser owns concise errors for legacy spellings; compiler receives only the
  canonical decorated declaration forms.
- Source, fixtures, implementation names, tooling metadata, and documentation
  converge on the canonical surface under PDR-0032.

## Alternatives rejected

- **Target-polymorphic `@constructor`:** rejected because one decorator would mean
  “derive from fields” on a class and “mark this method” on a member.
- **Silent compatibility lowering:** rejected by PDR-0032 because it retains a
  second language surface and lets stale source escape migration audits.
- **Keeping `static` as an alias:** rejected; it preserves a second name for the
  same placement axis and keeps field/method terminology split.
- **Generic parse errors:** rejected; retired spellings receive focused replacement
  guidance.
