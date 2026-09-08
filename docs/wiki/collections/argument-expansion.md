# Argument expansion

> Sources: argument-pack and expansion specification; COLL005
> Raw: [collection specification and program snapshot](../raw/collections/2026-09-08-collections-sources.md)
> Updated: 2026-09-08

Argument expansion turns positional and labeled expansion syntax into a canonical argument pack. The pack preserves selector slot order, labels, rest capture, and source provenance until callable checking consumes it.

## Boundary

The parser recognizes `*expr` and labeled arguments. Semantic callable checking validates arity, labels, generic expectations, and rest compatibility. The compiler lowers only the validated pack; runtime dispatch uses the resulting selector identity and values.

A trailing block is syntactic sugar for the final argument and does not create a new selector. COLL005 is proposed and not started. See [expressions](../language/expressions-and-control-flow.md) and [callable contracts](../type-system/callable-and-generic-contracts.md).
