# Product model

> Sources: product and collection specifications; COLL001
> Raw: [collection specification and program snapshot](../raw/collections/2026-09-08-collections-sources.md)
> Updated: 2026-09-08

Unit, Tuple, and Record are product values: their shape is determined by component count or field names, not by a mutable collection protocol. Product normalization gives equivalent construction paths a deterministic semantic shape.

## Ownership

The type system owns product type structure and row constraints. The collection runtime owns value construction and access. Semantic products preserve source identity and type evidence; they do not copy runtime layout rules into editor projections.

Records have sorted, unique fields in metadata and deterministic field access. [Records and rows](../type-system/records-and-rows.md) explains the type-level tail; [runtime representation](../runtime/representation-and-object-model.md) explains storage.

## Status

COLL001 is proposed. The specification is the contract; implementation and conformance evidence remain separate.
