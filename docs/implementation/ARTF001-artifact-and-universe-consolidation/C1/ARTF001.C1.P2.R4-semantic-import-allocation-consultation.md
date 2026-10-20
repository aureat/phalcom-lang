# Implementation incident: semantic import allocation accounting

## 1. Identity

- Program/checkpoint/plan/task: ARTF001 / ARTF001.C1 / ARTF001.C1.P2.R4 / T3, stage E
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`
- Branch and starting revision: `feat/artifact-universe-consolidation` at `3a72820e9`
- Artifact: `/tmp/artf001-universe.telosc` (20,667,377 bytes)

## 2. Escalation trigger

The fix may change the validation representation at a schema boundary. Two
plausible approaches have different memory and validation costs: budget a full
V4-to-V3 projection, or validate V4 records without constructing that copy.
This meets the repository escalation triggers for representation changes and
architecturally distinct fixes.

## 3. Required invariant

For semantic import, independently observed allocation requests must equal the
stage ledger. N-1 failure must remain typed and must roll back the child
transaction. Preserve the current V4 validation rules; do not hide the mismatch
by adjusting the ledger after allocation or weakening exact equality.

## 4. Expected architecture

R4.T3 requires exact accounting for each loader stage. Semantic import must
validate the current V4 world and reconstruct canonical semantic products
within the caller's allocation transaction. The allocation gate is an
independent measurement of allocator requests, including transient requests.

## 5. Observed behavior

The saved-artifact load now passes module materialization and type-graph import
with exact accounting:

| Stage | Ledger | Observed | Delta |
|---|---:|---:|---:|
| Module materialization | 1,138,500 | 1,138,500 | 0 |
| Type graph import | 1,393,544 | 1,393,544 | 0 |
| Semantic import | 317,818,249 | 439,991,050 | +122,172,801 |

The semantic import returns successfully, then fails the independent allocation
assertion at `phalcom-core/tests/artifact_allocation.rs:166`. Later query and
publication stages, N/N-1, rollback, and retry were not reached.

## 6. Minimal reproduction

```sh
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

Expected: semantic import observed bytes equal its ledger. Observed: 439,991,050
bytes versus 317,818,249, a 122,172,801-byte excess.

## 7. Relevant code path

```text
artifact-allocation test decodes V4 DTO before measuring stage E
  -> validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget
  -> validate_and_import_semantic_world_inner
  -> SemanticWorldV4::validate_with_allocation_budget
  -> SemanticWorldV4::to_v3_for_validation
  -> SemanticWorldV3::validate_with_allocation_budget
  -> TypeStore, structural contracts, and static-value import
```

- `phalcom-compiler-artifact/src/semantic/world.rs`: V4 validation calls
  `to_v3_for_validation`; that method collects new vectors, clones the other
  world families, and calls `CallableAnalysisRecordV2::to_v1`, which deep-clones
  each callable analysis record's owned collections.
- `phalcom-semantic/src/artifact/import.rs`: semantic import invokes V4
  validation before reconstruction. `require_semantic_import_allocation_accounting`
  currently returns `Ok(())`, despite the surrounding comment saying unaccounted
  imports should be rejected.
- `phalcom-core/tests/artifact_allocation.rs`: stage E independently measures
  allocator requests around the semantic importer.

## 8. Scoped implementation state

The earlier module-stage correction restores budget-aware frozen authority
collections, adds graph edit overlays, and moves `ProjectUniverse` construction
inside the module allocation transaction. The module graph tests pass 7/7; the
saved artifact reports exact module and type-graph stages.

The semantic importer also now accepts an aggregate `Runtime` phase summary as
covering an individual `InterfaceOnly` callable observation, matching the
exporter's existing coverage rule. Its focused regression passes 1/1. No
semantic allocation correction has been made.

## 9. Evidence and prior work

- The measured stage E starts after the test decodes the V4 DTO. The additional
  V4 validation projection therefore occurs inside the measured semantic
  import stage.
- `to_v3_for_validation` uses ordinary `.clone()`, `collect()`, and vector
  construction without charging the `AllocationBudget`. This is a direct
  unaccounted allocation path. Its individual share of the 122,172,801-byte
  excess has not been measured.
- Other semantic import work may also allocate outside the ledger. The current
  `require_semantic_import_allocation_accounting` helper does not block that
  work, so the aggregate delta cannot yet be attributed to the projection
  alone.
- The performance branch addressed module graph materialization in
  `c09ee934f` with budget-aware frozen constructors.
- The same branch recorded zero-delta V3 semantic import in `dde0e8ba9`
  (264,140,594 observed and ledger bytes in the commit record). That certified
  path consumes `SemanticWorldV3` directly and has no V4-to-V3 validation copy.
  The current consolidation's V4 projection is therefore new work beyond that
  precedent.

## 10. Implementer's hypothesis

The V4-to-V3 projection is likely a large contributor to the semantic-stage
excess because it duplicates the DTO, including callable analysis graphs,
immediately before validation. The exact contribution is unknown; additional
importer allocations may account for part of the delta.

## 11. Alternatives

1. Keep the shared V3 validation path and make V4-to-V3 conversion fully
   allocation-budgeted. This preserves validation reuse but retains a large
   temporary duplicate and must account for every nested allocation.
2. Validate V4 records in place through borrowed/shared validation logic. This
   avoids the full DTO copy but changes how the V3 and V4 validators share
   authority and requires preserving all current validation checks.

Skipping importer validation because decode already validated the DTO is not
equivalent: the public importer also accepts constructed V4 values.

## 12. Attempts

- Module materialization: restored budget-aware frozen map/set constructors and
  added graph overlays. Prediction: close the retained-index allocation gap
  without thawing the imported base. Result: module stage exact at 1,138,500
  bytes; graph tests 7/7.
- Semantic phase compatibility: aligned importer coverage with the exporter.
  Prediction: allow the saved artifact's valid aggregate/runtime phase
  combination through. Result: focused regression 1/1; the vertical gate then
  reached semantic import and exposed the allocation delta above.

## 13. Why local authority is insufficient

The narrow allocation sites are identified, but the choice affects whether
validation retains a second full semantic-world representation or shares one
validation path across schema versions. The V3 performance result does not
answer whether that duplicate should be budgeted or removed for V4. This is a
representation and validation-ownership decision, not a mechanical port of the
module map constructors.

## 14. Decision requested

For current V4 semantic import, should R4 preserve V3 validation reuse by
charging a complete temporary V4-to-V3 projection to stage E, or validate the
V4 DTO in place using shared borrowed validation logic? Please also confirm
that the allocation-accounting guard must remain rejecting until every
semantic import allocation is covered by the transaction.

Recommendation: prefer in-place or borrowed validation if it can preserve the
same validation rules. Do not account for a clone after the fact; if the
projection remains, charge each allocation at construction and establish that
the stage limit handles its temporary peak.

## 15. Status

This request is pending review. Module and type-graph exact accounting are
focused-tested; semantic import exact accounting is not. Do not promote R4.T3
or proceed to its N/N-1 and rollback gates until the semantic stage is exact.
