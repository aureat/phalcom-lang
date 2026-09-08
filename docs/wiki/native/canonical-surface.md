# Canonical native surface

> Sources: native surface records and selector identity
> Raw: [native surface source snapshot](../raw/native-surface/2026-09-07-phalcom-native-surface.md)
> Updated: 2026-09-08

The canonical surface is the generated, deterministic catalog of native members available to runtime and tooling consumers. A record carries owner, selector, dispatch side, visibility, type/callable contract, effects, return flow, intrinsic, and implementation metadata.

## Lookup and invariants

Lookup is keyed by stable universe/class identity plus selector identity. Deterministic fallback handles exact selector forms without weakening strict selector validation. Structural fingerprints and intrinsic checks detect catalog drift, duplicate keys, invalid owner/selector combinations, and inconsistent return shapes.

[Selector identity](../runtime/selector-identity.md) is the shared dispatch key. [Semantic type metadata](../type-system/semantic-type-metadata.md) is a separate transport graph; native records are inputs/projections, not a replacement for it.
