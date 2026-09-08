# Workspace incrementality

> Sources: semantic-analyzer chapters 09, 12, 13; SEMA003
> Raw: [semantic specification and program snapshot](../raw/semantic/2026-09-08-semantic-sources.md)
> Updated: 2026-09-08

Workspace incrementality retains source/module/semantic state across edits and publishes an immutable snapshot only after a candidate transaction is coherent. It connects [module sessions](../modules/sessions.md) to [editor snapshots](../editor/workspace-snapshots-and-incrementality.md).

## Lifecycle

A source mutation identifies affected roots, invalidates owned products, recomputes the necessary frontier, and atomically publishes the candidate. Failed analysis must not partially replace the last coherent snapshot. Consumers pin a snapshot and classify requests as exact, stale, or unmapped.

## Equivalence

Incremental results must preserve the observable semantic products and diagnostics of cold analysis for the same source state. Reuse is allowed only when dependency ownership and fingerprints prove that a product remains valid.

## Status

SEMA003 and related LSP programs are in progress or proposed. This is a contract and navigation page, not a claim that every incremental path is complete.
