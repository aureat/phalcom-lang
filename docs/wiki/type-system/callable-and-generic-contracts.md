# Callable and generic contracts

> Sources: TYPE003, TYPE004, SEMA001, callable specifications
> Raw: [type metadata source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-08

Callable contracts connect parameter labels, rest behavior, generic parameters, constraints, return slots, and declaration identity. They are shared by language calls, native declarations, semantic publication, and editor signature products.

## Contract layers

Symbolic notation parses the contract. Semantic inference solves constraints and publishes a callable result. Metadata serializes stable parameter ownership and relation forms. Runtime dispatch uses selector identity and does not reconstruct generic proof from display text.

A generic parameter has an owner and index, a name, kind, variance, and optional source span. Constraints are subtype or equivalence relations over published nodes. Unknown, dynamic, and unavailable slots remain explicit rather than being silently widened.

## Boundary

[Declaration pipeline](../native/declaration-pipeline.md) validates native callable text. [Semantic products](../semantic/products-and-reflection.md) publishes signatures for consumers. Proposed TYPE programs remain implementation work.
