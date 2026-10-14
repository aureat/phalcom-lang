---
id: LANG007.C4.P7
kind: implementation-handoff
title: "Recursive Generic Static Computation, Residualization, and Memoization — Handoff"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-14
updated: 2026-10-14
branch: main
---

# LANG007.C4.P7 — Handoff

`LANG007.C4.P7` is complete (`IMPLEMENTED / FOCUSED_TESTED`).
This handoff records the established interfaces, invariants, and instructions for downstream plans (`LANG007.C4.P8`, `LANG007.C4.P9`, `LANG007.C4.P10`).

## Read First

1. [C4 Checkpoint Record](../LANG007.C4-CHECKPOINT.md) for checkpoint-wide authority, status, and ledger.
2. [P7 Implementation Plan](LANG007.C4.P7-luna-patch-grade-implementation-plan.md) for architecture, task gates, and amendments.
3. [P7 Walkthrough](LANG007.C4.P7-walkthrough.md) for verified evidence and code modifications.
4. Incidents:
   - [`INC-004`](LANG007.C4.P7-INC-004-control-residual-acceptance.md) (control residual deferral via enclosing `Apply`)
   - [`INC-005`](LANG007.C4.P7-INC-005-symbolic-product-fingerprint.md) (cross-session product fingerprinting and logical project coordinates)
   - [`INC-006`](LANG007.C4.P7-INC-006-variant-residual-result-domain.md) (Variant result domain subtype check)
   - [`INC-007`](LANG007.C4.P7-INC-007-pack-residual-result-domain.md) (Pack proper result type and Symbol ordering)
   - [`INC-008`](LANG007.C4.P7-INC-008-checked-pack-construction.md) (checked source Pack construction and open iteration item domain)
   - [`INC-009`](LANG007.C4.P7-INC-009-pack-spread-inference.md) (exact Pack spread shape inference and static fusion)

## Established Takeover Interfaces and Invariants

1. **Static Application Memoization & Cycle Detection:**
   - `StaticApplicationKey` normalizes static definitions, values, residuals, and branch proofs.
   - `StaticApplicationMemo` caches exact subproblems and detects cycles (`StaticCycle`).
   - Recursion cycles format with `↻` markers and source frames.

2. **Static Residual Store & Normalized IR:**
   - Centralized in `phalcom-semantic/src/static_eval/residual.rs`.
   - `StaticResidualStore` manages interning of `StaticResidualData`: `Parameter`, `Apply`, `Proof`, `MemberProjection`, `If`, `Match`, `Pack`, `Tuple`, `Record`, `Data`, `Variant`.
   - Normalization and hashing are independent of arena allocation order.

3. **Residual Specialization Engine:**
   - Centralized in `phalcom-semantic/src/static_eval/specialize.rs` (`specialize_residual`).
   - Supports parameter substitution, proof goal substitution, and structural deconstruction.
   - Preserves No-New-Facts identity: returns original residual reference if no new facts specialize the node.

4. **Control Residuals & Untaken Branch Immunity (INC-004):**
   - Untaken branch bodies in `If` and `Match` are never evaluated during static evaluation.
   - If condition or scrutinee cannot be decided, control residuals defer via enclosing `Apply` to prevent invalid evaluations of untaken arms.

5. **Structural Construction Residuals & Pack Invariants (INC-006, INC-007, INC-008, INC-009):**
   - `Variant` residuals preserve exact `VariantId` and prove subtype against declared Sum via bounded canonical relation (`INC-006`).
   - `Pack` residuals carry a proper `result_type: TypeId`, report `ValueType(result_type)`, enforce unique Symbol labels, and order positional entries before labeled entries (`INC-007`).
   - Checked source Pack construction is published by the ordinary checker (`CheckedStaticConstruction`) with `PackPositional`, `PackLabeled`, and `PackExpansion` payloads (`INC-008`, `INC-009`).
   - Ordinary constructor shape inference binds exact Pack spreads (`*P`, `**P`, `***P`) when constructors have no fixed positional parameters. `extract_pack_expansion_elements` decomposes closed operand types (`INC-009`).
   - The static evaluator fuses exact spread elements in $O(\text{total entries})$ without intermediate collections.
   - Static `for` cursor uses $O(1)$ labeled indexing (`pack_labeled_value_at`) to eliminate quadratic lane copies.

6. **Residual Growth Budget (T11):**
   - `BudgetKind::ResidualNodes` charges for newly allocated residual nodes and variable-size payloads.
   - Cooperative cancellation token preempts budget exhaustion.

7. **Symbolic Product Fingerprints & Generalized Record Metadata (T12, INC-005):**
   - Centralized in `phalcom-semantic/src/db/static_fingerprint.rs`.
   - Hashes canonical declaration, type, value, proof, and environment meanings rather than store-qualified IDs.
   - Uses revision-independent logical project identities (`LogicalProjectIdentity`) for query fingerprints.
   - `StaticConstantProduct` and `StaticDefinitionProduct` retain their owning `StaticResidualStore`.
   - Metadata export in `metadata/export.rs` supports generalized `FieldKey` Records.

## Downstream Instructions for P8

- **P8 Freeze Met:** `LANG007.C4.P7` is frozen and verified. P8 can now safely consume the normalized static residual vocabulary and symbolic product fingerprints.
- **Wire Serialization Boundary:** P8 owns durable binary encoding, wire schema compatibility (e.g. metadata v14 if chosen), and cross-module artifact transport.
- **Store Attachment:** Any serialized/deserialized symbolic product must retain its owning `StaticResidualStore` rather than an orphaned reference.
- **Pre-existing Baseline Failure (Category D):** `snapshot_metadata_round_trip_preserves_static_alias_and_associated_term_identity` reports `NonExportableRecordKeyDomain` in the legacy export path. This is an existing Category D baseline issue owned by P8's metadata modernization work.
