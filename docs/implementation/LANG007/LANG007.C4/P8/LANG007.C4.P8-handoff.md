---
id: LANG007.C4.P8
kind: implementation-handoff
title: "Incremental Durability, Metadata, Cross-Module Static Products, and Materialization — Handoff"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-14
updated: 2026-10-14
branch: main
---

# LANG007.C4.P8 — Handoff

`LANG007.C4.P8` is complete (`IMPLEMENTED / FOCUSED_TESTED`).
This handoff records the established interfaces, invariants, and instructions for downstream plans (`LANG007.C4.P9`).

## Read First

1. [C4 Checkpoint Record](../LANG007.C4-CHECKPOINT.md) for checkpoint-wide authority, status, and ledger.
2. [P8 Implementation Plan](LANG007.C4.P8-detailed-implementation-plan.md) for architecture, task gates, and invariants.
3. [P8 Walkthrough](LANG007.C4.P8-walkthrough.md) for verified evidence and code modifications.

## Established Takeover Interfaces and Invariants

1. **Schema v13 and Heterogeneous Record Keys:**
   - `TYPE_METADATA_SCHEMA_VERSION = 13`.
   - `FieldKeyRef` (`Symbol`, `String`, `Int`, `Bool`) is the canonical wire key with ordering `Symbol < String < Int < Bool`.
   - `RecordFieldRef` and `ScopedRecordFieldRef` use `key: FieldKeyRef`, deserializing legacy v12 string field names as `FieldKeyRef::Symbol`.
   - `NonExportableRecordKey` and `NonExportableRecordKeyDomain` errors are retired.

2. **Durable Static Value DAG & Structural Fingerprinting:**
   - Centralized in `phalcom-type-meta/src/static_value.rs` and `phalcom-semantic/src/metadata/static_value.rs`.
   - `StaticValueTableExporter` projects all 15 `StaticValueData` forms into a topologically ordered (child-before-parent), deduplicated DAG.
   - Content fingerprints (`compute_static_value_fingerprint`) are independent of store allocation order or process-local IDs.

3. **Static Constant Export & Reconstructive Import:**
   - `MetadataExporter::with_static_constant_table` serializes static constants and value DAGs.
   - `SemanticMetadataImporter` reconstructs static values and constants into a fresh session's `StaticValueStore` and `StaticConstantTable` without executing provider source code.
   - `StaticConstantProduct` supports `ImportedStaticConstantOrigin` (`checked: None`, `imported: Some(...)`).

4. **Incremental Invalidation on Const Value Changes:**
   - `db/fingerprint.rs` and `db/query.rs` track imported constant value and dependency fingerprints.
   - Editing an exported const's value invalidates consumer definitions even when its declared type is unchanged.
   - If an initializer changes but produces the same canonical value, value identity is retained.

5. **Structural Runtime Materialization:**
   - `MaterializationDescriptor` supports `Tuple`, `Record`, and `Pack` alongside primitives and sequences.
   - `phalcom-core::modules::static_constant` materializes word-packed heap objects via `finish_tuple`, `finish_record`, and `finish_pack`.

## Downstream Instructions for P9

- **P9 Scope:** `LANG007.C4.P9` owns public ontology migration (retiring legacy aliases where planned), documentation, tooling polish, and checkpoint-wide release certification.
- **Wire Contract:** P9 consumes schema v13 as the definitive C4 wire format.
- **Pre-existing Baseline (Category D):** `ProductKind` in `structural.ph` during full Universe bootstrap (`vm/bootstrap.rs:170`) is a known pre-existing Category D baseline defect committed in `73ea73f4` prior to P7/P8.
