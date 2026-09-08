---
id: LSPX004
category: LSPX
kind: integration-and-performance
status: PROPOSED
completion: NOT_STARTED
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
