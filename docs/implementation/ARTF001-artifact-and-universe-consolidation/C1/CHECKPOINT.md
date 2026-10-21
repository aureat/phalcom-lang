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
| `ARTF001.C1.P2.R3` | SUPERSEDED | PARTIAL | UNVERIFIED | Complete P2 before P3: native declaration/alias/hierarchy reconstruction, native witness/derived-conformance, source-free static program execution, complete native `CallableAnalysis`, module execution authority, dependency-closed query import, immutable imported canonical generation, ordinary `SemanticWorkspaceSession` publication without source access, whole-load allocation N/N-1/retry, and hostile matrix certification. Superseded by P2.R4. |
| `ARTF001.C1.P2.R4` | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | Final P2 closure: current-schema P7 recovery, ordinary session publication, consumer consolidation, and certification. Supersedes P2.R3. |
| `ARTF001.C1.P3` | PROPOSED | NOT_STARTED | UNVERIFIED | Do not start until P2 publishes reconstructed current-main semantic/query authority through the ordinary session. |
| `ARTF001.C1.P4` | PROPOSED | NOT_STARTED | UNVERIFIED | Core canonical artifact compiler and source-free compilation path parity. |
| `ARTF001.C1.P5` | PROPOSED | NOT_STARTED | UNVERIFIED | Universe source cutover from `phalcom-core/core/universe` to `packages/universe` retaining `ProjectIdentity::Universe`. |
| `ARTF001.C1.P6` | PROPOSED | NOT_STARTED | UNVERIFIED | Universal artifact resolution across CLI, REPL, VM bootstrap, and tests with persistent cache ergonomics. |

## Current continuation state (2026-10-20)

- Latest fetched revision: `48c47753b` on `feat/artifact-universe-consolidation`; the remote branch still matched that revision. The continuation source changes are committed at `22a95bbb1` and `4807ecaf2`. P7 ancestry merged at `f1d2b6dc7b70fd0bab7c3f4abe5f2fe4def3645a` with P7 reference head `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94`.
- Active plan: `ARTF001.C1.P2.R4`, superseding `ARTF001.C1.P2.R3`.
- Current status: P2.R4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. P2 is open and P3 remains blocked pending R4 acceptance. R4.T3 is complete. The former R4.T5 canonical-query mutation blocker was resolved before `48c47753b`; the older incident entry below is historical. Full P2 acceptance has not been established.
- Pack consultation: closed. Pack remains runtime owner and semantic application form without an eighth type-member owner.
- Next action: close R4's compiler-image gates under P2.R4, then continue to P8's executable source-free VM path. P8 has started but is not complete; no source-free VM runtime proof exists. Keep R4 and P2 status separate from P8 progress.
- Branch rule: Continue on `feat/artifact-universe-consolidation`.

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

### P2 audit findings at the initial R2 handoff (historical snapshot)

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

### P2.R3 P7 ancestry-merge adaptation consultation — 2026-10-20

- **Trigger:** The required real P7 ancestry merge exposed a publication representation conflict: current `SemanticSnapshot` fields use standard maps, while the P7 importer stages budget-accounted frozen maps. Converting those rows to standard maps would leave allocations outside the exact import ledger; changing the whole snapshot or retaining a parallel sidecar would threaten current semantic authority and create duplicate state.
- **Decision:** Keep current semantic product types, owners, and authority. Use P7 `ParentMap`, `OrderedMap`, and `CallableAnalysisTable` selectively as the backing for imported immutable snapshot products. Do not restore the older P7 snapshot wholesale, add a sidecar source of truth, or estimate standard-map allocation costs.
- **Identity boundary:** Thread one transaction-scoped `StableOwnerCatalog` through artifact-supplied module, declaration, variant, implementation, callable, query, dependency, and static-symbol conversions into live identities. Already-relocated IDs and non-identity payloads do not need the catalog. Its path conversion must use the import allocation ledger before exact-count certification.
- **Operational static programs:** Port P7's `Operational` evaluator work/frame path into the current evaluator; do not replace `static_eval/machine.rs` wholesale. Stage and publish one native operational authority. Escalate if a current checked operation is absent from the canonical program or adapting execution would change current control-flow/result semantics.
- **Implementation state:** The P7 ancestry merge remains in progress at consolidation `cf6de4bfff06b81994c66592cd3da8756e961e6c` with P7 `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94` as `MERGE_HEAD`; no merge commit has been made. The focused semantic library check passes with 0 errors and 68 warnings after an initial 86 errors. The package sweep also passes `phalcom-type-meta`, `phalcom-compiler-artifact`, and `phalcom-modules`; `phalcom-core` fails with 43 compile errors in the P7 runtime/compiler integration. No tests were run. This establishes a compileable semantic-library slice, not an integrated compileable state. P2 remains partial. The architecture consultation below resolved the core boundary; scoped API adaptation remains.

### P2.R3 P7 core API merge-boundary consultation — 2026-10-20

- **Trigger:** The required P7 merge exposed 43 `phalcom-core` compile errors spanning obsolete artifact schemas/options, current versus P7 generic call-environment types, `RuntimeRecordRow` shape, compiler lowering identities, and collection APIs.
- **Finding:** The staged loader and generic VM extensions target P7's older `CanonicalUniverseFormalImage` route. P2 instead requires current-main native products to publish through the ordinary `SemanticSnapshot`, `SemanticDb`, and `SemanticWorkspaceSession`. The compile failures do not show that a new generic VM representation is needed.
- **Decision:** Level 1 clarification; no plan amendment. Preserve P7 artifact transport, budgeted reconstruction, selectively frozen imported product backing, and the static evaluator's required operational work/frame path. Keep current-main semantic/compiler/runtime contracts authoritative. Adapt or remove only staged P7 consumers that assume absent APIs. Do not migrate `RuntimeTypeEnvironment` to tagged static arguments to satisfy old call sites, retain the parallel `CanonicalUniverseFormalImage` route, or discard budget/accounting work in a blanket revert.
- **Implementation direction:** Rework the artifact loader toward current schemas and ordinary imported-session publication. Narrowly adapt `vm/dispatch.rs`, `modules/materialize.rs`, `compiler/lib/product_opt.rs`, and the `RuntimeRecordRow` consumers. Preserve exact owners, single semantic authority, allocation accounting, and atomic publication.
- **Verification:** The consultation reproduced `cargo check -p phalcom-core` at 43 errors; no tests were run. Rerun that focused package check after scoped changes, then run only directly affected source compiler/runtime and artifact-session publication tests at the user's test gate.
- **Status:** Implementer may resume on the current-main API integration and P2 artifact/session path. No code was changed by the adviser.

### P2.R3 handoff and follow-up image-lifecycle sequencing amendment — 2026-10-20

- **Authority:** The user-provided continuation handoff and the follow-up architecture consultation both direct recovery of the certified P7 image lifecycle before changing its publication boundary. This supersedes the earlier consultation's specific instruction to remove the `CanonicalUniverseFormalImage` route or move the loader to `SemanticWorkspaceSession` immediately; that earlier instruction is retained above as chronology.
- **Decision:** Restore and adapt `CanonicalUniverseFormalImage`, `CanonicalCompilerImageView`, the loaded canonical reader, source-free completeness, module-query publication, dependency fingerprints, and the loaded-publication census as one P7 compiler-image path. The image is the sole loaded artifact authority during recovery, not a parallel semantic engine or sidecar. Adapt it to current semantic products and V2/V4/V3 payloads. Do not redesign it into `SemanticWorkspaceSession` in the same patch.
- **Sequence:** First reach an integrated compileable state, then recover the P7 build/encode/load lifecycle and source-free completeness, then integrate the interrupted T1 phase and declaration-owner additions. Resume the ordinary current-session publication slices only after that lifecycle is recovered.
- **Verification boundary:** The active handoff prohibits tests in this continuation. Focused compile checks are allowed. Do not claim P2/T1 completion from a compile-only recovery.
- **Status:** The amendment changes sequencing and the temporary loaded-image authority boundary. P2 remains partial; no tests have been run.

### P2.R3 T1 source-import identity clarification — 2026-10-20

- **Trigger:** The V4 phase rows contain a `u32` binding value, but live `SourceImportBindingId` contains an import-site identity and a site-local binding ordinal. The plan requires the exact stable identity.
- **Decision:** Interpret `binding` as a checked, module-relative flattened source-binding ordinal. Enumerate `ModuleWorldV2` imports in vector order; each row index is the import-site ordinal, including sites with no bindings. A module import contributes one binding at local ordinal 0; selective imports and re-exports contribute one binding per item in item order. The wire ordinal is the checked prefix sum of earlier site binding counts plus the item-local ordinal. Decoding reverses those ranges into the exact source-import identity.
- **Scope and validation:** A module product's `StableModuleRef` scopes its ordinals; a callable's stable owner supplies the module scope. Require module products for every module, with exactly the dense ascending ordinal range for that module's bindings, including an empty range where appropriate. Callable observations remain sparse, must resolve within the owner's module, and must be consistent with the module phase summary. Never use executable linked-read slots, names, or spans as identity.
- **Plan impact:** No material architecture or wire-layout amendment is needed. Make the encoding and cross-row validation explicit in T1.3/T1.4 and cover it with mixed-import round-trip and malformed-row cases. Existing V4/semantic ABI versioning applies; no additional bump is needed unless a producer has emitted a different `binding` meaning. No such producer is present in the inspected tree.
- **Verification required:** Round-trip module, multi-item selective, multi-item re-export, and empty-item import sites; assert exact site/local identities for complete module products and sparse callable observations. Reject duplicate, missing, out-of-range, wrong-module, or phase-inconsistent rows and missing module products.
- **Status:** Implementer may resume. T1 remains pending until encoding, mapping, validation, and focused tests are implemented. No tests have been run for this slice.

- **Implementation follow-up (2026-10-20):** The V4 dependency-phase DTO now has the exact current states `InterfaceOnly` and `Runtime`. The lossy `CallableAnalysisRecordV1 -> V2` and `SemanticWorldV3 -> V4` conversions were removed because they defaulted new phase and module-execution authority to empty values. `cargo check -p phalcom-compiler-artifact` passes after this change. Source-import ordinal encoding, module/callable cross-row validation, and native product reconstruction remain open; no tests were run.

- **Implementation follow-up (2026-10-20):** `ModuleWorldV2` now exposes checked forward and reverse conversion between ordered import-site/local-binding coordinates and the module-relative `u32` ordinal. V4 payload validation uses that same source import table to require exactly one phase product per module, the dense ordinal range for its bindings, covered callable phase observations, and valid runtime-requirement module owners. `cargo check -p phalcom-compiler-artifact` passes. Exporter/importer conversion to the native module identities and products remains open; no tests were run.

### P2.R3 T1 dependency-phase wire clarification — 2026-10-20

- **Trigger:** The unfinished V4 DTO defined `Syntax`, `LoadTime`, and `Runtime`, while current `DependencyPhase` and callable phase evidence have only `InterfaceOnly` and `Runtime`.
- **Decision:** Make the V4 wire enum an exact two-state mirror: `InterfaceOnly` maps to `InterfaceOnly`, and `Runtime` maps to `Runtime`. Preserve the current join rule, where `Runtime` dominates. Do not reinterpret `Syntax` or `LoadTime` as current phases.
- **Plan impact:** This is a Level 1 clarification of exact current authority. No ABI bump is needed because no V4 producer has emitted artifacts. If a producer appears before implementation, the wire change requires a new artifact version rather than reinterpretation.
- **Verification required:** Round-trip both current phase values and reject invalid discriminants. Phase coverage checks must compare using the current phase ordering.
- **Status:** Implementer may resume. Update the unfinished DTO and T1.3/T1.4 notes before adding a producer.

### P2.R3 T1 intrinsic type-owner wire clarification — 2026-10-20 (superseded by the following amendment)

- **Trigger:** `DeclarationShellV2.intrinsic_type_owner` reused `IntrinsicRuntimeOwnerV1`, whose three variants cannot represent all seven current `IntrinsicTypeOwner` values.
- **Decision:** Define a distinct exhaustive `IntrinsicTypeOwnerV1` for the seven declaration-shell owner variants. Keep `IntrinsicRuntimeOwnerV1` as the three-case runtime binding owner for `DeclarationTypeBindingV1.runtime_owner`. Import both independently through the project-qualified `StableOwnerCatalog`, preserve `None`, and reject duplicate or conflicting shell authority.
- **Plan impact:** Level 1 clarification; no ABI bump is needed because `StructuralPrimaryContractsV2` and `SemanticWorldV4` remain unpublished. If either schema has been emitted before this correction, stop and version the changed contract.
- **Verification required:** Round-trip each intrinsic type owner and the independent runtime owner; reject duplicate owner authority. No tests are run during this continuation per the active handoff.
- **Status:** Its initial type-member interpretation was superseded after the shell-less authority audit below. The shell field is retained as a redundant runtime-owner witness; the seven-case intrinsic type-owner enum applies only to standalone rows per the user clarification recorded after the schema amendment.

### P2.R3 T1 shell-less intrinsic projection schema amendment — 2026-10-20

- **Trigger:** `register_intrinsic_static_domain` stores authority outside `DeclarationTypeInfo` and removes the nominal shell. The current V2 producer attached static-domain and type-owner fields only while iterating declaration shells, so shell-less compiler-owned type authority could not be exported. No validated source-free derivation was found.
- **Consultation decision:** Add independent `intrinsic_static_domains` and `intrinsic_type_owners` row families to `StructuralPrimaryContractsV2`, keyed by project-qualified `StableDeclarationRef`; remove the actual static-domain and seven-case type-member projections from declaration shells. Validate strict per-family ordering/uniqueness, resolve owners through `StableOwnerCatalog` even when shell-less, and import each projection independently with allocation charging. Do not infer static-domain authority from a kind, or type-owner authority from static/runtime ownership. A single owner can have both projection rows.
- **Plan/schema impact:** Level 2 architecture amendment; schema V2/V4 stay unchanged because they remain unpublished on this branch. If emitted bytes are found, version the affected structural and enclosing semantic artifact schemas before acceptance.
- **Status:** Accepted by architectural adviser and implemented in the current source tree. `cargo check -p phalcom-compiler-artifact --lib`, `cargo check -p phalcom-semantic --lib`, and `cargo check -p phalcom-core --lib` pass. Tests remain intentionally unrun per the active handoff.

### P2.R3 T1 shell runtime-owner witness clarification — 2026-10-20

- **User decision:** Keep `DeclarationShellV2.intrinsic_type_owner` as the three-case `IntrinsicRuntimeOwnerV1` runtime-owner witness. It is redundant with `binding.runtime_owner` and must match it exactly. `Pack` stays runtime-only and does not imply `IntrinsicTypeOwnerV1::TypePack`.
- **Wire separation:** The actual seven-case type-member authority remains in standalone `intrinsic_type_owners` rows, independent of shell/runtime authority. Static-domain authority remains in standalone rows as well.
- **Implementation:** V2 validation now rejects shell witness/binding mismatches; export copies the runtime owner into both fields. No tests were run per the active handoff.
- **Verification:** `cargo check -p phalcom-compiler-artifact --lib`, `cargo check -p phalcom-semantic --lib`, and `cargo check -p phalcom-core --lib` pass after the clarification. Tests remain unrun per the active handoff.

### P2.R3 T1 Pack semantic-owner consultation — 2026-10-20

- **Status:** Architecture adviser ruling received. Independent P2 consolidation may resume; no direct Pack type-member owner is added under the current specification.
- **Trigger:** The user asks that Pack have semantic authority beyond its current runtime owner. This changes semantic ownership and may change type-member dispatch, so it crosses the implementer's authority boundary.
- **Normative boundary:** [`type-level-domains.md`](../../../specs/type-level-domains.md) and [`structural-products.md`](../../../specs/structural-products.md) define first-class proper `Pack` types separately from the kinded `TypePack` domain. The current runtime-owner witness must not be treated as proof that Pack and TypePack share one owner.
- **Observed implementation:** `DeclarationTypeTable::store_authoritative` and `register_intrinsic_runtime_owner` associate runtime Tuple and Record with existing type-member owners, but associate runtime Pack with none. The source Pack declaration has semantic form `Intrinsic(Pack)` and runtime owner `Pack`. `register_intrinsic_static_domain` separately associates the `TypePack` kind with the `TypePack` type-member owner. Artifact V2 carries runtime ownership on the shell and actual type-member ownership in independent rows.
- **Required invariants:** Keep one canonical semantic owner table; keep the shell runtime-owner witness equal to `binding.runtime_owner`; preserve project-qualified artifact owner validation; do not conflate the first-class Pack type with the kinded TypePack domain without an explicit semantic ruling.
- **Advisory ruling:** Pack is already modeled beyond runtime ownership through `IntrinsicTypeApplication::Pack`, `TypeData::Pack` / `PackFromPacks`, and the structural Pack family. The current specs define members on Pack specializations (`View`, `PositionalEntries`, and `LabeledEntries`) but no direct intrinsic type-member operations owned by Pack itself. Keep the current three-case runtime owner; do not infer `IntrinsicTypeOwner::TypePack` or add an eighth intrinsic type owner without a specified direct Pack-member contract. Confidence is high for the current spec and P2 boundary.
- **Plan impact:** No amendment is needed to continue P2 with the current owner separation. A direct Pack projection API would require a Level 2 semantic amendment, normative specification, and updates to projection lookup, artifact transport, and acceptance tests. Independent P2 work may continue; P3 remains blocked until P2 closes.
- **Verification required:** After the handoff permits tests, preserve the distinction between Pack and TypePack, round-trip Pack binding/runtime authority independently from type-owner rows, and test any newly specified Pack members and result domains. No tests were run for this consultation.

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

## Current imported pipeline (not yet a P3 takeover)

The restored P7 core loader validates V2/V4/V3 sections, reconstructs the current `SemanticSnapshot` and query image, and builds a `CanonicalUniverseFormalImage`. The structural V2 import now reconstructs declaration shells plus independently carried static-domain and intrinsic type-owner projections, including owners without shells. These paths compile, but the imported image is not yet published as the ordinary `SemanticWorkspaceSession` base with local overlays and invalidation.

The separate `ArtifactImportTransaction::stage_payload_with_digest` path still returns `ImportedCompilerImage` as staging data. It retains the `StableOwnerCatalog`-resolved typed query owners and staged semantic authority; query-root reuse and final session publication remain incomplete P2 work. P3 stays blocked until the ordinary session publication contract is complete.

## Active plan

`ARTF001.C1.P2.R4` — Final P2 Closure: Current-Schema P7 Recovery, Ordinary Session Publication, Consumer Consolidation, and Certification. Supersedes P2.R3. Tasks T0–T9 define the complete path to P2 closure before unblocking P3.

### P2.R4 task ledger

| Task | Objective | Status |
|---|---|---|
| R4.T0 | Re-pin, normalize lifecycle records, terminate no-test period | COMPLETE |
| R4.T1 | Correct compatibility identity and port test/census infrastructure to current schemas | IN_PROGRESS |
| R4.T2 | Recover complete ModuleWorldV2/SemanticWorldV4/QueryImageV4 build/load lifecycle and close current semantic authority | IN_PROGRESS |
| R4.T3 | Re-establish exact allocation, rollback, failpoint, and retry certification | COMPLETE |
| R4.T4 | Finish dependency-bearing current query profile and fail-closed required-root completeness | COMPLETE |
| R4.T5 | Certify ordinary imported SemanticWorkspaceSession with overlays/invalidation/rollback | COMPLETE |
| R4.T6 | Consolidate duplicate consumer/import staging architecture and stabilize APIs | PENDING |
| R4.T7 | Fresh-process source denial, compiler parity, diagnostics parity, real package/runtime functionality | IN_PROGRESS — source-denied, ordinary-VM runtime, and source CLI gates pass; positive parity has an unresolved closure mismatch |
| R4.T8 | Broad regression, schema freeze, manifest closure, cleanup | IN_PROGRESS — current-schema query allocation fixture passes; workspace no-run is being retried |
| R4.T9 | P2 acceptance, walkthrough, checkpoint closure, P3 handoff | PENDING |

### P2.R4 compiler-image publication consultation — 2026-10-20

- **Trigger:** The current-source `artifact-allocation` discriminator reached semantic publication, then failed because compiler-image validation found advisory products that had already been generated. The request was only supplied to post-publication validation; `SemanticWorkspaceSession::update` did not consume it.
- **Finding:** `SemanticWorkspaceSession` owns optional semantic product generation. `build_advisory_workspace` and advisory query publication run unconditionally, while canonical image construction validates `SemanticProductRequest::canonical_compiler_image()` only after `update` returns. This is a publication-request propagation gap, not an artifact importer defect.
- **Decision:** Preserve the default full tooling behavior for existing session callers. Add request-aware session configuration and apply it before optional product generation. Compiler-image publication must retain all compiler-required formal/source semantic products, omit presentation-only and advisory products at their owning generation paths, and keep transaction rollback atomic. Do not request full products, generate then strip them, or weaken validation.
- **Plan impact:** Level 1 clarification. Amend R4.T2.1 to require request propagation into semantic publication before optional products are computed. Its focused gate compares compiler-image and default/full session publication, then runs canonical image publication. No wire-schema or ownership changes.
- **Status:** Request propagation is implemented and the focused `semantic-product-request` target passes 4/4. The canonical artifact build now reaches export but is blocked by the module-binding explanation mismatch below; T1 remains in progress and T2 has started to clear its vertical discriminator.

### P2.R4 canonical static-program publication consultation — 2026-10-20

- **Trigger:** The current canonical artifact build rejected a source-backed static definition because its semantic product had a checked body but no canonical operational program. The repair crosses session publication, static-value rooting, and artifact export.
- **Finding:** Source publication attaches `checked_body` but leaves `canonical_program` absent. V4 export requires the latter. Lowering during export would intern static values after the producer closes its static/type export graphs.
- **Decision:** The explicit compiler-image session request owns lowering from each checked static body to its canonical operational program. Publish both products, and intern any generated static values before snapshot publication and graph-root registration. Leave default/full sessions on their current checked-body product path. Keep the existing evaluator semantics and fail publication if canonical lowering fails.
- **Invariant:** For one semantic revision, the canonical program must be a deterministic semantics-preserving projection of its checked body, retaining owner, dependencies, applications, result, and failure behavior. Artifact import remains source-free and must not fabricate an AST or fall back to source.
- **Plan impact:** Level 1 implementation clarification under R4.T2.1/T2.8; no numbered plan amendment. Record source-versus-imported execution comparison under the existing T2.8 gate.
- **Status:** Source canonical-program publication is implemented. The exact artifact-allocation run advanced past this failure and is now blocked by the module-binding explanation mismatch below. Source-versus-imported execution parity remains pending.

### P2.R4 focused verification update — 2026-10-20

- `module_world_rejects_invalid_and_duplicate_module_import_identities` passes; all 13 `phalcom-modules` unit tests pass. `cargo test -p phalcom-modules` also reports two unresolved `builtin_catalog` integration failures: expected 22 expose declarations but found 23, and unresolved `universe.collections.iterator_laws` import.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic-product-request` passes 4/4, covering request-specific publication, canonical static-program publication, and `List<Dynamic>`/`List<Int>` specialization.
- The exact `artifact-allocation` gate currently fails during `build_universe_telosc` export, before N/N-1/retry or allocator equality runs. For `universe.errors.unsupported::unsupported`, its module fact has `explanation: Some(ExplanationId(3))` while the `<main>` owner row has `None`; all other displayed binding-state fields match. The original referential-integrity check remains strict.
- The `core` integration target does not compile because of 10 current fixture/API errors; the `modules::canonical_artifact` filter ran no tests. See the implementer handoff for the affected names and exact next steps.

### P2.R4 implementer takeover note — 2026-10-20

- This handoff captures the current blocker and test state; preserve the referential-integrity check until the owning semantic publication/projection is understood.
- The core `modules::canonical_artifact` integration filter remains blocked at test-target compilation by 10 fixture/API errors. The full modules test keeps two unresolved `builtin_catalog` integration failures after 13/13 unit tests pass.
- Detailed takeover instructions, exact commands, verification boundaries, and advanced review topics are in [`ARTF001.C1.P2.R4-implementer-handoff.md`](ARTF001.C1.P2.R4-implementer-handoff.md). R4.T1 remains in progress; this note does not promote T2 or T3 completion.

### P2.R4 explanation fingerprint and test-harness recovery — 2026-10-20

- **Root cause:** The session publishes a preliminary `ModuleStaticBindingIndex`, replaces the module `<main>` callable analysis during its second pass, and republishes the final index through the same semantic query key. `hash_module_static_binding_index` omitted `BindingState.explanation`, so the second query reused the preliminary index when the owner-local explanation was the only changed field. The final snapshot therefore combined a stale module fact with the current callable owner row.
- **Correction:** `BindingState.explanation` now participates in both module-static-binding input and product fingerprints. The exporter referential-integrity check remains strict. The focused `module-static-binding-fingerprint` test proves that an explanation-only change alters the fingerprint, invalidates query reuse, and publishes the replacement row.
- **Harness recovery:** Current V2 callable-analysis fixtures now wrap legacy payload construction in `CallableAnalysisRecordV2`, pass current module/import-phase inputs, and initialize current analysis fields. Core integration fixtures use current runtime type recipes, metadata export, opaque synthetic identity rules, and immutable executable-link construction. Both previously unresolved `builtin_catalog` fixtures are corrected; the focused target passes 9/9.
- **Executed evidence:** `cargo test -p phalcom-semantic --test module-static-binding-fingerprint` passes 1/1; `cargo test -p phalcom-compiler-artifact` passes all 106 tests; `cargo test -p phalcom-semantic --lib --no-run`, `cargo test -p phalcom-core --test core --no-run`, and `cargo test -p phalcom-core --test artifact-allocation --no-run` compile successfully; `compiler_product_optimization::test_decision_engine_pure_cases` passes 1/1. The comparator filter selected eight tests but exceeded the bounded 60-second local execution window and remains incomplete.
- **Next gate:** Run `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1`. It must prove current artifact build/load, exact N, independent allocator equality, typed N-1 failure, child rollback, and clean retries before R4.T2 or R4.T3 status is promoted.

### P2.R4 allocation-gate continuation — 2026-10-20

- **Executed:** `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1` fails after 98.41s during `build_universe_telosc` semantic-world export.
- **Current blocker:** V4 module execution authority rejects a duplicate runtime dependency edge for importer `universe.callable.function`, dependency `universe.errors.error`, source range `831..836` (`Error` in `on(Error)`, `phalcom-core/core/universe/src/callable/function.ph`). The validator's uniqueness key is `(importer, dependency, range)`. No artifact load, allocation census, exact N, allocator equality, N−1, rollback, or retry assertion ran.
- **Root-cause lead:** `phase_evidence.rs` has overlapping source-visible native-prelude and semantic `TypeForm` requirement projections that may both emit this edge. Confirm both serialized targets and provenance before correcting; same-span edges may carry distinct authority. Do not weaken duplicate validation or deduplicate by the validator key without establishing a lossless coalescing rule.
- **Current status:** R4.T1 remains IN_PROGRESS; R4.T2 remains IN_PROGRESS; R4.T3 remains PENDING. The previous explanation-mismatch blocker is resolved by the explanation fingerprint fix recorded above. T2/T3 acceptance is not promoted.
- **Next action:** Inspect the producer rows for this exact source site; correct the redundant projection at its owner, add same-target overlap and distinct-target preservation regressions, then rerun the exact allocation gate. Continue ordered T2/T3 and later T4–T9 only after the vertical artifact build/load gate advances.
- **Takeover record:** Updated details and the remaining-gate sequence are in [`ARTF001.C1.P2.R4-implementer-handoff.md`](ARTF001.C1.P2.R4-implementer-handoff.md).

### P2.R4 native-prelude observation overlap — 2026-10-20

- Starting revision: `da00e4c36` in `.worktrees/artifact`; the worktree was clean. The unrelated `main` checkout edit in `phalcom-semantic/src/session.rs` is outside this work unit.
- Producer evidence: the exact allocation gate failed after 117.13s. Both records for `universe.callable.function`, dependency `universe.errors.error`, range `831..836`, have reason `RuntimeDeclarationReference`, read target `Binding(errors.error::Error)`, consumer `universe.callable.function`, provenance target `Binding(errors.error::Error)`, product `RuntimeObject`, and timing `DuringModuleInitializer`. Only the producer differs: `CompilerGeneratedRead` versus `DeclarationRead`.
- Correction: native-prelude fallback runs after module semantic projections. It suppresses an observation only when a `DeclarationRead` matches every other requirement field. Different targets, products, or timing remain distinct. The artifact validator and its duplicate rejection are unchanged.
- Focused evidence: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib phase_evidence::native_prelude_tests -- --nocapture` passes 3/3 in 0.01s. The tests cover overlapping same-authority reads, distinct targets at one span, and distinct authority for one target.
- Next gate: rerun the exact core allocation test with `PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc`. The prior dispatch has no preserved result, no active process, and no exported file in the current environment. No build/load, allocation, source-denial, or speedup acceptance is promoted from the focused producer tests.

### P2.R4 runtime obligation identity ruling — 2026-10-20

The user supplied an architectural ruling after the repeated duplicate rejection.
The native-prelude suppression remains accepted. Independently, wire duplicate
identity `(importer, dependency, range)` is too coarse for the native contract.
R4.T2.6a now defines obligation identity as importer, dependency, range, reason,
read target, provenance target, product, and timing. Consumer must equal importer;
producer alone does not distinguish obligations. Transport must reject duplicate
obligations and preserve distinct same-span obligations. Full-record ordering is
required in native finalization and export. No schema/ABI bump is needed before
the R4 freeze because the existing wire already carries the fields.

The authorized correction includes precise wire validation, complete record
ordering, hostile/round-trip tests, and native export/import parity. Capture all
remaining coarse-location collision groups before any further producer change;
ownership-specific suppression is permitted only for proven duplicate obligations.
No global producer precedence or blanket deduplication is authorized.

The complete diagnostic run finished in 115.81s before further producer edits.
It found 521 groups sharing importer, dependency, and range: 374 groups contained
only repeated identical obligations, accounting for 616 extra records
(`CallableOwner`: 302; `DeclarationRead`: 72; `PatternOwner`: 242). The other 147
groups contained distinct obligations, with no repeated obligation within those
groups. No producer-only collision remained. The bounded correction therefore
targets repeated observations in those three owning producer helpers and retains
the distinct groups. Diagnostic instrumentation has been removed. The diagnostic
run still failed during export; it provides no load or allocation certification.

The bounded correction's focused gates pass: compiler-artifact suite 111/111;
`phase_evidence::native_prelude_tests` 4/4; and
`runtime_requirement_multiset_survives_native_export_v4_and_canonical_import`
1/1. The bridge transports five same-span native requirements through the
production exporter mapping, V4 encode/decode validation, and canonical import,
then compares complete native records. The vertical build/load gate is next;
T2 and T3 remain incomplete.

The single vertical run after this correction reached a different export check:
`static definition alias projection lacks structural alias authority`. It failed
after 113.22s of test execution (145.91s including compilation), before saving an
artifact or reporting any allocation stage. The runtime-requirement check no
longer blocks that run. Whole-load and source-free acceptance remain pending.

The new failure is a current-schema adapter defect under T2.4. V4 validates its
current structural contracts, then `to_v3_for_validation` builds a temporary V3
view containing hierarchy rows with `aliased_type: None` and omits every V2
type-alias row. The reused V3 static-definition validator then requires the
missing alias projection. The exporter has already checked the static product's
alias against the native alias table, and the native importer consumes the V2
alias rows. The bounded correction must preserve aliases in the validation view,
merge same-owner hierarchy/alias rows without losing superclass authority, and
retain rejection when structural alias authority is actually absent. This adapts
the private validation projection; it changes no canonical authority or schema.

The alias adapter correction is implemented and focused-tested: three alias
regressions and all 114 compiler-artifact tests pass. It preserves superclass and
alias form for shared owners, handles unordered rows, and retains duplicate rows
for validation. Its vertical run passed the alias check and reached query export,
then failed because a DeclarationShell query had no semantic product row.

The subsequent diagnostic-only census examined all 2,564 reusable candidates:
16 DeclarationShell mappings fail. Fifteen native Alias products have existing
V4 alias rows but no nominal shells. Native `reflection.package_object::Package`
is Nominal and absent from nominal, alias, static-domain, and intrinsic-owner rows.
The latter is a separate final-aggregate/query consistency issue; its cause has
not been established. The diagnostic run failed after 109.50s (129.15s including
compilation), saved no artifact, and emitted no allocation-stage results.
Instrumentation has been removed. No query representation or retention policy
has been changed.

**Consultation required:** DeclarationShell transport currently selects only the
nominal table, although the native product is a nominal/alias sum type. Selecting
a new table/index contract or discarding roots changes a cross-layer product
contract. Implementation of that issue stops under the mandatory escalation
rules. The incident, complete owner census, alternatives, and decisions requested
are in [`ARTF001.C1.P2.R4-query-shell-consultation.md`](ARTF001.C1.P2.R4-query-shell-consultation.md).
Current status remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED; T2/T3 acceptance,
source-free loading, and speedup remain unproven.

### P2.R4 source-denial controls and Package scope evidence — 2026-10-20

Direct source denial is now implemented at `UniverseSourceProvider::source_text`.
Provider construction remains inert. A focused parent test starts separate child
processes for direct root-source access, source prelude construction, canonical
formal-image construction, and canonical compiler-program construction; each
must fail at its expected denial guard. A fifth child proves provider construction
is allowed. These controls are shared with the later artifact-backed positive
test.

The canonical-artifact test file had no module registration, so earlier filters
could not execute it. It is now registered as `modules_canonical_artifact` in the
core test binary and compiles. The exact focused selector
`modules_canonical_artifact::source_denial_guards_are_effective_without_artifact`
passes 1/1 with 1028 tests filtered, in 0.02s, with artifact environment variables
unset. This proves denial efficacy, not artifact-backed source-free operation.

A temporary Package scope probe used its actual source interface and the
production symbolic linker/resolver. Package has no imports; Module is not in the
canonical prelude; unqualified Module resolves to None in Package's scope despite
being known. Object resolves through the prelude, and Module resolves locally in
its own module. Exactly one probe test passed in 0.07s. The probe is removed and
`resolver.rs` has no diff. Evidence is saved at
`/tmp/artf001-r4-package-scope-probe.log`.

This establishes the unresolved superclass name in Package's source. Missing
explicit import followed by blocked-declaration removal is the causal lead for
the absent final row; the exact removal event was not executed by this probe.
No Package source, query-freeze, or query-representation correction was made.
The architectural consultation remains pending. The user requested a handoff;
the current implementer handoff now replaces superseded restart instructions
with the verified state and narrow resume path. No commit or push was made at that earlier handoff.

### P2.R4 downstream proof preparation — 2026-10-20

Read-only inspection while R4.T2.6a is implemented established these later checks:

- **T3:** `artifact-allocation` already measures current V2/V4/V3 import stages,
  exact budgets, and retries. Its export failure prevents those assertions from
  running. Reuse the saved artifact only after the current exporter succeeds.
  Also inspect V4 validation scratch: `to_v3_for_validation` and callable
  `to_v1` currently deep-clone DTO collections without an allocation budget.
  The accounting guard returns `Ok(())`. These calls are inside the production
  decode/validation path and need explicit accounting or removal before exact
  allocation can pass; their allocator delta has not yet been measured.
- **T4:** `QueryKind::ALL` and `QUERY_KIND_MANIFEST` contain 46 kinds. The query
  allocation census contains 45 and omits `AttributeArgument`; its recorder
  silently returns for absent kinds. The census must be reconciled with the
  exhaustive inventory. `export_query_image_v3` still prunes candidates without
  an explicit required-root profile. Required-root completeness remains pending.
- **T5/T7:** `discover_image_closure` constructs the ordinary session through
  `SemanticWorkspaceSession::from_image` and supplies external canonical modules.
  The fresh-process test denies the top-level bootstrap constructors, but its
  comment claims a nonexistent `UniverseSourceProvider::read_source` denial hook.
  The actual source-text entry point, `source_text`, has no denial check. Add and
  exercise direct source-access denial before claiming zero canonical source
  reads. Constructing an inert provider remains permitted by T5.2.
- **Timing proof:** compare fresh child processes running the same user workload,
  with source construction versus artifact load included in the respective total.
  Record artifact construction separately and report load and workload durations.
  No speedup has been measured.

These are implementation follow-ups under the existing ordered tasks. They do
not promote T2–T7 acceptance or authorize early consumer consolidation.

### P2.R4 query-product family and final shell consistency ruling — 2026-10-20

The user supplied an audit ruling after independent inspection confirmed that,
at the time, QueryImageV3 conflated logical query kind with physical
product-table identity.
The V4 semantic world already carries alias authority in
`StructuralPrimaryContractsV2.type_aliases`; the gap is the query locator, not
missing semantic data. The query correction is assigned `R4.T2.6b` because
`R4.T2.7` is already the witness/derived/proof task in the active plan.

The accepted contract introduces an exhaustive physical
`PortableQueryProductFamilyV1`, versions QueryImage V3 to V4 and query
fingerprints ABI V2 to V3, and validates logical query/family compatibility,
family-specific row bounds, and exact stable owners. DeclarationShell roots
may address exactly a nominal shell row or a type-alias row. The structural
world rejects cross-family owner ambiguity. Portable fingerprints include the
selected family and row; imported live fingerprints continue to be recomputed
from native products.

Static-definition aliases are authorized by their exact alias row and owner;
they require no nominal shell. Package gains an explicit sibling Module import.
Blocked nominal declarations retire stale shell queries, and final query
publication rejects absent, duplicated, or variant-mismatched shell authority.
Requiredness remains a property of logical QueryKind, so aliases cannot be
reverse-pruned merely for using the TypeAlias product family.

This is a query schema/ABI amendment only. No ModuleWorld, SemanticWorld,
semantic artifact ABI, stable identity, or outer-container change is authorized
without separate evidence. The focused hostile/round-trip matrix has passed.
The vertical allocation gate reached module materialization and failed its exact
allocation comparison; the T3 representation ruling and later load stages
remain open. R4 stays IN_PROGRESS / PARTIAL / FOCUSED_TESTED and P3 remains
blocked.

### P2.R4 T2.6b implementation and T3 allocation result — 2026-10-20

- **Implemented:** QueryImage V4 separates logical query kinds from physical
  product families. DeclarationShell roots can reference exact nominal-shell
  or type-alias rows; structural validation rejects cross-family owner
  ambiguity; portable shell fingerprints include family and row content;
  static-definition aliases require their exact alias row without a nominal
  shell; Package explicitly imports sibling `Module`; blocked nominal shell
  queries are retired; final shell publication requires one matching native
  authority. Query requiredness remains a property of logical query kind.
- **Focused evidence:** The full `phalcom-compiler-artifact` suite passed
  earlier in this continuation. The V4 nominal/alias round-trip fingerprint
  regression (1/1), query schema rejection test (1/1), final shell authority
  regression (1/1), blocked-shell retirement regression (1/1), alias-only
  static-definition authority regression (1/1), and typed query-owner
  integration suite (4/4) pass. The core allocation target compiles against the
  V4 importer.
- **Vertical gate:**
  `PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1`
  produced a 20,667,377-byte artifact, then failed at module materialization
  before semantic/query import. Independent allocator requests were
  `2,131,498` bytes versus `1,094,188` ledger bytes (delta `+1,037,310`).
- **Bounded correction:** `ModuleId::clone_with_allocation_budget` allocated a
  budgeted component string, then copied it again through
  `ModuleComponent::from_identifier`. It now transfers that string with
  `from_owned_identifier`. Rechecking the saved artifact lowered observed
  module-stage requests to `2,043,796` (delta `+949,608`), so other unaccounted
  module materialization allocations remain.
- **Current blocker:** `load_module_world_inner` builds retained BTreeMap and
  BTreeSet indexes and graph products with ordinary `collect`/`from_entries`
  constructors. Graph storage is fixed to BTreeMap, unlike the existing
  source/frozen `ModuleAuthorityMap` and `ModuleAuthoritySet`.
- **User representation ruling (2026-10-20):** Use a frozen, budgeted imported
  module graph as the canonical base, with ordinary mutable overlays for local
  edits. Source-backed graphs remain mutable. Never mutate or thaw the imported
  canonical base in place; overlay lookup must preserve the base graph's
  identity and authority while edits add, replace, or hide local graph entries.
- **Plan amendment:** Adapt graph/index storage at artifact materialization to
  use budgeted frozen containers. Keep current source graph APIs mutable and
  route session edits through mutable overlays. Prove reads and edits across
  base-plus-overlay state before promoting T3; this representation clarification
  does not change artifact wire schemas.
- **Current blocker:** The representation contract is now decided, but the
  overlay boundary and graph API integration still need implementation and
  focused validation. The saved artifact is at `/tmp/artf001-universe.telosc`;
  later load stages, N/N-1, rollback, and retry assertions did not run.
- **Status:** R4.T2 remains IN_PROGRESS / PARTIAL; R4.T3 is IN_PROGRESS but
  uncertified. T2.6b focused behavior is implemented and tested; full canonical
  load, exact allocation, P2 closure, and P3 takeover remain unproven.
- **Incident packet:** [`ARTF001.C1.P2.R4-module-allocation-incident.md`](ARTF001.C1.P2.R4-module-allocation-incident.md) records the allocator evidence, code path, attempted correction, and representation decision requested.

### P2.R4 T3 module graph representation ruling — 2026-10-20

- **Incident:** [`ARTF001.C1.P2.R4-module-allocation-incident.md`](ARTF001.C1.P2.R4-module-allocation-incident.md)
- **Trigger:** Exact module-stage allocation accounting is short by 949,608
  observed bytes; retained BTreeMap/BTreeSet graph/index allocations are a
  likely source, and changing graph storage affects the edit contract.
- **User decision:** Use frozen base plus mutable overlays. Imported canonical
  module graph storage is frozen and budget-aware. Source-backed graphs remain
  mutable. Local workspace edits are represented in mutable overlays above the
  frozen base; the base is not mutated or implicitly thawed.
- **Architectural consequence:** Keep artifact allocation accounting at the
  imported base construction boundary and define lookup/update behavior across
  base and overlay without duplicating canonical project/module identity. The
  wire schema is unchanged. T3 must verify overlay precedence, deletion/hiding,
  and unchanged base reads through the owning module/session APIs.
- **Plan amendment:** R4.T3 now owns the budgeted frozen graph constructor and
  the smallest overlay integration required to preserve existing local edit
  behavior. Continue with focused tests, then rerun the saved-artifact vertical
  allocation discriminator before advancing to later load stages.
- **Verification state:** Implemented and focused-tested at the module layer.
  `cargo check -p phalcom-modules` passed, graph tests passed 7/7, and the saved
  artifact reports exact module materialization (1,138,500/1,138,500) and type
  graph import (1,393,544/1,393,544). The semantic stage now fails exact
  accounting; see the consultation packet. N/N-1, rollback, and retry have not
  run.

### P2.R4 T3 semantic-import allocation consultation — 2026-10-20

- **Packet:** [`ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md`](ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md)
- **Observed:** On `/tmp/artf001-universe.telosc`, module and type-graph stages
  pass exact accounting. Semantic import records 439,991,050 observed bytes
  against 317,818,249 ledger bytes, a +122,172,801-byte delta. The test fails at
  `phalcom-core/tests/artifact_allocation.rs:166`; later stages and N/N-1,
  rollback, and retry remain unrun.
- **Likely contributor:** `SemanticWorldV4::validate_with_allocation_budget`
  converts to V3 using `to_v3_for_validation`, which clones the world and
  callable-analysis collections without charging those allocations. The
  individual contribution is unmeasured, and other semantic importer paths may
  also allocate outside the ledger.
- **Prior work:** `perf/universe-characterization` fixed module graph
  materialization in `c09ee934f` and recorded exact V3 semantic import in
  `dde0e8ba9`. That V3 path has no V4-to-V3 validation projection, so it does
  not decide whether the new projection should be budgeted or removed.
- **Decision requested:** Choose between budget-accounting a complete V4-to-V3
  temporary projection and validating V4 in place through borrowed/shared
  validation logic. Confirm that the semantic allocation guard remains
  rejecting until the full importer is accounted. No semantic allocation fix
  has been attempted pending this review.
- **Status:** R4.T3 remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED. The module
  representation work is implemented; exact semantic import, later stages,
  N/N-1, rollback, retry, and P2 closure remain unproven.

### P2.R4 T3 borrowed V4 validation ruling — 2026-10-20

- **Incident:** [`ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md`](ARTF001.C1.P2.R4-semantic-import-allocation-consultation.md)
- **Decision:** Validate `SemanticWorldV4` in place through borrowed validation
  views shared with V3 common validation. Production validation must not build
  a complete V3 semantic-world copy. Preserve schema-specific structural
  authority and V4-only module execution and observed-import-phase checks.
- **Expanded boundary:** The same validator runs during V4 decode and semantic
  import, so the projection affects decode accounting as well as stage E.
  Measure V4 validation and decode-plus-validation separately, then rerun stage
  E and classify any residual delta.
- **Accounting contract:** Public budgeted validation/import APIs must reject
  while known successful-path allocations escape their transaction. A
  temporary diagnostic bypass is permitted only in test code. Keep independent
  allocator equality; do not adjust the ledger after observation.
- **Plan amendment:** R4.T3.1b records borrowed V1/V2 structural and callable
  views, temporary differential use of the old projection only in tests, no
  wire-schema/ABI change, and the required validation/decode/import gate order.
- **Status at ruling capture:** Decision settled; implementation and its
  focused evidence were pending at that point. The subsequent implementation
  and exact whole-load evidence are recorded in the following T3 section.

### P2.R4 T3 exact whole-load allocation and session boundary — 2026-10-20

- **Exact load gate:** With the saved canonical artifact, `cargo test -p
  phalcom-core --features allocation-diagnostics --test artifact-allocation --
  --nocapture --test-threads=1` passed. Current `N_image` is 450,922,288 bytes;
  the independent allocator observed the same total. Every named loader stage,
  query reconstruction phase, product-by-kind row, publication phase, and QX
  residual matched exactly. QueryKind census uses all 46 kinds. Fingerprint
  aggregation allocates zero bytes.
- **Semantic stages:** V4 validation matched at 2,557,138 bytes;
  decode-plus-validation matched at 121,856,466 bytes; semantic import matched
  at 316,234,625 bytes. Module materialization matched at 1,138,500 bytes and
  type graph import at 1,393,544 bytes. The full measured load, exact-budget
  import, and query reconstruction all completed.
- **Failure/retry:** N-1 returned typed cumulative `BudgetExceeded`; clean
  retry at N passed. The module materialization child-stage failure left the
  parent ledger equal to the preceding successful stages, and clean retry
  passed. The test also retains the direct semantic-import N-1 rollback check.
- **Session boundary:** Contract B is used: artifact loading ends at the
  immutable, complete `CanonicalUniverseFormalImage`; session construction is
  outside N and returns `SemanticWorkspaceSession` directly from that image.
  The focused gate verifies the loaded snapshot is complete/source-free and
  that both the initial and last-known-good session snapshots retain the same
  `Arc` identity. `from_image` constructs its inert source provider and reads
  only image-owned products. Session-local construction allocations are not
  included in the artifact-load budget.
- **Failpoints:** `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core
  --lib loaded_artifact_prepublication_failures_are_retryable -- --nocapture`
  passed: 1 test, 0 failures. The test exercises all four boundaries
  (`AfterModuleImport`, `AfterSemanticImport`, `AfterQueryImport`, and
  `AfterMetadataValidation`) and proves failed loads return no artifact and
  clean retries match the control identities.
- **Fixture compilation:** The 11 core unit-test fixture/API errors were
  repaired without production behavior changes. The semantic integration
  target's stale fixture representations were migrated by API family; its
  diagnostics fell from 451 errors to a successful `cargo check --test
  semantic`. No full semantic test run was performed.
- **T5 incident:** The selected `ordinary_image_session_keeps_universe_external_across_replacement`
  regression passes the initial snapshot identity checks, then panics during
  the first local update when `query_declaration_shell` failure handling tries
  to mutate an immutable canonical query. The scoped incident packet records
  the backtrace, evidence, alternatives, and decision requested. No production
  workaround was made. R4.T5 is blocked pending the ownership/fingerprint
  ruling; this does not invalidate the T3 loader transaction gates.
- **Status:** R4.T3 is COMPLETE / IMPLEMENTED / FOCUSED_TESTED. Exact
  allocation, N/N-1/retry, query classification, publication accounting,
  child rollback, and prepublication failpoint recovery are verified. R4.T2
  and the later R4 acceptance gates remain open.

The previous plans P2.R1, P2.R2, and P2.R3 remain preserved as implementation history. The current active plan is `ARTF001.C1.P2.R4`.

### P2.R4 R4.T5 canonical-query mutation incident — 2026-10-20

- **Trigger:** Ordinary local updates over an imported image reach
  `SemanticDb::set_state` for a key owned by the immutable canonical query
  image. This crosses the semantic-authority boundary; local correction would
  choose between query-root parity and session-bootstrap ownership.
- **Evidence:** The canonical-session overlay regression fails through
  `query_declaration_shell -> query_failure -> SemanticDb::set_state`. The
  semantic test target compiles, and the snapshot identity contract passes
  before the update failure. The exact key and query failure branch are not
  yet known.
- **Packet:**
  [`ARTF001.C1.P2.R4-T5-canonical-query-mutation-incident.md`](ARTF001.C1.P2.R4-T5-canonical-query-mutation-incident.md)
- **Decision requested:** Determine whether declaration-shell fingerprint /
  dependency parity or `from_image` bootstrap ownership should be corrected,
  and define non-mutating failure behavior for non-reusable immutable keys.
- **Historical status:** This blocker was resolved before revision
  `48c47753b`; retain the incident packet as evidence of the investigation.
  Do not treat its pending decision as current status.

### P2.R4 T2 current-schema artifact test recovery — 2026-10-20

- **Fixture migration:** Updated the semantic-artifact test fixture from the
  obsolete V3/V1 shapes to SemanticWorldV4, StructuralPrimaryContractsV2,
  CallableAnalysisRecordV2, current module authority collections, and complete
  module execution phase rows. External package authority now uses a root
  package module plus a nonroot `core` module, preserving the package receipt
  and relocated-owner assertions.
- **Current-schema validation:** Static-application denotations now resolve to
  the exact checked application row identified by their static body and
  ordinal. The canonical static-program validator already checks row ordering,
  signature, target, and argument validity.
- **Verification:** `cargo check -p phalcom-semantic --test semantic-artifact`
  passed. `cargo test -p phalcom-semantic --test semantic-artifact` passed
  20/20; `cargo test -p phalcom-compiler-artifact --test round_trip_test`
  passed 11/11; `blocked_nominal_shell_is_retired_while_alias_shell_remains_authoritative`
  passed 1/1; the named T2.6b nominal/alias query round-trip passed 1/1. The
  semantic integration target compiles after its broader fixture migration.
- **Status at this entry:** This closes the stale semantic-artifact test-target
  blocker and adds focused T2 evidence. R4.T2 remains IN_PROGRESS; no full
  semantic integration suite or full P2 exit gate has run. R4.T5 status is
  superseded by the later continuation record.

### P2.R4 and P8 continuation — 2026-10-20, revision 48c47753b

- **Session ownership fixes:** `SemanticWorkspaceSession::from_image` seeds
  `PreludeTypeMap` from the image snapshot. First local publication now wraps
  the canonical `DerivedConformanceRuleIndex` with a parent overlay instead
  of cloning it into local state. The affected canonical rule is
  `impl Universe.collections.iterator_laws local2 trait Iterable parameter428
  target2511`; the same canonical parent remains authoritative.
- **Regression correction:** The cold-session test now checks that a generic
  update publishes the edited binding as current and present. It retains the
  statistics, invalidation, and `Arc` identity checks; it does not weaken a
  semantic assertion.
- **Focused evidence:** `fresh_process_source_denied_artifact_compilation`
  passed 1/1 in 7.26s with `/tmp/artf001-universe.telosc` reused. Across three
  local revisions it found no canonical local query state, invalidation, or
  recomputation, and the canonical rule retained the same `Arc`. Required
  roots were current and validated: 336 `DeclarationShell`, 4
  `DeclarationSurface`, and 51 `CallableSignature`. Four slotmap modules and
  eight slotmap/runtime tests compiled with source denied and zero bootstrap.
  This was not a fresh artifact build or runtime execution.
- **Other focused evidence:** Query-policy 6/6; hostile missing/stale required
  root 1/1; semantic-artifact 20/20; artifact query import 4/4; compiler
  round-trip 11/11. The ordinary source-built session test target with three
  tests compiled successfully, then was canceled during canonical source bootstrap; it is
  incomplete, not a pass. Rollback, fail-closed, and source-canonical-root
  regressions were not rerun. Broad workspace checks were not run.
- **P8 status:** The approved `PERF003.C2.P8` plan is in progress. The
  instruction codec covers 128 opcodes with exhaustive conversion; bounded
  prototype encoding and the core layout projector are present. The final core
  library check passed, including the prototype owner-string-cap correction. There is no executable section, capture, materializer, or runtime
  proof yet. P2.R4 owns compiler-image completion; after that, continue with
  P8's source-free VM path. Do not mark P8 complete.
- **Delivery boundary:** The fetched remote remains at `48c47753b`. The user later requested committing and pushing the continuation; see the delivery record below.


### Continuation handoff — 2026-10-20

The user requested a handoff. Continue from
[`ARTF001.C1.P2.R4-implementer-handoff.md`](ARTF001.C1.P2.R4-implementer-handoff.md).
The compiler consumer fixes have focused source-denied compilation evidence;
source-free VM execution remains unimplemented and unproven. P8 has partial
instruction, prototype, constant-recipe, and projection infrastructure. The delivery core library check passed including the final prototype owner-string-cap
correction. Source changes are committed at `22a95bbb1` and `4807ecaf2`.
R4/P2 and P8 retain their separate incomplete lifecycle states.


### Delivery preparation — 2026-10-20

The user authorized committing and pushing until the consolidation worktree is clean.
`22a95bbb1` contains the image-session fixes and regressions; `4807ecaf2` contains
the partial executable DTO and projection infrastructure. The final core library
check passed in 7.17s, including the owner-string-cap correction; evidence is
`/tmp/artf001-delivery-check.log`. The documentation commit preserves the current
handoff. R4/P2 and P8 remain incomplete; no source-free VM runtime result is claimed.

### Bounded P2/P3 closure work — 2026-10-21

Repository callers confirm that `ImportedCompilerImage` and
`ArtifactImportTransaction` serve compatibility, validation, and
package-authority tests. Production publication uses the current native
importer and `CanonicalUniverseFormalImage`. The transaction is now
crate-private, and the compatibility view is hidden from generated public
documentation. R4.T6 remains open until the final caller/disposition gate is
reviewed.

`artifact_backed_compiler_output_runs_on_ordinary_vm` now calls
`VM::run_compiled` on the artifact-compiled SlotMap runtime program. The target
passed 1/1 in 114.50s and printed both SlotMap success markers followed by
`slotmap runtime tests complete`. This proves P2 runtime compatibility on the
ordinary source-backed VM; it is not source-free artifact VM evidence.

P2.R4 remains `IN_PROGRESS / PARTIAL`, so P3 G0 is false and no artifact VM
authority was published. The inherited validator no longer depends on an
already-published `LoadedUniverseArtifact`: it takes
`CanonicalUniverseFormalImage` plus `SemanticMetadataBundle` and returns the
private `ValidatedExecutableImage` after validation. Its contract test was
observed RED (`E0308`) and then passed 1/1. Full T1 proof limits and identity
coverage, T2–T10, and source-free VM execution remain open.

The current closure pass also completed the R4.T3 exact whole-load allocation
gate at `N=453,885,905` bytes with independent allocator/ledger equality,
`N-1` typed failure, rollback, and clean retry. R4.T4's explicit 46-kind query
policy, required-root closure, and query import tests passed (6/6, 1/1, and
4/4). R4.T5 ordinary image-session replacement, local overlay/invalidation,
shared-canonical isolation, and failure rollback tests passed (3/3 and 1/1).
R4.T6's compatibility importer API cleanup passed the semantic artifact suite
(20/20); its caller/disposition ledger still needs to be recorded before
promotion. The fresh current-head source-denied artifact compile passed with
three local revisions and the 4-module and 8-module SlotMap projects. The
artifact-compiled runtime smoke passed on the ordinary VM, and the ordinary
source CLI completed with both SlotMap success markers. The positive parity
test's first source-image attempt stopped on canonical Universe diagnostics.
After switching the source side to the normal source compiler entry point,
both sides compiled, but the comparator reported 12 differences: 41 Universe
modules on the source side versus 8 on the artifact side, plus differences in
runtime dependency and constructor-initialization products. The cause and the
required parity boundary remain unresolved; no compiler behavior or comparison
invariant has been weakened. Negative diagnostics and T8 broad gates remain
open; this does not yet constitute the P2 acceptance receipt.

The first required `cargo test --workspace --no-run` attempt failed compiling
`phalcom-semantic`'s `artifact-allocation` fixture, which used removed query
image V1 names and the old owner-key field. The fixture now targets the current
legacy closure-validation DTO (`QueryImageV2`) and keeps the same allocation,
cycle, duplicate, and invalid-index assertions. Its focused allocation test is
running; the workspace no-run gate must be repeated after that passes.
The repaired query-closure allocation discriminator passed 1/1; the workspace
no-run gate is now being repeated.
