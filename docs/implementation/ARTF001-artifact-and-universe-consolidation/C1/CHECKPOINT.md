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
| `ARTF001.C1.P2` | IN_PROGRESS | PARTIAL | FOCUSED_TESTED | Artifact/container and metadata substrate plus source-mode regressions are implemented. Live semantic/query import, project-qualified owner validation, and whole-load allocator accounting remain open. |
| `ARTF001.C1.P3` | PROPOSED | NOT_STARTED | UNVERIFIED | Do not start until P2 publishes reconstructed current-main semantic/query authority through the ordinary session. |
| `ARTF001.C1.P4` | PROPOSED | NOT_STARTED | UNVERIFIED | Core canonical artifact compiler and source-free compilation path parity. |
| `ARTF001.C1.P5` | PROPOSED | NOT_STARTED | UNVERIFIED | Universe source cutover from `phalcom-core/core/universe` to `packages/universe` retaining `ProjectIdentity::Universe`. |
| `ARTF001.C1.P6` | PROPOSED | NOT_STARTED | UNVERIFIED | Universal artifact resolution across CLI, REPL, VM bootstrap, and tests with persistent cache ergonomics. |

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
10. **Validation Precedes Semantic Publication (P2-INV-04, PENDING):** Container and section validation precedes returning an imported wire image. No publication into `SemanticSnapshot` or `SemanticDb` exists yet, and cross-section owner validation is partial.
11. **Exact Allocation Accounting (P2-INV-05, PENDING):** Whole-load decode, reconstruction, retained indexes, and publication still need measured N/N-1/retry evidence. Existing import-only ledger evidence is narrower.
12. **No Silent Compatibility Fallback (P2-INV-06):** Incompatible schemas, foreign native fingerprints, or malformed sections fail closed with typed errors.
13. **Unchanged Default Compiler Behavior (P2-INV-07):** Normal `ProgramCompiler` execution continues through the canonical source path. Tests opt into artifact loading explicitly.
14. **Universe Production Source Untouched (P2-INV-08):** `phalcom-core/core/universe` source authority remains untouched during P2.
15. **No P8 Executable Runtime Payload (P2-INV-09):** P2 does not serialize bytecode chunks, closures, or VM heap states.
16. **Project-Qualified Artifact Ownership (P2.R1-AMEND-01, PENDING):** Internal module identity is the exact `(StableProjectRef, module path)` pair. External semantic owners require an exact persistent project identity in the artifact allowlist. Session identities, wildcards, path-only matches, and unlisted projects are rejected.

## P2 verification evidence summary (current checkout)

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

### P2 audit findings still open

- `ImportedCompilerImage` contains `SemanticWorldV3` and `QueryImageV2` wire DTOs. It does not reconstruct them into current `TypeStore`, `StaticValueStore`, `SemanticSnapshot`, or typed `SemanticDb` products.
- `QueryImageV2` now has product-row references and fingerprints, and `phalcom-semantic::artifact::query_policy` classifies all 46 current query kinds. Live product reconstruction, dependency fingerprint validation, and project-qualified query owners remain open.
- `CompilerArtifactPayload::validate_cross_section` checks static-definition owners by path and selected module-query owners. It does not yet validate every nested stable owner against a project-qualified module inventory and external-project allowlist.
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
- **Plan impact:** The principal-architect amendment in `ARTF001-C1-P2-R2-plan.md` updates T2/T4 ownership and tests. The user selected exact `StableProjectRef::Package` bindings, including package name, version, and artifact fingerprint, in the existing workspace-owned catalog. This requires an implementation-backed catalog entry and exact matching; P2 cannot pass external-owner acceptance until that binding and its rejection cases are verified.
- **Verification required:** Exact existing-catalog relocation, source URI fingerprint mismatch rejection, unbound-but-allowlisted rejection, same-path modules under distinct projects, and confirmation that the imported session and overlay retain the same project IDs.

### Package artifact authority source

- **Trigger:** Exact `StableProjectRef::Package` registration requires a trusted source for `artifact_fingerprint`. The current container header carries package name and version only. The artifact digest covers the package reference encoded inside the artifact, so deriving the reference fingerprint from that digest would be self-referential. Project manifests also provide no artifact fingerprint.
- **Decision (2026-10-19):** The user selected a receipt from the caller's trusted package resolver. It binds the complete package/version/fingerprint reference, full validated `ArtifactDigest`, and an existing persistent `ResolvedProjectId`. Import validates the current artifact digest and header package/version, checks every receipt and target, then registers the exact binding in the supplied `ProjectUniverse`. Trust provisioning stays with the configured resolver; P2 adds no signature scheme or trust root. Missing authority fails closed.
- **Plan impact:** The principal architect updated the Level 2 amendment and T2 acceptance conditions in `ARTF001-C1-P2-R2-plan.md`. Do not derive a fingerprint from the container digest or trust an in-artifact receipt. Package-owner acceptance requires receipt, digest, exact-tuple, target, and conflict coverage.

### P2.R2 topology-fingerprint Level 2 amendment

- **Trigger:** Importing a package artifact against the required persistent resolver target failed with `module topology fingerprint mismatch`. The artifact module-world fingerprint was created with an empty graph; live topology validation hashes every project in the receiving `ProjectUniverse`, including graph-local IDs and import roots.
- **Decision (2026-10-19):** Give `ModuleWorldV2.topology_fingerprint` a portable, artifact-scoped meaning over the stable module identities, module kinds, parent/child structure, exposed children, and represented module links. Validate it against the wire rows. Then build the live workspace-scoped `ModuleTopology` using the caller's project graph and calculate its independent fingerprint. Receipt targets and unrelated receiving projects do not alter portable artifact validity; different receiving graphs may yield different live fingerprints.
- **Plan impact:** Principal-architect consultation required a Level 2 amendment to `ARTF001-C1-P2-R2-plan.md`. Module-world ABI V2 owns this meaning before release; if V2 has been published, bump `module_semantic_abi` before accepting it. P2 must add empty-graph/receipt-target equivalence, live-fingerprint difference, and represented-topology mutation rejection tests.
- **Verification required:** Exact same artifact loads under both valid graphs; returned module-world fingerprint reflects the receiving graph; changed represented identities, edges, or exposures fail wire-topology verification.
- **Status:** Adopted; implementation and focused tests remain open.

## Current API surface (not yet a P3 takeover)

P2 currently exposes the following container and staging API in `phalcom-semantic::artifact`. `ImportedCompilerImage` is not yet a published current-main semantic snapshot or query database:

```rust
// Binary serialization & validation
pub fn encode_artifact(image: &CompilerArtifactImage) -> Result<Vec<u8>, ArtifactError>;
pub fn decode_and_validate_artifact(
    bytes: &[u8],
    limits: &ValidationLimits,
    expected_identity: Option<&CanonicalCompatibilityIdentity>,
    budget: &mut AllocationBudget,
) -> Result<CompilerArtifactPayload, ArtifactError>;

// Staging into an ImportedCompilerImage data container
pub fn import_compiler_image(
    payload: CompilerArtifactPayload,
    limits: &ValidationLimits,
    budget: &mut AllocationBudget,
) -> Result<ImportedCompilerImage, ArtifactError>;

// Staged transactional builder
pub struct ArtifactImportTransaction<'a> { ... }
```

## Active plan

`ARTF001.C1.P2.R2` closure remediation — reconstruct artifact products into current semantic stores and the workspace session, complete query persistence and owner validation, and prove whole-load allocation behavior. The T2/T4 project mapping is owned by the supplied workspace graph. Exact Package bindings require receipts from the caller's trusted resolver. P3 remains proposed until P2 is accepted.
