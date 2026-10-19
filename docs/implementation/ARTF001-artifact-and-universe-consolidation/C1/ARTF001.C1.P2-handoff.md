---
id: ARTF001.C1.P2
kind: implementation-plan-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
base_revision: 3cf206757638b23884e59edb602c3ff0c3d28150
implementation_revision: ea136a0b5320c0a1c7ac78de5dc798b9d58e3b95
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-19
---

# ARTF001.C1.P2 — Handoff: Artifact Substrate and Import Closure

## Current State

P2 is **`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`**. The P2 audit at `ea136a0b` is correct: the existing code establishes a substantial portable artifact substrate, but it does not yet satisfy P2's live semantic/query import or whole-load accounting contract. Do not start P3 or describe P2 as complete.

The current checkout is branch `feat/artifact-universe-consolidation`, based at Git `HEAD` `ea136a0b5320c0a1c7ac78de5dc798b9d58e3b95`, with P2 closure work in the index and working tree. It includes `phalcom-type-meta` allocation changes and the derive crate, as well as artifact and semantic follow-up edits.

## Architecture decision to preserve

The consultation confirmed the single current-main semantic database direction. `SemanticWorkspaceSession` owns the `SemanticDb`; imported query products must become typed current-main products in one immutable imported generation with the ordinary local overlay. `SemanticSnapshot` remains the published canonical view. Do not add a second query engine or present wire DTOs as live semantic authority.

The P2.R1 architecture review found a second issue: module rows identify paths but not projects. Module identity must be the full `StableModuleRef`; the artifact must carry an exact allowlist of persistent external project identities. See the amendment in `CHECKPOINT.md`. The module-world schema must move to V2 and query-image schema to V3 before those owners can be validated.

## Work completed in this closure attempt

- P2.T12 source-based regressions passed: CLI smoke, SlotMap runtime package, and the focused Parse package test.
- Added `query_fingerprint_abi` to the canonical compatibility identity check.
- Routed metadata decoding through the shared allocation budget and retained typed allocation errors.
- Removed a duplicate `From<MetadataValidationError>` implementation and a stale check for a header field that does not exist in current main; `phalcom-type-meta` now checks and tests successfully.
- Query roots are required to be sorted and unique without allocating a validation-side set.
- Import staging consumes the payload and charges the retained module-world `Arc`; it still moves semantic/query DTOs without reconstructing them into live stores or session state.
- Query image V2 adds typed product-row references and input/product fingerprints. The portable query-kind set now accounts for all 46 current `QueryKind` values through an exhaustive mapping and explicit dispositions.

## Focused Evidence

The following completed checks apply to the current implementation tree:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta`: PASS, 30 tests.
- `cargo test -p phalcom-compiler-artifact`: PASS, 86 tests after the query image V2 changes.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic-artifact`: PASS, 3 tests.
- `cargo test -p phalcom-semantic --lib artifact::query_policy`: PASS, 2 tests.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke`: PASS, 3 tests.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime`: PASS (`slotmap-basic`, `dense-slotmap-basic`).
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package::parse_project_compiles_from_declared_entry`: PASS, one test selected.

After the documentation and API-comment corrections, `cargo fmt --all -- --check`, `git diff --check`, and `git diff --cached --check` all passed.

`cargo test --workspace --no-run`, workspace tests, and Clippy were not run in this closure attempt. The current tests' import N/N-1/retry case checks ledger reservations for import after decode; it does not measure actual allocator requests for the complete bytes-to-publication path.

## P2 acceptance gaps

1. `ImportedCompilerImage` retains `SemanticWorldV3` and `QueryImageV2` wire DTOs. There are no adapters that rebuild them into current `TypeStore`, `StaticValueStore`, `SemanticSnapshot`, or typed `SemanticDb` products.
2. Query image V2 has product-row references and fingerprints, and the semantic layer classifies all 46 current query kinds. It still lacks project-qualified query owners and validated dependency/product fingerprints.
3. Cross-section ownership checks cover static-definition owners by path and selected module-query owners. They do not yet validate every nested stable semantic owner against a project-qualified module inventory and exact external-project allowlist.
4. Allocation evidence is reservation-ledger evidence, not an exact measurement over decode, reconstruction, dependency-index retention, and publication; that full path currently does not exist.
5. The P2 compatibility omission for `query_fingerprint_abi` is fixed in the current tree. `source_digest` remains provenance/freshness information rather than a semantic compatibility key.

These gaps leave P2.G3, P2.G4, P2.G6's whole-load discriminator, and P2.G7 open. Container integrity and the source-mode discriminator are useful focused passes, not P2 acceptance.

## Next Action

Continue P2 closure remediation under the decisions recorded in `CHECKPOINT.md`:

- Add current-main reconstruction into private staging owners for type and static-value stores and the semantic product families required by the compiler image.
- Version module-world V2 rows with full stable module identity and a sorted, unique external-project allowlist. Version query-image V3 owners with project-qualified identities and validate them against product rows.
- Convert supported artifact query records into stable `QueryKey`s and typed `SemanticProduct`s; classify every current `QueryKind` explicitly as persisted, represented elsewhere, or omitted with a reason. Reject unsupported roots and missing products.
- Validate every nested stable owner and query dependency before publication. Accept external owners only for exact allowlisted persistent identities; reject session identities.
- Publish atomically through the ordinary semantic session/database model, retaining an immutable imported generation and normal local overlay. Prove failed import leaves the prior snapshot, DB, stores, and budget unchanged; prove local invalidation/shadowing.
- Measure and test the whole supported load boundary at N, N-1, and retry N, including retained indexes and publication.
- Rerun the focused artifact/semantic tests and the P2.T12 source regressions. Broader workspace certification remains separate unless explicitly required.

P3 remains `PROPOSED / NOT_STARTED / UNVERIFIED` until these P2 exit conditions pass and the P2 checkpoint record is updated.
