---
id: LSPX004
category: LSPX
kind: integration-and-performance
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# LSPX004 — incremental LSP and workspace modules analyzer

This program owns the incremental integration boundary between the language
server and the canonical workspace-module analyzer. It covers retained module
and semantic roots, bounded analysis frontiers, latest-wins cancellation,
compiler-effect-driven diagnostics, coherent editor products, and release
evidence for incremental LSP behavior.

`phalcom-modules` and `phalcom-semantic` remain the authorities for module and
formal semantic meaning. `phalcom-lsp` owns scheduling, document coherence, and
protocol adaptation only. Existing editor architecture, module architecture,
and source-location programs remain separate ownership areas.

## Current implementation checkpoint

IA-1 is fixed at the exact import-product publication seam. The module
session retains the body-only import root, graph changes carry an explicit
delta, and semantic publication consumes that canonical root. The LSP now
uses compiler diagnostic effects, cooperative epoch cancellation, Local-mode
deep-frontier selection, shared reference line-index conversion, and
semantic-token result IDs/deltas.

Crate-level validation is currently green for modules, semantic, and LSP.
This does not close the program: C1/C3 still require structural persistence
for the remaining workspace-sized roots, and C0/C4/C5/C8 evidence remains
open until deterministic counters, cancellation barriers, performance
fixtures, and release-wide gates are complete.
