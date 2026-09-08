# Raw source snapshot: editor

> Captured: 2026-09-08
> Source area: editor architecture and incremental-LSP program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/implementation/LSPX001-lsp-architecture/PROGRAM.md ---
---
id: LSPX001
category: LSPX
kind: completion-and-correction
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# LSPX001 — lsp architecture

This program contains editor-facing architecture, incrementality, snapshot
coherence, editor presentation, and callable semantic/LSP review records.
Module ownership and project/module topology live in `MODL001`; performance and
source-location work have separate LSPX programs.

--- docs/implementation/LSPX003-source-locations/PROGRAM.md ---
---
id: LSPX003
category: LSPX
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# LSPX003 — source locations

This program owns clickable source-location implementation and editor
integration.

--- docs/implementation/LSPX004-incremental-lsp-and-workspace-modules-analyzer/PROGRAM.md ---
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
