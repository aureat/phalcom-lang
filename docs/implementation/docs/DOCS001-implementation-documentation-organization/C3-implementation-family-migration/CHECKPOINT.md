---
id: DOCS001.C3
program: DOCS001
kind: migration
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# DOCS001.C3 — `docs/impl` family migration

## Acceptance objective

Move the coherent implementation families from `docs/impl/` under canonical
category/program directories while preserving their internal source structure
and technical content.

## Delivered batch

- ADT/GADT records moved to `TYPE001`.
- Typing-integration records moved to `TYPE002`.
- Semantic-completeness records moved to `SEMA001`.
- Semantic-correctness records moved to `SEMA002`.
- LSP architecture and callable LSP review records moved to `LSPX001`.
- Recursive pattern coverage moved to `SEMA004`.
- Semantic analyzer invariants moved to `SEMA003`.
- Print-family intrinsic moved to `UNIV001`.

## Remaining work

- Add plan-level metadata to individual migrated records.
- Rename legacy internal `part-*` and `sc-*` directories into named checkpoints.
- Audit and repair relative links affected by the path moves.
- Resolve duplicate universe records and other cross-program overlaps.
