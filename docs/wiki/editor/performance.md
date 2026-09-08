# Editor performance

> Sources: LSPX002 and LSPX004
> Raw: [editor program snapshot](../raw/editor/2026-09-08-editor-sources.md)
> Updated: 2026-09-08

Editor performance owns latency, bounded analysis frontiers, cancellation, and reuse measurements at the LSP boundary. It does not redefine semantic incrementality or module ownership.

## Contract

A request should pin one coherent snapshot, avoid duplicate analysis, and stop obsolete work when a newer document revision wins. Measurements must distinguish scheduling delay, analysis cost, presentation cost, and transport overhead.

LSPX002 and LSPX004 are proposed and unverified. [Workspace incrementality](../semantic/workspace-incrementality.md) owns recomputation correctness; [performance measurement](../performance/measurement-and-instrumentation.md) owns reproducible evidence.
