# Universe catalog

> Sources: native universe catalog
> Raw: [native metadata source snapshot](../raw/native-meta/2026-09-07-phalcom-native-meta.md)
> Updated: 2026-09-08

The universe catalog is the canonical vocabulary of runtime-owned classes, bindings, superclass relations, and type forms. `UniverseKey` gives declarations a stable symbolic owner without requiring a VM at metadata-parse time.

## Contract

The catalog records the native surface schema version, core object/value classes, callable and method-family classes, collection/module/project classes, errors, fibers/resources, and identity/manifest classes. Source-only helpers are not silently added to runtime-owned relations.

## Consumers

[Primitive contracts](primitive-contracts.md) use catalog keys in authored metadata. [Canonical surface](canonical-surface.md) lowers and indexes the resulting records. [Runtime representation](../runtime/representation-and-object-model.md) consumes the runtime projection. UNIV001 is proposed and unverified.
