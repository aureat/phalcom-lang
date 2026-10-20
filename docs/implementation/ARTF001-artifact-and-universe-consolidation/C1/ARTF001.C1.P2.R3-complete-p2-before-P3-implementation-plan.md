---
id: ARTF001.C1.P2.R3
kind: implementation-plan
program: ARTF001
checkpoint: ARTF001.C1
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
branch: feat/artifact-universe-consolidation
planning_revision: dc644bfb9fd20cf694eca48f0cedf817c7cc8f23
reference_branch: perf/universe-characterization
reference_revision: 077d93d3bfcd5d3bca0127c4aac18abb55c2ed94
merge_base: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
supersedes: ARTF001.C1.P2.R2
successor: ARTF001.C1.P3
prepared: 2026-10-20
---

# ARTF001.C1.P2.R3 — Complete P2 Before P3

## Native Semantic Authority, Query Closure, Imported Session Publication, and Certification

> **Execution directive:** This is the single corrective implementation plan that completes the remainder of `ARTF001.C1.P2`. Do not create a further P2.R4/P2.R5 decomposition merely because the work is large. Use `T0`–`T9` below as bounded implementation tasks inside this plan. `ARTF001.C1.P3` remains blocked until every P2 exit gate in this document is satisfied and the checkpoint record is updated with executed evidence.

---

# 1. Mission

Complete the artifact import side of the compiler architecture to the point where a validated `.telosc` image can be transformed, without canonical Universe source access, into **complete current-main native semantic authority** and atomically installed as the immutable canonical base of the ordinary `SemanticWorkspaceSession`.

P2 is complete only when the supported path is:

```text
.telosc bytes
    |
    v
container/schema/digest validation
    |
    v
exact project-qualified module/project binding
    |
    v
TypeStore + StaticValueStore reconstruction
    |
    v
complete current-main native semantic authority
    |
    v
complete current-main callable/proof/static authority
    |
    v
verified typed query products + exact dependency graph
    |
    v
one private imported canonical generation
    |
    v
one SemanticDb + one SemanticSnapshot
    |
    v
atomic SemanticWorkspaceSession publication
    |
    +--> immutable canonical imported base
    +--> ordinary current-main local source overlay
    +--> ordinary current-main invalidation/recomputation
```

No source parsing, source linking, or canonical Universe semantic reanalysis may be required to construct that imported session.

The immediate successor, **P3**, may then implement the production exporter/root collector. P3 must not be used to hide or complete importer/session architecture that belongs here.

---

# 2. Planning audit and pinned baseline

This plan was prepared from a repository audit pinned to:

```text
repository: aureat/phalcom-lang
branch:     feat/artifact-universe-consolidation
HEAD:       dc644bfb9fd20cf694eca48f0cedf817c7cc8f23
subject:    test(artifact): cover callable-local P7 evidence graph staging

P7 ref:    077d93d3bfcd5d3bca0127c4aac18abb55c2ed94
branch:     perf/universe-characterization

merge base: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
```

At the audit point the consolidation branch was **454 commits ahead and 111 commits behind** the P7 reference branch. The 111 commits are not a merge backlog. Completion is capability closure, not Git ancestry closure.

The current `CHECKPOINT.md` and P2.R2 records are materially stale: they describe the implementation around `37d2b0d8`, while the branch has since landed substantial callable, proof, witness, derived-conformance, static-program, and denotation work.

This planning audit inspected repository source, current implementation records, current artifact tests, and the certified P7 reference implementation. It did **not** rerun Cargo commands locally. Test results quoted as prior evidence below are repository-recorded evidence, not re-certification at `dc644bfb`.

---

# 3. What is already implemented at the planning revision

Do not restart or duplicate these capabilities.

## 3.1 Artifact and identity substrate

Implemented and retained:

- `.telosc` container framing, sections, digests, ABI/schema checks, bounded decode.
- `ModuleWorldV2`.
- `QueryImageV3`.
- `SemanticWorldV3`.
- project-qualified `StableModuleRef` ownership.
- explicit external-project allowlisting.
- caller-owned `ProjectUniverse` as the sole live project authority.
- trusted package-resolver receipt binding:
  - complete package identity;
  - version;
  - artifact fingerprint;
  - full validated 256-bit artifact digest;
  - already-existing persistent project identity.
- import allocates no project IDs.
- `StableOwnerCatalog` as the shared stable-owner relocation authority.
- current-main `TypeStore` reconstruction.
- current-main `StaticValueStore` reconstruction.

## 3.2 Semantic authority already staged/reconstructed

The current `ImportedCompilerImage` contains staged authority for:

- structural data/enum/variant semantics;
- associated-family authority;
- static-case catalogs/signatures;
- callable signatures;
- callable definitions;
- callable binding states;
- callable expression base facts;
- conditional-call capability;
- callable-local conformance evidence graphs;
- call specialization;
- module-static binding authority;
- constructor initialization;
- declaration dispatch and field signatures;
- conditional inherent authority;
- explicit conformance heads;
- trait headers/surfaces and diagnostics;
- witness-plan preflight:
  - associated-type plans;
  - completeness;
  - requirement views;
  - recursive requirement selection;
  - evidence references;
- guarded derived-conformance rules;
- enum requirements;
- static constants;
- canonical source-free static programs;
- static definitions as imported operational authority without fabricated AST;
- checked static-application denotation binding.

These are substantial P7 transplants. Many are still **staging products**, not yet the native tables expected by `SemanticSnapshot`.

## 3.3 Query work already implemented

The branch has:

- exhaustive policy for all 46 current `QueryKind` variants;
- exact portable-owner conversion where the wire identity is sufficient;
- project-qualified query-owner validation;
- transport row/product ownership validation for initial structural families;
- portable fingerprint validation;
- fresh live current-main input/product fingerprint computation for the implemented families;
- five reusable native query families:
  - `DataDeclaration`;
  - `EnumDeclaration`;
  - `AssociatedSurface`;
  - `TypeMemberDeclaration`;
  - `CallableDefinition`.

Dependency evidence is validated but is not promoted merely because it exists.

## 3.4 Existing transaction boundary

`ArtifactImportTransaction` validates and stages substantial semantic authority under a child allocation ledger, but its current `commit()` returns `ImportedCompilerImage` and explicitly **does not publish into a workspace session**.

`ImportedCompilerImage::publish_query_products_into_empty_db()` publishes only the currently supported query products into a fresh empty `SemanticDb`.

This is a useful intermediate boundary. It is **not** the final P2 atomic publication boundary.

---

# 4. Audit findings that change the old P2.R2 plan

These findings are mandatory inputs to this plan.

## 4.1 Current declaration authority is richer than P7's structural shell

Current live declaration authority is:

```rust
pub struct DeclarationTypeInfo {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub binding: DeclarationTypeBinding,
    pub generic_binder_header: Option<GenericBinderHeader>,
    pub generic_signature: Option<GenericSignature>,
    pub supertype_template: Option<GenericSupertypeTemplate>,
}
```

`DeclarationTypeBinding` separately carries:

```text
semantic application behavior:
    Ordinary
    Intrinsic(Tuple | Record | Pack)

runtime/native owner:
    Tuple | Record | Pack | None
```

`DeclarationTypeTable` additionally owns independently registered:

```text
intrinsic_static_domains
intrinsic_type_owners
```

The current wire `DeclarationShellV1` only carries:

```text
declaration
self_type
parameters
```

and `HierarchyAndAliasV1` only carries:

```text
declaration
superclass
aliased_type
```

That is insufficient unless every missing current-main field is formally proven derivable from other validated artifact authority.

**P2 must resolve this. It may require a structural schema bump. Missing fields must not be defaulted.**

## 4.2 Current `TypeAliasInfo` is also richer than the P7 alias row

Current alias authority includes:

```rust
TypeAliasInfo {
    declaration,
    kind,
    kind_shape,
    generic_signature,
    form,
    structural_form,
    dependencies,
    source,
}
```

`HierarchyAndAliasV1.aliased_type` alone does not encode this complete native product.

Source provenance may be represented as logical stable source span rather than raw source syntax, but every compiler-observable field must be encoded or uniquely derivable.

## 4.3 Current snapshot has post-P7 compiler-critical module execution products

Current `SemanticSnapshot` includes:

```rust
pub import_phase_products:
    Arc<BTreeMap<ModuleId, ModuleImportPhaseProduct>>,

pub generated_runtime_requirements:
    Arc<[RuntimeDependencyRequirement]>,
```

These are consumed by current executable-link finalization.

`SemanticWorldV3` contains neither family.

Current source-side derivation is **not source-free**. `compute_module_import_phase_products*()` reads:

- `SemanticLinkedProgram`;
- parsed sources;
- `SourceSemanticIndex`;
- callable analyses;
- attribute analyses;
- type store;
- static constants;
- source AST for some phase/runtime requirement cases.

Therefore P2 may not simply derive these fields after source denial.

**Default recommendation:** persist current canonical `ModuleImportPhaseProduct` and `RuntimeDependencyRequirement` authority in a versioned semantic artifact family, unless a complete source-free derivation proof is established first.

## 4.4 Current `CallableAnalysis` has post-P7 phase evidence

Current `CallableAnalysis` contains:

```rust
observed_import_phases:
    Arc<BTreeMap<SourceImportBindingId, DependencyPhase>>
```

The current `CallableAnalysisRecordV1` wire model does not.

These facts feed `ModuleImportPhaseProduct` construction and must not be silently replaced with an empty map.

`dependency_fingerprint` does **not** need to cross the wire: it should be recomputed with the current-main callable-body product fingerprint function after complete native reconstruction.

## 4.5 Current source-free static execution has no live-native destination yet

Current-main's `StaticDefinition` remains AST-backed:

```rust
StaticDefinitionBodyRepr =
    TypeForm(TypeAnnotation)
  | Expression(Expr)
  | Block(LexicalBlock)
```

and the existing `StaticEvaluator` consumes `CheckedStaticBody`, whose root is still AST-backed.

The branch correctly does **not** fabricate AST. It stages a canonical source-free static program instead.

P2 must add a legitimate current-main execution boundary that lets the **one existing evaluator semantics** consume imported canonical checked authority without creating:

- fake AST;
- a second evaluator;
- a second static semantic ontology.

## 4.6 Current session construction is still source-backed

Current:

```rust
SemanticWorkspaceSession::new()
    -> with_workspace(...)
    -> canonical_universe_semantic_baseline().instantiate(...)
```

There is no imported-generation constructor.

The session owns much more than a snapshot:

- one `SemanticDb`;
- `TypeStore`;
- `StaticValueStore`;
- base declaration/hierarchy/dispatch/callable tables;
- alias and generic-dependency state;
- module session/project graph;
- source/incremental state;
- current and last-known-good snapshots.

P2 must build an artifact-backed base without indirectly initializing the source baseline.

---

# 5. P2 end-state acceptance contract

P2 is accepted only when **all** of the following are true.

## G0 — coverage is explicit

Every current compiler-relevant:

- `SemanticSnapshot` field;
- `SemanticWorkspaceSession` base field;
- current `QueryKind`;
- compiler attachment family;
- imported P7 semantic family;

has an explicit P2 disposition.

Unknown disposition count:

```text
0
```

## G1 — native semantic authority is complete

The imported generation contains native current-main authority for every compiler-required semantic family.

No compiler-required family remains represented solely by a transport/staging DTO.

## G2 — callable authority is complete

Every imported callable that is required by the canonical image becomes a complete native `CallableAnalysis`.

For published callable analyses:

```text
deferred expression sidecars == 0
unresolved local ordinal refs == 0
missing proof refs            == 0
missing semantic deps         == 0
```

## G3 — query closure is complete

Every structurally reusable query root in the supported canonical profile is either:

1. reconstructed, fingerprinted, dependency-closed, and published `Ready`; or
2. explicitly removed from the reusable profile by a current-main architectural ruling proving it cannot be safely persisted.

No root becomes `Ready` with omitted dependencies.

## G4 — ordinary session publication works

Artifact bytes can create an ordinary current-main `SemanticWorkspaceSession` with:

```text
one SemanticDb
one TypeStore ontology
one StaticValueStore ontology
one coherent SemanticSnapshot
one retained ProjectUniverse
ordinary local overlay behavior
```

## G5 — source denial is proven

Imported session construction executes with canonical Universe source access denied and records:

```text
canonical baseline builds       == 0
canonical Universe parses       == 0
canonical Universe source links == 0
canonical semantic reanalysis   == 0
```

for the imported construction path.

## G6 — whole-load atomicity is proven

For measured whole-load allocation demand `N`:

```text
budget N     -> success
budget N - 1 -> typed allocation failure
retry N      -> success
```

and failed loads publish nothing and leave caller-owned authority unchanged.

## G7 — hostile artifact matrix is complete

Inner semantic/query corruptions reach their intended validation layer and fail before publication.

## G8 — P3 needs no importer redesign

P3 can consume a source-built complete semantic generation and project it to wire. It does not need to invent:

- new imported semantic authority;
- new imported session ownership;
- new query reuse semantics;
- new loader transaction boundaries.

Only then is P3 unblocked.

---

# 6. Mandatory architectural invariants

## INV-01 — one semantic ontology

There is exactly one live:

```text
TypeStore
StaticValueStore
SemanticProduct model
SemanticDb
SemanticSnapshot model
SemanticWorkspaceSession model
```

Wire DTOs are transport only.

## INV-02 — no fabricated semantic data

Never fill a missing artifact field with a plausible default merely to construct a current-main type.

Examples of prohibited fabrication:

```text
DeclarationTypeBinding::ordinary()
empty observed_import_phases
empty runtime requirements
empty alias dependencies
empty proof graph
empty dependency list
empty diagnostics/explanations when the wire owner says otherwise
```

unless the empty value is established by an explicit profile invariant and validated before publication.

## INV-03 — exact stable ownership

All imported ownership must resolve through the existing `StableOwnerCatalog` and caller-owned `ProjectUniverse`.

No parallel owner registry.

No artifact-local project IDs.

No path-only identity.

No project ID allocation during import.

## INV-04 — current-main native fingerprints are authoritative

Portable fingerprints establish transport integrity.

After relocation:

```text
recompute current input fingerprints
recompute current product fingerprints
recompute current dependency fingerprints
```

Do not use the portable fingerprint as the live `SemanticDb` fingerprint merely because it validated.

## INV-05 — open-world queries remain open-world

Never publish unconditional `Ready` roots for:

```text
GenericConformance
ConformanceDispatch
TraitDispatchBucket
CallableBody
CheckedStaticBody
```

unless a later adopted architecture changes the current persistence contract.

P2 does not weaken this contract.

## INV-06 — validation precedes visible publication

All of these complete privately first:

```text
container validation
project binding
type/static relocation
semantic reconstruction
proof reconstruction
query closure
native index construction
snapshot construction
session construction
```

Only then may publication become visible.

## INV-07 — one whole-load accounting boundary

The parent allocation ledger commits only after the final imported session/generation is constructed.

An intermediate staging `commit()` is not the P2 success boundary.

## INV-08 — no source fallback

Explicit artifact import either succeeds from the artifact or fails.

It must never silently invoke source compilation.

## INV-09 — P2 does not implement production export

Production:

```text
artifact/export.rs
artifact/roots.rs
canonical artifact builder/writer
```

remain P3/P4.

A `#[cfg(test)]` source-to-wire oracle is allowed if needed to prove importer parity on real current-main semantic generations. It must not become a production exporter API.

## INV-10 — source-mode behavior remains correct

Do not weaken:

- visibility;
- match/exhaustiveness proof;
- generic constraints;
- conformance coherence;
- static checking;
- runtime requirements;
- native metadata;
- linking;
- bootstrap requirements;

to make artifact tests pass.

---

# 7. Current semantic publication coverage manifest

## T0 must turn this table into executable repository policy

The following is the minimum current audit disposition. During implementation, verify every entry against actual compiler readers and encode the final result in a current-main artifact coverage manifest.

| `SemanticSnapshot` field | P2 disposition | Required authority |
|---|---|---|
| `id` | process-local derived | new imported session revision |
| `generation` | process-local derived | imported generation/session |
| `store` | WIRE | `type_graph` |
| `sources` | OMITTED_SOURCE_ONLY | empty for imported canonical base |
| `presentation_sources` | OPTIONAL_PRESENTATION | not analysis input |
| `surfaces` | WIRE | declaration dispatch surfaces |
| `dispatch` | DERIVED_ON_LOAD | exact surfaces |
| `callable_signatures` | WIRE | callable signature rows |
| `constructor_initializations` | WIRE | constructor rows |
| `callable_definitions` | WIRE | accepted definitions |
| `conformance_index` | WIRE | conformance heads |
| `conformance_witness_plans` | WIRE | witness plans + evidence |
| `derived_conformance_rules` | WIRE | derived rules + evidence |
| `trait_dispatch_index` | DERIVED_ON_LOAD | native conformance/trait authority |
| `field_signatures` | WIRE | dispatch surface rows |
| `declarations` | WIRE / SCHEMA_CLOSURE | complete declaration authority |
| `type_aliases` | WIRE / SCHEMA_CLOSURE | complete alias authority |
| `trait_headers` | WIRE | trait rows |
| `trait_surfaces` | WIRE | trait rows |
| `hierarchy` | WIRE / SCHEMA_CLOSURE | exact hierarchy/supertype authority |
| `diagnostics` | PROFILE_DERIVED_EMPTY or WIRE | must be explicitly proven |
| `semantic_graph` | DERIVED_ON_LOAD or WIRE | exact dependency graph; prove derivation |
| `callable_analyses` | WIRE / COMPLETE_NATIVE_REBUILD | all compiler-required callable facts |
| `internal_incidents` | OMITTED_ONLY_IF_PROFILE_REJECTS_INCIDENTS | never silently discard |
| `formal_projection` | DERIVED_ON_LOAD | complete callable analyses |
| `source_index` | OMITTED_SOURCE_ONLY | prove no source-free compiler reader requires it |
| `advisory` | OMITTED_ADVISORY | empty |
| `module_products` | DERIVED_ON_LOAD | `CanonicalExternalModuleWorld` |
| `data_semantics` | WIRE | structural contracts |
| `enum_semantics` | WIRE | structural contracts |
| `enum_requirements` | WIRE | enum requirement rows |
| `associated_surfaces` | WIRE | associated rows |
| `type_member_declarations` | WIRE | type-member rows |
| `associated_static_domains` | WIRE | associated static-domain rows |
| `static_case_catalogs` | WIRE | catalog rows |
| `static_case_signatures` | WIRE | signature rows |
| `static_definitions` | WIRE + execution adapter | canonical static program authority |
| `static_constants` | WIRE | static constant rows |
| `static_values` | WIRE | static-value graph |
| `module_static_bindings` | WIRE | module-static rows |
| `semantic_structure_shards` | OMITTED_SOURCE_INCREMENTAL | no canonical imported source shard |
| `import_phase_products` | **NEW WIRE unless source-free derivation proven** | current module execution semantics |
| `generated_runtime_requirements` | **NEW WIRE unless source-free derivation proven** | current executable-link requirements |
| `status` | DERIVED after completeness validation | `Complete` only after all gates |

T0 must also classify every `SemanticWorkspaceSession` field so imported construction does not accidentally consult source-owned state later.

Particular attention is required for:

```text
universe_source_provider
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
base_data_semantics
base_enum_semantics
base_associated_surfaces
base_type_member_declarations
base_enum_requirements
alias_sources
alias_dependencies
alias_contract_dependencies
alias_forms
generic header dependency indexes
declaration signature dependency indexes
canonical native diagnostics
last_snapshot / last_known_good
module_session / ProjectUniverse
```

Imported canonical aliases must operate from imported alias products and dependency authority, not from canonical source AST stored in `alias_sources`.

---

# 8. P7 salvage boundary for this P2 plan

## 8.1 P7 code/algorithms to port or adapt during P2

Use these as references; do not copy monolithically:

```text
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/import/diagnostic.rs
phalcom-semantic/src/artifact/import/explanation.rs
phalcom-semantic/src/artifact/import/proof.rs
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/artifact/manifest.rs
selected loaded-publication logic from:
    phalcom-semantic/src/canonical_image.rs
```

Port/adapt specifically:

- complete native conformance evidence materialization;
- complete witness construction;
- native derived-conformance construction;
- callable flow graph import;
- trait dispatch and contextual realization;
- iteration and positional expansion;
- sequence literals;
- associated/callable-reference/family application resolution;
- match/pattern/proof import;
- explanation arena;
- diagnostic cause/explanation reconstruction;
- callable dependencies and semantic dependencies;
- query closure/acyclicity/freshness validation;
- source-free completeness validation;
- loaded publication rollback/failpoint methodology.

## 8.2 P7 work intentionally deferred to P3/P4

Do **not** port these as production code under this plan:

```text
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/export/proof.rs
phalcom-semantic/src/artifact/export/explanation.rs
phalcom-semantic/src/artifact/roots.rs
phalcom-core/src/modules/canonical_artifact.rs
phalcom-core/src/modules/parity.rs
phalcom-core/examples/perf_audit_p7.rs
```

Their algorithms remain useful references for later phases.

---

# 9. Implementation tasks

# T0 — Re-pin, normalize records, and establish executable completeness contracts

**Purpose:** make P2 completion mechanically defined before adding more code.

## T0.1 Re-pin branch state

At implementation start:

```bash
git status --short --branch
git log -1 --oneline
git rev-parse HEAD
git rev-parse origin/feat/artifact-universe-consolidation
```

If the branch has advanced beyond the planning SHA:

1. remain on `feat/artifact-universe-consolidation`;
2. inspect all intervening artifact commits;
3. mark any T1–T9 work already completed;
4. do not recreate landed code;
5. update the plan implementation anchor in the checkpoint.

## T0.2 Replace the stale P2 progress description

Update `CHECKPOINT.md` to record the actual implementation families already staged at the active head.

Do not claim completion.

## T0.3 Add a current artifact coverage manifest

Recommended new module:

```text
phalcom-semantic/src/artifact/manifest.rs
```

The manifest should classify:

- every current `SemanticSnapshot` field;
- every current canonical-session base family;
- every `QueryKind`;
- compiler attachment families;
- every imported artifact family.

Recommended dispositions:

```rust
enum ArtifactAuthorityDisposition {
    Wire,
    DerivedOnLoad,
    LocalOverlayOnly,
    SourceOnly,
    PresentationOnly,
    AdvisoryOnly,
    ProcessLocal,
    IntentionallyOmitted,
}
```

The exact type may differ.

## T0.4 Add exhaustive tests

Required:

```text
artifact_manifest_covers_every_snapshot_field
artifact_manifest_covers_every_query_kind
artifact_manifest_covers_every_compiler_attachment_family
artifact_manifest_has_no_unknown_compiler_required_family
```

Where Rust does not permit reflection over struct fields, maintain a deliberate compile-adjacent field inventory and require changes to `SemanticSnapshot` to update it in the same patch.

## T0 exit gate

```text
unclassified compiler-required snapshot families == 0
unclassified QueryKind variants                   == 0
unclassified imported semantic families           == 0
```

No T1 schema decision proceeds from an implicit default.

---

# T1 — Close current-main wire/schema authority

**Purpose:** ensure the wire can actually represent every current semantic fact P2 promises to reconstruct.

This is the most important architecture task before further materialization.

## T1.1 Declaration reconstruction matrix

For each `DeclarationTypeInfo` field, record one and only one source:

```text
WIRE
DERIVED_FROM_VALIDATED_WIRE
TRUSTED_COMPILER_INTRINSIC_CATALOG
MISSING -> SCHEMA CHANGE REQUIRED
```

Fields:

```text
declaration
form
class_object_type
kind
binding.semantic_form
binding.runtime_owner
generic_binder_header
generic_signature
supertype_template.declaration
supertype_template.supertype
supertype_template.structural_form
intrinsic_static_domains
intrinsic_type_owners
```

Rules:

- `form`, `class_object_type`, and `kind` may be derived only if the derivation reproduces the exact canonical `TypeStore` identities and current kind construction.
- `generic_binder_header` may be derived from a validated generic signature only if equality is proven.
- `binding.runtime_owner` is carried by the three-case `IntrinsicRuntimeOwnerV1`; the shell's `intrinsic_type_owner` field is a redundant runtime-owner witness and must equal `binding.runtime_owner`. This witness never implies a type-member owner. The Pack consultation recorded in `CHECKPOINT.md` found that Pack already has semantic application and structural type/value representations, while the current specifications define no direct Pack-owned type-member operations. Keep the current owner separation; do not infer `TypePack` or add an eighth owner without a normative direct Pack-member contract.
- Actual `intrinsic_static_domains` and `intrinsic_type_owners` are separate strictly sorted row families keyed by project-qualified `StableDeclarationRef`; the latter uses the exhaustive seven-case `IntrinsicTypeOwnerV1`.
- Import resolves each actual projection row through the project-qualified `StableOwnerCatalog`, including owners with no declaration shell. Preserve each family independently, allow both projection rows for one owner, and reject duplicate rows within a family. Do not infer one actual owner family from the other, runtime ownership, or declaration names.
- ordinary fallback is forbidden if the source generation used an intrinsic binding.
- generic/applied supertype templates must not be collapsed to a bare superclass declaration.

If the existing structural schema cannot prove these fields, introduce a versioned structural-contract schema revision.

## T1.2 Alias reconstruction matrix

Represent or derive all of:

```text
kind
kind_shape
generic_signature
form
structural_form
dependencies
logical source span
```

Raw AST is not required.

If `source` is only diagnostic provenance, encode a stable logical source span rather than source syntax.

## T1.3 Add current module execution authority

Default implementation direction:

```text
SemanticWorldV4
    ...
    module_execution_authority: ModuleExecutionAuthorityV1
```

Conceptually:

```rust
struct ModuleExecutionAuthorityV1 {
    import_phase_products: Vec<ModuleImportPhaseProductV1>,
    runtime_requirements: Vec<RuntimeDependencyRequirementV1>,
}
```

Use stable project/module/import identities.

### Source-import binding ordinal contract

V4's `binding: u32` is a module-relative flattened source-binding ordinal. Derive it from the ordered `ModuleWorldV2` import rows:

1. Enumerate import rows in vector order. Each row index is its `ImportSiteLocalId`, including rows with no bindings.
2. A module import contributes one binding with site-local ordinal `0`. A selective import or re-export contributes one binding per item, in item order, with the item's ordinal local to that site.
3. Encode a binding as the checked prefix sum of the binding counts in earlier sites plus its site-local ordinal. Reject a module whose total cannot fit in `u32`.
4. Decode by locating that ordinal in the same site ranges to recover the exact `SourceImportBindingId`.

For `ModuleImportPhaseProduct`, the product's `StableModuleRef` scopes the ordinal. For callable observations, the stable callable owner supplies the importing module. Never use executable linked-read slots, local names, or spans as source-import identity.

Every module must have one phase product, including modules with no imports. Its rows must contain exactly the dense ascending range of source bindings in that module. Callable observations are sparse: each ordinal must exist in the owner's module, be unique, and agree with the module product's observed flag and phase coverage. The exporter and importer must use this same derivation; a second binding table is not allowed.

The portable dependency phase is an exact two-state mirror of the current native enum: `InterfaceOnly` maps to `InterfaceOnly`, and `Runtime` maps to `Runtime`. Preserve the native join rule, where `Runtime` dominates. Do not encode or reinterpret `Syntax` or `LoadTime`; neither is a current native phase. No ABI bump is needed while V4 remains unpublished. If a producer emits the unfinished enum first, introduce a new artifact version instead of reinterpreting emitted bytes.

Represent all current native fields:

### `ModuleImportPhaseProduct`

```text
module
binding identity
phase
observed
coverage
```

### `RuntimeDependencyRequirement`

```text
importer
dependency
range
reason
read_target
provenance.consumer
provenance.producer
provenance.target
provenance.product
provenance.timing
```

No current field may disappear in conversion.

The family is semantically generated and therefore belongs under the semantic artifact ABI even though its native live types are owned by `phalcom-modules`.

If implementation evidence proves an equally complete source-free derivation from other persisted products, record that ruling and use `DerivedOnLoad` instead. The current source implementation is not such a proof.

## T1.4 Extend callable wire authority for phase evidence

Add an exact portable representation for:

```text
CallableAnalysis.observed_import_phases
```

Do not encode raw process-local IDs.

Use stable module + stable source-import binding identity consistent with `ModuleWorldV2` symbolic import rows.

Use the source-import binding ordinal contract above for both module products and callable observations. A callable's stable owner determines the module scope; the `u32` is not a global or executable-slot identity.

## T1.5 Current callable fields intentionally derived after load

Do not serialize:

```text
CallableAnalysis.dependency_fingerprint
```

Recompute it with current:

```rust
db::fingerprint::callable_body_product_fingerprint(...)
```

`internal_incidents` should not become a normal portable product. The canonical compiler-image profile should reject source generations with internal incidents / `InternalFailure` authority. Only after that invariant is validated may imported native analyses use an empty incident list.

## T1.6 Versioning

Expected minimum:

```text
structural primary contracts schema -> bump if layout changes
semantic world schema              -> V4 if new families/fields added
semantic_artifact_abi              -> bump
```

Bump `query_fingerprint_abi` only if query key/fingerprint semantics change.

Do not bump outer container framing without a framing change.

Reject previous required semantic schema with a typed compatibility error if it cannot express current P2 authority.

## T1 tests

Add transport and hostile tests for:

```text
declaration intrinsic binding round-trip
all seven intrinsic type owners round-trip from standalone rows independently from runtime owners
shell-less intrinsic static-domain and type-owner rows round-trip independently
duplicate rows within either intrinsic projection family are rejected
both intrinsic projection families may independently name the same declaration
duplicate declaration-shell owners rejected
shell intrinsic runtime-owner witness must match the binding
Pack runtime owner does not create or imply a TypePack owner row
generic applied supertype preservation
alias kind/form/dependency preservation
module import phase rows round-trip
runtime requirement provenance round-trip
callable observed import phases round-trip
invalid source-import owner rejected
runtime requirement target/dependency mismatch rejected
duplicate module execution owner rejected
invalid coverage/phase discriminant rejected
InterfaceOnly and Runtime preserve their exact current phase semantics
old semantic schema rejected
```

Include a mixed-import round trip with a module import, multiple selective items, multiple re-export items, and an empty-item import site. Assert the exact reconstructed `(ImportSiteLocalId, ImportBindingLocalId)` pairs in complete module products and sparse callable observations. Reject duplicate, missing, out-of-range, wrong-module, and phase-inconsistent rows, plus missing module products.

## T1 exit gate

A field-by-field proof shows that every current compiler-required declaration/alias/module-execution/callable-phase fact is encoded or uniquely derived.

Missing facts:

```text
0
```

---

# T2 — Materialize declarations, aliases, hierarchy, and structural compiler authority

**Purpose:** turn the structural wire into complete native declaration authority.

Recommended new module:

```text
phalcom-semantic/src/artifact/declaration_import.rs
```

## T2.1 Reconstruct native declaration table

Construct a complete current:

```text
DeclarationTypeTable
```

including intrinsic projections.

Validate:

- declaration owner;
- form nominal origin;
- class-object type owner;
- kind equality;
- parameter owner/order;
- generic signature;
- binder header;
- semantic form;
- runtime owner;
- intrinsic static domain;
- intrinsic type owner;
- supertype template.

## T2.2 Reconstruct native alias table

Construct:

```text
TypeAliasTable
```

with exact:

- kind;
- structural kind shape;
- generic signature;
- form;
- structural presentation;
- dependencies;
- logical source span.

Reject:

- duplicate aliases;
- alias dependency owner mismatch;
- noncanonical dependency order where canonical order is required;
- form/kind disagreement;
- cycles prohibited by current semantics.

## T2.3 Reconstruct hierarchy

Construct:

```text
MapTypeHierarchy
```

from the complete supertype authority.

Preserve generic supertype templates separately from simple hierarchy parent edges.

Reject:

- absent parent owners;
- cross-project unauthorized parents;
- forbidden hierarchy cycles;
- inconsistent bare parent vs supertype template;
- class/object kind mismatches.

## T2.4 Staging allocation rule

Do not eagerly construct current `HashMap`-backed live indexes merely because rows are decoded.

Continue the existing pattern:

```text
sorted immutable staged rows
    |
    v
single T7 native publication/materialization
```

unless the native table construction itself is already budget-accounted and still private.

## T2 tests

Required semantic tests:

```text
declaration_type_info_round_trips_every_current_field
intrinsic_tuple_record_pack_bindings_survive_import
generic_supertype_template_survives_import
alias_complete_native_product_survives_import
alias_dependency_graph_survives_import
hierarchy_reconstructs_exact_parent_authority

missing_class_object_type_fails_closed
wrong_declaration_kind_fails_closed
intrinsic_binding_mismatch_fails_closed
generic_supertype_owner_mismatch_fails_closed
alias_kind_mismatch_fails_closed
alias_dependency_owner_mismatch_fails_closed
hierarchy_cycle_fails_closed
```

## T2 measurable success

For a source-built reference fixture:

```text
source DeclarationTypeTable
    semantic-observable equality
imported DeclarationTypeTable

source TypeAliasTable
    semantic-observable equality
imported TypeAliasTable

source hierarchy
    exact parent/template equality
imported hierarchy
```

No declaration/alias field is fabricated.

---

# T3 — Complete native witness, evidence, derived-conformance, and static authority

**Purpose:** convert the advanced P7 staging graphs already present into the actual current-main native products consumed by the compiler.

## T3.1 Centralize native conformance-evidence materialization

Add one shared materializer, for example:

```text
phalcom-semantic/src/artifact/evidence_materialize.rs
```

Conceptual responsibility:

```text
ImportedConformanceEvidenceGraph
    -> memoized native Arc<...> evidence objects
```

Requirements:

- preserve sharing;
- validate owner before materialization;
- reject cycles;
- enforce recursion/depth limits;
- avoid repeated deep clones;
- use one implementation for witness, derived-rule, and callable consumers.

Do not independently rebuild equivalent proof graphs in three import modules.

## T3.2 Build native `ConformanceWitnessPlan`

The current `ImportedWitnessPlanPreflight` already contains most semantic substance.

Materialize actual:

```text
ConformanceWitnessPlan
BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>
```

including:

- requirement views;
- requirement selections;
- conditional inherent fallback;
- associated-type plan;
- completeness;
- diagnostics;
- exact conformance evidence references.

Recompute current witness product fingerprint.

The wire fingerprint remains transport evidence only.

## T3.3 Build native derived-conformance index

Materialize:

```text
DerivedConformanceRule
DerivedConformanceRuleIndex
```

Validate:

- source impl identity;
- source module;
- target parameter;
- target template;
- trait ref;
- generic signature;
- associated type plan;
- requirement views;
- selections;
- evidence;
- coherence/duplicate constraints.

Recompute current fingerprint where current-main uses one.

## T3.4 Complete static constant table

Construct the live current:

```text
StaticConstantTable
```

from the already staged constant products.

Preserve:

- checked result/value;
- value fingerprint;
- dependencies;
- constant reads;
- diagnostics;
- imported/static lineage.

## T3.5 Define one source-free static execution contract

Do not fabricate `StaticDefinitionBodyRepr`.

Do not add a second evaluator.

Refactor at the checked/evaluator authority boundary so static execution may consume either:

```text
source-backed CheckedStaticBody
or
artifact-backed canonical checked static program
```

The exact API should follow current evaluator ownership.

A conceptual shape is acceptable only if it preserves one evaluator semantics:

```rust
enum StaticExecutionAuthority<'a> {
    CheckedSource(&'a CheckedStaticBody),
    ImportedCanonical(&'a ImportedCanonicalStaticProgram),
}
```

The final design may instead use a trait/view to avoid an enum.

Required invariant:

```text
same checked semantic operation
    -> same evaluator behavior
    -> same static value
    -> same diagnostics/failure
```

The imported form must carry all facts the evaluator currently reads:

- expression/check layout or equivalent canonical op;
- parameter alignment;
- nested checked static applications;
- static constructions;
- associated resolutions;
- sequence/match/pattern/proof operations as supported;
- dependencies;
- result domain;
- diagnostics.

## T3.6 Live static definition snapshot authority

`SemanticSnapshot.static_definitions` must have a legitimate current-main native product representing imported executable static authority.

Prefer extending the semantic **product/execution authority layer**, not manufacturing AST in `StaticDefinition`.

If `StaticDefinitionProduct` must be generalized, preserve source-mode APIs and make source-vs-imported body authority explicit.

## T3 tests

```text
native_witness_plan_equals_source_plan
native_witness_fingerprint_recomputed
derived_rule_native_materialization_matches_source
evidence_sharing_preserved
evidence_cycle_rejected
evidence_wrong_owner_rejected

imported_static_constant_table_matches_source
imported_static_program_evaluates_without_ast
imported_static_application_nested_execution_matches_source
imported_static_failure_diagnostic_matches_source
static_execution_does_not_consult_source_body
```

## T3 exit gate

The snapshot can be populated with native witness, derived-conformance, static constant, and static execution authority without source AST and without staging-only substitutes for compiler readers.

---

# T4 — Complete native `CallableAnalysis`

**Purpose:** finish the largest remaining P7 semantic transplant.

Existing modules are foundations, not complete analyses:

```text
callable_binding_import.rs
callable_expression_import.rs
callable_dispatch_import.rs
callable_specialization_import.rs
callable_evidence_import.rs
denotation_import.rs
diagnostic_import.rs
```

Add focused modules rather than returning to a monolithic importer.

Recommended decomposition:

```text
callable_trait_dispatch_import.rs
callable_contextual_import.rs
callable_flow_import.rs
callable_iteration_import.rs
callable_resolution_import.rs
callable_match_import.rs
proof_import.rs
explanation_import.rs
callable_analysis_import.rs
```

Exact file boundaries may vary.

## T4.1 Complete every expression

Build native `ExpressionAnalysis` with:

- base identity/range;
- type knowledge;
- callable;
- call specialization;
- conditional dispatch;
- conditional capability;
- trait dispatch;
- ambiguous trait-dispatch candidates;
- contextual realization;
- denotation;
- status;
- causal invalidity;
- explanation;
- call ID;
- read safety;
- binding read;
- binding write.

After assembly:

```text
deferred expression authority count == 0
```

for publishable canonical analyses.

## T4.2 Trait dispatch and contextual realization

Port/adapt P7 native reconstruction for:

- exact/evidenced dispatch;
- assumed dispatch;
- candidate provenance;
- selected callable;
- source impl;
- evidence;
- contextual basis realization;
- real embedding;
- coefficient source;
- realization execution plan.

Bind evidence refs through the shared T3 evidence materializer.

## T4.3 Flow graph

Reconstruct native:

```text
FlowGraph
nodes
edges
entry
exits
predecessors
successors
```

Preserve canonical edge order.

Reject:

- out-of-range endpoints;
- duplicate illegal edges;
- missing entry/exits;
- ordinal gaps where the native contract requires dense ordinals.

## T4.4 Iteration and expansion

Reconstruct:

```text
ForIterationLaneResolutionIndex
PositionalExpansionIterationResolutionIndex
KnownInvalidPositionalExpansionIndex
SequenceLiteralResolutionIndex
```

Preserve conformance evidence and exact source expression ownership.

## T4.5 Associated/callable-reference/family application resolution

Reconstruct native indexes for:

```text
AssociatedResolutionIndex
CallableReferenceResolutionIndex
FamilyApplicationResolutionIndex
```

including static vs dynamic family candidate selection.

## T4.6 Match, pattern, and proof authority

Port/adapt P7 proof/match algorithms for:

```text
MatchResolutionIndex
PatternSiteResolutionIndex
pattern resolution trees
pattern bindings
case instantiations
branch proof environments
exhaustiveness
coverage witnesses
local constraints
local proof terms
```

If a P7 wire proof field no longer has a native current-main standalone owner, classify it:

```text
native product
validation-only support authority
superseded / intentionally discarded after complete validation
```

Do not invent a second proof model merely to preserve P7 structure.

## T4.7 Flow summaries and exits

Reconstruct:

```text
entry_flow
BodyExitFacts.normal_returns
BodyExitFacts.throws
BodyExitFacts.unreachable
return_validation
constructor_super_initializer
```

Preserve direct-result dependencies.

## T4.8 Explanation arena and diagnostics

Port/adapt P7:

```text
artifact/import/explanation.rs
artifact/import/diagnostic.rs
```

Build:

```text
ExplanationArena
SemanticDiagnostic[]
DiagnosticCauseId closure
return explanation
```

Validate:

- every explanation ref belongs to the callable;
- cause ordinals are dense/valid;
- explanation graph is closed and acyclic;
- fixes/guidance/type refs are relocated;
- source spans use stable logical module authority.

## T4.9 Dependencies and phase evidence

Reconstruct:

```text
CallableAnalysis.dependencies
CallableAnalysis.semantic_dependencies
CallableAnalysis.observed_import_phases
```

The new phase evidence from T1 must be exact.

Recompute:

```rust
analysis.dependency_fingerprint =
    callable_body_product_fingerprint(&analysis)
```

Do not take that fingerprint from the old wire.

## T4.10 Status and internal incidents

Canonical artifact profile rules:

- `Complete` is allowed.
- other current valid statuses may be imported only if the canonical image contract explicitly supports them.
- `InternalFailure` / internal incidents must fail canonical compiler-image completeness unless a future stable transport is intentionally defined.

Do not quietly convert `InternalFailure` to `Complete`.

## T4 tests

A comprehensive callable fixture must exercise:

```text
generic call specialization
conditional inherent call
trait-dispatch call
ambiguous trait candidates
contextual realization
for iteration
positional expansion
invalid positional expansion
sequence literal
associated resolution
callable reference
dynamic/static family application
match/pattern
proof predicate
normal return
throw
constructor super call
explanation graph
diagnostic guidance/fix
semantic dependencies
observed import phases
```

Required assertions:

```text
complete native CallableAnalysis constructed
deferred sidecars == 0
source/imported product fingerprint equal
lowering-visible fields equal
phase evidence equal
diagnostics/explanations equal
```

Hostile tests must mutate every local ordinal family and prove fail-closed behavior.

## T4 exit gate

No compiler-required `CallableAnalysisRecordV1/V2` information remains only in wire/staging form.

---

# T5 — Complete module execution authority and derived snapshot products

**Purpose:** reconstruct the current post-P7 semantic products needed by executable linking and the remaining load-derived snapshot indexes.

## T5.1 Native module phase products

Materialize:

```text
BTreeMap<ModuleId, ModuleImportPhaseProduct>
```

Validate every binding against the imported linked module's exact symbolic import authority.

Reject:

- unknown binding;
- binding owned by another module;
- duplicate binding phase;
- coverage contradiction.

## T5.2 Native runtime requirements

Materialize:

```text
Arc<[RuntimeDependencyRequirement]>
```

Validate:

- importer exists;
- dependency exists;
- `read_target` module agrees with dependency;
- provenance consumer agrees with importer;
- provenance target agrees with read target when present;
- product/timing discriminants are valid;
- no unauthorized external target;
- canonical ordering/dedup rules.

Do not run source AST requirement extraction during artifact load.

## T5.3 Derived native indexes

Build from complete imported authority:

```text
SurfaceDispatchResolver
TraitDispatchIndex
FormalSemanticProjection
ModuleQueryProducts
semantic graph / dependency authority
module indexes
```

If `SemanticGraph` cannot be derived exactly from persisted module/query/semantic dependencies, promote it to a wire-required family rather than synthesizing an approximation.

## T5.4 Explicitly omitted source/tooling products

Prove that source-free compiler operation does not require:

```text
ParsedModuleUnit for Universe
SourceSemanticIndex
semantic_structure_shards
AdvisoryWorkspace products
presentation source text
```

Compiler-required facts formerly reached through source indexes must already be represented by canonical semantic products.

## T5.5 Internal source-free completeness validator

Add a validator such as:

```text
validate_imported_canonical_completeness(...)
```

It must require all compiler-critical native families, not merely nonempty tables.

Use exact owner coverage/census expectations where possible.

Examples:

```text
every callable definition requiring analysis has an analysis
every witness owner has its conformance/trait authority
every declaration surface member resolves
every static application target exists
every module phase product references real imports
every reusable query dependency can resolve
```

## T5 tests

```text
module_import_phase_products_match_source_generation
runtime_requirements_match_source_generation
runtime_requirement_provenance_survives_import
formal_projection_rebuilt_from_imported_analyses
module_query_products_rebuilt_from_module_world
dispatch_indexes_derive_exactly
source_free_completeness_rejects_missing_family
source_free_completeness_rejects_missing_owner
```

## T5 exit gate

A complete native `SemanticSnapshot` candidate can be assembled without canonical source products.

---

# T6 — Complete dependency-bearing query import

**Purpose:** finish T3 from the old plan against the now-complete native authority.

## T6.1 Generalize imported Ready product representation

Current zero-dependency representation is insufficient.

Extend to retain validated live dependencies, conceptually:

```rust
struct ImportedReadyQueryProduct {
    key: QueryKey,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    product: SemanticProduct,
    dependencies: Arc<[DependencyEdge]>,
}
```

Every dependency edge must contain the freshly validated live dependency product fingerprint.

## T6.2 Required reusable-root profile

Current policy allows these source-free roots after the location gate:

### Module-world roots

```text
UnlinkedInterface
ResolvedImports
LinkedInterface
```

### Semantic-world roots

```text
DeclarationShell
CallableDefinition        [already implemented]
CallableSignature
FieldSignature
DeclarationSurface
TraitHeader
TraitSurface
DataDeclaration           [already implemented]
EnumDeclaration           [already implemented]
AssociatedSurface         [already implemented]
TypeMemberDeclaration     [already implemented]
ModuleStaticBindingCatalog
ModuleStaticBindings
```

`HierarchyEdge` remains aggregate dependency evidence only.

`ParsedModule` remains source-dependent.

## T6.3 Source-free viability proof per family

Before implementing a new Ready root, write down:

```text
exact portable owner
native product source
source-free input authority
portable input fingerprint validation
portable product fingerprint validation
current live input fingerprint function
current live product fingerprint function
exact dependency set
dependency ordering rule
closure rule
cycle rule
publication rule
```

If any item is missing, the family is not ready to implement.

## T6.4 Use current query semantics, not P7 assumptions

Extract exact dependencies from current `db/query.rs`.

Examples that must be handled as current-main defines them:

- alias `DeclarationShell` dependencies;
- `ResolvedImports` dependencies on unlinked interfaces;
- `LinkedInterface` dependencies;
- `TraitSurface` dependency on trait header;
- callable/field signature prerequisites;
- declaration surface dependencies;
- module static binding dependencies.

Do not carry P7 edges forward if current query semantics have changed.

## T6.5 Root closure algorithm

For every imported reusable root:

```text
portable root
    |
    +--> validate owner
    +--> validate row owner
    +--> validate portable fingerprints
    |
    v
reconstruct live SemanticProduct
    |
    v
compute fresh input/product fingerprints
    |
    v
resolve exact dependency indices
    |
    +--> validate each portable dependency fingerprint
    +--> relocate exact dependency key
    +--> reconstruct/locate exact native dependency product
    +--> compute current live dependency fingerprint
    |
    v
closed dependency graph
    |
    +--> reject missing edge
    +--> reject stale edge
    +--> reject duplicate/contradictory edge
    +--> reject illegal cycle
    |
    v
stage Ready publication
```

Dependency evidence alone remains non-authoritative.

## T6.6 Query graph publication

Publish into one fresh current `SemanticDb`.

Required:

```text
QueryState::Ready
validated_revision == current imported db revision
typed SemanticProduct variant matches QueryKind
fresh input fingerprint
fresh product fingerprint
exact DependencyEdge list
```

## T6 hostile tests

For every dependency-bearing root class cover:

```text
wrong product row
wrong owner
portable input fingerprint mismatch
portable product fingerprint mismatch
live input mismatch
live product mismatch
dependency index out of bounds
dropped dependency
extra required dependency mismatch
dependency key mismatch
dependency fingerprint mismatch
duplicate dependency
cycle
foreign project dependency
```

## T6 exit gate

For every reusable family in the active profile:

```text
implemented Ready publication OR
explicit architectural ruling removing it from reusable profile
```

There are no “future importer will handle this” reusable roots left at P2 closure.

---

# T7 — Build the imported canonical generation and ordinary session publication

**Purpose:** cross the actual P2 architecture boundary.

Recommended new module:

```text
phalcom-semantic/src/artifact/session_import.rs
```

Recommended private product:

```text
ValidatedImportedCanonicalGeneration
```

The name may differ. The ownership must not.

## T7.1 Generation contents

It should own/share complete current-native authority:

```text
project universe
canonical external module world

TypeStore
StaticValueStore

DeclarationTypeTable
TypeAliasTable
MapTypeHierarchy
declaration surfaces
SurfaceDispatchResolver
CallableSignatureTable
field signatures
constructor initializations
callable definitions

ConformanceIndex
ConformanceWitnessPlan map
DerivedConformanceRuleIndex
TraitHeaderTable
TraitSurfaceTable
TraitDispatchIndex

DataSemanticTable
EnumSemanticTable
EnumRequirementTable

AssociatedFamilyTable
type-member declaration table
associated static-domain table
static-case catalog table
static-case signature table

StaticDefinitionTable / imported execution authority
StaticConstantTable
module-static binding indexes

complete CallableAnalysis map

ModuleImportPhaseProduct map
RuntimeDependencyRequirement list

ModuleQueryProducts
FormalSemanticProjection
semantic dependency graph

SemanticDb with imported Ready roots
SemanticSnapshot
```

## T7.2 Session origin model

Current `SemanticWorkspaceSession` assumes a source provider.

Refactor the provenance/authority boundary so an imported session does not require one.

A private provenance enum or optional presentation provider is acceptable, for example:

```text
SourceCanonicalBase
ImportedCanonicalBase
```

This is not a second semantic model; it is origin/provenance for the same native semantic base.

Do not call:

```rust
canonical_universe_semantic_baseline()
```

from imported construction.

## T7.3 Imported base vs local overlay

Imported canonical authority is immutable.

Local user/project source changes use the existing session update machinery.

Required behavior:

```text
imported Universe product
    remains immutable

local module update
    may add/invalidate local query state

unaffected imported roots
    remain reusable

local query may depend on imported root
    dependency fingerprint validates normally
```

Do not copy imported canonical products into source-owned mutable structures that later expect Universe AST.

## T7.4 Alias/incremental base state

Source-backed sessions currently retain `alias_sources`.

Imported canonical aliases have no AST.

Separate immutable imported alias authority from local source alias definitions.

Derive:

```text
base alias dependencies
base alias reverse dependencies
base alias forms
```

from imported native `TypeAliasInfo`.

Local overlay alias source state remains source-owned.

The same principle applies to generic/declaration signature dependency indexes: imported base dependencies must be native/stable, while local source contribution remains mutable overlay state.

## T7.5 Retain exact project graph

The imported session retains the same validated `ProjectUniverse`/project IDs used during import.

Local overlays must not rebind imported stable owners.

A project-graph replacement, if supported, must be staged and validated atomically with any owner rebinding.

## T7.6 Whole-load transaction nesting

Current staging `commit()` must not become the final parent-ledger commit too early.

Recommended transaction structure:

```text
caller AllocationBudget
    |
    +-- whole_load child
            |
            +-- decode/import nested children
            |       may commit into whole_load child
            |
            +-- native generation construction
            +-- SemanticDb
            +-- snapshot
            +-- SemanticWorkspaceSession
            |
            +-- FINAL SUCCESS
                    commit whole_load child to caller
```

This permits internal staging commits without charging the caller on a later session-publication failure.

## T7.7 Publication failpoints

Add test-only failpoints after:

```text
module import
semantic staging
native declaration materialization
native callable materialization
query closure
SemanticDb construction
snapshot construction
session construction
final Arc/publication
```

Every failure must leave no visible imported generation and preserve the prior caller project graph/session state.

## T7.8 Source-denial instrumentation

Add test-only instrumentation around canonical source baseline/provider entry.

A fresh-process imported-session test must fail immediately if any canonical source path is touched.

Do not rely solely on an already-warm process counter.

## T7 tests

Create:

```text
phalcom-semantic/tests/artifact_session_import_test.rs
```

Required tests:

```text
artifact_bytes_construct_source_free_semantic_workspace_session
imported_session_uses_one_semantic_db
imported_session_retains_exact_project_graph
imported_session_snapshot_is_complete
imported_session_contains_no_universe_parsed_sources

local_overlay_preserves_imported_owner_identity
local_overlay_reuses_unaffected_imported_roots
local_overlay_invalidates_dependent_local_queries
local_overlay_does_not_mutate_imported_base

publication_failure_before_final_commit_is_retryable
publication_failure_leaves_project_graph_unchanged
publication_failure_leaves_parent_budget_unchanged

fresh_process_import_does_not_build_source_baseline
```

## T7 exit gate

This API exists and passes:

```text
.telosc bytes
    -> SemanticWorkspaceSession
```

without source baseline construction.

That is the primary architectural P2 gate.

---

# T8 — Whole-load allocation, hostile input, rollback, and source-free certification

**Purpose:** certify the actual bytes-to-session boundary, not an import-only subset.

## T8.1 Exact whole-load allocation accounting

Measure the complete supported operation:

```text
bytes
-> container decode
-> section DTO decode
-> project binding
-> type/static reconstruction
-> semantic reconstruction
-> proof/callable reconstruction
-> query graph
-> native indexes
-> SemanticDb
-> SemanticSnapshot
-> SemanticWorkspaceSession
-> final publication
```

Let exact requested/ledger demand be `N`.

Required test:

```text
load with budget N:
    PASS
    expected final state

load with budget N - 1:
    FAIL AllocationError::Exhausted
    no visible generation
    caller ledger unchanged
    project graph unchanged

retry with N:
    PASS
    semantically equivalent final state
```

No synthetic reservation/padding is allowed.

## T8.2 Allocation audit

Audit retained allocations at least for:

```text
Vec
Box
Box<str>
Arc
Arc<[T]>
BTreeMap
HashMap
HashSet
proof graph memoization
callable analysis indexes
query dependency indexes
snapshot collections
session base collections
```

Any unbudgeted live map built during private publication must be either:

1. explicitly accounted by a measured allocation wrapper; or
2. moved behind a budgeted construction API.

## T8.3 Hostile matrix

Extend transport tests and add semantic-import hostile tests.

Minimum categories:

### Container/compatibility

```text
missing section
duplicate section
truncation
container digest
section digest
schema mismatch
semantic ABI mismatch
query ABI mismatch
native catalog fingerprint mismatch
project provenance mismatch
```

### Identity

```text
session project owner
unlisted external project
wrong package fingerprint
wrong resolver receipt digest
stale project target
same-path wrong-project owner
missing internal module
duplicate stable owner
```

### Type/declaration

```text
bad type index
bad kind index
type cycle
bad declaration form
class-object mismatch
intrinsic binding mismatch
generic parameter mismatch
supertype template mismatch
alias dependency mismatch
hierarchy cycle
```

### Witness/proof

```text
wrong trait owner
wrong impl owner
bad evidence ordinal
evidence cycle
selection nesting overflow
associated-type owner mismatch
derived-rule owner mismatch
duplicate proof key
```

### Callable

```text
bad expression ordinal
bad binding ordinal
bad call specialization evidence
bad trait dispatch evidence
bad contextual realization owner
flow edge endpoint
iteration evidence
positional expansion owner
sequence owner
associated resolution owner
family candidate owner
match/pattern ordinal
proof scope
explanation cycle
diagnostic cause ordinal
semantic dependency owner
observed import binding owner
```

### Static

```text
static definition owner
static program op reference
static application target
static constant read
static value lineage
module binding fingerprint/owner
```

### Module execution

```text
phase binding absent
phase binding wrong module
runtime dependency absent
read target mismatch
provenance consumer mismatch
provenance target mismatch
unauthorized runtime project
```

### Queries

```text
wrong root kind
wrong product row
stale portable fp
stale live fp
dependency index
dependency owner
dependency fp
dropped dependency
illegal cycle
open-world Ready attempt
```

### Limits

```text
allocation
entry count
string length
recursion depth
proof depth
callable owner-local family limits
```

For inner semantic corruption tests, mutate the inner section and recompute legitimate outer section/container digests so the test reaches the intended validation layer.

## T8.4 Fresh-process source denial

Run imported construction in a fresh child process.

Require the source-denial hook/counter to prove no canonical source path was used.

Do not accept “fresh session in the same process” as sufficient.

## T8 exit gate

Whole-load N/N−1/retry, failure rollback, source denial, and the complete hostile matrix pass.

---

# T9 — P2 closure, checkpoint update, and P3 handoff

**Purpose:** make P2 administratively and technically complete.

## T9.1 Final verification sequence

Run focused checks first.

### Artifact/semantic focused gate

```bash
cargo fmt --all -- --check

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check -p phalcom-semantic --lib

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_query_import_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_session_import_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_allocation_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_hostile_import_test

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib artifact::query_policy

git diff --check
```

Use the actual final test filenames if implementation chooses different names.

### Source-mode regression gate

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test cli-smoke

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
modules_parse_package::parse_project_compiles_from_declared_entry
```

### Broad P2 integration gate

After focused gates pass:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check --workspace --all-targets
```

Run broader semantic/module/core suites required by the active checkpoint baseline and classify any inherited failures according to the repository lifecycle convention. Do not silently repair unrelated failures.

## T9.2 Complete P7 P2 salvage ledger

For every relevant P7 import/query/publication symbol record:

```text
PORTED_INTACT
PORTED_ADAPTED
SUPERSEDED_BY_CURRENT_MAIN
HISTORICAL_EVIDENCE_ONLY
INTENTIONALLY_NOT_PORTED
DEFERRED_TO_P3
```

Audit at least:

```text
artifact/import.rs
artifact/import/diagnostic.rs
artifact/import/explanation.rs
artifact/import/proof.rs
artifact/query.rs
artifact/manifest.rs
canonical_image.rs loaded-publication/query-closure portions
```

Unknown P2 salvage disposition:

```text
0
```

## T9.3 Produce lifecycle records

At completion:

1. update `CHECKPOINT.md`;
2. mark `ARTF001.C1.P2.R3` complete;
3. create the R3 walkthrough;
4. create the P3 handoff;
5. record exact commit SHA and executed verification evidence;
6. record any explicit deferred baseline failures;
7. change P3 from `PROPOSED` to ready/active only after all P2 gates are satisfied.

## T9.4 P2 final completion statement

The checkpoint may state P2 complete only if this statement is true:

> Validated artifact bytes can be imported transactionally into complete current-main native semantic authority and a source-free ordinary `SemanticWorkspaceSession`; the imported generation uses the caller's exact project graph, one current-main `SemanticDb`, complete safe query reuse with exact dependency closure, ordinary local overlays, and whole-load rollback/allocation guarantees. No canonical Universe parsing, linking, or semantic reanalysis occurs on the artifact-backed construction path.

---

# 10. Query-family completion matrix

This matrix is part of the P2 acceptance record.

| Query family | Policy | P2 action |
|---|---|---|
| `ParsedModule` | EligibleStructural + source-only location | omitted as root |
| `UnlinkedInterface` | EligibleStructural | implement Ready from module world |
| `ResolvedImports` | EligibleStructural | implement Ready + exact deps |
| `LinkedInterface` | EligibleStructural | implement Ready + exact deps |
| `DeclarationShell` | EligibleStructural | implement after T2 |
| `HierarchyEdge` | EligibleStructural + evidence-only | dependency evidence only |
| `CallableDefinition` | EligibleStructural | already implemented; revalidate in final graph |
| `CallableSignature` | EligibleStructural | implement after native signature authority |
| `FieldSignature` | EligibleStructural | implement |
| `DeclarationSurface` | EligibleStructural | implement + exact signature/shell deps |
| `TraitHeader` | EligibleStructural | implement |
| `TraitSurface` | EligibleStructural | implement + exact header deps |
| `DataDeclaration` | EligibleStructural | already implemented |
| `EnumDeclaration` | EligibleStructural | already implemented |
| `AssociatedSurface` | EligibleStructural | already implemented |
| `TypeMemberDeclaration` | EligibleStructural | already implemented |
| `ModuleStaticBindingCatalog` | EligibleStructural | implement |
| `ModuleStaticBindings` | EligibleStructural | implement + exact deps |
| `GenericConformance` | OpenWorld | never unconditional Ready |
| `ConformanceDispatch` | OpenWorld | never unconditional Ready |
| `TraitDispatchBucket` | OpenWorld | never unconditional Ready |
| `CallableBody` | OpenWorld | native snapshot authority, not unconditional Ready |
| `CheckedStaticBody` | OpenWorld | native/imported static execution authority, not unconditional Ready |
| all remaining current kinds | Unsupported | do not publish as Ready |

If current-main query policy changes during implementation, update this table and the exhaustive policy test in the same commit.

---

# 11. Required source/native equivalence tests

Synthetic fixtures are necessary but insufficient.

Before P2 closure, add a test-only projection/oracle capable of producing representative wire authority from a source-built current-main semantic generation **without becoming the P3 production exporter**.

Use it to compare at least:

## Structural

```text
DeclarationTypeTable
TypeAliasTable
MapTypeHierarchy
DataSemanticTable
EnumSemanticTable
```

## Trait/conformance

```text
TraitHeaderTable
TraitSurfaceTable
ConformanceIndex
ConformanceWitnessPlan
DerivedConformanceRuleIndex
TraitDispatchIndex
```

## Callable

```text
CallableSignatureTable
CallableAnalysis
formal projection
```

## Static

```text
static definitions
static constants
static values
module static bindings
source-free evaluation result
```

## Module execution

```text
ModuleImportPhaseProduct
RuntimeDependencyRequirement
```

## Query

```text
Ready root keys
input fingerprints
product fingerprints
dependency edges
```

The equivalence target is **compiler-observable native equality/parity**, not DTO byte equality.

---

# 12. Explicit anti-shortcut rules

The implementer must not close a gate by doing any of the following.

## Architecture weakening

Forbidden:

- keeping `CanonicalUniverseFormalImage` as a second live authority outside the bounded P7 lifecycle recovery. During that recovery, it is the sole loaded compiler-image authority; do not also publish a competing session-backed image. Any later move to ordinary `SemanticWorkspaceSession` publication happens after the P7 lifecycle works and in a separate slice;
- creating an artifact-specific query database;
- creating an artifact-specific type system;
- creating an artifact-local `ProjectUniverse`;
- using path strings as semantic identity;
- allocating replacement project IDs;
- using wire row index as durable owner identity.

## Semantic fabrication

Forbidden:

- default ordinary declaration binding;
- default empty witness selections;
- empty callable phase evidence;
- empty runtime requirements;
- empty dependencies for a dependency-bearing query;
- empty diagnostics/explanations where the owner has them;
- invented AST for imported static definitions.

## Fingerprint weakening

Forbidden:

- publishing portable fingerprint as live fingerprint without recomputation;
- changing current query fingerprint functions simply to match artifact bytes;
- ignoring a stale dependency because the root product matches;
- omitting hierarchy/query dependency evidence.

## Testing weakening

Forbidden:

- changing expected tests to accept partial authority;
- marking unsupported/open-world roots Ready;
- bypassing current source-mode tests;
- testing source denial only in a process that already initialized the source baseline;
- treating early digest rejection as proof of an inner semantic hostile case.

## Transaction weakening

Forbidden:

- committing caller allocation budget before session construction finishes;
- mutating caller project graph before all authority validates without rollback;
- leaving a partially populated DB/snapshot visible on failure.

---

# 13. Recommended implementation/commit sequence inside the single plan

This remains one numbered plan. The following is a safe commit order.

## 13.1 Continuation sequencing amendment — 2026-10-20

The user-provided handoff and follow-up architecture consultation supersede the immediate loader direction recorded in the earlier P2.R3 core API consultation. That consultation remains part of the chronology, but its instruction to remove the P7 image route or move straight to ordinary session publication does not govern this recovery.

Use this order before resuming the table below:

1. Reach an integrated compileable state while preserving the P7 loader and staged work.
2. Restore the P7 `CanonicalUniverseFormalImage` / `CanonicalCompilerImageView` lifecycle, including the loaded canonical reader, source-free completeness, module-query publication, dependency fingerprints, and publication census. Adapt the lifecycle to the current semantic products and V2/V4/V3 schemas, current query policy and owners, product references, portable fingerprint ABI v2, and current live fingerprints.
3. Keep that image as the single loaded artifact compiler-image authority during recovery. Do not replace it with a `SemanticWorkspaceSession`, add a parallel sidecar, or combine image recovery with a session-boundary redesign.
4. Once source-free build/encode/load works, integrate the interrupted T1 phase-product and declaration-owner changes, then resume the ordinary-session publication slices in this plan.

This order preserves the plan's final current-main semantic authority and session-publication acceptance objective. The temporary image path is the handoff's required recovery boundary, not a second semantic engine. Tests remain unrun during the active handoff; focused compile checks are allowed and do not close T1 or P2.

| Slice | Commit objective | Exit condition |
|---|---|---|
| R3.T0 | refresh checkpoint + exhaustive authority manifest | zero unclassified families |
| R3.T1a | declaration/alias schema closure | complete field matrix + wire tests |
| R3.T1b | module execution + callable phase wire authority | all current post-P7 fields representable |
| R3.T2 | native declaration/alias/hierarchy materialization | source/native equivalence |
| R3.T3a | shared evidence materializer + native witness | native witness table |
| R3.T3b | native derived rules | native derived index |
| R3.T3c | static execution authority | source-free static eval parity |
| R3.T4a | trait dispatch/contextual expression sidecars | expression deferred count reduced |
| R3.T4b | flow/iteration/sequence/resolution indexes | native indexes |
| R3.T4c | match/proof/explanation/diagnostics | native proof/diagnostic closure |
| R3.T4d | final `CallableAnalysis` assembly | deferred count zero |
| R3.T5 | phase/runtime requirements + derived snapshot indexes | complete snapshot candidate |
| R3.T6a | module-world Ready roots | three module roots closed |
| R3.T6b | declaration/trait/signature Ready roots | all semantic structural roots closed |
| R3.T6c | module-static Ready roots + global query closure | all reusable roots closed |
| R3.T7a | `ValidatedImportedCanonicalGeneration` | complete private generation |
| R3.T7b | ordinary `SemanticWorkspaceSession` construction | source-free session works |
| R3.T7c | overlay/invalidation/rollback | ordinary local behavior proven |
| R3.T8a | whole-load budget N/N-1/retry | exact allocation gate |
| R3.T8b | hostile matrix + fresh-process source denial | failure/source-free gates |
| R3.T9 | broad verification + lifecycle records | P2 accepted; P3 unblocked |

Each slice should leave the branch buildable and focused-tested.

---

# 14. File-level implementation map

## `phalcom-compiler-artifact`

Expected changes:

```text
src/semantic/aggregates.rs
    declaration/alias structural schema closure

src/semantic/world.rs
    semantic world schema revision
    callable observed-import phase rows
    module execution authority family

src/semantic/world/*
    transport validation for new rows

src/query.rs
    only if final query dependency/profile representation needs amendment

src/header.rs / compatibility constants
    semantic ABI/schema bumps
```

Do not move current semantic decisions into this crate.

## `phalcom-modules`

Expected focused helpers:

```text
compiler_artifact.rs
    stable module/import/runtime requirement conversion

linker.rs
    only minimal constructors/validation APIs required to build native
    ModuleImportPhaseProduct / RuntimeDependencyRequirement safely

project/session ownership
    retain exact existing project IDs and graph
```

Do not broaden package resolution or create artifact-owned project authority.

## `phalcom-semantic/src/artifact`

Current modules should be retained and completed.

Expected additions/refactors:

```text
manifest.rs
declaration_import.rs
evidence_materialize.rs
proof_import.rs
explanation_import.rs
callable_trait_dispatch_import.rs
callable_contextual_import.rs
callable_flow_import.rs
callable_iteration_import.rs
callable_resolution_import.rs
callable_match_import.rs
callable_analysis_import.rs
module_execution_import.rs
session_import.rs
```

The exact module count is not normative. Separation of authority is.

## `phalcom-semantic`

Likely current-main integration points:

```text
declarations.rs
type_alias.rs
snapshot.rs
session.rs
db/mod.rs
db/query.rs
db/fingerprint.rs
static_definition.rs
static_eval/*
derived_conformance.rs
impls / generic_evidence / trait_dispatch as required by constructors
phase_evidence.rs (reference/parity; avoid source path during import)
```

Do not alter fingerprint semantics unless an independent current-main correctness bug is proven.

---

# 15. New tests recommended by this plan

The plan should leave tests organized by boundary rather than one giant file.

Recommended files:

```text
phalcom-semantic/tests/artifact_native_authority_test.rs
phalcom-semantic/tests/artifact_callable_import_test.rs
phalcom-semantic/tests/artifact_query_import_test.rs
phalcom-semantic/tests/artifact_session_import_test.rs
phalcom-semantic/tests/artifact_allocation_test.rs
phalcom-semantic/tests/artifact_hostile_import_test.rs
```

Existing:

```text
phalcom-semantic/tests/semantic_artifact_test.rs
```

may retain focused integration fixtures, but avoid allowing it to become an unstructured thousands-line catch-all.

Wire-only validation remains in `phalcom-compiler-artifact/tests/`.

---

# 16. Verification classification

## During implementation

Use:

```text
FOCUSED_TESTED
```

for coherent completed slices.

Do not claim whole-P2 verification because a subset of transplants pass.

## P2 completion minimum

P2 may finish:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only after all P2-specific gates in T9 execute successfully.

If the repository's broader release suite also executes successfully, the checkpoint may use the stronger classification allowed by the lifecycle convention. Do not infer `RELEASE_COMPLETE` from focused P2 evidence.

---

# 17. P2 completion checklist

The implementer should be able to mark every item below.

## Authority

- [ ] Current snapshot/session coverage manifest is exhaustive.
- [ ] Complete native declaration table exists.
- [ ] Complete native alias table exists.
- [ ] Complete native hierarchy exists.
- [ ] Native witness plans exist.
- [ ] Native derived conformance index exists.
- [ ] Source-free static execution authority is integrated with the one evaluator.
- [ ] Complete native callable analyses exist.
- [ ] Current observed import phases are preserved.
- [ ] Module import phase products exist.
- [ ] Generated runtime requirements exist.
- [ ] All required derived snapshot indexes exist.

## Queries

- [ ] All 46 query kinds retain explicit policy.
- [ ] All active reusable roots are implemented.
- [ ] Dependency-bearing roots carry exact `DependencyEdge`s.
- [ ] Portable fingerprints are validated.
- [ ] Live fingerprints are recomputed.
- [ ] Dependency graph is closed and acyclic.
- [ ] Open-world roots are not unconditional Ready.

## Session

- [ ] Imported generation is private until complete.
- [ ] Ordinary `SemanticWorkspaceSession` can be created from artifact authority.
- [ ] Source canonical baseline is not initialized.
- [ ] Exact ProjectUniverse is retained.
- [ ] Local overlays work.
- [ ] Imported base is immutable.
- [ ] Failure rollback is complete.

## Certification

- [ ] Whole-load exact N passes.
- [ ] N-1 fails.
- [ ] Retry N passes.
- [ ] Hostile matrix passes.
- [ ] Fresh-process source denial passes.
- [ ] Source-mode CLI smoke passes.
- [ ] SlotMap runtime source-mode regression passes.
- [ ] Parse focused source-mode regression passes.
- [ ] Workspace/all-target check passes or inherited blockers are correctly recorded.
- [ ] P7 P2 salvage ledger has zero unknown dispositions.
- [ ] `CHECKPOINT.md` reflects actual final state.
- [ ] R3 walkthrough exists.
- [ ] P3 handoff exists.

---

# 18. Definition of done for P3 handoff

P3 receives exactly this importer contract:

```rust
source-built current-main canonical semantic generation
    |
    | P3 responsibility: root discovery + deterministic wire projection
    v
CompilerArtifactImage / .telosc bytes
    |
    | already complete P2 responsibility
    v
ValidatedImportedCanonicalGeneration
    |
    v
SemanticWorkspaceSession
```

P3 must **not** need to answer:

- how to reconstruct current declarations;
- how to represent current module phase/runtime requirements;
- how to rebuild complete `CallableAnalysis`;
- how to execute imported static definitions;
- what query roots are safe;
- how dependencies are validated;
- how to construct the artifact-backed semantic session;
- where the whole-load atomic boundary is.

If P3 still has to solve any of those, P2 is not complete.

---

# 19. Final architectural outcome of this plan

After `ARTF001.C1.P2.R3` completes, the repository will not yet generate the production `universe.telosc`; that is P3.

It **will** have the complete consumer architecture that makes production artifact generation safe to begin:

```text
portable artifact representation
        +
complete native importer
        +
complete current semantic authority
        +
query dependency closure
        +
source-free imported session
        +
whole-load transactional guarantees
```

That is the correct boundary between P2 and P3.

P3 can then focus purely on the producer side:

```text
current source-built canonical authority
    -> root closure
    -> export
    -> deterministic .telosc
```

without redesigning the compiler's imported semantic architecture.

# 20. Plan amendment — independent intrinsic projection rows (2026-10-20)

- **Trigger:** Live `DeclarationTypeTable` evidence showed that type-only intrinsic static-domain registrations remove the declaration shell. The V2 exporter previously attached both static-domain and intrinsic type-owner authority only to declaration-shell rows, so these compiler-owned projections had no wire representation. No source-free derivation from the kind or runtime owner was established.
- **Decision:** Add separately sorted, project-qualified `intrinsic_static_domains` and `intrinsic_type_owners` row families to `StructuralPrimaryContractsV2`, keyed by `StableDeclarationRef`. Remove actual static-domain and seven-case type-owner projections from `DeclarationShellV2`. Each family is imported independently through `StableOwnerCatalog`; neither is inferred from the other or from runtime ownership. A declaration may carry both rows, and may have no shell.
- **Schema impact:** Keep structural schema V2 and enclosing semantic schema V4 because these schemas are unpublished on this branch. If evidence shows either schema has been emitted, stop and version the affected contract before accepting bytes.
- **Implementation/verification impact:** T1 owns row validation, export/import, allocation charging, and schema constructor updates. No tests are run during this continuation per the active handoff; compile checks do not close T1 or P2.

# 21. User clarification — shell intrinsic owner is redundant runtime authority (2026-10-20)

- **Decision:** Retain `DeclarationShellV2.intrinsic_type_owner` as an `Option<IntrinsicRuntimeOwnerV1>` witness for compatibility with the current schema draft. It is redundant with `binding.runtime_owner`; validation requires exact equality. Its `Pack` case remains runtime-only and does not imply the independent seven-case `IntrinsicTypeOwnerV1::TypePack` row.
- **Scope:** The seven-case intrinsic type-member authority remains represented only by standalone `intrinsic_type_owners` rows. Intrinsic static-domain authority remains represented only by standalone `intrinsic_static_domains` rows. These rows stay independent and may be shell-less.
- **Schema impact:** V2/V4 remain unpublished on this branch, so no ABI bump is needed. If either schema has been emitted, version the affected structural and enclosing semantic artifact schemas before accepting bytes.
- **Verification:** After the wire/export validation update, `cargo check -p phalcom-compiler-artifact --lib`, `cargo check -p phalcom-semantic --lib`, and `cargo check -p phalcom-core --lib` all passed. The semantic crate reported 68 warnings and core one unused-method warning. The handoff still prohibits running tests during this continuation; T1 and P2 remain open.

# 22. Consultation ruling — Pack type-member authority (2026-10-20)

- **Trigger:** The user requests that Pack be modeled with semantic authority beyond its current runtime owner. This may change semantic ownership and projection dispatch, so it requires architecture guidance before implementation.
- **Current contract:** The three-case shell field remains a redundant witness for `binding.runtime_owner`; actual type-member owners stay in independent project-qualified rows. The normative specifications distinguish first-class proper `Pack` from the kinded `TypePack` domain.
- **Ruling:** Pack is already modeled beyond runtime ownership through `IntrinsicTypeApplication::Pack`, `TypeData::Pack` / `PackFromPacks`, and the structural Pack family. The current specs define operations on Pack specializations but no direct intrinsic type-member operations owned by Pack itself. Keep the current three-case runtime owner; do not infer `TypePack` or add an eighth owner without a direct Pack-member contract.
- **Plan impact:** No amendment is needed for independent P2 continuation. A direct Pack projection API would require a Level 2 semantic amendment, normative specification, and changes to projection lookup, artifact transport, and acceptance tests. P3 remains proposed until P2 closes. Full source evidence and the consultation are recorded in `CHECKPOINT.md`.
- **Status:** Architecture adviser returned high confidence for the current specification and P2 boundary. Implementer may resume P2 work independent of a new direct Pack projection contract.
