# Patterns and matching

> Sources: ADT/GADT and pattern-related language specifications; SEMA008–SEMA009
> Raw: [language specification and program snapshot](../raw/language/2026-09-08-language-sources.md)
> Updated: 2026-09-08

Pattern syntax destructures values through literals, tuples/records, bindings, and constructors. The parser owns shape and binding syntax; [ADT/GADT families](../type-system/adt-gadt-families.md) owns constructor identity and indexed-family constraints.

## Semantic stages

Matching proceeds from parsed pattern shape to constructor legality, type refinement, usefulness/exhaustiveness, and witness diagnostics. These are separate products. A parser that accepts a constructor pattern does not prove that it is legal for the scrutinee.

Recursive coverage and constructor completion are proposed implementation programs. This page therefore describes the intended ownership and evidence flow, not a completion claim.

## Boundary

[Capability and flow](../semantic/capability-and-flow.md) records narrowing evidence from a successful pattern. [Report model](../diagnostics/report-model.md) owns user-facing witness formatting.
