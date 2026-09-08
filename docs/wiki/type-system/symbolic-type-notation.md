# Symbolic type notation

> Sources: `phalcom-type-syntax`
> Raw: [type syntax source snapshot](../raw/type-system/2026-09-07-phalcom-type-syntax.md)
> Updated: 2026-09-08

Symbolic type notation is the VM-free grammar used when native and tooling contracts need a type expression without running the program. It models unknown, never, self, named and universe-qualified types, parameters, applications, unions, tuples, and callable parameter groups.

## Callable shape

A parameter tuple separates positional, labeled, and optional rest parameters. Once a label appears, later positional parameters are rejected. Callable syntax combines optional type parameters, parameters, a return type, and `where` constraints using subtype `<:` or equivalence `==`.

## Parser boundary

The parser owns complete-input consumption and reports unexpected end, character, token, or contextual syntax errors. It does not resolve names, prove constraints, or assign runtime classes. [Semantic type formation](../semantic/type-formation-and-inference.md) consumes language expressions; [primitive contracts](../native/primitive-contracts.md) embeds static specifications.

## Display and evidence

Display reconstructs the principal surface spelling, including generic arguments, unions, labels, rest markers, and constraints. Focused tests cover type forms and callable constraints; this page does not claim workspace-wide conformance.
