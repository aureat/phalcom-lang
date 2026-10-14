# Canonical Universe generated-requirement cycle incident

Status: canonical bootstrap blocker closed; MODL001.C2.P4 remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED.

## Scope and required invariant

Consolidation of integration `52838aa5`, main `a09de379` (already inherited), and
collections `60e6a3db`. Worktree: `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom`;
branch: `feat/consolidation-universe`. PERF and concurrent enum repairs are excluded.

MODL001.C2.P4 requires immutable finalized reads, every generated runtime owner
represented before finalization, and strict runtime DAG rejection. Compiler-side
late insertion and relaxed cycle validation are prohibited.

## Evidence and attempted correction

1. Conservative Universe wrapping failed `UncoveredRuntimeRead` for a root
   re-export to `errors.argument`. The owning linker was corrected to project
   canonical re-export owners before DAG validation. All 18 linker tests pass.
2. Runtime bootstrap then reached `callable.function`, whose `attempt()` reads
   prelude `Error`, and failed because the conservative wrapper had no generated
   read. The canonical builder did not pass symbolic linkage into semantic
   analysis and therefore omitted phase/generated products.
3. One serious correction applies existing staged APIs to canonical construction:
   `link_semantic` → fallible semantic analysis → generated requirements plus
   existing metadata owner projection → `finalize_executable_with_requirements`.
   Core and LSP checks pass. Exact runtime discriminator fails strict DAG validation.

Reproducer (pinned toolchain; clear local flags):

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture
```

Completed result: 0 passed, 1 failed, 989 filtered, 140.05 seconds.

Reported cyclic initialization set:

```text
collections.bytes, collections.iterator_laws, collections.list,
collections.map, collections.range, collections.record, collections.set,
collections.tuple, errors.argument, errors.result, option.option,
scalar.bool, scalar.char, scalar.number, scalar.string,
scalar.string_index, text.builder, text.substring
```

This is a reported cycle set, not an independently established single SCC or
an edge-by-edge root-cause proof.

## Causal hypothesis and decision requested

Canonical generated declaration/reification reads now participate in the eager
source-module DAG. Some may be satisfied by already allocated native-floor
objects rather than source initialization, or the legacy source graph may need
structural migration. No bounded evidence yet distinguishes these alternatives.

Decide the canonical contract between native-floor availability, generated
runtime reads, metadata-owner requirements, and source initialization. If floor
satisfaction is valid, define the owning canonical product and exact proof for
that classification. If source cycles are real, amend the source architecture
and consolidation scope. Do not introduce a blanket Universe exemption.

Triggers: nonlocal compiler/runtime/bootstrap contract; materially different
architectural consequences between floor classification and source migration;
active P4 strict-DAG requirement conflicts with canonical source acceptance.
See `docs/workflow/shared-consultation-escalation-protocol.md`.

## Takeover

Inspect only:

- `phalcom-core/src/modules/canonical_universe.rs`: canonical staged candidate.
- `phalcom-core/src/modules/compile.rs`: shared runtime metadata owner collector.
- `phalcom-semantic/src/phase_evidence.rs`: generated requirement producers.
- `phalcom-modules/src/linker.rs`: strict executable finalization.
- `phalcom-core/src/vm/bootstrap.rs`: native-floor and source bootstrap boundary.
- MODL001.C2 checkpoint/ruling and this integration record.

The staged canonical candidate is saved for review; it is not accepted runtime
closure. No further corrective attempt or relaxed validator was applied.

## Adopted continuation decision — 2026-10-14

The user approved the attached architectural audit's direction and the
bootstrap-closure implementation plan. Continue under `MODL001.C2.P4` with
strict cycle rejection. Preserve exact requirement provenance and classify
availability by required product and consumption stage. Use a VM-independent
native-floor contract for canonical source compilation and a fully initialized
Universe contract for ordinary user compilation. A proof is exact to a module
or binding and its promised stage; it cannot be inferred from Universe module
membership or a reserved slot. Finalized read slots remain allocated even if
an availability proof removes their initialization edge.

The continuation starts at `d203e71a` in this isolated worktree. Focused
baselines passed: `phalcom-modules --test linker` (18/18) and
`phalcom-semantic --test semantic integration::phase_evidence` (12/12). The
previous exact canonical bootstrap failure is the same-revision baseline
recorded above; it has not been rerun. First implementation task is deterministic
edge provenance/SCC reporting, with no classification change. Source migration
is limited to cycles proven to be genuine eager-initialization dependencies.
Other P4 acceptance gaps remain open.

## Outcome — 2026-10-14

The approved availability closure retained strict DAG validation and removed
only edges covered by exact module/binding, product, and stage proofs. No
genuine source-initialization cycle remained, so no source module was edited
to reorder initialization or change its initializer body.

An intermediate conservative source-value classification exposed one generated
SCC with a deterministic path. The path showed a trait conformance import and
callable/pattern owner requirements. Typed canonical-identity proofs and exact
native owner-binding proofs resolved those requirements without a DAG
exemption; the remaining authored source-value import is exercised separately
by the ordinary user linked-read check.

The first full runtime attempt after finalization reached `universe:time.timestamp`
and exposed a separate materialization defect: linked export/read projection
replaced the native-floor immediate `None` binding with the hidden `None` class
handle. The resulting `Option.==` / `Object.is` recursion was traced to the
wrong live binding value. Materialization now preserves the immediate singleton
in eager and lazy Universe paths; the hidden class identity remains in the
canonical class registry. `Object.is` and `is!` compare class identities with
`===`, consistent with R-INV-1.2. Both mirrored Object sources carry that
correction.

Focused evidence:

- `phalcom-modules --test linker`: 24/24; exact generated cycle path retains
  requirement provenance.
- `phalcom-modules --test graph`: 5/5; sorted SCCs and deterministic cycle
  path.
- `phalcom-semantic --test semantic integration::phase_evidence`: 12/12.
- Native-floor inventory against a fresh native VM: 1/1, including canonical
  values/identities and negative checks for reserved source-only bindings.
- Exact canonical bootstrap discriminator: 1/1 in 188.46s, including source
  reopen identity, metadata installation, immediate `None`, and a user
  selective import whose finalized linked read resolves to the exported
  Universe binding.
- `object_model_invariants::isa_is_reflexive_and_superclass_closed`: 1/1,
  including `None.is(Option)` and `None.is(Object)`.

Workspace and release certification remain deferred. The other P4 generated
producer families and cross-stage CV/T8 gaps remain open.

## Integrated closure verification — 2026-10-14

PR #21 merged the native-floor/runtime-availability repair into
`feat/consolidation-universe` as `57c7cb66`. During reconciliation the
previously unverified bootstrap-order invariant exposed an unsatisfied
runtime edge `collections.interval -> errors.argument`, opposite to the
authored-import-only source index's initialization schedule. The correction
retains the source index's initializer closure but imposes the finalized
runtime DAG as the dependency-first schedule, with stable authored-order ties.
No effective graph edge was silently removed.

A separate invariant repair prevents `SourceInitializedValue` availability
proofs for modules whose source initializers are not executed. The exact
source-value proof regression, native-floor catalog inventory, executable
canonical bootstrap/user linked-read discriminator, and `None.is` object
identity test all passed on integration head `b1ee59f4`, as did the module
graph 5/5, linker 25/25, semantic phase evidence 12/12, source-order 3/3,
and pattern scratch 3/3 focused tests.

Authoritative scoped CI:
https://github.com/aureat/phalcom-lang/actions/runs/38613486708 (SUCCESS).
Historic failures above remain incident evidence only. Full workspace and
release certification are separate and incomplete; P4 remains PARTIAL.
