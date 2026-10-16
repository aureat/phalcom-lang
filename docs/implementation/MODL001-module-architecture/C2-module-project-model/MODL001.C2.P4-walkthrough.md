---
id: MODL001.C2.P4-WALKTHROUGH
program: MODL001
checkpoint: MODL001.C2
plan: MODL001.C2.P4
kind: implementation-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: /Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom
branch: feat/consolidation-universe
starting_revision: d203e71a
---

# MODL001.C2.P4 walkthrough — canonical bootstrap availability closure

## Scope

This continuation closes the canonical Universe bootstrap generated-requirement
cycle blocker under the approved P4 amendment. P4 remains
**IN_PROGRESS / PARTIAL / FOCUSED_TESTED**; other T4 producer-coverage and
T7/T8 cross-stage gaps remain open. No workspace or release certification was
requested or run.

## Implementation

- Added typed provenance for generated runtime requirements, including producer,
  source range, consumer, exact module/binding target, required product, and use
  timing. The finalized executable product retains ordered requirements and
  read mappings.
- Added immutable availability proofs with installation authority and stage.
  Finalization allocates linked-read slots independently of graph edges and
  removes an edge only for an exact target/product proof. Mismatched proofs,
  reserved source bindings, and binding proofs for whole-module imports retain
  conservative edges.
- Classified native catalog bindings as floor runtime objects, source-backed
  bindings and whole-module values as source-initialized values, and
  trait/type-alias references plus callable-owner metadata as canonical
  identity. Native-floor binding identity proofs remain distinct from
  initialized-value proofs.
- Added deterministic SCC sorting and a cycle path whose edges retain
  requirement provenance. Strict runtime-cycle validation remains unchanged.
- Built the native-floor contract from UNIVERSE_BINDINGS and the module
  materialization boundary. Built the fully initialized Universe contract for
  ordinary user compilation. The linker contracts contain no VM handles.
- Added exact reads for unresolved canonical native prelude bindings such as
  None, so lowering has finalized slots for compiler-emitted linked reads.
- Fixed the runtime issue exposed after cycle closure: linked export and read
  materialization could replace the immediate None binding with the hidden
  None class handle. Eager and lazy paths now preserve the singleton value;
  class identity remains in the canonical class registry. Object.is and is!
  use identity comparisons for class-chain checks.
- Added focused integration assertions for the None singleton, canonical
  Option reopen identity and behavior, installed metadata, and a finalized
  user linked read.

No genuine source-initialization cycle remained after exact availability
matching. No Universe initializer body or declaration order was changed, and no
DAG exemption was added.

## Verification

    RUSTFLAGS='' cargo test -p phalcom-modules --test linker
      24 passed
    RUSTFLAGS='' cargo test -p phalcom-modules --test graph
      5 passed
    RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::phase_evidence
      12 passed
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib native_floor_contract_matches_fresh_vm_values_and_excludes_source_only_slots -- --nocapture
      1 passed in 97.96 seconds
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
      1 passed; 989 filtered out
    RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants::isa_is_reflexive_and_superclass_closed -- --exact --nocapture
      1 passed; 989 filtered out
    RUSTFLAGS='' cargo test -p phalcom-modules --test linker finalization_rejects_cycles_created_by_generated_runtime_requirements -- --exact
      1 passed; 22 filtered out

The first post-finalization bootstrap attempts exposed the None overwrite and
Object.is recursion. Those attempts failed with call-depth overflow before
the materialization correction. The corrected exact bootstrap discriminator
passed in 188.46 seconds, including an ordinary user project that selectively
imports an exported Universe binding and reads its finalized linked slot. An
earlier run before adding that user-import assertion passed in 143.36 seconds.
A stricter intermediate classification correctly exposed a residual SCC;
exact identity products resolved it without source initializer changes.
The direct class-membership regression passed in 95.60 seconds. One initial
attempt to select the regression used the wrong qualified test name and
selected zero tests; the corrected fully qualified filter above selected and
passed the test.

## Mode consistency slice — 2026-10-16

Carried `CompileMode` through `AnalyzedProgram` and `CompiledProgram`, exposed
`CompiledProgram::compile_mode()`, and added `VM::validate_program_compile_mode`
checks before module allocation, closure compilation, and initialization.
Threaded active mode through interpreter and module context entry points.
Added `SemanticWorkspaceSession::update_module_workspace_with_execution_mode_and_cancel`
to propagate execution mode choices into retained-session compilation.

Verification:
- `modules_linking::finalized_compile_modes_reject_mismatched_vm_before_mutation_and_execute_matching_guards`: 1 passed (246.72s)
- `modules::compile_session::tests::retained_fixture_mode_transitions_preserve_the_selected_execution_policy`: 1 passed (3.18s)
- `modules_linking` full suite: 20 passed (322.15s)
- `integration::phase_evidence`: 20 passed (23.22s)

## Remaining work

- Complete other T4 generated-requirement producer families.
- Close remaining CV cross-stage coverage and executable T8 acceptance,
  including the existing NonExportableRecordKeyDomain metadata-export gap.
- Keep P4 partial until those acceptance criteria are met.
