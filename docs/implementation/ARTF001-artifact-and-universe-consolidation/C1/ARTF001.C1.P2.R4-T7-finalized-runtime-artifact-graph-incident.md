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

## Complete canonical-universe collision inventory — 2026-10-22

The exact strict parity test was run with temporary reporting at the
`canonical_universe.rs` composition boundary, after concatenating
`snapshot.generated_runtime_requirements` and
`runtime_metadata_type_requirements`. Artifact construction still fails at V4
semantic-world validation, before load or program comparison. The report
captured **78 colliding semantic obligations (156 observations) in 17
importers**. No other collision family appeared.

All 78 obligations share these fields:

```text
dependency: universe.scalar.number
reason: RuntimeDeclarationReference
read_target: None
target: Module(universe.scalar.number)
product: CanonicalIdentity
timing: BeforeSourceExecution
consumer: importer
producers: CallableOwner + MetadataTypeOwner
```

The ranges below are byte offsets within the named importer's source file.
Each range denotes one identity collision with both listed producer rows.

| Importer | Count | Source ranges | Producers per range |
|---|---:|---|---|
| `collections.range` | 19 | `1549..1561, 1861..1873, 2510..2522, 2778..2790, 4189..4201, 4658..4731, 4700..4731, 4911..4931, 4948..4973, 4996..5016, 5033..5058, 5330..5350, 5367..5392, 5411..5431, 5448..5473, 5948..5963, 6310..6325, 6853..6868, 7292..7303` | `CallableOwner`, `MetadataTypeOwner` |
| `collections.bytes` | 1 | `5348..5361` | `CallableOwner`, `MetadataTypeOwner` |
| `concurrency.fiber` | 10 | `8190..8205, 12013..12031, 17266..17293, 17334..17346, 19238..19280, 19251..19279, 19336..19348, 19418..19446, 19804..19832, 19917..19945` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.arrow` | 1 | `2264..2487` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.chain` | 1 | `7815..7900` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.fold` | 6 | `2788..2890, 2954..3131, 3477..3634, 3794..3951, 4015..4252, 4315..4484` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.journal` | 1 | `3116..3201` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.reader_result` | 1 | `836..874` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.state` | 1 | `4080..4398` | `CallableOwner`, `MetadataTypeOwner` |
| `functional.state_result` | 1 | `1579..1622` | `CallableOwner`, `MetadataTypeOwner` |
| `math.complex.kernel` | 1 | `1023..1058` | `CallableOwner`, `MetadataTypeOwner` |
| `math.reduction` | 1 | `1779..1934` | `CallableOwner`, `MetadataTypeOwner` |
| `numbers.complex` | 1 | `5725..5756` | `CallableOwner`, `MetadataTypeOwner` |
| `scalar.string` | 5 | `4771..4781, 4799..4809, 5380..5390, 5412..5422, 14902..14915` | `CallableOwner`, `MetadataTypeOwner` |
| `text.substring` | 11 | `7642..7655, 13495..13505, 13523..13533, 20423..20435, 20438..20450, 22415..22427, 22447..22459, 22782..22794, 22814..22826, 25107..25118, 25137..25148` | `CallableOwner`, `MetadataTypeOwner` |
| `time.calendar.date` | 3 | `1740..1751, 1770..1781, 1784..1795` | `CallableOwner`, `MetadataTypeOwner` |
| `time.calendar.datetime` | 10 | `2011..2028, 2053..2074, 2314..2327, 4680..4701, 4720..4741, 4744..4765, 4795..4816, 4818..4839, 4841..4862, 4864..4885` | `CallableOwner`, `MetadataTypeOwner` |
| `time.timezone` | 5 | `1908..1920, 1995..2008, 2033..2046, 2073..2086, 2113..2126` | `CallableOwner`, `MetadataTypeOwner` |

The rows prove that `MetadataTypeOwner` is a secondary observation when a
`CallableOwner` already owns this exact module-level canonical-identity
obligation. This does not establish global precedence: the metadata observation
must remain when no existing owner has the same full semantic obligation
identity. Next, share the V4 obligation key with the native linker, make this
projection defer only to an exact pre-existing owner, add a native composition
regression, and make finalization reject any remaining duplicate obligation.

## Additional source-entry collision — 2026-10-22

After the canonical artifact built, validated, and loaded, the strict positive
parity rerun reached ordinary source compilation of `packages/slotmap` and the
new native duplicate guard rejected a separate obligation:

```text
importer: resolved SlotMap project / main
dependency: universe.scalar.string
range: 670..682 in packages/slotmap/src/main.ph (`propositions` in the for loop)
reason: RuntimeDeclarationReference
read_target: None
target: Binding(universe.scalar.string::String)
product: RuntimeObject
timing: DuringModuleInitializer
consumer: importer
producers: ConformanceTypeOwner x 4
```

This is one semantic obligation with four exactly identical observations from
the same producer family. The targeted source-mode SlotMap test reproduced it
in 161.97 seconds. The metadata projection had already deferred its exact
duplicate rows, so this is an independent producer-local repetition. The
existing `push_binding_owner_requirement` coalesces exact repeated
`PatternOwner` observations because pattern sites and resolved views overlap,
but it did not apply the same producer-local rule to repeated
`ConformanceTypeOwner` evidence views. Correct that owner boundary to retain
one exact row for that producer while preserving different bindings, targets,
products, timings, ranges, and other producer families. Keep the native
finalizer guard fail-closed for any cross-producer semantic duplicate that
remains.

The phase-evidence producer now coalesces exact duplicate `ConformanceTypeOwner`
rows at that owner boundary. Its focused regression passes, including a
different binding at the same span. A fresh source-mode SlotMap test no longer
fails during finalization and reaches VM execution, but then fails at
`main.ph:29` with `specialized call: selected method has no canonical runtime
callable metadata`. No other collision rows were printed. This runtime failure
is recorded separately from obligation uniqueness; strict source/artifact
parity has not yet been rerun after the correction.
