---
kind: integration-record
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
date: 2026-10-16
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
- `e9670ed0`, `b4be8ada`: label cycle-internal generated requirements by
  `semantic-generated` versus `metadata-type-owner` producer, bound the
  rendered diagnostic to 128 distinct edges, and preserve SCC membership
  throughout iterator projection. No change to the executable edge set.

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

### Historical blocker snapshot and verification (superseded by merge `57c7cb66`)

The canonical generated-requirement cycle is **not resolved**. The earlier
failed bootstrap remains the latest completed execution evidence; the new
diagnostic has not been exercised in a cargo run. The next owning-worktree
action is:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
```

The next bounded Rust gates are:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib pattern_temp_depth_tests -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib finalized_bootstrap_order_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib canonical_cycle_diagnostic_ -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker -- --nocapture
```

After those targeted gates, run semantic phase evidence, canonical bootstrap,
and relevant collections integration tests. First establish the exact rejected edge provenance; do not
remove graph-only edges merely because their `read_target` is `None`.
Do not relax runtime cycles, add late compiler reads, or claim bootstrap
acceptance without executable tests. Remote GitHub updates were committed;
this execution environment has neither a Phalcom checkout nor Rust Cargo.

### Historical diagnostic-only cycle producer differential (superseded by typed linker provenance)

- `d8791b4f`: when (and only when) canonical executable finalization rejects a
  runtime cycle, rerun immutable finalization over **authored imports only**,
  **authored + semantic-generated requirements**, and
  **authored + metadata-type-owner requirements**. Report acyclic/cyclic/other
  outcomes without publishing candidate executables or altering the original
  failure. This identifies whether a requirement family creates a cycle by
  itself or whether families interact.
- `d45108fe`, `70572fbd`: add a synthetic case where authored imports and
  each generated requirement family are independently acyclic, but the
  combination forms a strict runtime cycle. The test specifically protects
  against incorrectly exonerating one family based on an isolated probe.
- **Execution status: UNVERIFIED.** These Rust changes and the focused test were
  committed through GitHub but cannot be Cargo-tested in the current session.
  The diagnostic can prove which requirement families participate in a cycle;
  it does **not** prove that any particular edge can be dropped.

Proposed test filter after local checkout/build:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib canonical_cycle_differential_ -- --nocapture
```

**Architectural hypothesis requiring evidence:** the semantic producer
aggregates reads from declared callable bodies regardless of whether those
bodies execute during module initialization. The executable finalizer currently
adds every generated read/owner requirement as an *eager initialization edge*.
Consequently, deferred callable reads and metadata identity requirements could
overconstrain the initialization DAG. This is not yet proved as the sole cause
of the reported canonical SCC. Implementer must distinguish (1) allocation of
an immutable read target, (2) actual availability of the runtime value, and
(3) dependency-first execution of a source initializer, without treating
preallocated-but-uninitialized slots as available values. Do not simply erase
owner-only requirements or relax the DAG.

### Integrated bootstrap closure — merged and focused-verified (2026-10-14)

This record preserves the historical failed-bootstrap investigation above.
The separate preservation branch `feat/consolidation-bootstrap-repair` at
`3d61f2b4` implements the approved MODL001.C2.P4 native-floor availability
contract and reports the exact canonical bootstrap discriminator passing.
It also fixes the immediate `None` binding overwrite and class identity
comparison, adds deterministic cycle-provenance reporting, and retains immutable
finalized read slots while satisfying eager-edge obligations by exact proofs.

The isolated integration candidate merges the preservation history with the
latest diagnostic/cleanup consolidation baseline. Where the two histories
changed `canonical_universe.rs` independently, the candidate retains the
repair's availability-aware construction and restores the bootstrap execution
order check with provenance-compatible regression fixtures. The old ad hoc
cycle-detail and diagnostic-only differential helpers are superseded by the
typed cycle path in `phalcom-modules`; their combined-family regression now
lives in the modules linker suite. The independent list-pattern scratch
layout correction is preserved.

**Integration outcome: MERGED / FOCUSED_VERIFIED.** The canonical bootstrap
blocker is closed on `feat/consolidation-universe` by merged PR #21,
commit `57c7cb66`. The separate local repair `3d61f2b4` was merged with
the latest consolidation baseline and verified on the integration head
`b1ee59f4` by the scoped GitHub Actions run
https://github.com/aureat/phalcom-lang/actions/runs/38613486708.

The first integration run detected an additional runtime correctness defect:
the authored-import source index scheduled `collections.interval` before
`errors.argument`, while an unsatisfied effective finalized runtime edge
required `errors.argument` before `collections.interval`. The accepted
correction retains the source index as owner of the initializer *closure* but
stably topologically orders that closure against the finalized runtime DAG.
Authored order remains the deterministic tie-breaker for independent modules;
the source bootstrap order invariant remains enforced. There is no DAG exemption
or late compiler read insertion.

The availability contract now publishes `SourceInitializedValue` proofs only
for modules whose source initializers actually execute, not all linked/catalogued
modules. A negative/positive regression verifies that constraint. Native-floor
availability continues to require exact installed binding and product identity.

**Focused results on `b1ee59f4`:** core `cargo check` passed; modules graph
5/5; modules linker 25/25; semantic phase evidence 12/12; bootstrap schedule
regression 3/3; pattern scratch-layout regression 3/3; post-bootstrap source
proof regression 1/1; fresh native-floor catalog inventory 1/1; canonical
VM bootstrap plus finalized ordinary-user linked read 1/1; class-membership
and `None` identity regression 1/1. Both focused workflow jobs completed
successfully. The commit was subsequently merged as `57c7cb66`.

**Workspace-wide CI remains non-green outside this consolidation change:**
`phalcom-ast` parser/snapshot integration failures, Clippy warnings in
untouched lexer/type-metadata code, broad Rustfmt drift, and VS Code
extension-host responsiveness. Do not misrepresent these unrelated failures
as a green workspace/release certification, or absorb unrelated repairs into
the accepted bootstrap scope.

**Remaining:** MODL001.C2.P4 is still IN_PROGRESS / PARTIAL / FOCUSED_TESTED;
other T4 requirement producer families and T7/T8 cross-stage acceptance remain
open. PERF and concurrent enum feature work remain excluded.

## Functional consolidation acceptance — 2026-10-16

The bounded functional acceptance in the advisory decision is met on the
current `feat/consolidation-universe` tree. This record is `COMPLETE /
IMPLEMENTED / FOCUSED_TESTED` for canonical bootstrap plus representative
ordinary-user compile, finalized linking, and execution. It does not close
`MODL001.C2.P4` or certify the workspace.

The ordinary-user discriminator now explicitly imports the documented
`universe.collections.list.List` API, constructs `List<Int>`, appends 19 and
23, reads both values, and exports their sum (42). It also retains the existing
canonical bootstrap, `None`/Option identity, metadata, and exact
`SourceInitializedValue` linked-read assertions. The active production source
provider reads `phalcom-core/core/universe/src/collections/list.ph`; that file
was missing `export List`, despite the specification and native Universe
catalog exposing the collection. Adding the export makes this ordinary import
and execution succeed.

The positive protocol/default-to-adapter fixture now compiles and executes,
returning 23. Its paired runtime-cycle fixture still rejects the genuine cycle.
Two missing-mapping regressions still prove finalized compiler/prelude paths
fail closed without appending a read slot. The previously recorded
`NonExportableRecordKeyDomain` lowering failure did not reproduce in the
current positive execution discriminator; its causal relationship to the
List export is not established.

Focused current-tree evidence:

```text
phalcom-core modules_linking::mat_01_export_slots_populated: 1 passed
phalcom-core modules_linking::protocol_default_to_adapter_cycle_is_only_semantic_on_adapter_side: 1 passed; execution result 23
phalcom-core modules_linking::protocol_default_adapter_runtime_cycle_is_rejected: 1 passed
phalcom-core modules_linking (full suite): 20 passed
phalcom-core --lib fails_closed_without_appending: 2 passed
phalcom-core --lib retained_fixture_mode_transitions: 1 passed
phalcom-modules --test linker: 25 passed
phalcom-modules --test graph: 5 passed
phalcom-semantic --test semantic integration::phase_evidence: 20 passed
```

The exact core filters were run with `-- --exact --nocapture`; all results
above are from the current source tree. The full semantic suite, workspace
checks, and release certification were not run. The historical report of 94
semantic failures has not been reclassified by this focused work.

`MODL001.C2.P4` remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Remaining
producer-family coverage and cross-stage CVs are still owned by that
checkpoint. This functional milestone does not authorize PERF integration or
a production Universe cutover.

## Recursive checker stabilization — 2026-10-17

Starting from `324ee315`, the unchanged Horner regression reproduced a process
abort on its 2 MiB analysis thread. The expression carrier exceeded its 760-byte
budget. `BlockReason::UnavailableSemanticOwner` embedded a full `QueryKey`,
enlarging every `AnalysisStatus`; solver-local terms also occupied every
expression carrier even when absent.

The correction keeps the exceptional owner key and optional solver result in
owned boxes. Exact owner identity, equality, fingerprints, query outcomes, and
single ownership remain unchanged. Publication, contextual realization,
inference constraints, and explanations run in their original order in a
post-descent helper. No evidence or diagnostics are removed. On this arm64 debug
build, the recursive publication frame fell from the recorded 13,568 bytes to
2,688 bytes. The stack allowance and carrier assertion are unchanged.

The semantic library tests also contained a missed call-site update from the
mode slice. The budget-exhaustion test now explicitly supplies Debug to the
updated private API; its assertions are unchanged.

Executed with `CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016`
and empty `RUSTFLAGS`/`RUSTC_WRAPPER`:

```text
cargo test -p phalcom-semantic --test semantic capabilities::algebraic_realization -- --nocapture
  9 passed; 0 failed; 2240 filtered out; 8.20s
```

This proves the bounded Horner analysis and carrier regressions are repaired,
alongside the existing positive and negative realization cases. It does not
establish arbitrary-depth stack safety or close P4. The 94-failure full-suite
baseline has not been rerun after this correction; its carrier entry is now
resolved by focused evidence.

The new package probe also reproduced an 8 MiB lowering-thread overflow after
semantic analysis succeeded. The native crash report showed repeated
`compile_expr_want_inner` / `compile_bilateral_binary` frames. Binary expressions
now dispatch before the large non-binary lowering match. The existing binary
lowering body was moved without changing contextual-realization precedence,
operand order, lazy Boolean blocks, operator selection, or scratch-slot lifetime.

Additional completed semantic checks:

```text
cargo test -p phalcom-semantic --test semantic incremental::query_ownership -- --nocapture
  7 passed; 0 failed; 2242 filtered out; 2.95s
cargo test -p phalcom-semantic --lib session::publication_tests -- --nocapture
  13 passed; 0 failed; 179 filtered out; 9.19s
```

Production-path regression:

```text
cargo test -p phalcom-core --test core ordinary_float_horner_package_executes_all_coefficients -- --nocapture
  1 passed; 0 failed; 994 filtered out; 128.88s
```

The test uses the production package compiler and full Universe VM and asserts
exactly `2080.0`, the sum of coefficients 1 through 64 at `x = 1`. The original
8 MiB package-test stack is unchanged.

Broader gate: `cargo test -p phalcom-core --test language-corpus -- arithmetic booleans --nocapture`
selected two corpus groups and failed both (89.31s) before fixture execution:
`finalized executable link has no runtime read for class System from universe:concurrency.fiber`.
The retained fixture compiler still uses `ExecutableLinkedProgram::from_conservative_link`
instead of projecting the semantic snapshot's generated requirements. This is
an unresolved linkage blocker; the corpus run supplies no operator-runtime
acceptance evidence. The exact Horner package uses the production finalizer and
passes. Do not relax finalized-read checks to unblock the corpus.
