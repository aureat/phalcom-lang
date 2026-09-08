# ADT and GADT families

> Sources: `docs/spec/adts.md`, TYPE001, SEMA008–SEMA009
> Raw: [type metadata source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-08

ADT/GADT families give constructor identity a type-level role. A family owns its constructor set and indexed relationships; matching can refine the scrutinee only when the constructor relation is established by the semantic authority.

## Invariants

Constructor declarations need stable family/constructor identity, legal parent and result relationships, and a distinction between ordinary nominal inheritance and indexed family refinement. Coverage analysis must preserve constructor witnesses and recursive structure. Constructor completion must not invent members from a package or runtime catalog.

## Status boundary

The normative ADT material and semantic analyzer rules define the intended contracts. TYPE001, SEMA008, and SEMA009 are separate implementation programs with proposed or partial status. [Patterns and matching](../language/patterns-and-matching.md) covers surface form; [coverage](../semantic/pattern-coverage-and-constructors.md) covers proof products.
