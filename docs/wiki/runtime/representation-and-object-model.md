# Representation and object model

> Sources: object-model specification; RUNT001–RUNT002
> Raw: [runtime specification and program snapshot](../raw/runtime/2026-09-08-runtime-sources.md)
> Updated: 2026-09-08

The object model separates classes, instances, blocks, methods, native objects, and immediate values while preserving message-send semantics. A representation record is an implementation seam, not a new language type.

## Ownership

Class identity and superclass relations connect to the [universe catalog](../native/universe-catalog.md). Product storage is owned by [collections](../collections/product-model.md). Selector identity is a shared dispatch contract and is documented in [selector identity](selector-identity.md).

Runtime representation programs describe implementation work; they do not override normative object-model rules.
