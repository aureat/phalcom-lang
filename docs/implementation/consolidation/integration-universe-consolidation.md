---
kind: integration-record
status: BLOCKED
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
- Shadow Universe parser/interface census: 1 passed after correcting effects/fold/folds imports to `collections.iteration`.
- Functional bootstrap and runtime acceptance: FAILED; strict generated-requirement cycle requires architectural decision.

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
passed after this correction. Compiler/runtime retest cleared that graph-coverage failure but exposed missing generated reads.

The inherited `functional/chain.ph` twins differ only in their import owner:
legacy `collections.iterator_laws` versus shadow `collections.iteration`. This
is the intentional shadow migration, not behavioral drift to erase by copying.

## Canonical staged construction

The runtime retest cleared `UncoveredRuntimeRead`, then failed six materialization
tests because `callable.function` lacked a finalized `Error` read. Canonical
Universe construction still used `link_all` and never supplied symbolic linkage
to semantic analysis, so it did not receive the generated-requirement products
used by ordinary programs.

Canonical construction now follows the same existing staged contract:
`link_semantic` → fallible semantic analysis with symbolic linkage → generated
requirements plus the existing metadata-owner projection → executable
finalization. The source provider, project identity, and eager bootstrap policy
remain unchanged. Core and LSP checks passed after this correction. The exact
`modules_linking::mat_01_export_slots_populated` runtime discriminator failed strict DAG validation (0 passed, 1 failed, 140.05 seconds).


Final state: the three branch histories are consolidated, but runtime acceptance
is blocked. See [architectural incident and handoff](canonical-universe-runtime-cycle-incident.md).
Core/semantic/LSP checks and focused linker/phase-evidence checks do not certify
a working VM. Full workspace, collection/property suites, static metadata parity,
and source-free PERF gates were deliberately not run. The latest canonical
staging correction is a reviewable candidate, not accepted bootstrap closure.

## Consolidation stabilization and cleanup — 2026-10-14

Target branch: `feat/consolidation-universe`; the implementation remains
`BLOCKED / PARTIAL / UNVERIFIED`. The historical checks above were completed
on earlier revisions and do not verify the latest stabilization commits.

### Canonical bootstrap guard and diagnostics

- `5aeeffa8`, `e369d7dc`: require the canonical source-index initializer
  order to respect each finalized runtime dependency between modules actually
  executed by that bootstrap schedule. The check does not classify external
  native-floor satisfaction and does not alter the linker DAG.
- `8091f584`, `88729c3f`: preserve strict cycle rejection and augment the
  canonical linking error with generated requirements whose endpoints lie in
  the rejected strongly connected component, retaining module identity, source
  range, dependency reason, and linked read target where present.
- `b19df039`: isolate the bootstrap-order validator and add focused synthetic
  reversed-order, dependency-first, and outside-closure regression tests.
  **Tests were committed but not executed.**

### Pattern-frame cleanup

- `6de2796a`: correct executable List `*rest` temporary-depth accounting:
  the extracted tail and copy-loop counter occupy two live slots. Nested rest
  patterns reserve deeper slots as required. Remove the fallback that reserved
  fresh compiler locals during pattern execution, which could invalidate
  repeated `while let` or alternative-pattern scratch layouts. Add depth
  regression tests. **No Cargo build or runtime gate was executed.**
- The post-`60e6a3db` collections delta is not safe to copy wholesale:
  consolidation has subsequent pattern-frame preallocation, generalized Record
  metadata export, and updated semantic/Pack logic. Preserve these newer
  developments and reconcile isolated collection fixes only after source-level
  equivalence review. Do not import the concurrent enum inference change.

### Remaining blockers and verification

The canonical generated-requirement cycle is **not resolved**. The earlier
failed bootstrap remains the latest completed execution evidence; the new
diagnostic has not been exercised in a cargo run. The next owning-worktree
action is:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
```

Then run the two new focused unit-test filters via the core library target,
followed by module linker, semantic phase evidence, and relevant collections
integration tests. First establish the exact rejected edge provenance; do not
remove graph-only edges merely because their `read_target` is `None`.
Do not relax runtime cycles, add late compiler reads, or claim bootstrap
acceptance without executable tests. Remote GitHub updates were committed;
this execution environment has neither a Phalcom checkout nor Rust Cargo.
