---
id: MODL001.C2.P4-HANDOFF
program: MODL001
checkpoint: MODL001.C2
plan: MODL001.C2.P4
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: /Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom
branch: feat/consolidation-universe
---

# MODL001.C2.P4 continuation handoff

## Mission and authority

Continue the amended phase-aware source-dependency implementation in this
worktree. The adopted authority is
[the architectural ruling](MODL001.C2.P4-architectural-ruling-and-implementer-directive.md)
and the user's INC-1 decision recorded in
[the incident record](MODL001.C2.P4-runtime-requirement-closure-incident.md).
Do not create P5; continue T4–T8 under P4.

The fixed pipeline is:

```text
source/interfaces
  -> symbolic semantic links with canonical identities
  -> semantic phase evidence and generated requirements
  -> one executable finalizer for canonical reads, runtime graph, and DAG check
  -> lowering/materialization from ExecutableLinkedProgram only
```

Keep strict runtime-cycle rejection, existing import syntax, and crate ownership
(`phalcom-modules` must not depend on `phalcom-semantic`). Preserve all
concurrent Universe, nanoid, and modernization edits. The user authorized
committing and pushing the scoped consolidation implementation.

## Current implementation state

- **T0–T3:** Complete and focused-tested. Symbolic module identities/products,
  semantic resolution, and retained source-import phase evidence are in place.
  Resolver integration passed 4/4.
- **T4:** Partial. Snapshot products include per-import phases and
  `RuntimeDependencyRequirement`s. Producers cover superclass, class-object /
  type-form, contextual realization, structural pattern, runtime attributes,
  nested-loop Error/Zip, and source-derived callable/witness/type/static roots.
  A retained semantic query now covers supported class and class-member
  attribute arguments, including receiver/generic contract contexts and
  per-callable invariant contexts. Selected callable and witness requirements
  are derived from those products. Break/continue trap ownership is modeled
  against materialized loop-control paths. Core propagates Debug/Release/
  Unchecked so stripped attribute uses remain observed but interface-only;
  finalizer assertions verify mode-specific read slots. Unsupported attribute
  declaration forms and the complete ruling family matrix remain open. The
  current semantic `phase_evidence` integration suite passed 18/18 after the
  finalizer assertions were added.
- **T5:** Partial, focused-tested. Modules owns requirement transport and the
  finalizer projects canonical read slots and generated runtime edges together
  before strict DAG validation. The linker integration suite passed 24/24.
  Positive source fixtures pass analysis/finalization. A new CV-04 fixture
  carries a whole-module import and a selected re-export through finalization
  and `ProgramCompiler::compile_analyzed`, asserting distinct canonical read
  targets and runtime edges; broader execution acceptance remains open.
- **T6:** Partial, focused-tested. All three `ProgramAnalyzer` paths preserve
  `ExecutableLinkedProgram`; finalized compiler/prelude paths fail closed
  rather than append reads. `RUSTFLAGS='' cargo check -p phalcom-core` passed.
  The contextual-read test and two missing-mapping tests each passed 1/1.
- **T7:** Partial. CV-06 and CV-07 pass at the semantic boundary (the focused
  semantic suite passed 18/18 on the current tree). CV-03 has linker-boundary
  evidence for conservative fallback and strict generated-cycle rejection.
  CV-04 now has program-analysis/finalizer/compiler evidence for canonical
  whole-module and re-export targets; its broader execution matrix remains
  open. CV-05 passes analysis/finalization in the type-cycle fixture, but
  executable acceptance remains open. CV-01/02/08 remain partial across
  layers; CV-09 has positive finalized-graph and negative-cycle evidence, with
  execution on the intended Universe iteration source still open.
- **T8:** Partial. The mutual type-cycle finalization fixture passes. The
  positive protocol/default-to-adapter fixture now compiles and executes with
  result 23, and its paired runtime-cycle fixture still rejects the genuine
  cycle. A prior `ProgramCompiler::compile_analyzed` attempt returned
  `NonExportableRecordKeyDomain`; that error did not reproduce in the current
  execution fixture and its cause remains unestablished. T8 remains partial
  because the broader cross-stage CV coverage is incomplete.

The core fixtures need a test-only Universe source override because default
`SemanticWorkspaceSession::new()` hits the private `numbers.capabilities` child
in the committed source graph. The override adds that child to the test
session's `numbers` exposure. This override remains test-only. Separately, the
canonical `Object.is` class identity correction changed both mirrored Universe
Object sources as recorded in the canonical bootstrap outcome below.

## Current focused evidence

Commands run in this continuation:

```text
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::phase_evidence
  12 passed
RUSTFLAGS='' cargo test -p phalcom-modules --test linker
  24 passed
RUSTFLAGS='' cargo test -p phalcom-modules --test graph
  5 passed
RUSTFLAGS='' cargo check -p phalcom-core
  passed
RUSTFLAGS='' cargo test -p phalcom-core --lib contextual_realization_receiver_projects_canonical_runtime_read
  1 passed
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_linking::semantic_type_cycle_finalizes_without_runtime_import_slots -- --nocapture
  1 passed
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_linking::protocol_default_to_adapter_cycle_is_only_semantic_on_adapter_side -- --nocapture
  1 passed
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_linking::protocol_default_adapter_runtime_cycle_is_rejected -- --nocapture
  1 passed
```

Earlier focused checks remain relevant: resolver integration passed 4/4; the
compiler/prelude missing-mapping tests each passed 1/1. The failed compile
attempt is recorded above and is not a pass. No workspace/release suite was run.

## Canonical bootstrap closure — 2026-10-14

The approved native-floor availability amendment is implemented for the
canonical bootstrap blocker. Requirements retain producer, source range,
consumer, exact target, product, and timing provenance. Finalization keeps read
slots independently of graph edges, removes only exact proof-satisfied edges,
and continues to reject genuine cycles with sorted SCCs and a deterministic
provenance-bearing edge path.

The native-floor contract is derived from the native installation catalog and
semantic identity catalog, then checked against a fresh \`VM::new_native()\`,
including canonical class identities and absent reserved source-only values.
Exact native bindings receive runtime-object proofs; source-backed declarations
may receive canonical-identity proofs without claiming an initialized value.
The fully initialized Universe contract supplies source-initialized binding
proofs to ordinary user compilation. A runtime trace
found that linked export/read materialization replaced the immediate \`None\`
binding with its hidden class handle; eager and lazy materialization now retain
the immediate value, while the class identity stays in its registry.
\`Object.is\` and \`is!\` compare classes by identity. No source initialization
cycle remained after exact availability matching, and no initializer body or
declaration order was changed to work around it.

The discriminator also compiles and runs an ordinary user project importing
the exported `universe.errors.unsupported.unsupported` binding, then verifies
its finalized linked read resolves to the populated source binding. Its
requirement is retained as `SourceInitializedValue`, and the full-bootstrap
proof removes the eager edge while preserving that read slot. The first probe
used `Option` from a core source module that does not export it and was
correctly rejected; the corrected probe uses an existing exported binding.

Focused verification completed:

\`\`\`text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib native_floor_contract_matches_fresh_vm_values_and_excludes_source_only_slots -- --nocapture
  1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
  1 passed in 188.46s; includes None preservation, Option reopen identity, metadata installation, and a user selective import with a finalized linked read
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model_invariants::isa_is_reflexive_and_superclass_closed -- --exact --nocapture
  1 passed; includes None.is(Option) and None.is(Object)
RUSTFLAGS='' cargo test -p phalcom-modules --test linker finalization_rejects_cycles_created_by_generated_runtime_requirements -- --exact
  1 passed; genuine generated cycle remains rejected with deterministic provenance
\`\`\`

P4 remains \`IN_PROGRESS / PARTIAL / FOCUSED_TESTED\`. Other T4 producer
families, CV cross-stage coverage, and executable T8 acceptance remain open.
Workspace/release certification was deferred.

## Bounded functional consolidation acceptance — 2026-10-16

The advisory-defined consolidation scope is now functionally accepted on the
current branch. The active production source provider reads
`phalcom-core/core/universe/src/collections/list.ph`, whose `List` declaration
was not exported even though `universe.collections.list.List` is documented
and catalogued. Adding `export List` enabled the existing ordinary-user
discriminator to import `List<Int>`, append 19 and 23, read both, and execute
to result 42. That test continues to assert canonical bootstrap/native-floor
identity and the exact finalized source-initialized linked read for
`universe.errors.unsupported.unsupported`.

The positive protocol/default-to-adapter fixture now compiles and executes
with result 23; the paired runtime dependency cycle still fails finalization.
The two missing compiler/prelude mapping tests still fail closed without
late-read insertion. The previously documented `NonExportableRecordKeyDomain`
failure is not reproduced by this current execution fixture. Its cause is
unresolved, so no metadata architecture change is inferred.

Current-tree focused results:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
  1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::protocol_default_to_adapter_cycle_is_only_semantic_on_adapter_side -- --exact --nocapture
  1 passed; VM result 23
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::protocol_default_adapter_runtime_cycle_is_rejected -- --exact --nocapture
  1 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib fails_closed_without_appending -- --nocapture
  2 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test linker
  25 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test graph
  5 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::phase_evidence
  12 passed
```

The core user-runtime and protocol tests ran on the same current source tree as
the added export. No full semantic suite, workspace gate, or release
certification was run. The historical 94 semantic failures are not classified
by this work.

P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; outstanding T4 producer
families and T7/T8 cross-stage CVs remain separate work. Do not reopen the
historical metadata-export issue unless it reproduces or a required P4 gate
directly depends on it.

## Fresh stability baseline — 2026-10-16, `ce8c6777`

The full semantic integration test binary, excluding only the separately-run
stack regression, completed with 2,123 passed, 94 failed, 23 ignored, and 1
filtered out. The named 94-failure inventory in
`phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` reproduced. The 23
ignored tests were not run. The separately selected
`ordinary_float_horner_expression_is_stack_safe_without_realization` test
overflowed its explicit 2 MiB analysis thread and aborted its test process
with SIGABRT. `typed_expression_remains_a_lightweight_recursive_carrier`
failed at 896 bytes versus its 760-byte budget. These two are confirmed
findings; the remaining 93 failures are reproduced but unresolved as to
implementation defect, oracle defect, or unsupported behavior. No
environmental cause was established.

The seven focused functional baseline gates all passed. This preserves M1
acceptance while keeping P4 partial. Do not use the 94-test count alone to
assign failures to P4 or to change expected behavior.

## P4.T4 attribute evidence and mode projection — 2026-10-16

The user selected mode-specific requirement projection. `SemanticWorkspaceInput`
now carries the compile mode, and the core entry-selection path passes
Debug/Release/Unchecked to semantic phase projection. A fully analyzed import
used only by a stripped `@ensures` or `@invariant` is retained as observed
`InterfaceOnly`; an emitted `@requires` remains `Runtime` in Release, and an
ordinary emitted body use still joins to `Runtime`. This preserves the P4 rule
that genuinely unobserved or unsupported selective imports remain Runtime.

The mode regression checks semantic diagnostics remain mode-independent,
generated requirements match emitted guards, and executable finalization
allocates the expected immutable linked-read slots for each mode. It passed
along with the complete directly affected phase-evidence module:

```text
CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::phase_evidence -- --nocapture
  18 passed; 0 failed
```

This closes the supported class/member attribute-argument slice and refines
the earlier mode-filtering implementation. Other attribute declaration forms,
unsupported/incomplete emitted expressions, the rest of the T4 family matrix,
and CV-01/02/04/08 remain open. The full semantic suite, workspace checks, and
release certification were not run.

The CV-04 focused source-project test also passed:

```text
CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::whole_module_and_reexport_reads_keep_canonical_runtime_targets -- --exact --nocapture
  1 passed in 149.06s
```

It verifies finalized runtime edges and immutable read mappings for both the
facade module object and the original exported declaration, and confirms
compiler lowering retains that read layout. CV-04 still needs the wider
execution matrix; CV-01/02/08 and CV-09 remain open at their specified stages.

## Next work

1. Reconcile the current failing-test inventory against the ledger, beginning
   with failures intersecting generated requirements or incremental phase
   products. Do not change semantic expectations without checking the owning
   spec.
2. Extend/re-audit remaining T4 attribute declaration forms and generated
   requirement families, keeping unsupported emitted expressions incomplete
   and conservative.
3. Complete the missing CV-01/02/04/08 cross-stage evidence, including that
   every emitted `GetLinked` read resolves through the immutable finalized
   mapping.
4. Reconcile the synthetic protocol/default-to-adapter fixture with the
   intended Universe iteration source shape for CV-09.
5. Keep P4 `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` until those acceptance gaps
   are resolved. Stage and deliver only the scoped work unit.
