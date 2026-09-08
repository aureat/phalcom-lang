# Workspace snapshots and incrementality

> Sources: LSPX004; SEMA003; semantic consumer chapters
> Raw: [editor program snapshot](../raw/editor/2026-09-08-editor-sources.md)
> Updated: 2026-09-08

The editor must pin one immutable workspace/semantic snapshot per request. Updates are latest-wins at the scheduling layer, while semantic publication remains transactional and coherent.

## Correctness

An incremental request is exact only when its source/revision identity maps to the pinned snapshot. Stale or unmapped requests use declared fallback behavior. [Workspace incrementality](../semantic/workspace-incrementality.md) owns invalidation and recomputation; LSP owns cancellation and protocol timing.
