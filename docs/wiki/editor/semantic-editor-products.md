# Semantic editor products

> Sources: LSPX001 and semantic consumer specification
> Raw: [editor program snapshot](../raw/editor/2026-09-08-editor-sources.md)
> Updated: 2026-09-08

Editor products are projections of canonical semantic facts: hover text, declaration/navigation targets, completion items, callable signatures, and diagnostics. Presentation may omit detail or mark uncertainty, but it cannot strengthen unknown or advisory facts into established types.

## Boundary

[Semantic products](../semantic/products-and-reflection.md) owns the source facts and identity. [Diagnostics](../diagnostics/report-model.md) owns report structure. The editor owns protocol shape and user interaction.
