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


The staged-linking integration exposed generated-requirement cycles during
canonical Universe finalization. The approved 2026-10-14 availability amendment
closed that bootstrap blocker while retaining strict DAG validation. Evidence
and the runtime `None` binding correction are in the
[canonical bootstrap incident](../../consolidation/canonical-universe-runtime-cycle-incident.md).

### Canonical runtime-availability closure — active P4 amendment (2026-10-14)

The user approved the architectural direction in the canonical bootstrap audit.
P4 remains active; this amendment closes the canonical bootstrap blocker only.
Strict DAG rejection, immutable finalized reads, and fail-closed behavior remain
fixed. Requirement availability is classified by exact target, consumed
product, and use stage. Native-floor proofs are immutable and contain no VM
handles; exact catalog bindings prove installed runtime objects, while semantic
identity proofs do not claim source values are populated. Reserved source slots
have no floor value proof. Read-slot allocation remains independent from
initialization ordering. Other P4 T4/T7/T8 coverage gaps remain open.

- Starting revision: `d203e71a` on `feat/consolidation-universe`.
- Baselines: module linker 18/18; semantic phase-evidence integration 12/12.
- Active continuation: P4 T9 provenance/SCC evidence, T10 availability-aware
  finalization, and T11 native-floor verification are implemented and
  focused-tested. No residual source-initialization cycle was established; the
  exact native `None` binding is preserved through materialization.
- Focused evidence: modules linker 24/24; modules graph 5/5; semantic phase
  evidence 12/12; fresh native-floor inventory 1/1; canonical bootstrap
  discriminator 1/1; class-membership identity regression 1/1. See the
  [walkthrough](MODL001.C2.P4-walkthrough.md) and updated
  [handoff](MODL001.C2.P4-handoff.md) for exact commands and outcomes.
- Remaining scope: other P4 T4 producer families and T7/T8 cross-stage CVs are
  still open. No workspace/release certification was run.
- Certification boundary: exact canonical bootstrap and selected focused
  cross-layer regressions; no workspace/release certification.

### Consolidated canonical bootstrap acceptance — 2026-10-14

Integrated native-floor bootstrap availability closure and related
compiler/VM repairs were merged by PR #21 into `feat/consolidation-universe`
at `57c7cb66`. The integrated source head `b1ee59f4` passed focused CI
(https://github.com/aureat/phalcom-lang/actions/runs/38613486708):
`phalcom-core` check; graph 5/5; linker 25/25; semantic phase evidence
12/12; canonical bootstrap order 3/3; pattern scratch 3/3; negative
post-bootstrap source-value proof 1/1; native-floor inventory 1/1; executable
canonical Universe bootstrap plus user finalized linked read 1/1; and
`None` class-membership 1/1.

Integration additionally repaired source initializer scheduling: the source
index selects the executable source closure, whereas the finalized effective
runtime DAG supplies execution dependencies. Stable topological sorting keeps
the former authored order wherever dependencies permit; it rejects real
cycles instead of suppressing generated edges. Post-bootstrap
`SourceInitializedValue` proofs are now limited to executed source modules,
not all linked modules. The immediate `None` binding remains preserved.

**Checkpoint remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED.** Bootstrap blocker
closure is not T4-family completion or T7/T8 CV/release certification.
Workspace CI still fails in untouched AST parser snapshots, lint/format
baseline, and VS Code E2E; do not claim workspace/release green.

### Bounded consolidation functional acceptance — 2026-10-16

The advisory-defined functional scope is implemented and focused-tested on
`feat/consolidation-universe`: canonical bootstrap and native-floor identity
regressions remain covered; an ordinary user program imports canonical
`List<Int>`, appends and reads values, and returns 42 through finalized
linkage; the positive protocol/default-to-adapter fixture compiles and runs
with result 23; the paired runtime cycle remains rejected; and two missing
mapping tests fail closed without appending reads. The ordinary-user fixture
was enabled by adding the missing `export List` to the active production
Universe source at `phalcom-core/core/universe/src/collections/list.ph`.

Current focused results: core `mat_01_export_slots_populated` 1/1; positive
protocol/default-to-adapter execution 1/1; paired runtime-cycle rejection
1/1; fail-closed missing mappings 2/2; modules linker 25/25; modules graph
5/5; semantic phase evidence 12/12. See the current consolidation record and
P4 handoff for exact commands. The historical `NonExportableRecordKeyDomain`
failure did not reproduce in the current positive execution check; its cause
is unknown.

This bounded result does not close P4: T4 producer-family completeness and
remaining T7/T8 cross-stage CVs stay open. The full semantic suite, workspace
checks, and release certification were not run. Functional consolidation
acceptance must not be reported as full P4 or workspace completion.

### Fresh semantic stability baseline — 2026-10-16

On `ce8c6777`, the full semantic integration binary excluding only the
separately-run stack regression completed with 2,123 passed, 94 failed, 23
ignored, and 1 filtered out (946.52 seconds). The same 94 failures in
`phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` reproduced. The excluded
`ordinary_float_horner_expression_is_stack_safe_without_realization` test was
run by its fully qualified name and aborted when analysis overflowed its
explicit 2 MiB thread stack. The `TypedExpression` size assertion also
reproduced at 896 bytes versus a 760-byte maximum. These two findings are
confirmed; the other 93 failed tests remain unclassified pending bounded
oracle/implementation triage. The 23 ignored tests were not run.

All seven focused functional-acceptance gates passed on this baseline. P4
remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; the baseline does not change
the bounded consolidation acceptance or complete T4/T7/T8 work.

### P4.T4 attribute-analysis consultation — 2026-10-16

The implementation review confirmed that syntax-only attribute-root collection
cannot prove selected callable, specialization, trait-dispatch, or witness
requirements. The approved route is an additive semantic attribute-argument
query/product with a stable key composed from canonical owner identity,
attribute name and occurrence, argument ordinal, and analysis context. Source
ranges remain metadata; snapshot-local `SourceSiteId` is not a query key.
Module-initialization attributes are analyzed with module imports/values and no
receiver or method generics. `@requires`/`@ensures` are analyzed as separate
receiver-shaped closures with the owning callable's receiver/generic context
and without callable value parameters. The product feeds the existing phase
evidence collector and records normal semantic dependencies for reuse and
invalidation. This is within P4.T4; no plan amendment is required unless live
checker evidence disproves the context model.

The user selected **mode-specific runtime requirement projection**. The
implementation now retains semantic argument products for supported class and
class-member attribute sites, derives selected callable and witness
requirements from those products, and classifies stripped contract/invariant
uses as observed `InterfaceOnly` in the selected compile mode. Emitted uses
still join to `Runtime`; unsupported or incomplete emitted sites retain the
conservative incomplete/runtime fallback. Core passes Debug/Release/Unchecked
through semantic analysis into finalization. A focused regression verifies
the generated requirements, per-import phases, and finalized linked-read
slots.

The directly affected semantic `phase_evidence` module passed 18/18 on the
current worktree after finalizer assertions were added. T4 remains partial:
other attribute declaration forms and the complete producer-family matrix
still need audit. T7/T8 cross-stage gates remain open.

CV-04 now has an end-to-end source-project discriminator for a whole-module
import plus a selected re-export: analysis/finalization retain the module
object target and original declaration target as distinct reads and runtime
edges; compiler lowering emits both finalized reads; and VM execution returns
the original source class through both paths. The same fixture checks every
`GetLinked` in the entry initializer maps to the expected canonical target.
The fixture now reads the selected re-export inside a nested closure and scans
every live closure owned by its compiled modules. Each emitted read must match
both finalized and materialized target tables and have an ordered runtime
dependency. The focused core test passed 1/1 in 139.17 seconds after this
extension. This closes the fixture's CV-04 path and adds fixture-wide CV-08
evidence; coverage across other program shapes and read producers remains open.

The mutual class type-import cycle fixture now also asserts the entry import's
exact canonical read target, compiles, and executes; the entry receives the
canonical `A` class while the mutually imported `A`/`B` type references keep
their zero-slot, edge-free runtime projection. The focused core test passed
1/1 in 140.06 seconds. This provides current CV-01/02 evidence and closes the
type-cycle execution case for CV-05; conformance-cycle and remaining CV-08
coverage stay open.

The minimal CV-09 protocol/default-to-adapter fixture now asserts both
mode-specific import summaries, both semantic interface edges, the single
canonical protocol identity, and the one-way runtime edge. Its default method
compiles and executes with result `23`; the focused core test passed 1/1 in
139.34 seconds. This satisfies the plan's minimal split-fixture gate. The full
Universe iteration-source migration and cutover remain explicitly deferred.
