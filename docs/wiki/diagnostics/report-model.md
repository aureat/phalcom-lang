# Report model

> Sources: diagnostic report and section model
> Raw: [diagnostic specification and program snapshot](../raw/diagnostics/2026-09-08-diagnostics-sources.md)
> Raw: [diagnostic implementation source snapshot](../raw/diagnostics/2026-09-07-phalcom-diagnostics.md)
> Updated: 2026-09-08

A report is a protocol-neutral product assembled from a title, optional code, source snippets, typed sections, notes, and help. Severity maps to a semantic rendering role; the report layer does not run analysis.

## Stable assembly

Formatting orders the headline, snippets, non-empty rich sections, and notes/help. Explanation, guidance, context, and type-trace sections retain different visual roles without duplicating ANSI policy. [Semantic products](../semantic/products-and-reflection.md) owns the source reason; this domain owns its presentation model.

The earlier package-level rendering snapshot remains available as [diagnostic implementation provenance](../raw/diagnostics/2026-09-07-phalcom-diagnostics.md).
