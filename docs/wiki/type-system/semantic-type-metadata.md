# Semantic type metadata

> Sources: `phalcom-type-meta`
> Raw: [type metadata source snapshot](../raw/type-system/2026-09-07-phalcom-type-meta.md)
> Updated: 2026-09-08

Semantic type metadata is the immutable, store-independent transport model for published type facts. It contains artifact headers, stable identities, kind and type graphs, scoped type-lambda bodies, generic signatures, declarations, callable/field surfaces, module/runtime roots, occurrences, fingerprints, and validation errors.

## Graph and identity rules

Kinds and global/scoped type nodes are indexed graphs with topological reference order. Stable project/module/declaration identities separate builtins, packages, source artifacts, and sessions. Scoped types alpha-normalize binders and distinguish bound variables from free global references.

## Compatibility

Metadata headers advertise schema, semantic-model, native-surface, retention, feature, identity, and source/interface fingerprint information. Decoding enforces byte budgets, deserializes JSON, and validates version/feature compatibility plus graph references. Record-row constructs require the corresponding schema feature floor.

## Authority boundary

This is a published artifact format, not the inference engine. [Semantic authority and identity](../semantic/authority-and-identity.md) owns how facts become canonical; [canonical native surface](../native/canonical-surface.md) consumes compatible native records. Fingerprints support reuse but do not turn stale metadata into current truth.
