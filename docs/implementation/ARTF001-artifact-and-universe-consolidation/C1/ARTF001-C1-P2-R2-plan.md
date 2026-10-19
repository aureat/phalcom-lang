# ARTF001.C1.P2.R2 — Implementation Plan

**Status:** IN_PROGRESS
**Completion:** PARTIAL
**Verification:** FOCUSED_TESTED
**Branch:** `feat/artifact-universe-consolidation`
**Original baseline revision:** `9f0d2f66af0196739dc562b0df19f140576ec861`
**Current code revision at lifecycle update:** `37d2b0d810e78e8ca08b71cc8fcad123eedcceb6`

---

## Current execution status (2026-10-20)

The P2 plan remains open. At the current code revision, project-qualified module identities, exact package resolver receipts, type/static-value relocation, primary data/enum/variant aggregates, exhaustive query persistence classification, and typed owner-to-`QueryKey` conversion are implemented and focused-tested. Query products are not reconstructed or published into `SemanticDb`; session/snapshot publication, complete dependency validation, whole-load allocation accounting, and the complete hostile matrix remain outstanding. See `CHECKPOINT.md` and `ARTF001-C1-P2-R2-handoff.md` for the current durable state and takeover sequence.

Continue on `feat/artifact-universe-consolidation` and push to `origin/feat/artifact-universe-consolidation`. Do not create a new branch. P3 remains proposed until P2's acceptance gates pass.

## Original Baseline Confirmation (as of 9f0d2f66)

| Suite | Command | Result |
|---|---|---|
| `phalcom-type-meta` lib | `cargo test -p phalcom-type-meta` | 0 passed (no lib tests yet) |
| `phalcom-compiler-artifact` all | `cargo test -p phalcom-compiler-artifact` | **86 passed** |
| `phalcom-semantic --test semantic-artifact` | as specified | **3 passed** |
| `phalcom-semantic --lib artifact::query_policy` | as specified | **2 passed** |

---

## Original Codebase Assessment (as of 9f0d2f66)

### What already exists (good)

- `ModuleWorldV1` — full module wire schema with fingerprints, topology, semantic rows
- `SemanticWorldV3` — complete semantic type graph, callable, dispatch wire DTOs
- `QueryImageV2` — query roots with `PortableQueryKindV1`, string `stable_owner_key`, product refs, fingerprints, dependency evidence
- `PortableQueryKindV1` — exhaustive 46-kind enum
- `ArtifactImportTransaction` — transactional staging with allocation budget
- `AllocationBudget` — hierarchical child budgets with commit/rollback
- `SemanticDb` — full transaction, rollback, publish_product_ready, invalidate
- `StableProjectRef` / `StableModuleRef` / all stable identity types in `phalcom-type-meta`
- `CanonicalCompatibilityIdentity` with ABI version fields
- `load_module_world_with_allocation_budget` — reconstructs `CanonicalExternalModuleWorld`

### Critical gaps to fix

1. **T1: Project-qualified identity** — `ModuleWorldV1` has path-only `Vec<String>` (no `StableProjectRef`). `PortableQueryKeyV1.stable_owner_key` is a plain `String`. `load_module_world_inner` hardcodes `ModuleId::universe(path)`. Schema bumps required: `MODULE_WORLD_SCHEMA_V2`, `QUERY_IMAGE_SCHEMA_V3`.

2. **T2: Semantic reconstruction** — `ImportedCompilerImage` retains raw `SemanticWorldV3` / `QueryImageV2` DTOs. No live type/static reconstruction into current-main `TypeStore`/`StaticValueStore`.

3. **T3: Query product reconstruction** — No typed `QueryKey` conversion from wire `PortableQueryKeyV1`. No `SemanticProduct` reconstruction from semantic world rows. No reuse eligibility classification per PERF003.

4. **T4: Session publication** — No `SemanticWorkspaceSession::from_imported_canonical()`. Current session construction uses source-backed `canonical_universe_semantic_baseline()`.

5. **T5: Whole-load allocation** — `ModuleWorldV1::validate()` uses unmetered `BTreeSet`. Allocation propagation gaps at several stages.

6. **T6: Hostile artifact tests** — Some cases missing (forged project owner, session identity on wire, etc.).

---

## Implementation Order

### Level 2 Amendment — Workspace-owned project identity binding

This amendment incorporates the P2.R1 ownership contract and the architecture consultation received for R2. T1's project-qualified wire identities and `external_projects` allowlist do not by themselves produce live `ModuleId`s: `load_module_world_inner` currently constructs Universe-only IDs, and the artifact import API has no workspace project-graph input. The existing metadata resolver is not a valid fallback: it matches `SourceArtifact` by URI without checking its fingerprint and allocates a new synthetic identity for `Session` references.

**Fixed owner and lifetime:** The existing workspace module identity catalog owns the binding from exact `StableProjectRef` values to already allocated `ProjectIdentity` values. Its owner is `WorkspaceModuleSession` and its `ProjectUniverse`; the importer receives that catalog from the caller and derives its stable-project and stable-module lookup view from the same graph. Do not allocate project IDs during artifact import, create an artifact-local project identity authority, or use `WorkspaceId` as a substitute for the graph. The validated imported generation and resulting `SemanticWorkspaceSession` must retain the supplied module session/project graph, including its existing `ResolvedProjectId` values, for the lifetime of imported authority. Local overlays use that same graph and IDs. A graph replacement must update or invalidate the imported binding atomically with the graph; it must not silently rebind existing imported owners.

**Exact identity and authority checks:** Compare the complete `StableProjectRef` variant and all of its fields. For `SourceArtifact`, both `logical_uri` and `source_fingerprint` must match the workspace catalog; URI may index a lookup but cannot authorize it. For `Package`, require a caller-supplied trusted package-resolver capability to provide a validated receipt binding the complete `(package, version, artifact_fingerprint)` tuple to the full validated 256-bit `ArtifactDigest` and an already-existing persistent `ResolvedProjectId`. Recompute and validate the artifact digest, check the header package/version, verify the full package owner tuple against the receipt, confirm that the target still exists and is persistent, then register the exact binding in the supplied `ProjectUniverse` before publication. The fingerprint is not derived from the artifact digest, and no package identity receipt or fingerprint is self-authenticated by being embedded in the artifact. For `Builtin`, match the complete namespace/version against a catalog builtin (currently the Universe builtin). Reject `Session` unconditionally, as well as unknown, ambiguous, stale, conflicting, or unbound identities. An external project must also be present by exact equality in `ModuleWorldV2.external_projects`; allowlisting alone does not establish that the project or any owner exists. Resolve each `StableModuleRef` through that project binding, then form the `ModuleId` from the resolved `ProjectIdentity` and the row's exact module path. Validate uniqueness and all nested semantic owners, query roots, dependency owners, product owners, and fingerprints against the reconstructed module world and its resolved project authorities. Complete these checks in private staging before a generation can be published.

This work only binds imported stable owners to existing workspace graph identities. It does not add source compilation for package projects or change ordinary module resolution.

**Adopted decision — exact `Package` bindings and resolver receipts:** The user selected exact package bindings and a package-resolver-provided authenticated receipt as their authority. `ProjectUniverse`, owned by the supplied `WorkspaceModuleSession`, registers each complete `StableProjectRef::Package` against one already-existing persistent `ResolvedProjectId` only after the caller-supplied trusted package-resolver capability provides a validated receipt. The receipt binds the exact `(package, version, artifact_fingerprint)` tuple, the full 256-bit `ArtifactDigest`, and that existing project ID. The importer recomputes and validates the artifact digest, checks the header package/version, verifies the full stable package owner tuple against the receipt, confirms the target still exists and is persistent, and registers the binding in the supplied `ProjectUniverse` before publication. Retain and compare the complete receipt binding, including its digest and project ID; lookup never authorizes by package/version alone and import allocates no project IDs. Reject receipt mismatch, digest mismatch, conflict, ambiguous mapping, stale or absent target, and non-persistent target. If the capability or receipt is absent, fail closed for package owners. No conversion that drops `artifact_fingerprint` may authorize import. Do not derive `artifact_fingerprint` from `ArtifactDigest`, and do not treat a value embedded in the artifact as authenticated merely because container integrity validation covers its bytes. Represent the trusted-resolver boundary as an opaque validated receipt supplied by the caller's trusted resolver capability. The importer accepts receipts only through that capability; P2 does not invent or implement a signature scheme, trust root, or in-artifact receipt format. Trust provisioning and receipt authentication remain owned by the caller's configured resolver. Keep registration and lookup in or derived from `ProjectUniverse`, not in a parallel artifact-owned registry.

**Adopted decision — portable artifact topology fingerprint:** The live `ModuleTopology` fingerprint is workspace-scoped and includes graph-local project IDs and import roots. `ModuleWorldV2.topology_fingerprint` must instead fingerprint only topology evidence represented by that artifact: stable module identities, module kinds, parent/child structure, exposed children, and represented module links. The loader validates this portable fingerprint against the wire rows, then constructs the live `ModuleTopology` using the supplied `ProjectUniverse` and computes its separate workspace-scoped fingerprint. Importing the same artifact into different valid workspace graphs may therefore produce different live topology fingerprints. Receipt targets and unrelated projects in the receiving graph must not change the portable artifact fingerprint. Module-world ABI V2 defines this portable meaning; no additional ABI bump is required before V2 is released. If any V2 artifact is treated as already published, bump `module_semantic_abi` before accepting it. Add tests that the same artifact validates with an empty graph and with an unrelated persistent receipt target, that its live fingerprints can differ, and that changing represented module identity, topology edges, or exposures fails portable validation.

### T1: Project-qualified module and query schemas

**Files to change:**
- `phalcom-compiler-artifact/src/module_world.rs` — add `ModuleWorldV2`, `ModuleV2` with `StableModuleRef`
- `phalcom-compiler-artifact/src/query.rs` — add `QueryImageV3`, typed `PortableQueryOwnerKeyV1` enum
- `phalcom-compiler-artifact/src/identity.rs` — audit/fix `TeloscPackageIdentity::from(&StableProjectRef)` lossy conversion
- `phalcom-compiler-artifact/src/header.rs` — bump ABI version constants
- `phalcom-modules/src/compiler_artifact.rs` — update `project_module_world` and `load_module_world_inner` for V2
- `phalcom-semantic/src/artifact/mod.rs` — update imports for V2/V3
- New test: `phalcom-compiler-artifact/tests/project_qualified_identity_test.rs`

**Key decisions:**
- `ModuleWorldV2` adds `external_projects: Vec<StableProjectRef>` and each `ModuleV2` carries `identity: StableModuleRef`
- `PortableQueryOwnerKeyV1` enum replaces string key with typed variants matching all 46 query kinds
- Schema constants: `MODULE_WORLD_SCHEMA_V2 = 2`, `QUERY_IMAGE_SCHEMA_V3 = 3`
- Old schemas rejected with typed `IncompatibleAbi` errors
- `StableProjectRef::Session` variants must be rejected at import

### T2: Semantic reconstruction

**Files to change/add:**
- `phalcom-semantic/src/artifact/identity.rs` (new) — owner catalog validation
- `phalcom-semantic/src/artifact/relocate.rs` (new) — stable-to-local relocation maps
- `phalcom-semantic/src/artifact/semantic_import.rs` (new) — type/static reconstruction
- `phalcom-semantic/src/metadata/import.rs` — resolve stable owners through the supplied workspace catalog; remove URI-only matching and synthetic `Session` fallback from the artifact import path
- `phalcom-modules/src/project.rs` — register and resolve complete persistent `StableProjectRef::Package` bindings in `ProjectUniverse`, rejecting conflicts and stale targets
- `phalcom-modules/src/compiler_artifact.rs` — pass the caller's workspace project catalog into module-world reconstruction
- `phalcom-compiler-artifact/src/semantic/type_store.rs` — expose reconstruction helpers
- New test: `phalcom-semantic/tests/artifact_reconstruction_test.rs`

**Strategy:** Reconstruct module and semantic owners using the exact workspace catalog passed into import, then rebuild semantic products into current-main stores. Do not assume Universe-only support: preserve R1's exact allowlisted persistent external-owner contract. `RepresentedByModuleWorld` families are reconstructed from `CanonicalExternalModuleWorld`; `RepresentedBySemanticWorld` families are reconstructed from semantic rows. All resulting objects use existing current-main IDs and types.

> [!IMPORTANT]
> T2 is the most complex task. `SemanticWorldV3` contains wire representations of complex live objects (callable signatures, data declarations, and related products), so reconstruct those from the wire rows into the existing semantic types. Stubs are not acceptable. Module and semantic rows must resolve through the same validated workspace identity catalog, including all nested project-qualified owners.

**Additional T2 acceptance conditions:**

- Import receives a caller-supplied, already populated `WorkspaceModuleSession` or its exact `ProjectUniverse` graph. `WorkspaceId` alone and a newly created empty module session do not provide project identity authority.
- Every stable project binding is one-to-one with an existing `ProjectIdentity`; each module row forms a unique `ModuleId` from that identity and its exact path. Same-path modules in different projects remain distinct.
- A `SourceArtifact` with a matching URI but a different `source_fingerprint` is rejected. A listed external project with no catalog binding or no corresponding module/semantic owner authority is rejected. `Session` is rejected without allocating a synthetic identity.
- Register a `Package` binding only after a receipt from the caller-supplied trusted resolver capability binds the complete package/version/fingerprint tuple to the full validated 256-bit artifact digest and an existing persistent `ResolvedProjectId`. Import recomputes and validates the digest, checks header package/version, verifies the full owner tuple, confirms the target still exists and is persistent, then registers in the supplied `ProjectUniverse` before publication. Missing capability or receipt fails closed. Prove receipt tuple mismatch (including matching package/version with a different fingerprint), digest mismatch, stale or absent target, non-persistent target, conflicting registration, and exact tuple acceptance. Do not derive `artifact_fingerprint` from the digest, accept a raw unauthenticated receipt, allocate IDs, or treat in-artifact data as its own authentication. No conversion that drops `artifact_fingerprint` may authorize import.
- Owner, external allowlist, product, query, dependency, and fingerprint checks all complete before returning publishable reconstructed authority.
- Add focused cases for exact catalog binding, same-URI fingerprint mismatch, unbound-but-allowlisted project, cross-project same-path modules, and unchanged owner identity after relocation, alongside the existing reconstruction matrix.

### T3: Query product reconstruction and persistence policy

**Files to change/add:**
- `phalcom-semantic/src/artifact/query_import.rs` (new) — typed key conversion, reuse eligibility
- `phalcom-semantic/src/artifact/query_policy.rs` — extend with `QueryReuseEligibility`
- `phalcom-semantic/src/db/key.rs` — add `QueryKey::from_portable_owner`
- New test: `phalcom-semantic/tests/artifact_query_import_test.rs`

### T4: Session construction and atomic publication

**Files to change/add:**
- `phalcom-semantic/src/artifact/session_import.rs` (new) — `ValidatedImportedCanonicalGeneration`
- `phalcom-semantic/src/session.rs` — add `from_imported_canonical()` constructor
- `phalcom-semantic/src/artifact/mod.rs` — wire through publication path
- New test: `phalcom-semantic/tests/artifact_session_import_test.rs`

**Additional T4 acceptance conditions:**

- Fresh artifact-backed construction receives the caller's populated `WorkspaceModuleSession`/project graph as an explicit input to both T2 import and T4 session construction, then retains that same graph, package bindings, and project IDs in `SemanticWorkspaceSession`. The constructor must not create an empty module session and cannot infer the graph from `WorkspaceId`.
- Overlays and invalidations operate through the retained module session. For an overlay that leaves project topology unchanged, prove the same `ProjectIdentity`, `ResolvedProjectId`, `ModuleId`, and stable-owner binding remain in force; imported canonical products remain immutable and unaffected products remain reusable.
- If the project graph must change, stage the replacement graph and imported-owner rebinding together, validate the full candidate, then publish atomically. A failed import/publication leaves both prior semantic state and prior project graph/bindings unchanged.
- The catalog and all external authority checks are complete before the generation/session publication boundary. Artifact-backed construction still performs zero canonical Universe parsing, linking, or semantic reanalysis.
- Add tests for caller-supplied project graph retention across fresh construction and local overlays, and for a missing/unbound graph failing before publication.

### T5: Whole-load allocation accounting

**Files to change:**
- `phalcom-compiler-artifact/src/module_world.rs` — replace unmetered `BTreeSet` in `validate()`
- `phalcom-type-meta/src/allocation.rs` — verify propagation paths
- Extend existing allocation tests
- New test: `phalcom-semantic/tests/artifact_allocation_test.rs`
- New test: `phalcom-type-meta/tests/metadata_budgeted_equivalence_test.rs`

### T6: Hostile compatibility tests

- Extend `phalcom-compiler-artifact/tests/hostile_matrix_test.rs`
- New test: `phalcom-compiler-artifact/tests/query_owner_integrity_test.rs`

---

## Scope Assessment and P2 Boundary

> [!WARNING]
> **Escalation risk on T2**: Full semantic reconstruction of all 46 query families from wire DTOs requires deep integration with current-main `TypeStore`, `TypeId`, `KindId`, etc. The existing `SemanticWorldV3` uses `ArtifactKindId`, `ArtifactTypeId` as opaque wire indices. Reconstructing live objects requires re-mapping these to current-main IDs.
>
> Under the project-identity amendment, the practical strategy is:
> - `RepresentedByModuleWorld` families: reconstruct from `CanonicalExternalModuleWorld` already loaded by T1
> - `RepresentedBySemanticWorld` families: reconstruct via semantic world wire rows → live products using `TypeStore` reconstruction from the type graph
> - Stable owners: resolve through the exact caller-supplied workspace project catalog and the R1 allowlist; do not collapse the runtime profile to Universe-only.
>
> The key constraint (INV-01): no second type store. We reconstruct into a fresh current-main `TypeStore` from the portable type graph.

---

## Checkpoint Record

- [ ] T1: Gate D identity tests pass
- [ ] T2: Gate D reconstruction tests pass
- [ ] T3: Gate D query import tests pass
- [ ] T4: Gate E session tests pass
- [ ] T5: Gate E allocation tests pass
- [ ] T6: Hostile matrix complete
- [ ] Gate A: fmt + compile clean
- [ ] Gate B: phalcom-type-meta + phalcom-compiler-artifact
- [ ] Gate C: semantic-artifact + query_policy
- [ ] Gate F: CLI smoke + SlotMap + Parse
- [ ] Gate G: workspace build + clippy
- [ ] Gate H: committed + pushed
