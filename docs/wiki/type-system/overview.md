# Type system

> Sources: `phalcom-type-syntax`, `phalcom-type-meta`, TYPE001–TYPE004, SEMA001
> Raw: [type metadata source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-08

The type-system domain defines symbolic type notation, callable and generic contracts, ADT/GADT families, record rows, and transportable semantic metadata. It neighbors [language](../language/overview.md) for syntax, [semantic](../semantic/overview.md) for authoritative inference, [native](../native/overview.md) for host contracts, and [runtime](../runtime/overview.md) for representation.

## Reading order

- [Symbolic type notation](symbolic-type-notation.md)
- [Callable and generic contracts](callable-and-generic-contracts.md)
- [ADT/GADT families](adt-gadt-families.md)
- [Records and rows](records-and-rows.md)
- [Semantic type metadata](semantic-type-metadata.md)

The syntax model is VM-free and parser-owned. Metadata is a serialized semantic artifact with schema/version validation; neither page is a substitute for the semantic analyzer's formal authority.
