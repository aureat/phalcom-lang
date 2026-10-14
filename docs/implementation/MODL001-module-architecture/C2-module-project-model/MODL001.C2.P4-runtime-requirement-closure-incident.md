# IMPLEMENTATION INCIDENT — INC-1 Runtime Requirement Closure

## 1. Identity

- Program/checkpoint/plan/task: `MODL001.C2.P4.T4–T6`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections`
- Branch: `feat/telos-collections`
- Starting revision: `83527b08`
- Current revision: `83527b08` plus uncommitted P4 changes

## 2. Escalation trigger at incident opening

The accepted ruling's §24 triggers 3 and 5 apply: a generated runtime
requirement cannot be identified and projected to an executable read before
bytecode compilation with the live transport product, and the finalized read
layout is still mutable through compiler fallback paths. This also triggers
the repository rules for changing a cross-layer compiler/runtime contract.

## 3. Required invariant

- **INV-03:** unknown or unsupported evidence remains Runtime.
- **INV-04:** executable programs have a strictly validated runtime DAG.
- **INV-05:** every runtime read and declaration-materialization requirement is
  covered by that DAG and the finalized read layout.
- **INV-06:** semantic links remain available after an import is omitted from
  executable reads.
- T6 requires program compilation/materialization to consume only
  `ExecutableLinkedProgram`; a missing mapping is an internal error, never a
  late read insertion.

## 4. Expected architecture

P4 combines two lanes before executable finalization:

```text
source import phase evidence
  + generated executable requirements with canonical runtime targets
  -> one finalizer that allocates reads, builds the runtime graph, and validates
```

The module layer owns the transport and final slots. Semantic analysis and the
core semantic projection identify generated owners without making
`phalcom-modules` depend on `phalcom-semantic`.

## 5. Observed behavior at incident opening

The live phase collector produces one generated edge family: external
superclasses. `RuntimeDependencyEdge` identifies only importer, dependency,
range, and reason. It carries no declaration/symbol/read target. The finalizer
adds these edges before DAG validation, but only authored symbolic imports are
projected into `LinkedReadSpec` slots.

The ruling also names class-object/type-form values, contextual realization,
runtime patterns/class checks, attributes/decorators, type/reification roots,
selected callable owners, witness owners, and cross-module static constants.
No producers for those families were found in the live P4 collector/core
projection. Two compiler paths still append reads after linking:

- `Compiler::emit_global_reference` in
  `phalcom-core/src/compiler/lib/mod.rs`;
- `Compiler::emit_declaration_class_reference` in
  `phalcom-core/src/compiler/lib/expr.rs`.

## 6. Minimal reproduction

This is a structural contract reproduction; no behavioral test was run for the
missing generated read mapping.

```text
RuntimeDependencyEdge { importer, dependency, range, reason }
  -> finalizer adds graph edge
  -> no source binding/read target is available to allocate
  -> compiler fallback appends linked_reads and emits GetLinked
```

Expected: finalization publishes both the edge and canonical read mapping;
compiler either uses that mapping or fails with an internal invariant.

## 7. Relevant code path

```text
SemanticWorkspaceSession snapshot assembly
  -> phase_evidence::compute_module_import_phase_products
  -> RuntimeDependencyEdge list
  -> ModuleLinker::finalize_executable_with_requirements
  -> ProgramCompiler lowering / P3 materialization
```

The semantic-to-core projection and canonical generated-read map are absent
between snapshot assembly and finalization. The live fallback locations are
`emit_global_reference` and `emit_declaration_class_reference` above.

## 8. Scoped implementation diff at incident opening

The current uncommitted P4 work adds the T5 module finalizer, wires all three
`ProgramAnalyzer` paths through semantic linking and executable finalization,
adds core type-cycle and S4 protocol/adapter fixtures, and records imported
trait conformance roots as interface-only evidence. The T6 stage type is not
preserved through `AnalyzedProgram`/`CompiledProgram`, and the two compiler
fallbacks remain. The subagent drafted direct semantic tests for CV-04/05/07
and conformance-root evidence; they have not yet been validated.

## 9. Evidence at incident opening

- `phalcom-semantic/src/phase_evidence.rs` contains one producer of
  `generated_requirements`, in the superclass branch.
- `phalcom-modules/src/graph.rs` defines `RuntimeDependencyEdge` with module
  endpoints and reason only.
- `phalcom-modules/src/linker.rs` projects `LinkedReadSpec` only while iterating
  authored `SemanticLinkedModule.imports`; generated requirements add graph
  edges but no read slots.
- `phalcom-core/src/compiler/lib/mod.rs` has a fallback that pushes a
  `RuntimeLinkedRead` and emits `GetLinked`.
- `phalcom-core/src/compiler/lib/expr.rs` has a second fallback that pushes a
  `RuntimeLinkedRead` for a canonical declaration and emits `GetLinked`.
- `RUSTFLAGS='' cargo test -p phalcom-modules --test linker` previously passed
  18/18 for the T5 implementation.
- `RUSTFLAGS='' cargo check -p phalcom-core` passed for the initial T6 pipeline
  integration, before the conformance-root collector change and current
  unvalidated semantic test additions.
- A session-based semantic test probe panicked during
  `SemanticWorkspaceSession::new()` because the dirty worktree's Universe
  source graph reports `numbers.capabilities` as private. The subagent replaced
  these probes with direct fixtures; their results are pending.

## 10. Implementer's causal hypothesis

T4's generated requirement lane was implemented as module-level graph edges,
but it needs a canonical executable target/read identity as well. Without that
information, T5 cannot allocate the read layout from the same authority as the
runtime graph, and T6 compiler fallbacks can still repair the layout late.

## 11. Alternative hypotheses

1. Some generated requirements are graph-only and need no executable read, but
   the ruling's class-object, contextual realization, pattern, callable, and
   witness cases still need owner-specific classification and proof.
2. Existing runtime import slots may cover some generated references, but no
   current finalizer contract verifies this for every emitted read.
3. The requirement families may need a bounded corrective plan after P4's
   current source-import phase work is recorded as partial.

## 12. Attempts already made

- T5 added generated runtime edges before DAG validation and preserves strict
  cycle rejection. Linker tests passed 18/18.
- T6 changed program analysis order to semantic-link, analyze, then finalize;
  the core package check passed before the last semantic source change.
- A local attempt to preserve `ExecutableLinkedProgram` through core public
  types was stopped and reverted after discovering that legacy constructors
  and generated runtime reads need an explicit canonical requirement product.
  No unchecked constructor or compatibility escape was retained.

## 13. Why local authority is insufficient

The fix changes the cross-layer representation and ownership of generated
runtime requirements and read identity. The current ruling fixes the owner
boundaries but does not specify the transport key for generated reads or the
canonical compiler lookup map. Choosing a shape locally could duplicate
identity or split graph/read authority, violating INV-01 and INV-05.

## 14. Decision requested

Should P4 be amended to add a core-owned generated runtime requirement/read
product (with canonical module/symbol targets) that the module finalizer
projects into immutable slots and DAG edges, or should this closure be split
into a corrective `MODL001.C2.P5` while P4 remains partial? The recommended
path is a Level 2 P4 amendment if the principal architect confirms this is the
same acceptance objective; otherwise create P5 before resuming T4–T8.

## 15. Verification

No new focused command has been run after the conformance-root change or the
S4 fixture addition. Workspace/release verification remains deferred as the
P4 plan requires.

## 16. User decision and resolution

On 2026-10-13 the user selected **amend P4** rather than create corrective
P5. Keep the same checkpoint acceptance objective and record a Level 2 plan
amendment. The amendment introduces a modules-owned transport requirement
that couples each generated runtime graph edge to an optional canonical
`LinkedReadSpec` target. Core projects semantic/runtime metadata into these
requirements before finalization; the linker allocates immutable canonical
read slots and graph edges together; program compilation/materialization
preserves `ExecutableLinkedProgram` and fails closed on missing mappings.
Legacy all-runtime compiler entry points must be isolated behind a validating
conservative wrapper. Complete all T4 requirement families and retain strict
runtime-cycle rejection before closing T5–T8.

## 17. Implementation progress after the amendment — 2026-10-13

The original INC-1 contract gap is implemented in the current P4 worktree:

- `phalcom-modules` owns `RuntimeDependencyRequirement`, which pairs a runtime
  edge with an optional canonical `LinkedReadSpec`; the finalizer validates
  target/dependency agreement, projects canonical read slots and edges, and
  rejects runtime cycles before publishing `ExecutableLinkedProgram`.
- Semantic snapshots publish these requirements. Semantic/core producers now
  cover superclass, class-object/type-form, contextual realization, structural
  pattern, runtime attributes, nested-loop Error/Zip, and source-derived
  callable/witness/type/static roots. Unsupported owners remain Runtime and
  mark coverage incomplete; T4 is still partial. In particular, nonempty
  runtime attribute arguments leave module coverage incomplete, and exact
  generated callable/witness reads for unsupported attribute-expression forms
  are not closed. Break/continue trap ownership also remains incomplete.
- Core orchestration filters snapshot-generated requirements to the program's
  runtime importer closure, finalizes before compilation, and preserves
  `ExecutableLinkedProgram` through analyzer/compiler products. Finalized
  compiler/prelude paths fail closed instead of appending linked reads.
- Focused validation passed: module linker 18/18; semantic phase-evidence
  integration 11/11; core package check; type-cycle and S4 positive
  analysis/finalization fixtures; and the paired S4 runtime-cycle rejection.
- A separate attempt to compile the positive fixture passed analysis and
  finalization but failed while exporting legacy runtime metadata with
  `NonExportableRecordKeyDomain`. This is an open executable acceptance gap,
  not a claimed pass or a verified clean-checkout baseline.
- Core fixtures use a test-only Universe source override to expose the
  already-imported `numbers.capabilities` child during semantic-session
  bootstrap. No production Universe source was changed for this P4 work.

INC-1's transport and stage-boundary decision is therefore implemented, but
P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. T4 family closure and
cross-layer CV-01/02/04/05/08 plus executable T8 acceptance remain open. See
the current [P4 handoff](MODL001.C2.P4-handoff.md) for exact focused commands
and the next actions.
