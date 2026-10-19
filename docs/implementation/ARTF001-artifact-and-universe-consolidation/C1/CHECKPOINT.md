---
id: ARTF001.C1
program: ARTF001
kind: checkpoint
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# ARTF001.C1 — Artifact and Universe Consolidation Checkpoint

## Objective

Establish the unified mainline integration branch, freeze starting baselines, resolve durability and persistence authority between current `main` and certified PERF003 P7, and execute the step-by-step artifact compiler and Universe cutover program.

## Plan Ledger

| Plan | Status | Completion | Verification | Objective |
|---|---|---|---|---|
| `ARTF001.C1.P1` | COMPLETE | IMPLEMENTED | BASELINED | Repository setup, current-main baseline, P7 and main persistence census, authority reconciliation matrix, and P2 handoff. |
| `ARTF001.C1.P2` | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | Artifact transport, project-qualified identity, exact resolver-receipt package binding, type/static relocation, primary structural aggregates, and query-owner/persistence-policy slices are implemented and focused-tested. Full semantic product reconstruction, typed query publication, session construction, complete dependency validation, whole-load allocation accounting, and the hostile matrix remain open. |
| `ARTF001.C1.P3` | PROPOSED | NOT_STARTED | UNVERIFIED | Do not start until P2 publishes reconstructed current-main semantic/query authority through the ordinary session. |
| `ARTF001.C1.P4` | PROPOSED | NOT_STARTED | UNVERIFIED | Core canonical artifact compiler and source-free compilation path parity. |
| `ARTF001.C1.P5` | PROPOSED | NOT_STARTED | UNVERIFIED | Universe source cutover from `phalcom-core/core/universe` to `packages/universe` retaining `ProjectIdentity::Universe`. |
| `ARTF001.C1.P6` | PROPOSED | NOT_STARTED | UNVERIFIED | Universal artifact resolution across CLI, REPL, VM bootstrap, and tests with persistent cache ergonomics. |

## Current continuation state (2026-10-20)

- Code revision: `37d2b0d810e78e8ca08b71cc8fcad123eedcceb6` on `feat/artifact-universe-consolidation`, matching `origin/feat/artifact-universe-consolidation`. The implementation files were clean and synchronized before this lifecycle-record update.
- Current status: P2 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. The latest source slice routes reconstructed type, static-value, structural-aggregate, and typed query owners through one `StableOwnerCatalog` backed by the caller's `ProjectUniverse`.
- Package authority: Exact `StableProjectRef::Package` bindings use the caller's trusted resolver receipt. The receipt binds the full package/version/fingerprint tuple, validated artifact digest, and an existing persistent project ID. Import allocates no project IDs.
- Next action: Continue T3 by rebuilding supported query products from semantic rows, checking current input/product fingerprints and dependency evidence, and staging correctly typed products in the one current-main `SemanticDb`. Then assess T4 session publication as its own cross-layer gate.
- Branch rule: Continue and commit directly on `feat/artifact-universe-consolidation`, pushing to `origin/feat/artifact-universe-consolidation`. Do not create a branch. P3 remains proposed until P2 meets its acceptance contract.
- Record-update scope: This handoff refresh is documentation-only; it did not rerun implementation tests. Four untracked toolchain specification files (`README.md`, `package-model-and-manifest.md`, `registry.md`, and `local-registry.md`) were included in the delivery as requested and are outside P2 implementation scope.

## Starting Baseline & Provenance

| Parameter | Value |
|---|---|
| Integration branch | `feat/artifact-universe-consolidation` |
| Worktree path | `.worktrees/artifact` |
| Base commit (`origin/main`) | `39334961e61cc3b9e251d03bcd348bceb50abbc6` |
| Reference P7 branch | `origin/perf/universe-characterization` |
| Reference P7 commit | `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94` |
| P7 measured implementation commit | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` |
| P7 test revision commit | `c1524361b9606dc537671997fe5069f2aa66035b` |
| Common merge base | `e2ae4a0fd360e4075ce8ad229a3e39e1531fb771` |
| Divergence | `origin/main` ahead 273 commits; `perf/universe-characterization` ahead 111 commits |
| Toolchain | `rustc 1.99.0-nightly (af3d95584 2026-07-09)`, `cargo 1.99.0-nightly (59800466c 2026-07-07)` |
| Host platform | Darwin 25.5.0 arm64 (Apple Silicon) |

## Baseline verification summary

- `cargo fmt --all -- --check`: PASS (0 diffs).
- `cargo check --workspace --all-targets`: PASS (0 errors, 0 warnings).
- `cargo test -p phalcom-modules`: 7 passed, 2 failed (`bcat_02`, `bcat_08` in `builtin_catalog`). Classified as `BASELINE_CURRENT_MAIN` (stale module exposure fixtures).
- `cargo test -p phalcom-semantic --lib`: 191 passed, 1 failed (`tracking_resolver_demands_the_exact_unready_type_member_owner`). Classified as `BASELINE_CURRENT_MAIN`.
- `cargo test -p phalcom-core --lib`: PASS (211/211 passed in 144.39s).
- Broader workspace test: skipped per explicit user instruction ("Ignore test baseline.").
- All observed failures classified and recorded in `ARTF001.C1.P1-baseline-failures.md`. None block artifact consolidation.

## Established invariants and pending acceptance criteria

1. **Current Main Has Semantic Authority (P1-INV-03):** Language rules, type relations, AST, checker rules, compiler lowering, and VM execution on current `main` are authoritative. P7 artifact code adapts to current `main`, not vice versa.
2. **Singular Semantic Ontology (P1-INV-04):** There is exactly one live semantic type model (`TypeStore`, `StaticValueStore`) and one durable metadata model (`phalcom-type-meta`). Artifact DTOs in `phalcom-compiler-artifact` serve strictly as wire containers, not alternative semantic representations.
3. **Allocation Ledger Exists:** The P7 allocation transaction model (`AllocationBudget`, `ChildAllocationBudget`, `AllocationError`) is present. The current N/N-1 test measures ledger reservations for import after decode; it does not establish exact allocator-requested bytes for decode-through-publication.
4. **Universe Production Authority Unchanged During P1 (P1-INV-05):** `phalcom-core/core/universe` remains production authority during P1-P4. `packages/universe` migration is planned for P5.
5. **Universe Builtin Identity Preserved (P1-INV-06):** In future cutover (P5), `packages/universe` retains `ProjectIdentity::Universe`; it is never loaded as an ordinary external project.
6. **No Production Code in P1 (P1-INV-01):** P1 is strictly architectural reconnaissance, baselining, authority assignment, and takeover planning.
7. **Current-Main Semantic Products Authoritative (P2-INV-01):** Artifact encoding/decoding adapts to current-main data structures; current semantic representations are never regressed.
8. **No Duplicate Semantic Ontology (P2-INV-02):** Artifact wire DTOs are encoding containers only. Durable authority resides solely in `phalcom-type-meta` and live semantic authority in `phalcom-semantic`.
9. **No Store-Local Identity on Wire (P2-INV-03):** Internal indices, arena references, and raw addresses are forbidden on the wire; references use stable symbolic identity (`StableProjectRef`, `StableModuleRef`, `StableDeclarationRef`).
10. **Validation Precedes Semantic Publication (P2-INV-04, PARTIAL):** Container, section, project, type, static-value, primary structural-owner, and typed query-owner validation occurs before returning staged import data. Publication into `SemanticSnapshot` or `SemanticDb` does not exist yet; remaining semantic families, query products, fingerprints, and dependency closure still require validation.
11. **Exact Allocation Accounting (P2-INV-05, PENDING):** Whole-load decode, reconstruction, retained indexes, and publication still need measured N/N-1/retry evidence. Existing import-only ledger evidence is narrower.
12. **No Silent Compatibility Fallback (P2-INV-06):** Incompatible schemas, foreign native fingerprints, or malformed sections fail closed with typed errors.
13. **Unchanged Default Compiler Behavior (P2-INV-07):** Normal `ProgramCompiler` execution continues through the canonical source path. Tests opt into artifact loading explicitly.
14. **Universe Production Source Untouched (P2-INV-08):** `phalcom-core/core/universe` source authority remains untouched during P2.
15. **No P8 Executable Runtime Payload (P2-INV-09):** P2 does not serialize bytecode chunks, closures, or VM heap states.
16. **Project-Qualified Artifact Ownership (P2.R1-AMEND-01, PARTIAL):** Module rows and imported type/static/primary structural/query owners use exact `(StableProjectRef, module path)` authority. External owners require exact allowlisting and a persistent binding in the caller's project graph; package bindings additionally require the trusted resolver receipt. Remaining semantic wire families and product/dependency cross-checks are part of P2 closure.

## P2 verification evidence summary (cumulative; latest code slice at `37d2b0d8`)

- `cargo fmt --all -- --check`: PASS after the P2 record corrections.
- `git diff --check` and `git diff --cached --check`: PASS after the P2 record corrections.
- `cargo test --workspace --no-run`: NOT RUN for this P2 closure attempt.
- `cargo test -p phalcom-type-meta`: PASS (30 tests: 2 allocation budgeting, 23 schema compatibility, 5 static integer).
- `cargo test -p phalcom-compiler-artifact`: PASS (86 tests after the query image V2 changes).
- `cargo test -p phalcom-semantic --test semantic-artifact`: PASS (3/3 tests: round-trip export/import, hostile corrupt metadata rejection, exact N / N-1 / retry allocation accounting).
- `cargo test -p phalcom-semantic --lib artifact::query_policy`: PASS (2/2 tests: exhaustive current query-kind mapping and static-family dispositions).
- `cargo test -p phalcom-core --test cli-smoke`: PASS (3/3 tests in 131.31s).
- `cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime`: PASS (`ok: slotmap-basic`, `ok: dense-slotmap-basic`, `slotmap runtime tests complete`).
- `cargo test -p phalcom-core --test core modules_parse_package::parse_project_compiles_from_declared_entry`: PASS (one selected test; 136.15s).
- R2 follow-up: `cargo test -p phalcom-compiler-artifact --test module_world_allocation_test`: PASS (1 test; legacy module-world validation scratch is included in the allocation ledger, and N-1 failure leaves the parent budget unchanged). This is focused validation evidence, not whole-load publication accounting.
- R2 follow-up: `cargo test -p phalcom-semantic --lib metadata::import::stable_module_identity_tests`: PASS (2 tests; exact builtin version, SourceArtifact URI plus source fingerprint, stale fingerprint rejection, and Session rejection).
- R2 continuation after pushed snapshot `c4c8ca04`: `cargo test -p phalcom-compiler-artifact --test project_qualified_identity_test` PASS (11/11, including portable topology mutation rejection); `cargo test -p phalcom-compiler-artifact --test round_trip_test` PASS (2/2); `cargo test -p phalcom-modules --test package_authority` PASS (6/6); `cargo test -p phalcom-semantic --test semantic-artifact` PASS (4/4, including importing identical package bytes into two valid workspace graphs and observing distinct live topology fingerprints); focused library checks for `phalcom-compiler-artifact` and `phalcom-modules` PASS.
- R2 owner-resolution correction: `cargo test -p phalcom-semantic --test semantic stable_declaration_import_requires_the_exact_canonical_path_shape` PASS (1 selected test). Stable declaration import rejects multi-segment paths instead of silently collapsing distinct wire owners onto their last path component.
- R2 semantic-store relocation: `cargo check -p phalcom-semantic --lib` PASS; `cargo test -p phalcom-semantic --lib artifact::type_import::tests` PASS (4/4: forward type/kind references through a nested lambda, type and lambda cycle rejection, malformed tuple rejection); `cargo test -p phalcom-semantic --lib artifact::static_import::tests` PASS (2/2: scalar/tuple/record/pack reconstruction and cycle rejection); `cargo test -p phalcom-semantic --test semantic-artifact` PASS (4/4, including N/N-1/retry import accounting and static-store publication). `ImportedCompilerImage` now retains reconstructed current-main `TypeStore` and `StaticValueStore` products. Semantic aggregate/proof, typed-query, and session reconstruction remain open; allocation accounting for internal store allocations remains part of T5.
- R2 structural-product slice: `cargo check -p phalcom-semantic --lib` PASS; `cargo test -p phalcom-semantic --test semantic-artifact structural_data_contract_reconstructs_as_native_semantic_product` PASS (one selected test). The importer now reconstructs native `DataInfo`, `EnumInfo`, and `VariantInfo` tables from structural contracts, including relocated type IDs, declarations, generic signatures, constructor/field shapes, case environments, and static algebraic identity. This does not cover declaration shells, hierarchy/aliases, enum requirements, callable/conformance/proof/operational products, or the required nontrivial round-trip matrix.
- R2 package-authority regression recheck: `cargo test -p phalcom-modules --test package_authority` PASS (6/6) after the module-world identity validation changes.
- R2 query-policy authority port: the architectural consultation verified that the handoff's source is the certified PERF003 P7 `phalcom-semantic/src/canonical_image.rs` at `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94`, which defines `CanonicalQueryPersistence`, the family classification, and freeze-time checks. The P7 classification has 45 query kinds; current main adds `AttributeArgument`, conservatively classified `Unsupported` until separately proven. `cargo test -p phalcom-semantic --lib artifact::query_policy` PASS (4/4: exhaustive current 46-kind mapping, open-world classification, static family location, and unique portable mapping). This is the policy port only; typed key/product import, owner/fingerprint/dependency validation, and query publication remain open.
- R2 typed query-owner slice: `cargo test -p phalcom-semantic --test artifact_query_import_test` PASS (4/4); `cargo test -p phalcom-semantic --lib artifact::query_policy` PASS (5/5); `cargo test -p phalcom-semantic --test semantic-artifact` PASS (5/5) after the stricter staging validation. `StableOwnerCatalog` requires an internal owner to match an exact module row, and an external owner to use a project in the artifact allowlist with an exact persistent binding in the caller's project graph. `QueryKey::from_portable_owner` relocates typed module, declaration, callable, field, type-member, and other exactly representable owners through that authority; mismatched owners and wire forms that omit live identity (including generic-conformance propositions, exact enum-case dispatch targets, and associated static-domain case identity) fail closed. Reusable-root admission requires both PERF003 structural eligibility and a product location represented as a root; this excludes source-dependent `ParsedModule` and dependency-evidence-only `HierarchyEdge`.
- R2 nested semantic-owner authority: `TypeGraphImporter`, `StaticValueImporter`, and structural aggregate reconstruction receive the same `StableOwnerCatalog` as typed query-owner validation. Stable declaration, callable, implementation, variant, static-data, and static-variant owners pass exact module-row/external-project checks before live identities are formed. Focused verification: type importer 4/4, static importer 2/2, query-owner integration 4/4, and semantic-artifact integration 5/5. The structural fixture includes the referenced `core` module in its module world. Query product reconstruction, current fingerprints, full dependency closure/freshness, semantic product reconstruction beyond primary data/enum/variant tables, and publication remain open.

### P2 audit findings still open

- `ImportedCompilerImage` now reconstructs portable type and static-value graphs into current-main `TypeStore` and `StaticValueStore` products and reconstructs primary data/enum/variant structural tables, but still retains `SemanticWorldV3` and `QueryImageV3` wire DTOs. Declaration shells, hierarchy/aliases, enum requirements, callable/conformance/proof/operational products, `SemanticSnapshot`, and typed `SemanticDb` products remain incomplete.
- `QueryImageV3` has typed project-qualified owners and product-row references/fingerprints. The P7 persistence contract and root-location gate are now ported, and exact owner conversion is implemented for representable identities. Live query-product reconstruction, product/input fingerprint validation, dependency closure/freshness, and typed `SemanticDb` publication remain open.
- The exact owner catalog now covers the reconstructed type/static graphs, primary structural aggregates, query roots, and dependency evidence. Semantic-world families not yet reconstructed through that catalog, product-row ownership checks, current fingerprints, and dependency closure/freshness remain open.
- The N/N-1/retry test proves the current import-ledger reservation boundary after a separately budgeted decode. It does not prove exact allocator-requested bytes over decode, live semantic reconstruction, query/index construction, and publication.
- Compatibility validation now includes `query_fingerprint_abi`, closing that specific omission, but the other P2 acceptance gaps remain.

## P2 consultation and closure decision

- **Trigger:** The audit showed that P2.G7 was recorded as satisfied even though `import_compiler_image` returned wire DTOs and had no path into `SemanticWorkspaceSession`, `SemanticSnapshot`, or typed `SemanticDb` products.
- **Decision:** Keep one current-main `SemanticDb` with an immutable imported generation and the ordinary local overlay; publish canonical products through `SemanticSnapshot`. Reconstruct into current stores and typed products before publication. Do not add a second query engine or treat wire DTOs as semantic authority.
- **Plan impact:** The architecture is retained. P2 closure scope expands to include the missing reconstruction/publication adapter, complete query profile and cross-section validation, and whole-load allocation evidence. P3 remains proposed.

### P2.R1 architecture amendment — project-qualified owners

- **Trigger:** The live `ModuleV1` wire row records only a module path, while semantic and query references carry or need project identity. A path match cannot distinguish equal paths in different projects.
- **Decision (2026-10-19):** Use each module row's full `StableModuleRef` as its identity. Add an explicit sorted, unique allowlist of persistent external `StableProjectRef` identities. Internal owners must resolve to an exact module row; external owners are accepted only when their exact project identity is allowlisted. Reject session identities, wildcards, unlisted projects, absent internal modules, and ambiguous matches. Query owners and dependency evidence use project-qualified identities and must agree with referenced products.
- **Versioning:** Bump the module-world section schema for stable module identities and the external-project allowlist. Bump the query-image schema for project-qualified query owners. Keep the outer container schema unchanged. Bump `module_semantic_abi` and `query_fingerprint_abi`; leave `stable_identity_schema` unchanged while reusing the existing stable reference encodings. Bump `semantic_artifact_abi` if the semantic-owner validation contract receives a separate version.
- **Plan impact:** This is a Level 2 material amendment to P2. Update the active P2 handoff and P3 consumers. Focused gates must cover same-path modules in different projects, all semantic owner families, query and dependency owners, exact allowlist acceptance/rejection, and rejection of old module/query schemas. Keep P3 proposed until P2 closes.

### P2.R2 consultation and Level 2 amendment

- **Trigger:** `ModuleWorldV2` and semantic/query rows carry persistent project-qualified identities, but the live module-world loader still constructs Universe-only `ModuleId`s. Artifact staging receives no workspace project graph. The existing metadata resolver is unsuitable because it matches source projects by URI without checking the fingerprint and maps session identities to newly allocated synthetic projects.
- **Decision:** The binding from exact `StableProjectRef` values to existing `ProjectIdentity` values belongs to the workspace-owned `WorkspaceModuleSession` / `ProjectUniverse`. Artifact import must receive the populated graph, validate exact identity fields and authority before publication, and retain that same graph for local overlays. It must not invent project IDs or create a parallel artifact-owned identity registry.
- **Plan impact:** The principal-architect amendment in `ARTF001-C1-P2-R2-plan.md` updates T2/T4 ownership and tests. The user selected exact `StableProjectRef::Package` bindings, including package name, version, and artifact fingerprint, in the existing workspace-owned catalog. This required an implementation-backed catalog entry and exact matching; that receipt path and its focused acceptance/rejection cases are now implemented. Full semantic/query owner-product validation and session retention remain P2 requirements.
- **Implementation status:** The caller-supplied trusted resolver receipt path is implemented through `stage_artifact_project_authorities` and `ProjectUniverse`. `phalcom-modules/tests/package_authority.rs` covers exact identity/digest/target binding, conflicts, non-persistent targets, missing receipts, wrong digests, and header tuple matching; `semantic_artifact_test.rs` covers successful import while retaining the supplied graph. This does not complete semantic/query publication.
- **Verification required:** Identity and package receipt acceptance/rejection tests now cover the exact catalog path. T4 must still prove that artifact-backed session construction and local overlays retain the same project IDs and stable-owner mappings. Remaining cross-section product/dependency owner checks are P2 closure work.

### Package artifact authority source

- **Trigger:** Exact `StableProjectRef::Package` registration requires a trusted source for `artifact_fingerprint`. The current container header carries package name and version only. The artifact digest covers the package reference encoded inside the artifact, so deriving the reference fingerprint from that digest would be self-referential. Project manifests also provide no artifact fingerprint.
- **Decision (2026-10-19):** The user selected a receipt from the caller's trusted package resolver. It binds the complete package/version/fingerprint reference, full validated `ArtifactDigest`, and an existing persistent `ResolvedProjectId`. Import validates the current artifact digest and header package/version, checks every receipt and target, then registers the exact binding in the supplied `ProjectUniverse`. Trust provisioning stays with the configured resolver; P2 adds no signature scheme or trust root. Missing authority fails closed.
- **Plan impact:** The principal architect updated the Level 2 amendment and T2 acceptance conditions in `ARTF001-C1-P2-R2-plan.md`. Do not derive a fingerprint from the container digest or trust an in-artifact receipt. Package-owner acceptance requires receipt, digest, exact-tuple, target, and conflict coverage.

### P2.R2 topology-fingerprint Level 2 amendment

- **Trigger:** Importing a package artifact against the required persistent resolver target failed with `module topology fingerprint mismatch`. The artifact module-world fingerprint was created with an empty graph; live topology validation hashes every project in the receiving `ProjectUniverse`, including graph-local IDs and import roots.
- **Decision (2026-10-19):** Give `ModuleWorldV2.topology_fingerprint` a portable, artifact-scoped meaning over the stable module identities, module kinds, parent/child structure, exposed children, and represented module links. Validate it against the wire rows. Then build the live workspace-scoped `ModuleTopology` using the caller's project graph and calculate its independent fingerprint. Receipt targets and unrelated receiving projects do not alter portable artifact validity; different receiving graphs may yield different live fingerprints.
- **Plan impact:** Principal-architect consultation required a Level 2 amendment to `ARTF001-C1-P2-R2-plan.md`. Module-world ABI V2 owns this meaning before release; if V2 has been published, bump `module_semantic_abi` before accepting it. P2 must add empty-graph/receipt-target equivalence, live-fingerprint difference, and represented-topology mutation rejection tests.
- **Verification required:** Exact same artifact loads under both valid graphs; returned module-world fingerprint reflects the receiving graph; changed represented identities, edges, or exposures fail wire-topology verification.
- **Status:** Implemented, focused-tested, committed, and pushed by `37d2b0d8`. `ModuleWorldV2` computes a domain-separated portable fingerprint without temporary allocation; decode and loading validate it. The loader computes a separate live workspace fingerprint from the supplied `ProjectUniverse`. The same package bytes load with and without an unrelated persistent workspace project, and represented identity, edge, and exposure mutations fail the portable check.

### P2.R2 type-graph relocation consultation

- **Trigger:** `TypeGraphV1::validate` permits in-bounds forward references and does not define row ordinal order, while the current `TypeStore` interns immutable forms only after their dependencies exist.
- **Decision:** Keep artifact row IDs opaque and resolve dependencies privately into current-main IDs. Accept arbitrary acyclic row order; reject dependency cycles that current `TypeStore` forms cannot represent. Do not impose a new wire ordering contract.
- **Plan impact:** No Level 2 amendment is required. The importer recursively relocates all type-graph row families and checks reconstruction invariants before retaining the type store. Static-value DAG rows now relocate into a fresh current-main `StaticValueStore`, validating static types and declaration/variant ownership. This is a T2 slice only; semantic aggregates/proofs, query products, and session publication remain open.
- **Verification:** Forward type and kind references through a nested lambda relocate successfully. Type/lambda cycles, static-value cycles, malformed tuple shapes, and noncanonical owner paths fail closed. Focused type/static importer tests and the semantic artifact import/N/N-1/retry suite pass. Internal store-allocation accounting remains a T5 requirement.

### P2.R2 query-persistence authority consultation

- **Trigger:** The active handoff requires reconciliation with PERF003's `CanonicalQueryPersistence`, `canonical_query_persistence`, and `QueryFreezeRejection`, but a targeted source search on the artifact branch found no such symbols. Query reuse is semantic authority and affects incremental invalidation, so the implementer did not invent a replacement policy.
- **Finding:** The authority exists at the pinned certified P7 commit `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94` in `phalcom-semantic/src/canonical_image.rs`; the P1 census and P2 handoff both identify that source. This is an expected porting gap, not a disproved architecture premise.
- **Decision:** Port the three P7 dispositions and its per-root requirements: current validation revision, exact closed-world owner, typed product/family match, product and input fingerprints, closed dependency fingerprints, and acyclic dependency graph. Keep product location (`QueryProfileSupport`) separate from query reuse eligibility. Classify the newly added current-main `AttributeArgument` kind as `Unsupported` without additional proof. Open-world and unsupported products are not published as reusable `Ready` roots, though required canonical products may remain in the snapshot.
- **Plan impact:** No Level 2 amendment is required. A material divergence from P7 or final deferral of query-root reuse would require an amendment.
- **Status:** The exhaustive policy classifier is implemented and focused-tested; root reconstruction/publication checks remain open.

## Current API surface (not yet a P3 takeover)

At source revision `37d2b0d8`, `ImportedCompilerImage` contains a reconstructed `CanonicalExternalModuleWorld`, the retained caller project graph, metadata, relocated current-main type and static-value stores, primary data/enum/variant aggregates, and the still-portable `SemanticWorldV3` and `QueryImageV3`. It is staged import data, not a published current-main `SemanticSnapshot` or populated `SemanticDb`.

The current pipeline is owned by `ArtifactImportTransaction::stage_payload_with_digest`. `StableOwnerCatalog` resolves exact project-qualified identities for type/static relocation, primary structural reconstruction, and typed query owners. `QueryKey::from_portable_owner` and `query_root_is_reusable` provide owner conversion and policy/location checks. There is no `SemanticProduct` reconstruction, product/dependency fingerprint validation, or query database publication yet.

## Active plan

`ARTF001.C1.P2.R2` closure remediation — reconstruct artifact products into current semantic stores and the workspace session, complete query persistence and owner validation, and prove whole-load allocation behavior. The T2/T4 project mapping is owned by the supplied workspace graph. Exact Package bindings require receipts from the caller's trusted resolver. P3 remains proposed until P2 is accepted.

### P2 task state at source revision `37d2b0d8`

| Task | Current state |
|---|---|
| T1 | Project-qualified module/query schemas and workspace-owned project bindings are implemented; focused identity, topology, and package-authority tests passed. Remaining end-to-end owner/product/dependency checks stay in P2 closure. |
| T2 | Partial: type and static-value graphs plus data/enum/variant structural aggregates are reconstructed into current stores. Other declaration, hierarchy, callable, conformance, proof, and operational products remain open. |
| T3 | Partial: exhaustive persistence policy, root-location gate, stable owner catalog, and typed `QueryKey` owner conversion are implemented. Product reconstruction, fingerprint recomputation, dependency closure/freshness, and `SemanticDb` publication remain open. |
| T4 | Not started: no source-free `SemanticWorkspaceSession` constructor or atomic session/snapshot publication exists. |
| T5 | Not started: current N/N−1/retry evidence covers tracked import reservations after decode, not allocator-requested bytes across decode through publication. |
| T6 | Partial: focused malformed-input, identity, relocation, and policy tests exist; the required full hostile compatibility matrix remains incomplete. |

The original `ARTF001.C1.P2-handoff.md` and `ARTF001.C1.P2-walkthrough.md` are historical records for the first closure attempt. The current operational handoff is `ARTF001-C1-P2-R2-handoff.md`.
