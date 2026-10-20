# Implementation incident: module graph allocation accounting

> The module-stage mismatch is resolved. The current semantic-import mismatch
> is tracked in
> [`ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md`](ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md).

## 1. Identity

- Program/checkpoint/plan: ARTF001 / ARTF001.C1 / ARTF001.C1.P2.R4
- Task: R4.T3, exact allocation stage C
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`
- Branch: `feat/artifact-universe-consolidation`
- HEAD: `3a72820e9`

## 2. Escalation trigger

At incident start, exact accounting appeared to require changing retained module graph storage. That crossed the graph mutation and loader allocation contracts, so a representation ruling was required. The user has since selected frozen base with mutable overlays.

## 3. Required invariant

For each load stage, independently observed allocator requests must equal the stage ledger. The fix must preserve the canonical module graph and ordinary session edit behavior. It must not debit observed bytes after allocation or weaken exact equality.

## 4. Expected architecture

R4.T3 requires exact stage accounting, N/N-1 behavior, rollback, and clean retry. Current canonical module authority is retained as an immutable artifact base; local source edits use ordinary workspace overlay state.

## 5. Observed behavior

The first saved-artifact run failed during module materialization before semantic and query import. After the user's representation ruling and the graph/index changes, the same artifact passes module materialization and type-graph import exactly. It now reaches semantic import, where a separate allocation mismatch remains.

Initial module stage: 2,131,498 independently observed bytes versus 1,094,188 ledger bytes, a +1,037,310-byte delta.

After restoring budget-aware frozen indexes and graph rows, module materialization is 1,138,500 observed bytes versus 1,138,500 ledger bytes. Type-graph import is 1,393,544 observed bytes versus 1,393,544 ledger bytes. Semantic import has a separate +122,172,801-byte delta; see the linked consultation packet.

The artifact is saved at `/tmp/artf001-universe.telosc` (20,667,377 bytes). Query and publication stages, N/N-1 checks, rollback, and retry have not been reached.

## 6. Minimal reproduction

```sh
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

This was the original module-stage reproduction. It now passes the module and type-graph comparisons; the current failure is the semantic-import comparison at `phalcom-core/tests/artifact_allocation.rs:166`.

## 7. Relevant code path

```text
load_module_world_with_allocation_budget
  -> load_module_world_inner
  -> build ModuleId, interface, topology, and graph indexes
  -> budget-aware frozen map/set and graph constructors
  -> publish CanonicalExternalModuleWorld
```

- `phalcom-modules/src/compiler_artifact.rs`: artifact materialization now uses budget-aware frozen maps and sets.
- `phalcom-modules/src/graph.rs`: imported graph rows use frozen base storage with mutable overlays for edits.
- `phalcom-modules/src/authority.rs`: existing `ModuleAuthorityMap` and `ModuleAuthoritySet` support mutable source storage and frozen budgeted storage.
- `phalcom-modules/src/identity.rs`: budgeted module-identity cloning.

## 8. Scoped implementation diff

The T2.6b correction adds QueryImage V4 physical product families, nominal/alias shell row validation and fingerprints, exact alias static-definition authority, blocked nominal-query retirement, final native shell checks, and Package's explicit sibling Module import. The module allocation correction moves the already-budgeted component string, restores frozen budget-aware retained maps and graph rows, adds mutable graph overlays for local edits, and accounts for `ProjectUniverse` construction inside the stage transaction.

## 9. Evidence

- The original `load_module_world_inner` collected retained indexes into ordinary `BTreeMap`/`BTreeSet` values.
- The original graph `from_entries` constructors collected their input vectors into ordinary `BTreeMap` storage.
- Before correction, the graph types did not use the existing budget-aware frozen map/set constructors.
- Replacing the second component-string copy reduced independently observed allocation by 87,702 bytes on the same saved artifact and stage.
- These materialization sites now use budget-aware frozen storage; the exact module-stage comparison passes.
- The performance branch contains a directly relevant, previously implemented
  allocation fix. Commit `c09ee934f` added budgeted constructors to module graph
  types and used `ModuleAuthorityMap` / `ModuleAuthoritySet` for declarations,
  exports, exposed-child sets, linked exports, binding indexes, topology indexes,
  and graph rows in `load_module_world_inner`.
- Comparing `perf/universe-characterization` with the consolidation branch shows
  those exact materialization sites were changed back to ordinary BTreeMap /
  BTreeSet construction in the V2 project-authority adaptation. The current
  loader still has the pre-existing `ModuleAuthorityMap` and `ModuleAuthoritySet`
  model available; the principal missing piece for the selected design is
  edit-time graph mutation over a frozen base.

## 10. Implementer's causal hypothesis

Ordinary retained tree-map/set allocations caused the module-stage mismatch. Their individual shares were not isolated, but restoring budget-aware frozen construction closed the stage delta. The semantic-import mismatch is separate and is tracked in the linked packet.

## 11. Alternative hypotheses

Before correction, fingerprinting, validation, or helper conversions were possible additional contributors to the module delta. The aggregate module comparison is now exact; its individual allocation sites were not isolated. The current semantic-import delta is separate and described in the linked packet.

## 12. Attempts already made

- Replaced `ModuleComponent::from_identifier` with `from_owned_identifier` in `ModuleId::clone_with_allocation_budget`. Prediction: remove the second string allocation while preserving identity. Result: observed module-stage bytes fell by 87,702; a substantial mismatch remains.
- Restored budget-aware frozen construction for retained maps, sets, and graph rows; kept local edits in graph overlays; and moved `Arc<ProjectUniverse>` construction inside the child budget. Result: module stage exact at 1,138,500 bytes, type-graph import exact at 1,393,544 bytes, `cargo check -p phalcom-modules` passed, and graph tests passed 7/7.

## 13. Why local authority is insufficient

At the time of this incident, budgeted graph construction appeared to conflict with graph edit methods. The user settled that boundary by choosing a frozen base with mutable overlays. The implementation preserves mutable source graphs and local edits without thawing the imported base.

## 14. User decision and implementation contract

The user selected frozen base with mutable overlays.

- Artifact-loaded canonical graph and retained indexes use frozen,
  budget-aware storage.
- Source-backed graph construction and graph APIs remain mutable.
- Local edits are held in mutable overlays above the imported base.
- The imported canonical base is never mutated or implicitly thawed.
- Lookup/update semantics across base and overlay must retain one canonical
  project/module identity and be tested at the owning module/session boundary.
- No artifact wire-schema change is authorized by this ruling.

Implemented under the user's ruling: retained imported authority collections
use budget-aware frozen storage; source graphs remain mutable; graph edits live
in overlays; and the wire schema is unchanged. Focused graph tests pass 7/7.
The saved artifact reports exact module materialization and type-graph import.
The separate semantic-import delta is not covered by this representation
decision.

## 15. Consultation question and recommendation

The user answered the original representation question by selecting frozen
base with mutable overlays. The allocation-aware constructor pattern from
`c09ee934f` was restored, with overlays for graph edits. The module
representation question is closed at the measured module boundary. The
separate semantic-import question is in the linked consultation packet.
