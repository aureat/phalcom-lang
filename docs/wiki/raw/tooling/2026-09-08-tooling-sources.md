# Raw source snapshot: tooling

> Captured: 2026-09-08
> Source area: REPL/documentation program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/implementation/CLIT001-repl/PROGRAM.md ---
---
id: CLIT001
category: CLIT
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# CLIT001 — repl

This program owns REPL bootstrap, compiler/session execution, interactive
oracle and presentation, commands, and final integration.

--- docs/implementation/DOCS001-implementation-documentation-organization/PROGRAM.md ---
---
id: DOCS001
category: DOCS
kind: migration
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# DOCS001 — implementation documentation organization

## Purpose

Move implementation plans into a stable category/program/checkpoint/plan
hierarchy without changing their technical scope or falsely upgrading their
implementation status.

## Migration rules

- Preserve substantive plan and specification content.
- Assign stable identifiers rather than using Plan A/B or Part 1/2 names.
- Keep corrective and regression plans with the checkpoint whose acceptance
  objective they serve.
- Represent incomplete and deferred work in metadata.
- Remove migration-only copies and legacy naming once substantive content has a
  canonical owner.
- Preserve unrelated dirty and untracked work.

## Checkpoints

| Checkpoint | Objective | Status |
|---|---|---|
| [DOCS001.C1](C1-organization-contract-and-pilot/CHECKPOINT.md) | Define the canonical organization and migrate the first pilot | IN_PROGRESS |
| [DOCS001.C2](C2-inventory-and-disposition/CHECKPOINT.md) | Inventory and disposition all existing plan-like documents | IN_PROGRESS |
| [DOCS001.C3](C3-implementation-family-migration/CHECKPOINT.md) | Migrate coherent families from former implementation roots | IN_PROGRESS |
| [DOCS001.C4](C4-work-lifecycle-migration/CHECKPOINT.md) | Migrate work modules, pending plans, refactors, and completed records | IN_PROGRESS |
| DOCS001.C5 | Validate links, identifiers, metadata, and legacy-path retirement | PROPOSED |

--- docs/implementation/DOCS003-phaldoc/PROGRAM.md ---
---
id: DOCS003
category: DOCS
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# DOCS003 — phaldoc

This program owns the phaldoc analysis and documentation-generation work.
