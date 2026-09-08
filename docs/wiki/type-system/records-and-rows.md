# Records and rows

> Sources: collection/type specifications and type metadata schema
> Raw: [type metadata source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-08

Record types describe named fields; row types describe an open tail that may be extended or constrained. The representation appears in both collection products and serialized semantic graphs, but ownership differs: collection pages explain values, while this page explains type structure.

## Metadata representation

Global open-record tails resolve to parameters of record-row kind. Scoped open-record tails resolve to an in-scope row binder or a free parameter with that kind. Fields must be sorted and unique so normalization, fingerprints, and equality are deterministic.

## Boundary

[Product model](../collections/product-model.md) owns runtime Unit/Tuple/Record values. [Type formation](../semantic/type-formation-and-inference.md) owns row constraints and satisfaction. Schema validation rejects feature/version combinations that cannot represent record rows.
