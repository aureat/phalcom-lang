# LSP architecture

> Sources: LSPX001; semantic consumer rules
> Raw: [editor program snapshot](../raw/editor/2026-09-08-editor-sources.md)
> Updated: 2026-09-08

The language server owns protocol scheduling, document coherence, request cancellation, and adaptation. It does not own module resolution or formal type truth. Those remain in [modules](../modules/overview.md) and [semantic authority](../semantic/authority-and-identity.md).

## Request path

A document update enters workspace state, analysis publishes a coherent snapshot, an LSP request pins that snapshot, and a presentation adapter returns hover, completion, navigation, or diagnostics. Stale/unmapped requests follow explicit fallback policy rather than mixing revisions.
