# Collection traversal

> Sources: collections-next traversal/range specifications; COLL004
> Raw: [collection specification and program snapshot](../raw/collections/2026-09-08-collections-sources.md)
> Updated: 2026-09-08

Traversal separates eager collection operations from lazy iterator pipelines. Eager operations produce a concrete collection or scalar result; lazy operations retain a cursor/pipeline until a consumer requests values.

## Boundedness

An eager exhaust operation must not silently consume an unbounded source. The semantic/compiler boundary may issue a boundedness diagnostic when the operation requires termination proof. The runtime owns cursor progression and cleanup after a pipeline is consumed or abandoned.

## Mutation boundary

List mutation helpers and map/set updates are owned by their collection contracts. Traversal combinators must not assume immutability merely because a source is iterable.
