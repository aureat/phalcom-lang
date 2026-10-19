---
id: ARTF001.C1.P2
kind: implementation-plan-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
planning_revision: 3cf206757638b23884e59edb602c3ff0c3d28150
implementation_revision: ea136a0b5320c0a1c7ac78de5dc798b9d58e3b95
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-19
---

# ARTF001.C1.P2 — Walkthrough: Artifact Substrate and Import Closure

## Outcome

P2 has a substantial implementation, but the P2 audit identified acceptance gaps that prevent `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The repository state remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; P3 is not ready to start.

The implementation introduces the `phalcom-compiler-artifact` crate and current artifact container, module-world, metadata, semantic-world, and query-image wire schemas. Stable declaration/project/module identity is represented with durable refs, and static metadata is routed through the current `phalcom-type-meta` model. The default compiler remains source-based.

The importer currently validates and stages a module world and metadata bundle, then moves the `SemanticWorldV3` and `QueryImageV2` DTOs into `ImportedCompilerImage`. It does not reconstruct those DTOs into live current-main `TypeStore`/`StaticValueStore` state, publish a `SemanticSnapshot`, or install typed query products in `SemanticDb`. The type and query DTOs therefore must not be described as an imported live semantic generation.

## Corrective changes in the current working tree

- Added `query_fingerprint_abi` to `CanonicalCompatibilityIdentity` comparison.
- Changed metadata section decode to use the load allocation budget; allocation failures retain their typed `AllocationError`.
- Changed import staging to consume `CompilerArtifactPayload`, move wire sections, and charge the retained external module-world `Arc`.
- Made query-root validation enforce sorted unique keys without a validation-side set.
- Added typed product-row references and input/product fingerprints to QueryImage V2, with cross-section row-bound checks.
- Added an exhaustive mapping from all 46 current `QueryKind` values to portable query kinds and explicit profile dispositions.
- Fixed two compile blockers in ongoing dirty metadata work: a duplicate error conversion and a stale check for a field absent from the live metadata header.
- Recorded the P2.R1 architecture amendment: module rows and reusable query owners must be project-qualified, with an exact persistent external-project allowlist. Module-world moves to V2 and query-image moves to V3; P3 remains gated.

## P2.T12 source-mode evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke`: PASS, 3/3.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime`: PASS; `slotmap-basic` and `dense-slotmap-basic` both reported `ok`.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package::parse_project_compiles_from_declared_entry`: PASS, one matching test selected.

## Other focused evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta`: PASS, 30 tests.
- `cargo test -p phalcom-compiler-artifact`: PASS, 86 tests.
- `cargo test -p phalcom-semantic --lib artifact::query_policy`: PASS, 2 tests.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic-artifact`: PASS, 3 tests.
- The current semantic-artifact allocation test demonstrates N success, N-1 typed failure, and retry N for the import ledger after the payload has already been decoded. This is not a measured allocator total and does not include live-store reconstruction, query/index retention, or publication.

`cargo test --workspace --no-run`, workspace tests, Clippy, and release verification were not run for this closure attempt. Existing P1 baseline failures remain recorded in `ARTF001.C1.P1-baseline-failures.md`; no claim is made that this attempt recertified them.

## Audit Reconciliation

The compatibility check now includes `query_fingerprint_abi`. The query profile now covers all current query kinds and identifies typed row targets, but remaining findings are confirmed against the live tree:

- `QueryImageV2` fingerprints and row references are not yet reconstructed into typed products or installed in `SemanticDb`; dependency fingerprints are not validated against live products.
- `QueryImageV2` still stores owner strings. Project-qualified owners and dependency evidence are required by the P2.R1 amendment.
- Cross-section validation checks static-definition owners by path and selected module query owners, not all nested stable owners. The module inventory itself has no project-qualified module identity or external-project allowlist yet.
- Import stops before current-main semantic/session publication.
- The N/N-1 ledger test does not prove exact allocator-requested bytes for a whole artifact load.

The consultation decision is to retain one current-main `SemanticDb` with an immutable imported generation and its ordinary local overlay, and publish canonical products through `SemanticSnapshot`. No second query engine or DTO-sidecar authority is acceptable.

## Gate Status

| Gate | Status | Evidence / remaining condition |
|---|---|---|
| P2.G1 — artifact crate builds | PASS | Artifact tests passed in this checkout. |
| P2.G2 — canonical durable identity and metadata | PARTIAL | Durable metadata adapters exist; typed live-store reconstruction remains. |
| P2.G3 — wire products correspond to current-main authority | PARTIAL | Wire schemas exist; live current-main import does not. |
| P2.G4 — transactional live import | PARTIAL | Staging fails closed before return; no `SemanticDb`/snapshot publication. |
| P2.G5 — normal compiler remains source-based | PASS | CLI smoke, SlotMap runtime, and focused Parse test passed. |
| P2.G6 — round trips, hostile inputs, exact whole-load allocation | PARTIAL | Current crate and semantic artifact suites passed; whole-load measured allocation and publication atomicity are absent. |
| P2.G7 — P3 can use API without redesign | NOT MET | P3 would still need to design typed semantic/query reconstruction and session publication. |

## P3 Prerequisite

Complete P2 import closure: reconstruct the portable semantic world and query records into current-main stores and typed products; validate all owners/dependencies; publish atomically through the session/DB; verify immutable-base/local-overlay behavior and exact whole-load allocation. Then update the checkpoint and handoff with the actual evidence. Keep Universe production source and default compiler behavior unchanged until later plans.
