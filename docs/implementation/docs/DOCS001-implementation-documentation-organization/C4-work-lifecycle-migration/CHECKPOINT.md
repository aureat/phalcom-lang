---
id: DOCS001.C4
program: DOCS001
kind: migration
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# DOCS001.C4 — work-lifecycle migration

## Acceptance objective

Move the remaining `docs/work/modules/`, `docs/work/pending/`,
`docs/work/refactors/`, and `docs/work/completed/` records into owning
category/program/checkpoint directories without using source folders as
lifecycle state.

## Delivered batch

- Module architecture records moved to `MODL001`.
- Pending records grouped by technical ownership.
- The compiler module-split refactor moved to `COMP001.C1`.
- Completed records retained as `COMPLETE` as-built checkpoints under their
  owning programs.
- Companion artifacts, including the native-primitives PDF, retained.

## Remaining work

- Add plan-level identifiers to legacy records.
- Normalize old internal `part-*`, `u*`, and bucket names into checkpoint names
  where that improves navigation.
- Audit links and update the migration manifest with final counts.
