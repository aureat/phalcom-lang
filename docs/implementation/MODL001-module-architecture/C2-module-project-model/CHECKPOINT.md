---
id: MODL001.C2
program: MODL001
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# MODL001.C2 — module project model

This checkpoint contains pending module syntax, project resolution, linking,
compiler/LSP integration, runtime initialization, and stabilization records.
Patch artifacts are retained as historical inputs, not treated as applied
source changes.

## Plans

| Plan | Scope | State |
|---|---|---|
| `MODL001.C2.P1` | Module syntax and project resolution | Proposed |
| `MODL001.C2.P2` | Module linking graphs and LSP integration | Proposed; follows P1 |
| `MODL001.C2.P3` | Module runtime initialization and execution | Proposed; follows P2 |
| `MODL001.C2.P4` | Phase-aware source dependencies for the S4 iteration-split prerequisite | IN_PROGRESS / PARTIAL / FOCUSED_TESTED; T0–T3 done, T4 partial, T5–T6 partial/focused-tested, T7 partial, T8 partial/focused-tested; see [continuation handoff](MODL001.C2.P4-handoff.md) |

## P4 architecture incident state — 2026-10-13

The incident is resolved by the user-supplied approved `MODL001.C2.P4-architectural-ruling-and-implementer-directive.md`, dated 2026-10-13, supplied from the `perf-universe-characterization` worktree. The ruling reviewed collections revision `b679028c8ff634e17828003572bd2a6e1a0d79ad`, which is the clean starting revision for implementation. P4 is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

## Adopted ruling and T0 amendment

- Preserve historical P1 → P2 → P3 → P4 plan ordering; live P2/P3 substrate is the integration basis, without declaring their historical plans completed.
- Modules owns item-level `SourceImportBindingId`, canonical symbolic targets, distinct semantic/executable link products, and executable finalization. It must not depend on semantic.
- Semantic consumers own retained per-binding use evidence, reasons, explicit completeness, and content fingerprints. Resolver lookup alone cannot prove interface-only use. Missing, stale, unsupported, unobserved, or incomplete evidence stays Runtime.
- Core closes generated declaration, callable/witness, runtime type/reification, pattern, attribute, and contextual requirements before finalization. Source import evidence and generated executable requirements are separate input lanes to one finalizer.
- Finalization jointly projects slots and runtime edges and strictly validates the DAG. Only the finalized product enters program lowering/materialization. Program lowering must fail on absent authority instead of adding linked reads.
- Whole-module imports, direct re-exports, external superclasses, static constants, and unused imports remain Runtime. No syntax, runtime SCC tolerance, identity duplication, or production Universe cutover is authorized.
- P2/P3 pipeline/input contracts and the P4 task sequence are amended accordingly. Required evidence includes symbolic resolution without executable slots, type-enforced stage isolation, mixed-use/runtime cycles, generated reads, and cold/incremental parity.

### INC-1 decision — 2026-10-13

The user selected **amend P4** under the same checkpoint acceptance objective.
This is a Level 2 plan amendment: generated requirements need a modules-owned
transport that carries the runtime edge plus an optional canonical
`LinkedReadSpec` target. Core projects semantic/runtime products before
finalization; the finalizer allocates the immutable read layout and runtime DAG
together. `ExecutableLinkedProgram` must remain the compiler/materializer input
type, and missing mappings must fail closed. Continue T4–T8 in P4; do not
create P5. See [INC-1](MODL001.C2.P4-runtime-requirement-closure-incident.md).

The Slice 1–3 documents remain supporting review inputs. The ruling is the adopted authority. The user decision is recorded in [INC-1](MODL001.C2.P4-runtime-requirement-closure-incident.md); continue the amended T4–T8 work under this checkpoint.

## Implementation state — 2026-10-13

P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. The user resolved INC-1
by amending P4. The modules-owned generated-requirement transport and the
finalized compiler boundary are implemented, while T4 producer coverage and
cross-layer CV acceptance remain incomplete.

Current task state and evidence:
- **T0–T3 — DONE / FOCUSED_TESTED.** The architecture amendment, symbolic link
  products, semantic resolver migration, and retained source-import phase
  evidence are in place. Focused resolver integration passed 4/4.
- **T4 — PARTIAL / FOCUSED_TESTED.** Snapshots publish `import_phase_products`
  and modules-owned `RuntimeDependencyRequirement`s. Producers cover
  superclass, class-object/type-form, contextual realization, structural
  pattern, nested-loop Error/Zip, and source-derived callable/witness/type/static
  roots. User-attribute constructors and straightforward argument roots are
  projected, while nonempty attribute arguments and unsupported expression
  forms deliberately leave coverage incomplete; exact generated callable /
  witness reads there are not closed. Break/continue trap ownership also
  remains incomplete. The focused semantic `phase_evidence` suite passed
  11/11; the full ruling family matrix is not closed.
- **T5 — PARTIAL / FOCUSED_TESTED.** The finalizer projects canonical read
  slots and generated runtime edges together, then validates the strict DAG.
  Focused linker tests passed 18/18. Positive type-cycle and S4 fixtures pass
  through finalization; CV-01/02/04 still lack complete cross-stage evidence.
- **T6 — PARTIAL / FOCUSED_TESTED.** All three `ProgramAnalyzer` paths preserve
  `ExecutableLinkedProgram`; finalized compiler/prelude paths fail closed
  instead of appending reads. The focused core crate check passed, as did the
  contextual runtime-read test (1/1) and two finalized-mapping fail-closed
  tests (1/1 each). End-to-end emitted-read coverage remains open.
- **T7 — PARTIAL / FOCUSED_TESTED.** CV-06 and CV-07 pass at the semantic
  boundary. CV-03 has linker-boundary evidence. CV-01/02/04/05/08 remain
  partial across layers; see the current handoff and INC-1 progress record.
- **T8 — PARTIAL / FOCUSED_TESTED.** The type-cycle fixture and S4 positive
  fixture pass analysis/finalization; the paired S4 runtime-cycle rejection
  passes. A separate attempt to lower/execute the positive fixture reaches
  legacy metadata export and fails with `NonExportableRecordKeyDomain`.
  Executable acceptance is therefore still open. The core fixtures use a
  test-only Universe source override to expose the already-imported
  `numbers.capabilities` child; no Universe source was changed for P4.

The test-only Universe override resolves the fixture's semantic-session setup
failure without changing production bootstrap. The metadata-export failure is
not yet established as a clean-checkout baseline and is not claimed as one.
Preserve concurrent Universe, nanoid, and modernization edits. Do not close P4
or mark T8 complete until the executable acceptance gap and remaining CVs are
resolved.

### Integration consolidation boundary (2026-10-14)

The collections snapshot `60e6a3db` was merged with integration `52838aa5` as
`41181e0b`; follow-up stabilization is isolated on `feat/consolidation-universe`.
The integrated pipeline retains transactional, fallible semantic publication and
modules-owned executable finalization. Optional checked static initializers
now conservatively mark phase coverage incomplete when absent. A focused
phase-evidence regression checks that executable imports remain Runtime.

Consolidation exposed `UncoveredRuntimeRead` at canonical Universe bootstrap:
conservative re-export resolution canonicalized read owners without adding the
corresponding direct runtime edge. The modules linker now projects canonical
owners before strict DAG validation. The three-module re-export regression and
all 18 linker tests passed. This retains the finalized-read invariant and does
not authorize late compiler read insertion. Runtime retest is recorded in the
[consolidation record](../../consolidation/integration-universe-consolidation.md).

P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; producer-family and
cross-stage acceptance gaps above remain open. PERF and the production source
cutover are excluded from this consolidation work.
