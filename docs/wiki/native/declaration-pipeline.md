# Declaration pipeline

> Sources: native declaration normalization and procedural expansion
> Raw: [native declaration source snapshot](../raw/native-decl/2026-09-07-phalcom-native-decl.md)
> Raw: [procedural expansion source snapshot](../raw/native-macros/2026-09-07-phalcom-native-macros.md)
> Updated: 2026-09-08

The declaration pipeline turns an authored native attribute plus attached documentation into a normalized, validated declaration. It is the seam between source ergonomics and the catalog generator.

## Stages

1. Parse the owner and selector plus name/value fields.
2. Normalize enum-like metadata, type strings, documentation, and optional fields.
3. Validate selector identity, visibility, internal naming, replacement/deprecation rules, duplicates, and unknown fields.
4. Hand the normalized declaration to macro expansion and surface generation.

The parser and validator return structured errors rather than panicking. [Primitive contracts](primitive-contracts.md) owns the meaning of metadata fields; [generated drift](generated-surface-drift.md) owns census/order/output checks.

The procedural expansion implementation is preserved as [the macro source snapshot](../raw/native-macros/2026-09-07-phalcom-native-macros.md); it is an implementation seam of this pipeline, not a separate conceptual domain.
