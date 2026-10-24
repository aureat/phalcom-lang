---
id: ARTF001.C1.P2.R4
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
date: 2026-10-24
---

# Continue after query-owner migration

Use `CHECKPOINT.md` as shared state. Work in `.worktrees/artifact` on
`feat/artifact-universe-consolidation`; preserve the separate dirty main tree.
The migration is based on the assessment's `84e58af20`, including its remote
receipt, immutable-parent, provenance and Universe fixes.

The stale query integration target is retired. Run
`artifact::query::query_owner_catalog_regressions` and
`artifact::compatibility_tests` as library unit tests. Do not re-expose the
compatibility importer in production or replace the established session and
receipt fixes.

Current-head checks passed after reconciliation: 35 library tests, four
image-session/zero-copy tests and 13 round trips, plus both semantic compile
checks. Push and observe the focused workflow. Next build a real SlotMap executable
package over a current full Universe artifact; remove copied package source;
load and run in a fresh process under `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`.
Compare output with source execution. Prove fresh-process/path determinism and
hostile rejection with valid retry. Execute existing SHA-1 vectors on both VM
routes. Broader gates and a final acceptance receipt remain separate work.
