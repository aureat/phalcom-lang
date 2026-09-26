---
id: LANG007.C3.P3.R1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
prepared: 2026-09-25
---

# LANG007.C3.P3.R1 walkthrough

## Result

P3.R1 corrected the binding-projection defect found at the P3 T4 endpoint.
One canonical `DeclarationTypeInfo` now carries the semantic application
projection and the optional runtime/native owner projection as sibling facts.
This preserves the source declaration identity while allowing Tuple and Record
to have higher-kinded semantic forms and independent VM owners.

P3.R1 was implemented at revision `82c9dd0382ece4932de4a52e98fdd36452b122c8`.
The structural-data completion plan now owns `LANG007.C3.P4`; this walkthrough
records only the earlier corrective closure.

## Architecture established

- `DeclarationTypeInfo::binding.semantic_form` selects ordinary application
  or intrinsic Unit/Tuple/Record application behavior.
- `DeclarationTypeInfo::binding.runtime_owner` identifies the existing
  runtime/native owner when one exists.
- Semantic annotation formation consumes `semantic_form`.
- Native surface import, runtime form selection, and runtime facade lowering
  consume `runtime_owner`.
- Ordinary declaration-shell refresh preserves already-registered projections
  when a refreshed shell has the default binding. Fingerprints include both
  projections.
- Tuple application continues through `CompleteTypePack` and
  `TupleFromPacks`; Record remains backed by `RecordRowId` and
  `TypeData::Record`. No second owner registry or row-to-TypeId conversion was
  introduced.

## Focused verification recorded at closure

- semantic declarations: 8 passed;
- semantic type annotations: 51 passed;
- semantic native Tuple runtime-owner import: 1 passed;
- semantic declaration-shell fingerprints: 34 passed;
- semantic associated/type-member closure: 10 passed;
- semantic incremental type-member suite: 24 passed;
- core Unit intrinsic facade/runtime-class identity: 1 passed.

These are historical P3.R1 results, not P4 verification. The shared checkpoint
records the current P4 status and action.
