# ARTF001.C1.P2.R2 — Implementation Handoff: Complete Artifact Import and Semantic Authority Publication

- **Program:** `ARTF001`
- **Checkpoint:** `ARTF001.C1`
- **Phase:** `ARTF001.C1.P2`
- **Corrective slice:** `ARTF001.C1.P2.R2`
- **Status:** `IN_PROGRESS`
- **Completion:** `PARTIAL`
- **Verification:** `FOCUSED_TESTED`
- **Repository:** `aureat/phalcom-lang`
- **Branch:** `feat/artifact-universe-consolidation`
- **Starting revision:** `9f0d2f66af0196739dc562b0df19f140576ec861`
- **Upstream main anchor:** `39334961e61cc3b9e251d03bcd348bceb50abbc6`
- **PERF003 reference revision:** `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94`
- **Successor phase:** `ARTF001.C1.P3` — BLOCKED

---

# 1. Mission and required end state

Complete `ARTF001.C1.P2` to its originally intended architectural and functional standard, including the corrections established by the P2 audit and P2.R1 project-identity amendment.

The objective is **not merely to make artifact decoding, wire validation, or transport round-trips pass**.

The objective is to establish a complete, portable, transactionally imported compiler-image substrate that reconstructs and publishes **current-main live semantic authority** through the existing semantic session and query database.

At completion, the repository must support the following operational path:

```text
Portable .telosc artifact bytes
    |
    v
Container framing, integrity and compatibility validation
    |
    v
MODULE_WORLD V2
SEMANTIC_WORLD V3 (unless independently versioned)
QUERY_IMAGE V3
METADATA
    |
    v
Exact project-qualified identity validation
    |
    v
Private, allocation-budgeted import transaction
    |
    +--> CanonicalExternalModuleWorld
    |
    +--> Current-main TypeStore
    |
    +--> Current-main StaticValueStore
    |
    +--> Reconstructed semantic products and proofs
    |
    +--> Typed QueryKey / SemanticProduct pairs
    |
    +--> Verified query dependency graph
    |
    v
Validated immutable imported canonical generation
    |
    v
Atomic publication
    |
    v
SemanticWorkspaceSession
    |
    +--> ONE SemanticDb
    +--> ONE current-main semantic type system
    +--> ONE coherent SemanticSnapshot
    +--> Immutable imported canonical authority
    +--> Ordinary local revision/query overlay
    +--> Correct invalidation and recomputation
```

The importer must produce objects usable directly by the existing semantic compiler. Importing into a new session must not require parsing, linking, or semantically analyzing Universe source again.

## 1.1 Definition of success

P2 is complete only when all of the following are demonstrated:

1. The binary artifact carries project-qualified stable identities and complete, validated ownership evidence.
2. The importer reconstructs actual current-main types, static values, semantic aggregates, callable/proof products, and other required compiler-image products.
3. Imported query roots resolve to actual typed `SemanticProduct` values under their correct current-main `QueryKey`s.
4. Query reuse follows the semantic persistence contract. Source-dependent, open-world, stale, failed, and unproven products cannot become improperly reusable.
5. Reconstructed canonical authority is installed into the existing `SemanticWorkspaceSession` and its ordinary `SemanticDb`/`SemanticSnapshot` model.
6. Artifact-backed session construction is demonstrably source-free.
7. All cross-section and nested stable identities are validated before publication.
8. The complete bytes-to-publication operation is transactionally atomic and allocation-budgeted.
9. Exact N/N−1/retry allocation tests, hostile-artifact tests, source-free tests, and local-overlay tests pass.
10. Existing source-based compiler behavior remains correct.

**No single transport round-trip test, query-kind inventory, or green artifact-crate suite substitutes for these requirements.**

## 1.2 Explicit non-goals

This corrective slice must not:

- Implement P3's canonical Universe artifact producer or full production-image projection.
- Implement P4's complete user-package compilation via imported Universe authority.
- Change the production Universe source location.
- Switch the default compiler, CLI, REPL, or VM to automatic artifact loading.
- Implement executable bytecode, VM heap, or closure serialization.
- Redesign the language's current type or static-value semantics.
- Merge the PERF003 branch wholesale into main.
- Introduce a second semantic engine, type system, or query database.

These remain separate phases.

P2 must nevertheless provide the real importer and session-construction capability required for P3 and P4.

---

# 2. Starting state and evidence

The starting commit `9f0d2f66` contains:

- `phalcom-compiler-artifact` binary framing, codecs, digests, sections, and bounded transport DTOs.
- `ModuleWorldV1` and its current Universe-specific loader.
- `SemanticWorldV3` transport structures.
- `QueryImageV2` with query product-row references, input/product fingerprints, and dependency evidence.
- Exhaustive mapping of all 46 current semantic query kinds to portable query kinds.
- Explicit query transport dispositions.
- Durable metadata transport through `phalcom-type-meta`.
- Budget-aware metadata decoding and validation infrastructure.
- Initial artifact import transaction infrastructure.
- Compatibility validation including `query_fingerprint_abi`.

Current reported passing results:

| Existing check | Recorded result |
|---|---|
| `cargo test -p phalcom-type-meta` | 30 passed |
| `cargo test -p phalcom-compiler-artifact` | 86 passed |
| `cargo test -p phalcom-semantic --test semantic-artifact` | 3 passed |
| `cargo test -p phalcom-semantic --lib artifact::query_policy` | 2 passed |
| `cargo test -p phalcom-core --test cli-smoke` | 3 passed |
| SlotMap runtime program | PASS |
| Parse package focused integration | PASS |
| `cargo fmt --all -- --check` | PASS |
| Git diff whitespace checks | PASS |

These are starting evidence, not P2 completion certification.

## 2.1 Known incomplete implementation

At the starting revision:

```rust
pub struct ImportedCompilerImage {
    pub external_world: Arc<CanonicalExternalModuleWorld>,
    pub metadata_bundle: SemanticMetadataBundle,
    pub semantic_world: SemanticWorldV3,
    pub query_image: QueryImageV2,
}
```

This structure still retains semantic and query **wire DTOs** rather than a fully reconstructed and published live compiler image.

Likewise, the import transaction effectively performs:

```rust
self.staged_semantic_world = Some(payload.semantic_world);
self.staged_query_image = Some(payload.query_image);
```

Moving those DTOs avoids some cloning but does not complete semantic reconstruction.

The existing `SemanticWorkspaceSession::with_workspace()` constructs its baseline from `canonical_universe_semantic_baseline()`, which is source-backed.

The current module-world loader reconstructs module identities with `ModuleId::universe(path)`, losing arbitrary project-qualified module ownership.

All these limitations must be addressed.

---

# 3. Mandatory architectural invariants

These invariants take precedence over convenience, passing a particular test, or reducing the implementation scope.

## INV-01 — Current-main semantic authority is singular

There must be exactly one authoritative live semantic model.

The current implementations of:

- `TypeStore`
- `StaticValueStore`
- `SemanticDb`
- `SemanticProduct`
- `SemanticSnapshot`
- `SemanticWorkspaceSession`

remain authoritative.

The artifact crate owns wire transport and bounded validation, not an independent runtime semantic representation.

**Forbidden:** introducing artifact-specific live type IDs, a competing semantic product store, or a parallel query engine.

## INV-02 — Stable identities remain project-qualified

A module's durable identity is its complete `StableModuleRef`:

```rust
StableModuleRef {
    project: StableProjectRef,
    path: Box<[Box<str>]>,
}
```

Neither module paths nor textual owner strings alone are sufficient for identity.

Use the current `phalcom-type-meta` stable identity ontology.

Never replace it with a second artifact-specific identity system.

## INV-03 — Only validated persistent external identities may be referenced

An artifact may reference external semantic owners only when their complete persistent `StableProjectRef` is present in the explicit external-project allowlist.

Validation must reject:

- Session identities.
- Wildcards.
- Unknown or unlisted project identities.
- Partial project matches.
- Path-only matches.
- Unresolved internal owners.
- Ambiguous ownership.
- Project identities with incorrect artifact/source fingerprints.

A listed external project identity is permission to reference that exact external authority; it is not permission to invent arbitrary declarations within it. Required external semantic contracts must still be validated.

## INV-04 — No store-local identity crosses the wire

Persistent wire identities cannot contain unrelocated:

- `TypeId`, `KindId`, or other store-local arena indices.
- VM pointers or object addresses.
- Ephemeral solver IDs.
- Local workspace/session IDs.
- In-process-only fingerprints presented as portable identities.

Artifact-local row numbers are allowed only as transport references whose meaning is validated and relocated during import.

Owner-relative stable ordinals are permitted only where their established stability contract is proven.

## INV-05 — Validation precedes publication

The importer must complete all compatibility, structural, semantic owner, relocation, product, and dependency validation before publishing any resulting authority.

A failed import must leave the previous session, snapshot, query database, stores, and caller allocation budget unchanged.

## INV-06 — No unsafe query reuse

Transportability does not establish reusability.

A product is reusable only if its query family and specific instance satisfy the current-main semantic reuse contract.

The importer must preserve the open-world and dependency-closure restrictions established in certified PERF003.

## INV-07 — No silent recovery or downgrade

Invalid or incompatible artifacts must fail with typed errors.

Do not silently:

- Substitute source compilation inside an explicit artifact import.
- Invent missing products.
- Drop required dependency edges.
- Replace unavailable proofs with successful placeholders.
- Mark a query `Ready` without a typed validated product.
- Decode a rejected old schema as a new one.

Source fallback, where later introduced, must be explicit and separately observable.

## INV-08 — Exact allocation-accounting contract

All owned allocations included in the supported load boundary must be covered by the transaction ledger.

The required tests are:

```text
Measure supported whole-load requests = N.

Budget N:
    PASS
    remaining = 0

Budget N - 1:
    FAIL
    typed AllocationError::Exhausted
    no published state mutation
    no committed caller-budget debit

Retry with N:
    PASS
    equivalent published result
```

The measurement must cover the whole supported bytes-to-publication path, not an import-only subset.

## INV-09 — Source-free artifact-backed construction

Artifact-backed semantic-session construction must not invoke canonical Universe parsing, linking, or semantic reanalysis.

It must also avoid indirectly initializing the source-backed canonical baseline merely to obtain default session state.

## INV-10 — Production Universe remains unchanged during P2

`phalcom-core/core/universe` remains the production source authority for P2.

The future `packages/universe` cutover belongs to P5.

Stable semantic identity must not depend on the physical filesystem location of Universe source.

## INV-11 — No regression of existing compiler behavior

Existing source-mode CLI, module linker, semantic analysis, SlotMap runtime, and Parse integration must remain correct.

Do not weaken semantic checking, match coverage, visibility, native metadata, or bootstrap requirements to satisfy artifact tests.

## INV-12 — One atomic publication boundary

The importer may build multiple private staging objects, but publication into a visible semantic session must be atomic.

No partially imported generation may be externally visible.

---

# 4. Architecture and implementation requirements

## 4.1 Required ownership boundaries

Implement the following responsibilities in their owning crates.

| Crate | Responsibility |
|---|---|
| `phalcom-type-meta` | Canonical stable identities, durable metadata, budgeted decoding and validation |
| `phalcom-compiler-artifact` | Wire schemas, section validation, referential-integrity prechecks, fingerprints, bounded decoding |
| `phalcom-modules` | Current-main linked module-world projection/reconstruction and module identity relocation |
| `phalcom-semantic` | Live type/static relocation, semantic product reconstruction, query persistence policy, session/database publication |
| `phalcom-core` | Higher-level artifact production and downstream compilation integration in P3/P4 |

Do not move semantic decision-making into the wire crate merely because it has access to serialized rows.

The wire crate may validate transport contracts; the semantic crate must validate current-main semantic equivalence and query reuse eligibility.

---

# 5. T1 — Complete project-qualified module and query identity

**Priority:** First
**Blocking:** T2, T3, T4

## Implementation targets

Primary files:

- `phalcom-compiler-artifact/src/module_world.rs`
- `phalcom-compiler-artifact/src/query.rs`
- `phalcom-compiler-artifact/src/payload.rs`
- `phalcom-compiler-artifact/src/container.rs`
- `phalcom-compiler-artifact/src/header.rs`
- `phalcom-compiler-artifact/src/identity.rs`
- `phalcom-modules/src/compiler_artifact.rs`
- `phalcom-modules/src/identity.rs`
- `phalcom-semantic/src/artifact/mod.rs`
- `phalcom-semantic/src/db/key.rs`

## Required schema

Introduce `ModuleWorldV2`.

Each module row must carry its complete stable identity.

Conceptual shape:

```rust
pub struct ModuleV2 {
    pub identity: StableModuleRef,

    // Existing linked/unlinked/module metadata fields.
    // Preserve their current semantics.
}

pub struct ModuleWorldV2 {
    pub entry: ArtifactModuleId,
    pub initialization_order: Vec<ArtifactModuleId>,
    pub modules: Vec<ModuleV2>,
    pub semantic_rows: Vec<SemanticRowV2>,

    // Exact persistent project identities which may own
    // referenced external semantic objects.
    pub external_projects: Vec<StableProjectRef>,

    pub topology_fingerprint: u64,
}
```

This is a conceptual shape, not an instruction to blindly copy fields. Preserve all current required module-world products.

`external_projects` must be:

- Canonically ordered.
- Unique.
- Composed exclusively of permitted persistent identity forms.
- Validated against the artifact's actual ownership and compatibility contract.

Use exact `StableProjectRef` equality, including fingerprint-bearing fields.

## Required query schema

Introduce `QueryImageV3`.

Replace string-only owner identity with typed project-qualified owner identity appropriate to each query family.

A recommended design uses a dedicated portable owner-key enum whose variants retain:

- Project-qualified module identity.
- Declaration identity.
- Callable identity.
- Field identity.
- Implementation/conformance identity.
- Static-domain/case identity.
- Other current query-key owners.

The full design must cover all 46 current query kinds without collapsing distinct owner types into the same string.

Preserve product-row references, input fingerprints, product fingerprints, and dependency evidence.

Every root and dependency owner must resolve through the same identity catalog.

## Required validation

- A module is identified by its full stable module reference.
- All artifact-local module references resolve to valid module rows.
- No two module rows have the same full stable identity.
- Same-path modules from different projects remain distinct.
- Every internal semantic owner resolves to the exact module row.
- Every external semantic owner resolves to an exact allowlisted persistent project.
- Query root owner matches the owner of its referenced product row.
- Dependency keys and aggregate owners are validated using the same identity rules.

Do not broaden `phalcom-modules` runtime support for unrelated projects merely to pass the new identity tests. If the supported P2 profile is Universe-specific, preserve that profile boundary while making its wire ownership model correct and future-extensible.

## Schema and ABI requirements

- Bump module-world section schema to V2.
- Bump query-image section schema to V3.
- Bump `module_semantic_abi`.
- Bump `query_fingerprint_abi`.
- Bump `semantic_artifact_abi` if semantic owner interpretation changes.
- Preserve outer container schema V2 unless a genuine container framing change requires otherwise.
- Preserve `stable_identity_schema` if existing stable reference encodings are unchanged.
- Reject old required section schemas with typed compatibility errors.
- Update all fixtures and header/section consistency checks.

Audit `TeloscPackageIdentity::from(&StableProjectRef)`.

Its existing lossy mapping must not be used as a canonical validation or reversible identity conversion. Introduce a fallible or explicitly scoped conversion if required.

## Required tests

Create or extend:

`phalcom-compiler-artifact/tests/project_qualified_identity_test.rs`

Include:

```text
same_path_in_distinct_projects_remains_distinct
duplicate_full_module_identity_rejected
missing_internal_module_owner_rejected
unlisted_external_project_rejected
exact_allowlisted_external_project_accepted
external_project_fingerprint_mismatch_rejected
session_project_owner_rejected
query_owner_project_mismatch_rejected
legacy_module_schema_rejected
legacy_query_schema_rejected
```

**T1 PASS:** Every test passes and no path-only matching remains in authoritative semantic owner validation.

---

# 6. T2 — Reconstruct live current-main semantic authority

**Priority:** Second
**Blocking:** T4

## Implementation targets

Primary:

- `phalcom-semantic/src/artifact/mod.rs`
- New focused modules under `phalcom-semantic/src/artifact/`
- `phalcom-semantic/src/types/store/`
- `phalcom-semantic/src/static_value/`
- `phalcom-semantic/src/snapshot.rs`

Reference:

- `phalcom-compiler-artifact/src/semantic/world.rs`
- `phalcom-compiler-artifact/src/semantic/type_store.rs`
- `phalcom-compiler-artifact/src/semantic/static_value.rs`
- `phalcom-compiler-artifact/src/semantic/world/proof.rs`
- `phalcom-compiler-artifact/src/semantic/world/explanation.rs`

## Required architecture

Introduce private reconstruction components with explicit responsibilities.

Suggested organization:

```text
phalcom-semantic/src/artifact/
    mod.rs
    identity.rs
    relocate.rs
    semantic_import.rs
    query_import.rs
    session_import.rs
    allocation.rs
    query_policy.rs
```

The exact file structure may vary, but responsibilities must remain separated.

Use a staged relocation context containing:

- Validated project/module owner catalog.
- Mapping from stable type identities to newly constructed current-store IDs.
- Mapping for static-value DAG nodes.
- Stable declaration/callable/implementation resolution.
- Reconstructed semantic aggregate tables.
- Type and static-value stores.
- Validated semantic product indexes.

Suggested conceptual API:

```rust
struct SemanticRelocationContext {
    // Private maps and reconstruction state.
}

struct ReconstructedCompilerAuthority {
    external_world: Arc<CanonicalExternalModuleWorld>,
    store: Arc<TypeStore>,
    static_values: Arc<StaticValueStore>,

    // Current-main semantic aggregates and product tables.
    // No wire DTOs as live authority.
}
```

These are conceptual types. Reuse existing current-main types rather than defining alternative representations.

## Required reconstruction strategy

1. Validate the stable ownership catalog.
2. Establish stable-to-local relocation maps.
3. Reconstruct types and static domains into a fresh `TypeStore`.
4. Reconstruct static-value DAGs into the current `StaticValueStore`.
5. Resolve declaration, callable, field, associated family, variant, conformance, and static owner references.
6. Reconstruct structural, callable, conformance, proof, and operational products required by the supported compiler-image profile.
7. Validate cross-product references and semantic invariants.
8. Produce a complete private semantic authority for later query/session publication.

Reconstruction must preserve current-main rules involving:

- Tuple/record kinds and structural type shapes.
- Type parameters and scoped types.
- Static domains and static constants.
- Data and enum declaration semantics.
- Associated static domains.
- Callable definitions and signatures.
- Dispatch/conformance evidence.
- Match and executable semantic proof requirements.
- Module semantic graphs and runtime dependency requirements, where required by the imported profile.

Do not assume that decoding a wire row directly produces the correct current-main type or proof.

Reconstruction must relocate references and validate ownership.

## Explicit prohibition

Do not make the following count as success:

```rust
struct ImportedCompilerImage {
    semantic_world: SemanticWorldV3,
    query_image: QueryImageV3,
}
```

Those may exist as transient staging inputs.

The completed imported authority must expose reconstructed current-main products, not merely retain serialized descriptions.

## Required tests

Create:

`phalcom-semantic/tests/artifact_reconstruction_test.rs`

Required test cases:

```text
type_store_reconstructed_from_portable_graph
static_values_reconstructed_with_exact_semantics
declaration_and_callable_owners_relocated
associated_static_domain_owners_relocated
semantic_cross_references_resolve
missing_semantic_owner_rejected
wrong_project_semantic_owner_rejected
foreign_store_local_identity_not_reused
reconstructed_products_match_source_semantics
required_semantic_product_missing_fails_closed
```

Use nontrivial fixtures including nested generic types, structural record/tuple forms, data/enum products, static values, callable signatures, and meaningful cross-module dependencies.

**T2 PASS:** Reconstructed objects are native current-main semantic objects; their semantics and owner identities match the source-produced reference; invalid graphs cannot produce publishable authority.

---

# 7. T3 — Reconstruct typed query products and enforce persistence safety

**Priority:** Second, after T1
**Blocking:** T4

## Implementation targets

- `phalcom-semantic/src/artifact/query_policy.rs`
- `phalcom-semantic/src/db/key.rs`
- `phalcom-semantic/src/db/product.rs`
- `phalcom-semantic/src/db/state.rs`
- `phalcom-semantic/src/db/dependency.rs`
- `phalcom-semantic/src/db/mod.rs`
- `phalcom-compiler-artifact/src/query.rs`

## Architectural decision

**Separate product location from query reuse eligibility.**

A product being represented in `SemanticWorldV3` is not sufficient reason to publish its producing query as reusable.

Introduce explicit classification along the following conceptual dimensions:

```rust
enum ArtifactProductLocation {
    ModuleWorld,
    SemanticWorld,
    DependencyEvidence,
    Omitted,
}

enum QueryReuseEligibility {
    ImmutableStructural,
    ConditionalClosedWorld,
    RecomputeInLocalContext,
    NotReusable,
}
```

The actual implementation may use equivalent enums or a combined exhaustive policy table.

Every one of the 46 current query kinds must have an explicit policy.

## Required PERF003 reconciliation

Review certified PERF003's:

- `CanonicalQueryPersistence`
- `canonical_query_persistence`
- `QueryFreezeRejection`
- Structural dependency closure rules.
- Negative local candidate-set restrictions.
- Stale validation restrictions.
- Query product/fingerprint matching requirements.

The following families require particular care:

```text
GenericConformance
ConformanceDispatch
TraitDispatchBucket
CallableBody
CheckedStaticBody
```

PERF003 identified these as open-world-sensitive.

Do not automatically mark their imported roots reusable merely because the wire product is present.

The current-main semantics may justify narrower forms of reuse, but each such relaxation requires a written proof, correct dependency invalidation, and a discriminating test.

## Required importer behavior

For each eligible root:

1. Convert its typed portable owner into the exact current-main `QueryKey`.
2. Resolve the referenced row to a reconstructed current-main `SemanticProduct`.
3. Verify the expected query family matches the product variant.
4. Verify that the product actually belongs to the specified owner.
5. Independently validate or recompute its product fingerprint using the approved current-main fingerprint contract.
6. Validate the input fingerprint and provenance.
7. Resolve dependency indices into typed `QueryKey`s.
8. Verify observed dependency fingerprints.
9. Verify dependency closure and freshness.
10. Publish only if the query reuse policy permits that product in the imported generation.

Products that are useful as canonical semantic snapshot authority but not safe as reusable query results must remain available through the appropriate snapshot/aggregate representation without being improperly inserted as `Ready` query products.

Do not silently discard required product data just because its corresponding query is not reusable.

## Single database requirement

Use the existing:

```rust
SemanticDb
QueryKey
QueryState
SemanticProduct
DependencyEdge
DependencyIndex
```

In particular, preserve the semantics of:

```rust
SemanticDb::publish_product_ready(...)
SemanticDb::is_reusable(...)
SemanticDb::validate_reuse(...)
SemanticDb::invalidate(...)
```

Existing publication APIs may be reused inside private construction or an appropriate transaction, but must not be invoked piecemeal on an externally visible session before all import checks pass.

Do not add a second query cache beside `SemanticDb`.

## Required tests

Create:

`phalcom-semantic/tests/artifact_query_import_test.rs`

Required tests:

```text
all_46_query_kinds_have_explicit_import_policy
typed_query_key_reconstructs_correct_owner
typed_semantic_product_matches_query_family
wrong_in_bounds_product_row_rejected
wrong_product_fingerprint_rejected
wrong_input_fingerprint_rejected
unknown_dependency_owner_rejected
dependency_fingerprint_mismatch_rejected
dependency_cycle_without_proof_rejected
non_ready_query_state_not_imported_as_ready
open_world_query_not_unconditionally_reused
new_local_conformance_invalidates_affected_queries
new_local_dispatch_candidate_invalidates_affected_queries
local_overlay_preserves_immutable_canonical_products
```

**T3 PASS:** Every imported reusable root resolves through `SemanticDb` as a correctly typed current-main product with verified dependencies, and open-world changes cannot preserve stale conclusions.

---

# 8. T4 — Atomic publication through the ordinary semantic session

**Priority:** Third
**Blocking:** T5

## Implementation targets

- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/canonical_baseline.rs`
- `phalcom-semantic/src/db/mod.rs`
- `phalcom-semantic/src/artifact/mod.rs`
- New artifact session-integration module

## Required architecture

Provide a dedicated artifact-backed session construction route.

Conceptual API:

```rust
impl SemanticWorkspaceSession {
    pub fn from_imported_canonical(
        imported: ValidatedImportedCanonicalGeneration,
        workspace: WorkspaceId,
    ) -> Result<Self, ArtifactImportError> {
        // Construct from validated current-main authority.
        // Must not invoke source-backed Universe construction.
    }
}
```

The exact API may differ.

The required behavior may not.

The session must own:

1. The reconstructed canonical semantic stores.
2. The canonical module world.
3. Current-main semantic aggregate tables.
4. A single `SemanticDb` containing validated imported reusable roots.
5. A coherent `SemanticSnapshot`.
6. Ordinary local overlay/invalidation machinery.
7. The same workspace identity and revision semantics used by normal sessions.

## Imported generation design

Recommended ownership:

```text
Immutable imported canonical generation
    |
    +--> module products
    +--> type/static semantic stores
    +--> structural/callable products
    +--> validated reusable query roots
    +--> dependency evidence
    |
    v
SemanticWorkspaceSession
    |
    +--> canonical generation (shared immutable)
    +--> local semantic revision state
    +--> ordinary current-main SemanticDb
    +--> ordinary current-main SemanticSnapshot
```

An imported generation is an immutable authority boundary, not a second semantic database.

Use shared immutable ownership where appropriate to avoid unnecessary deep copies.

Local revisions must not mutate imported canonical data.

## Atomicity implementation

The publication workflow must be:

```text
BEGIN
    Validate container and identity
    Decode DTOs under private budget
    Reconstruct module and semantic authority
    Reconstruct and validate typed queries
    Build dependency indexes
    Construct complete proposed snapshot/session state
    Validate all publication preconditions
    Finalize budget transaction
    Atomically install complete generation
COMMIT
```

All fallible work must happen before the final visible installation.

For an existing session, use a staging candidate or rollback-safe transaction preserving the prior state. Avoid partially mutating the active session.

For fresh construction, keep the incomplete session private and return it only after full validation.

## Source-free requirement

`SemanticWorkspaceSession::with_workspace()` currently uses `canonical_universe_semantic_baseline()`.

Do not call it as a shortcut inside the artifact-backed constructor.

Likewise, do not call `UniverseSourceProvider` in a mode that reparses canonical source to obtain semantic authority.

It is acceptable for the new constructor to initialize ordinary local workspace state or non-source-dependent primitives. It is not acceptable to rebuild the canonical Universe from source.

Add instrumentation that distinguishes:

```text
canonical_source_parse_count
canonical_link_count
canonical_semantic_analysis_count
artifact_import_count
artifact_relocated_product_count
```

For artifact-backed construction, the first three counters must be zero within the measured construction boundary.

## Required tests

Create:

`phalcom-semantic/tests/artifact_session_import_test.rs`

Required tests:

```text
fresh_session_constructs_from_imported_authority
imported_session_contains_current_main_type_store
imported_session_contains_typed_ready_query_products
imported_session_exposes_coherent_semantic_snapshot
artifact_construction_does_not_build_source_baseline
artifact_construction_does_not_parse_universe
artifact_construction_does_not_link_universe
artifact_construction_does_not_reanalyze_universe
local_declarations_overlay_imported_authority
local_invalidation_recomputes_affected_queries
unaffected_canonical_products_remain_reusable
failed_import_preserves_previous_snapshot
failed_import_preserves_previous_query_database
failed_import_preserves_previous_stores
publication_failure_rolls_back_atomically
```

Require a meaningful representative canonical fixture. Empty products or a single synthetic module cannot be the only evidence.

**T4 PASS:** A fresh source-free semantic session exposes correct imported typed semantic authority and correctly processes local revisions through the ordinary query engine.

---

# 9. T5 — Whole-load allocation, validation and rollback closure

**Priority:** Fourth
**Blocking:** Final acceptance

## Implementation targets

- `phalcom-type-meta/src/allocation.rs`
- `phalcom-type-meta/src/budgeted.rs`
- `phalcom-type-meta/src/encode.rs`
- `phalcom-type-meta/src/validate.rs`
- `phalcom-compiler-artifact/src/container.rs`
- `phalcom-compiler-artifact/src/module_world.rs`
- `phalcom-compiler-artifact/src/query.rs`
- `phalcom-compiler-artifact/src/payload.rs`
- `phalcom-semantic/src/artifact/`
- `phalcom-semantic/src/db/`

## Required whole-load boundary

Measure and account for:

```text
bytes
  → container view / directory
  → module DTO decode
  → semantic DTO decode
  → query DTO decode
  → metadata DTO decode
  → metadata validator scratch
  → project/owner validation
  → live module graph reconstruction
  → live type/static reconstruction
  → semantic aggregate reconstruction
  → typed query products
  → dependency indexes
  → retained publication structures
  → final session publication
```

## Allocation rules

- Use one parent transaction for the complete supported load.
- Use child budgets for independently bounded suboperations.
- Do not commit child reservations until that stage succeeds.
- Avoid unmetered `clone()`, `collect()`, `BTreeSet`, `BTreeMap`, `Box`, `Arc`, formatting, and temporary graph construction inside the measured boundary.
- Account for validation scratch structures as well as retained products.
- Treat all allocation-related failures as typed allocation failures.
- Prevent integer overflow in allocation-size computations.
- Keep the caller budget unchanged on failed import.
- Avoid redundant full semantic graph copies.

The starting implementation's module-world validator uses allocating sets without an allocation budget. These operations must be converted to budgeted structures or allocation-free canonical-order validation.

## Specific error-propagation correction

`MetadataValidationError::Allocation` must propagate through:

```text
MetadataValidationError::Allocation
    → MetadataDecodeError::Validation
    → MetadataTransportError::Decode
    → ArtifactError::Allocation
```

Do not translate this into `ReferentialIntegrity`.

Other allocation paths must receive the same treatment.

## Metadata decoder hardening

The new `phalcom-type-meta-derive` decoder must be audited against current-schema Serde behavior.

In particular:

- Exact handling of `serde(default)` and supported default forms.
- Required-field behavior.
- Duplicate fields.
- Unknown fields.
- Escaped Unicode and surrogate pairs.
- Numeric bounds.
- Static integers.
- Tuple and enum representations.
- Special custom metadata types.
- Consistent nesting and resource limits.

Avoid using heuristic token-string inspection of Serde attributes where a structured `syn` attribute parser is required for correctness.

Unsupported attribute semantics should cause a clear compile-time error or require an explicit custom implementation, rather than silently generating incorrect decoding behavior.

## Required measurements

Record at minimum:

```text
whole_load_budget_requested_bytes
whole_load_allocator_requested_bytes
whole_load_retained_bytes
whole_load_peak_live_bytes
whole_load_temporary_bytes
reconstructed_type_count
reconstructed_static_value_count
reconstructed_semantic_product_count
imported_ready_query_count
imported_nonreusable_product_count
dependency_edge_count
canonical_source_parse_count
canonical_link_count
canonical_semantic_analysis_count
```

Use allocator instrumentation isolated from unrelated test-harness allocations.

Define the accounting contract precisely: requested allocator bytes and budget charges must reconcile for the measured operations. Retained memory and cumulative allocations must be reported separately.

Do not claim exact allocation certification using a reservation ledger alone.

## Required tests

Create:

`phalcom-semantic/tests/artifact_allocation_test.rs`

Required tests:

```text
whole_load_exact_n_succeeds
whole_load_n_minus_one_fails_typed_exhausted
whole_load_retry_n_succeeds
whole_load_failure_keeps_caller_budget_uncommitted
whole_load_failure_preserves_session_generation
metadata_validator_allocation_failure_remains_typed
module_validation_allocations_are_accounted
semantic_relocation_allocations_are_accounted
query_dependency_index_allocations_are_accounted
publication_allocations_are_accounted
allocator_requests_reconcile_with_ledger
no_unmetered_full_graph_clone
```

Add differentiated allocation-failure injection at decode, reconstruction, dependency-index, and publication-preparation boundaries.

Create or extend:

`phalcom-type-meta/tests/metadata_budgeted_equivalence_test.rs`

Required tests:

```text
budgeted_decoder_matches_serde_current_schema
budgeted_decoder_accepts_valid_unicode
budgeted_decoder_preserves_default_fields
budgeted_decoder_preserves_custom_type_semantics
budgeted_decoder_rejects_duplicate_fields
budgeted_decoder_rejects_malformed_integer
budgeted_decoder_rejects_malformed_escape
budgeted_decoder_rejects_excess_nesting
budgeted_decoder_exhaustion_is_atomic
```

**T5 PASS:** Measured whole-load allocation requests reconcile with budget charges, exact N succeeds, N−1 fails with the prescribed typed error and no visible publication, and retry with N succeeds.

---

# 10. T6 — Hostile artifact and compatibility certification

Create or extend:

`phalcom-compiler-artifact/tests/hostile_matrix_test.rs`

`phalcom-compiler-artifact/tests/query_owner_integrity_test.rs`

Required rejection matrix:

| Hostile condition | Expected result |
|---|---|
| Truncated container/section | Typed decode/container error |
| Duplicate required section | Rejected |
| Unknown required section | Rejected |
| Unsupported section schema | Typed compatibility error |
| Incorrect container digest | Rejected |
| Incorrect section digest | Rejected |
| Foreign native catalog | Rejected |
| Wrong query fingerprint ABI | Rejected |
| Wrong module semantic ABI | Rejected |
| Forged project-qualified owner | Rejected |
| Valid index pointing to wrong semantic row | Rejected |
| Duplicate module stable identity | Rejected |
| Same path in different projects | Preserved as distinct |
| Unallowlisted external project | Rejected |
| Session identity on wire | Rejected |
| Missing internal owner | Rejected |
| Dependency key with wrong owner | Rejected |
| Dependency fingerprint mismatch | Rejected |
| Product fingerprint mismatch | Rejected |
| Incompatible product/query kind | Rejected |
| Stale or failed product marked reusable | Rejected |
| Unsupported negative open-world proof | Not published as reusable |
| Excessive allocation request | Typed exhaustion, atomic failure |

Where necessary, regenerate section and container digests after intentional semantic mutations so tests reach semantic validation rather than failing trivially at the digest layer.

Use the normal public decoding/import path wherever possible.

**T6 PASS:** The importer rejects structurally valid but semantically inconsistent artifacts, without corruption of live state or silent fallback.

---

# 11. Required execution and verification commands

Execute from the integration worktree:

```bash
cd /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact-universe-consolidation

git status --short
git rev-parse HEAD
git branch --show-current
```

Expected at takeover:

```text
HEAD starts with 9f0d2f66
branch = feat/artifact-universe-consolidation
working tree clean
```

If HEAD has legitimately advanced, record its exact revision and audit the intervening changes before continuing.

## Gate A — Formatting and compilation

```bash
cargo fmt --all -- --check
git diff --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

Required:

```text
exit status = 0
no formatting differences
no whitespace errors
no Rust compilation errors
```

No compiler-error suppression or exclusion of changed targets is acceptable.

## Gate B — Artifact and metadata substrate

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
```

Required:

```text
exit status = 0
0 failed tests
no ignored new acceptance tests
all original focused tests continue passing
```

The existing starting baselines are 30 and 86 passing tests respectively. The completed implementation should contain additional discriminating tests. Do not use the historical counts as the final required counts.

## Gate C — Semantic artifact tests

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact::query_policy
```

Required:

```text
exit status = 0
0 failed tests
all previous 3 semantic-artifact tests pass
all previous 2 query-policy tests pass
```

Existing tests must be updated if their old assumptions conflict with the stricter contracts. They must not be deleted or weakened.

## Gate D — New identity and semantic reconstruction tests

The implementer must add the named test targets before running these commands:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test project_qualified_identity_test

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test query_owner_integrity_test

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact_reconstruction_test

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact_query_import_test
```

Required:

```text
all specified test targets exist
all tests run (not 0 tests)
0 failed
0 ignored
exit status = 0
```

## Gate E — New session and allocation tests

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact_session_import_test

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact_allocation_test

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test metadata_budgeted_equivalence_test
```

Required:

```text
all specified test targets exist
all tests run
0 failed
0 ignored
exit status = 0
```

Additionally, record measured N and observed allocator figures. A green test with no reconciled allocation evidence is insufficient.

## Gate F — Existing source-mode regressions

These tests are relatively expensive. Execute after the focused artifact/semantic tests have passed, not after every edit.

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke

RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_parse_package::parse_project_compiles_from_declared_entry
```

Required CLI smoke result:

```text
3 passed
0 failed
```

Required SlotMap program output includes:

```text
ok: slotmap-basic
ok: dense-slotmap-basic
slotmap runtime tests complete
```

Required Parse result:

```text
1 passed
0 failed
```

Do not weaken test assertions or substitute check-only compilation for the runtime execution discriminator.

## Gate G — Workspace build and code quality

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run

RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings

cargo fmt --all -- --check
git diff --check
```

Required:

- Workspace test compilation passes.
- No new code-quality violations.
- Clippy passes, or any pre-existing independent failure is captured precisely with baseline and evidence; no silent omission.
- Formatting and diff checks pass.

Existing known historical failures in the broader `phalcom-modules` and `phalcom-semantic` test baselines must not be silently treated as P2 regressions or silently excused if their behavior changes.

If broader tests are run, classify failures as `BASELINE_CURRENT_MAIN`, `INTRODUCED_BY_P2`, or `UNDETERMINED`, with evidence.

## Gate H — Final repository state

```bash
git status --short
git log -n 5 --oneline
git diff --check
```

After committing:

```text
working tree clean
all intended changes committed
branch pushed to origin/feat/artifact-universe-consolidation
remote HEAD equals reported implementation commit
```

Do not assert push success without verifying the remote.

---

# 12. Required explicit acceptance report

Produce a final P2 closure report with these exact sections.

## 12.1 Implementation inventory

List every modified and added file, grouped by:

- Wire schemas.
- Project-qualified identity.
- Semantic relocation.
- Query import.
- Session publication.
- Allocation accounting.
- Tests.
- Documentation.

## 12.2 Architectural invariant verification

For every `INV-01` through `INV-12`, record:

```text
Invariant:
Implementation:
Verification test:
Observed result:
Status: PASS / FAIL
```

No invariant may be marked PASS solely because the implementation was designed to satisfy it.

## 12.3 Gate ledger

For each Gate A–H:

```text
Gate:
Exact command(s):
Exit status:
Tests executed:
Tests passed:
Tests failed:
Tests ignored:
Relevant output:
Result: PASS / FAIL / NOT_RUN
```

Mark a gate `NOT_RUN` if it was not executed.

Never report expected or inferred results as observed results.

## 12.4 Artifact operational evidence

Provide a trace of:

```text
artifact decoded
project owners validated
type/store identities relocated
semantic products reconstructed
typed query products installed
dependency graph constructed
session published
snapshot available
source parsing count = 0
canonical linking count = 0
canonical semantic reanalysis count = 0
```

Include actual reconstructed product counts and typed query counts.

## 12.5 Allocation evidence

Provide:

```text
Total measured whole-load requests N:
Allocator-requested bytes:
Ledger-requested bytes:
Retained live bytes:
Peak live bytes:
Temporary allocation bytes:

Budget N result:
Budget N-1 result:
Typed failure:
Caller budget after failure:
Published generation after failure:
Retry N result:
```

The report must identify and reconcile any differences between allocation measurements and charges.

## 12.6 Source-mode regression evidence

Record actual commands and outputs for:

- CLI smoke.
- SlotMap runtime.
- Parse package integration.

## 12.7 Provenance

Record:

```text
Starting revision:
Final revision:
Branch:
Remote tracking branch:
Commit SHA(s):
Working tree status:
Outstanding unrelated baseline failures:
```

---

# 13. Explicit stop and escalation conditions

Stop and document an implementation incident before taking an architectural shortcut if any of the following occurs:

1. A required wire owner cannot be reconstructed into a stable current-main identity without inventing an identity.
2. A semantic product cannot be reconstructed without access to source-only authority.
3. Imported semantic products appear to require a second persistent type store or query engine.
4. A product's apparent transport eligibility conflicts with its query reuse semantics.
5. An open-world query cannot be safely invalidated or revalidated.
6. The existing `SemanticSnapshot` contract cannot represent the imported authority without placeholder semantic data.
7. Whole-load allocation accounting exposes unmetered allocations that require changes outside the identified ownership layers.
8. A metadata schema/Serde compatibility behavior cannot be preserved by the custom decoder.
9. Typed allocation failures cannot be propagated without revising public error contracts.
10. Correct publication would require bypassing current-main semantic, match, callable, native, or bootstrap validation.
11. A proposed simplification removes a dependency, fingerprint, owner, or proof that current-main compilation uses.
12. A previously passing focused source-mode regression fails after the changes.

For each incident, report:

```text
Invariant at risk:
Exact observed failure:
Minimal reproduction:
Owning subsystem:
Relevant source locations:
Architectural options:
Recommended resolution:
Testing consequences:
```

Do not weaken the architecture or tests while awaiting an architectural decision.

---

# 14. Forbidden implementation shortcuts

The following do not satisfy P2 and must not be used:

- Returning `SemanticWorldV3`/`QueryImageV3` DTOs as the completed imported semantic authority.
- Building a separate imported query store instead of using the existing `SemanticDb`.
- Retaining raw store-local IDs from the producer process.
- Resolving semantic owners using module path alone.
- Treating `StableProjectRef` variants as equivalent because their display strings match.
- Auto-allowlisting every external project encountered.
- Mapping session-local identities into a fake stable builtin project.
- Publishing product-row references without verifying their true semantic owners.
- Trusting serialized product fingerprints without verifying their semantic contract.
- Marking every represented query product automatically reusable.
- Treating `GenericConformance`, `ConformanceDispatch`, or callable body proofs as globally closed-world without evidence.
- Dropping dependency edges to make reuse validation pass.
- Populating `Ready` query states without matching typed products.
- Reconstructing Universe from source inside the artifact-backed constructor.
- Calling the source-backed baseline constructor and claiming source-free execution.
- Disabling match coverage, conformance checks, or native compatibility guards.
- Using catch-all errors that obscure allocation exhaustion.
- Counting only tracked reservations while calling allocation behavior exact.
- Artificially inflating N to make N−1 tests pass.
- Deleting or weakening failing tests.
- Claiming P2 complete from a nontrivial transport test without operational session proof.
- Changing production Universe sources or automatically enabling artifacts during this slice.

Prefer an explicit unsupported operation and a well-classified incomplete gate over an implementation that fabricates semantic authority.

---

# 15. Implementation order and checkpoints

Execute the work systematically.

| Order | Work | Required checkpoint before proceeding |
|---|---|---|
| 1 | T1: project-qualified module/query schemas | Identity and compatibility tests pass |
| 2 | T2: current-main semantic reconstruction | Real native semantic products round-trip |
| 3 | T3: query product reconstruction and persistence policy | Typed query and invalidation tests pass |
| 4 | T4: session construction and atomic publication | Source-free, overlay, rollback tests pass |
| 5 | T5: whole-load accounting | Allocator reconciliation and N/N−1/retry pass |
| 6 | T6: hostile compatibility certification | Entire hostile matrix passes |
| 7 | Existing source-mode regressions | CLI, SlotMap and Parse pass |
| 8 | Workspace build, documentation, commit/push | Clean and verified repository state |

T2 and T3 may progress in parallel after T1 if their owner contracts are stable.

Do not let parallel implementation produce two incompatible relocation maps or semantic product ownership systems.

At each checkpoint, record:

```text
Tasks completed:
Files changed:
Commands executed:
Measured results:
Invariant verification:
Remaining blockers:
Next task:
```

If implementation is incomplete, report partial progress accurately. Do not mark a gate passed based on planned work.

---

# 16. Final P2 acceptance decision

`ARTF001.C1.P2` may be marked:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only when:

- All mandatory P2 corrective goals are implemented.
- All new identity, semantic reconstruction, query, session, hostile-artifact and allocation tests pass.
- The source-free session construction discriminator passes.
- Query invalidation after local semantic changes passes.
- Whole-load measured N/N−1/retry accounting passes.
- Existing CLI, SlotMap and Parse regressions pass.
- All required compatibility/schema changes are recorded.
- The checkpoint and handoff accurately describe the as-built APIs and evidence.
- The final implementation is committed, pushed, and the repository is clean.

If any mandatory architectural gate remains incomplete, retain:

```text
status: IN_PROGRESS
completion: PARTIAL
```

Do not change P3's status solely because P2 transport tests pass.

## Final outcome required from the implementer

The end state is **a compiler artifact substrate that restores actual semantic authority**, not a serialized snapshot that merely resembles the compiler's state.

The definitive proof is a fresh artifact-backed semantic session that:

1. Has valid native current-main stores and typed query products.
2. Has a coherent, usable `SemanticSnapshot`.
3. Required zero canonical Universe source parsing, linking, and reanalysis during artifact construction.
4. Correctly responds to local semantic modifications and invalidates unsafe cached results.
5. Rejects malformed or incompatible artifacts before publication.
6. Satisfies measured complete-load allocation and rollback invariants.

Once this is demonstrated, close P2 and hand off to `ARTF001.C1.P3`, whose job is to produce and import the real canonical Universe compiler image using this completed substrate.

**Success means reaching these operational results while preserving the architecture. Passing tests by reducing the required semantics, validation, or proofs is explicitly not success.**
