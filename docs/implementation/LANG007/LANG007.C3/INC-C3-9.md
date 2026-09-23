# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T8 — metadata catalog and validation
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Scope: working-tree implementation; no commit created

## 2. Escalation trigger

The pre-v8 validator could check that a static constructor owner matched a nominal
or associated domain, but the serialized artifact had no declaration/case catalog.
It could not prove case membership, payload arity/kinds, or associated owner
environment arity/kinds. The persisted schema contract required an architectural
decision.

## 3. Required invariants

- Stable domain and case identity comes from declaration identity, never store-local
  numeric IDs or source spelling.
- Associated owner specialization remains a heterogeneous `StaticArgument` environment
  on the domain reference.
- Current membership comes from canonical declaration products, not retained TypeStore
  interning history.
- Schemas v1-v7 retain their established validation behavior.
- Metadata consumers validate kinded `Applied` arguments instead of assuming every
  argument is a proper type.

## 4. Architecture decision

The T8 architectural adviser reviewed the missing catalog boundary on 2026-09-23 and
recommended this typed schema-v8 contract.

Use schema v8 with a typed static-domain catalog keyed by
`StableStaticDomainDeclarationRef::{Nominal, Associated}`. Each record stores the
ordered owner parameter kind/mode pairs and sorted case names with ordered payload
kinds. Associated owner arguments remain on `StaticDomainRef::Associated` and are
checked against the catalog's owner shape. Decoded v1-v7 bundles default the added
catalog field to empty and continue through their existing weaker validators; readers
whose maximum schema is v7 reject v8.

Snapshot-backed export receives the current nominal data/enum semantic products and
associated static-domain products. The standalone TypeStore-only exporter remains a
detached mode for callers without a semantic snapshot; production snapshot paths attach
canonical products and do not enumerate store history.

## 5. Implementation

- Bumped type metadata schema to v8 and added the typed catalog records.
- Carried nominal static-domain products through published data/enum semantic products
  and their query fingerprints.
- Exported catalog entries from those products and the snapshot's associated-domain
  products, including cases that have no observed `StaticData` term.
- Added v8 validation for missing/duplicate declarations and cases, associated owner
  environment arity/kinds, static-data case membership and payload arity/kinds, and
  global/scoped `Applied` argument and residual result kinds. Heterogeneous row
  arguments validate sorted/unique fields, Type-valued fields, and RecordRow tail
  parameter kinds.
- Kept the v1-v7 feature floors unchanged and updated the current-schema compatibility
  sentinel.

## 6. Verification

Passed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta -p phalcom-semantic -p phalcom-core --lib
```

No tests were added or run. Catalog rejection cases, v1-v7 preservation, cross-module
composition, import reconstruction, provider edit invalidation, and cold/incremental
parity remain unverified.

## 7. Status

The schema/exporter/validator slice is `IMPLEMENTED / UNVERIFIED`. T8 remains partial;
this incident does not close provider/import reconstruction or incremental acceptance.
