---
kind: integration-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
date: 2026-10-14
---

# Integration Universe consolidation

## Scope and fixed inputs

Consolidate `main`, `feat/telos-collections`, and `feat/integration-universe`
on the integration branch. PERF is under active implementation and is excluded:
no merge, checkout changes, plan amendments, or production Universe cutover.

- Integration start: `52838aa575d3eca7df8397a8b51c8bc15e5cfcaa`.
- Main input: `a09de379d97375566872109513f78780aaec368c`, already an ancestor of integration.
- Collections input: `60e6a3db8d754ed5db601298e7121a9f35af0463`.

The supplied consolidation analysis predates the integration LANG008 changes.
Preserve those changes and transactional semantic publication. Merge collections'
modules-owned generated requirements and finalized executable products.

## Current integration evidence

- Core, semantic, and LSP crate checks passed after conflict resolution.
- Module linker integration tests: 18 passed, zero failures.
- Semantic phase-evidence tests: 12 passed, including the optional-initializer regression.
- Functional bootstrap and runtime acceptance: pending.

The compiler's staged helper preserves fallible semantic updates, followed by
`finalize_executable_with_requirements`. Optional checked static initializers
require conservative incomplete phase coverage when checked authority is absent.

## Concurrent work and remaining seam

User-directed isolation: snapshot merge commit `41181e0b` was saved on
integration. Stabilization now proceeds on `feat/consolidation-universe` in
`/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom`.
Concurrent enum-inference edits and the untracked audit remain in the original
integration checkout and are excluded from this stabilization branch.

Collections' normative enums specification retires `@variant`. Integration's
interval additions were mechanically migrated to direct case declarations in
both source trees; the interval twins compare byte-for-byte equal. A missed
fallible-update test call was also corrected. Canonical Universe bootstrap
acceptance still depends on completed compiler/runtime evidence.

MODL001.C2.P4 remains partial. Its existing checkpoint owns producer coverage,
cross-stage CV gaps, and executable S4 acceptance. This merge does not close
those gates or certify the workspace. The legacy Universe remains production
source authority; PERF consolidation and production cutover remain downstream.

## Bootstrap correction: canonical re-export runtime owners

Observed: the first compiler/runtime linking run passed 12 tests and failed six
materialization tests at canonical Universe bootstrap. The common error was
`UncoveredRuntimeRead` from the Universe root to `errors.argument`.

Cause: conservative linking canonicalizes a selected/re-exported read to the
original owner, but its runtime graph only retained the authored intermediary
edge. The newly integrated executable validator requires the actual read owner
to have a direct runtime edge. Phase-aware finalization already projects that
canonical owner.

Correction: the modules-owned conservative linker now projects canonical export
owners while resolving imports, before strict runtime DAG validation and
initialization scheduling. No compiler mutation, validation bypass, or semantic
ownership transfer was introduced. The original authored edges remain. A
three-module re-export regression proves the executable wrapper accepts the
finalized reads and schedules the owner before its consumer. All 18 linker tests
passed after this correction. Compiler/runtime retest is pending.

The inherited `functional/chain.ph` twins differ only in their import owner:
legacy `collections.iterator_laws` versus shadow `collections.iteration`. This
is the intentional shadow migration, not behavioral drift to erase by copying.
