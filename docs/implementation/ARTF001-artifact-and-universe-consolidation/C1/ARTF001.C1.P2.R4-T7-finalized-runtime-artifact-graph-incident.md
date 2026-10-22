# P2.R4 T7.2 finalized runtime graph incident

## Identity

- Plan: `ARTF001.C1.P2.R4`
- Gate: `R4.T7.2` positive compiler parity
- Branch: `feat/artifact-universe-consolidation`
- Revision: `4f0861428d82b962a094f588450e9d0577960e68`

## Required invariant

Preserve the full source import and semantic graph authority. Keep runtime
read coverage and the strict artifact runtime DAG validator. Do not weaken the
positive parity comparator or change the P2/P3 boundary.

## Observed behavior

After source bootstrap was changed to use source imports only for candidate
closure and the finalized semantic runtime graph for dependency order, the
positive SlotMap parity test advanced to artifact loading. Loading fails with
`cyclic runtime dependency graph` in `validate_runtime_dag` before compiler
comparison.

`CanonicalUniverseFormalImage::build` captures the foundation link produced by
`canonical_universe_linked_program`. That helper calls
`SemanticLinkedProgram::to_unfinalized_program()`, which documents that it
projects every symbolic import into runtime reads and edges without claiming
that runtime initialization has been validated. Phase-aware executable
finalization later omits interface-only import edges. The formal-image source
products therefore contain a provisional runtime graph that may include a
legal interface-only cycle, while the canonical executable product has a
strict DAG.

`build_universe_telosc` currently projects `view.formal.modules()` directly.
That carries the provisional runtime read slots and graph into the artifact.
The source interface cycle `reflection.module` / `reflection.package_object`
is established by the source-cycle consultation. No validator relaxation or
parity change was made.

## Relevant path

```text
canonical_universe_linked_program
  -> SemanticLinkedProgram::to_unfinalized_program
  -> CanonicalModuleProducts captured by CanonicalUniverseFormalImage
  -> build_universe_telosc(view.formal.modules())
  -> ModuleWorldV2 runtime graph
  -> validate_runtime_dag rejects a cycle
```

The phase-aware executable link is produced separately by
`ModuleLinker::finalize_executable_with_availability` and is retained in the
canonical Universe compiler program.

## Adviser ruling

Preserve source imports and semantic relationships, but take runtime slots,
import bindings, dependencies, graph, and initialization order together from
the finalized executable link. Store a phase disposition for every import.
Interface-only imports remain semantic imports without runtime slots. Runtime
reads whose eager edges are omitted by availability evidence retain their
slots and exact availability witnesses; missing or mismatched evidence must
fail validation. Keep strict DAG validation. The interface-cycle round-trip
must pass, while making both cycle edges runtime dependencies must fail.

This changes the R4.T7.2 artifact representation contract. The checkpoint and
active plan now record the amendment. The wire-version details must follow the
existing TELosc compatibility contract; no compatibility gate is waived.

## Status

The evidence establishes a mismatch between provisional semantic-link
runtime fields and finalized executable runtime fields. Artifact representation
changes are pending focused implementation and validation. P2.R4 remains
`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; P3 remains blocked.

## Latest continuation — exact compiler-derived availability authority

The first finalized ModuleWorldV3 projection cleared the provisional-cycle
failure but the source-free loader rejected a finalized witness produced by
`runtime_metadata_type_requirements`: its `MetadataTypeOwner` requirement was
not present in `SemanticWorldV4.module_execution_authority.runtime_requirements`.
This compiler-derived requirement set was used for finalization but omitted
from semantic-world export.

Adviser ruling: persist the finalizer's exact canonical requirement list in
`ModuleExecutionAuthorityV1` and use that same list for graph finalization and
artifact export. Do not exempt `MetadataTypeOwner` witnesses from exact
matching; a producer/product label is not independent authority to omit an
eager edge. The active plan now includes this amendment. The small
round-trip/tampering discriminator passes 1/1, and `phalcom-semantic` and
`phalcom-core` library checks pass. The exact SlotMap parity run after the
correction then failed during artifact construction, before load or comparison,
with `CanonicalImage(IncompletePublication("limit exceeded: duplicate runtime dependency requirement"))`.
The exact duplicate rows have not yet been identified. Preserve duplicate-
obligation validation and inspect the full finalizer requirement set before
retrying parity.
