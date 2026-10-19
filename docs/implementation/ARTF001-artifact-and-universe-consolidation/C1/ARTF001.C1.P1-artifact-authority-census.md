---
id: ARTF001.C1.P1
kind: architectural-census
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINED
---

# ARTF001.C1.P1 — Artifact and Durability Authority Census

## 1. Executive Summary

This report establishes the architectural and durability census for Program `ARTF001.C1` (Artifact and Universe Consolidation). It analyzes the persistence and compiler models across two branches:
- **Current `main`** (`39334961e61cc3b9e251d03bcd348bceb50abbc6`), which owns current Telos semantics, type system, AST, compiler lowering, incremental database, and recent LANG007.C4 static values/definitions metadata.
- **Certified PERF003 P7** (`077d93d3bfcd5d3bca0127c4aac18abb55c2ed94`), which owns the proven `.telosc` artifact container, source-free compiler image contract, allocation accounting transaction model, and certified benchmark harnesses.

The fundamental rule of this consolidation is:
```text
Current main has semantic authority.
P7 artifact machinery adapts to current main.
There is exactly one semantic ontology and one durable metadata model post-consolidation.
```

---

## 2. Current-Main Architecture

Current `main` has advanced 273 commits beyond the merge base with `perf/universe-characterization`. Key owners in the durability and compiler pipeline include:

### 2.1 Type Metadata and Static Definitions (`phalcom-type-meta`)
- `src/static_value.rs`: Defines `StaticValueNode` (15 variants: Unit, Bool, Char, Int, FloatBits, String, Symbol, Type, StaticTerm, Tuple, Sequence, Record, Pack, Data, Variant), `StaticValueNodeId`, `StaticValueNodeEntry`, `StaticConstantRecord`, `StaticValueDomainRef`.
- `src/static_definition.rs`: Defines `StaticDefinitionRecord`, `StaticParameterRecord`.
- `src/field_key.rs`: Defines `FieldKeyRef` (Symbol, String, Int, Bool) for canonical heterogeneous Record keys.
- `src/bundle.rs`: Defines `SemanticMetadataBundle` with header fingerprints and type/declaration tables.
- `src/generic.rs`, `src/declaration.rs`, `src/scoped_type.rs`, `src/type_node.rs`: Provide durable type graph representations.

### 2.2 Semantic Engine & Database (`phalcom-semantic`)
- `src/static_value/store.rs`: Interned, snapshot-stable `StaticValueStore`, `StaticValueRef`, `StaticValueData`, `StaticValueId`.
- `src/static_definition.rs`: Checked semantic `StaticDefinition`, `StaticDefinitionSignature`, `CheckedStaticApplication`.
- `src/static_constant.rs`: Checked `StaticConstantId`, `CheckedStaticExpression`, `StaticEvalOutcome`.
- `src/db/`: Fine-grained query database (`SemanticDb`, `QueryKey`, `ProductFingerprint`, `InputFingerprint`, `Scheduler`). Supports 40+ query keys including `StaticDefinition`, `StaticConstant`, `CheckedStaticBody`.
- `src/session.rs`: `SemanticWorkspaceSession` managing incremental rebuilds, snapshot replacement, and database state.

### 2.3 Compiler & Universe Pipeline (`phalcom-core`)
- `src/modules/canonical_universe.rs`: Process-shared `CanonicalUniverseProgram` caching `CompiledProgram`, `NativeSourceIndex`, and availability contracts.
- `src/modules/compile_session.rs`: `ProgramCompilationSession` for incremental module compilation with persistent synthetic identities.
- `src/modules/compile.rs`: `ProgramCompiler` compiling entry selections through `canonical_universe_program()`.

---

## 3. Certified P7 Architecture

P7 introduced a full source-free compilation subsystem designed to eliminate the ~9.5s cold compilation startup by pre-serializing the canonical Universe.

### 3.1 Container & Codec (`phalcom-compiler-artifact`)
- Dedicated crate with zero business logic dependencies.
- `container.rs`: `TeloscContainerBuilder` and `TeloscContainerView` implementing deterministic container serialization, directory layout, and whole-file hashing.
- `header.rs`: `TeloscHeaderV2` containing magic, schema versions, package identity, compatibility fingerprints, and section counts.
- `section.rs`: `SectionDirectoryEntryV1`, `SectionKind` (MODULE_WORLD, SEMANTIC_WORLD, QUERY_IMAGE, METADATA, PRESENTATION, SOURCE_MAPS, EXECUTABLE), and `SectionFlags` (REQUIRED, COMPRESSED).
- `digest.rs`: Domain-separated BLAKE3-256 hashing (`DOMAIN_ARTIFACT`, `DOMAIN_SECTION`, etc.).
- `limits.rs`: `ValidationLimits` fail-closed bounds on bytes, counts, depths, and string sizes.

### 3.2 Allocation Transaction Accounting (`phalcom-type-meta::allocation`)
- `AllocationBudget`: Tracks cumulative bytes requested by decoders.
- `ChildAllocationBudget`: Section-local isolated sub-budgets. Committed only on section success; failed sections drop uncommitted bytes.
- Exact whole-load budget N ensures deterministic memory envelopes.

### 3.3 Semantic & Query Wire Representation
- `module_world.rs`: Portable, source-free `ModuleWorldV1` capturing module topology, exports, declarations, and bindings.
- `semantic/world.rs`: `SemanticWorldV3` holding 22 tables of semantic products (type graphs, structural contracts, static values, callable analyses, witness plans, etc.).
- `query.rs`: `QueryImageV1` storing precomputed query roots and dependency evidence.
- `metadata.rs`: `MetadataSectionV1` wrapping `SemanticMetadataBundle` with preflight limits and envelope verification.

### 3.4 Runtime & Core Integration
- `phalcom-semantic/src/canonical_image.rs`: `CanonicalUniverseFormalImage` and `CanonicalSemanticQueryImage`.
- `phalcom-core/src/modules/canonical_artifact.rs`: Builds and loads `.telosc` containers, binding them into `CanonicalCompilerImageView`.

---

## 4. Persistence Authority Reconciliation Matrix

Every concept shared by current-main durability and P7 artifact persistence is assigned one unambiguous target owner below.

| Concept | Current-Main Live Owner | Current-Main Durable Owner | P7 Owner | Overlap? | Target Owner After Consolidation | Migration Strategy | P2 Action | Risk / Unresolved Question |
|---|---|---|---|---|---|---|---|---|
| **1. StableProjectRef** | `phalcom-modules::identity::ProjectIdentity` | `phalcom-type-meta::identity::StableProjectRef` | `phalcom-type-meta::StableProjectRef` / `TeloscPackageIdentity` | Yes | `phalcom-type-meta::StableProjectRef` | Canonical durable project identity in `type-meta`; wire container uses identical structure. | Reuse `StableProjectRef` in `phalcom-compiler-artifact::identity`. | None |
| **2. StableModuleRef** | `phalcom-modules::ModuleId` | `phalcom-type-meta::identity::StableModuleRef` | `phalcom-type-meta::StableModuleRef` + `ArtifactModuleId(u32)` | Yes | `phalcom-type-meta::StableModuleRef` (durable), `ArtifactModuleId` (wire DTO) | Cross-module identity is `StableModuleRef`; `ArtifactModuleId` is strictly a container section-local compact index. | Keep `ArtifactModuleId` in `module_world.rs` as wire index; resolve to `StableModuleRef`. | None |
| **3. StableDeclarationRef** | `phalcom-semantic::identity::DeclarationId` | `phalcom-type-meta::identity::StableDeclarationRef` | `phalcom-type-meta::identity::StableDeclarationRef` | Yes | `phalcom-type-meta::identity::StableDeclarationRef` | Identical across both branches; keep in `phalcom-type-meta`. | Direct reuse. | None |
| **4. Type Identity** | `phalcom-semantic::types::id::TypeId` | `phalcom-type-meta::type_node::TypeNodeId` | `ArtifactTypeId(u32)` / `TypeGraphV1` | Yes | Live: `TypeStore` (`TypeId`); Metadata: `TypeNodeId`; Wire: `ArtifactTypeId` | In-memory live authority is `TypeStore`. `TypeGraphV1` serializes the immutable prefix of `TypeStore` into the artifact. | Adapt P7 `TypeGraphV1` wire codec to serialize/deserialize all current-main `TypeData` variants. | Main has added types (e.g. record row variants, static data). Codec must cover all. |
| **5. Type Metadata** | `phalcom-semantic::metadata` | `phalcom-type-meta::SemanticMetadataBundle` | `METADATA` section wrapping `SemanticMetadataBundle` | Yes | `phalcom-type-meta::SemanticMetadataBundle` | `METADATA` section wraps `SemanticMetadataBundle` verbatim. Preflight limits validate envelope before bundle decoding. | Import `MetadataSectionV1` and preflight inspector into `phalcom-compiler-artifact`. | Low; main added static value metadata to bundle. |
| **6. StaticValue** | `phalcom-semantic::static_value::store::StaticValueData` | `phalcom-type-meta::static_value::StaticValueNode` | `StaticValueGraphV1` (`StaticValueV1`) in artifact | Yes | Live: `StaticValueStore` (`StaticValueData`); Durable: `StaticValueNode`; Wire: `StaticValueGraphV1` | Live semantic authority is `StaticValueStore`. Durable metadata is `StaticValueNode`. Wire DTO is `StaticValueGraphV1` which mirrors `StaticValueData` 1:1. No parallel semantic types. | Adapt `StaticValueGraphV1` in artifact crate to match current-main `StaticValueData` and `FieldKeyRef`. | Ensure exact IEEE-754 float bits, large integers, pack slices, and record fields round-trip. |
| **7. StaticDefinition** | `phalcom-semantic::static_definition::StaticDefinition` | `phalcom-type-meta::static_definition::StaticDefinitionRecord` | `StaticDefinitionRecordV1` in `world.rs` | Yes | Live: `StaticDefinition`; Durable: `StaticDefinitionRecord`; Wire: `StaticDefinitionRecordV1` | Current-main `StaticDefinition` is semantic authority. `StaticDefinitionRecordV1` is the wire DTO in `SEMANTIC_WORLD`. | Align `StaticDefinitionRecordV1` fields with current-main `StaticDefinitionRecord` and `StaticDefinitionSignature`. | Low. |
| **8. StaticConstant** | `phalcom-semantic::static_constant::StaticConstantId` | `phalcom-type-meta::static_value::StaticConstantRecord` | `StaticConstantRecordV1` in `world.rs` | Yes | Live: `StaticConstantId`; Durable: `StaticConstantRecord`; Wire: `StaticConstantRecordV1` | Wire DTO maps directly to `StaticConstantRecord` in `type-meta` and `StaticConstantId` in `semantic`. | Align wire record fields with current-main. | Low. |
| **9. FieldKey** | `phalcom-semantic::field_key::FieldKey` | `phalcom-type-meta::field_key::FieldKeyRef` | `FieldKeyV1` in `type_store.rs` | Yes | `phalcom-type-meta::field_key::FieldKeyRef` (durable), `FieldKey` (live) | Unify wire `FieldKeyV1` with `FieldKeyRef` (Symbol, String, Int, Bool). | Update `FieldKeyV1` in artifact crate to mirror `FieldKeyRef`. | Low. |
| **10. Generic / Static Arguments** | `phalcom-semantic::types::parameter::GenericSignature` | `phalcom-type-meta::generic::GenericSignatureRef` | `GenericSignatureV1`, `StaticArgumentV1` | Yes | Live: `phalcom-semantic`; Durable: `phalcom-type-meta`; Wire: `GenericSignatureV1` | Wire DTOs in `phalcom-compiler-artifact` convert to/from canonical `GenericSignature`. | Ensure wire DTOs handle kinded vs ordinary static parameters. | Low. |
| **11. Module World** | `phalcom-modules::session::WorkspaceModuleSession` | None | `phalcom-compiler-artifact::module_world::ModuleWorldV1` | Yes | Wire: `ModuleWorldV1`; Live: `CanonicalExternalModuleWorld` | P7 introduced this layer for source-free linking. Adopt into `phalcom-compiler-artifact` and `phalcom-modules`. | Import `module_world.rs` into `phalcom-compiler-artifact`; prepare adapter in `phalcom-modules`. | Verify all current-main module expose rules are captured. |
| **12. Semantic World** | `phalcom-semantic::snapshot::SemanticSnapshot` | None | `phalcom-compiler-artifact::semantic::world::SemanticWorldV3` | Yes | Live: `CanonicalUniverseFormalImage`; Wire: `SemanticWorldV3` | Adopt P7's 22-table `SemanticWorldV3` schema in `phalcom-compiler-artifact`. Live authority remains `SemanticSnapshot`. | Port `SemanticWorldV3` schema definitions into `phalcom-compiler-artifact`. | High volume of tables; each must match current-main semantic types. |
| **13. Query Image** | `phalcom-semantic::db::SemanticDb` | None | `phalcom-compiler-artifact::query::QueryImageV1` | Yes | Live: `CanonicalSemanticQueryImage`; Wire: `QueryImageV1` | Query image acts as immutable root provider for `SemanticDb`. It does NOT duplicate the database engine. | Port `QueryImageV1` wire DTO and `CanonicalSemanticQueryImage`. | Ensure query kinds cover all cold compile queries. |
| **14. Query Dependency Identity** | `phalcom-semantic::db::dependency::DependencyEdge` | None | `PortableQueryKeyV1` | Yes | Live: `QueryKey`; Wire: `PortableQueryKeyV1` | Convert between `QueryKey` and `PortableQueryKeyV1` at artifact boundary. | Update `PortableQueryKindV1` enum to include current-main query variants. | Low. |
| **15. Query Fingerprints** | `phalcom-semantic::db::key::ProductFingerprint(u64)` | `Fingerprint128` (in `type-meta`) | `ProductFingerprint(u64)` | Yes | `phalcom-semantic::db::key::ProductFingerprint` | In-memory authority is `ProductFingerprint`. Wire format uses raw `u64`. | Reuse `ProductFingerprint` during export/import. | Low. |
| **16. Callable Semantic Products** | `CallableDefinition`, `CallableSignature`, `CallableAnalysis` | None | `Callable*RecordV1` in `world.rs` | Yes | Live: `phalcom-semantic`; Wire: `Callable*RecordV1` | Semantic authority stays in `phalcom-semantic`; DTOs adapt to current-main callable structures. | Align DTOs with current-main callable analysis fields. | Medium. |
| **17. Callable Proof Products** | `VerificationConditions`, `GenericConformance` | None | `proof.rs`, `explanation.rs` in `world.rs` | Yes | Live: `phalcom-semantic`; Wire: `ProofRecordV1` | Adapt proof wire DTOs to current-main verification conditions. | Include proof DTOs in `world.rs`. | Medium. |
| **18. Match-Lowering Products** | `phalcom-core::modules::semantic_lowering::MatchLoweringSpec` | None | Lowered from `SemanticSnapshot` directly | Yes | `phalcom-core::modules::semantic_lowering` | No wire DTO needed. Lowering consumes `SemanticSnapshot` directly from the imported formal image. | Verify source-free match lowering from imported image. | Low. |
| **19. Metadata Section** | `phalcom-semantic::metadata` | `phalcom-type-meta::SemanticMetadataBundle` | `MetadataSectionV1` | Yes | `phalcom-type-meta::SemanticMetadataBundle` (durable), `MetadataSectionV1` (wire) | Encapsulate bundle directly inside `MetadataSectionV1`. | Import `metadata.rs` into `phalcom-compiler-artifact`. | None. |
| **20. Materialization Descriptors** | `phalcom-core::modules::artifact::ModuleMaterializationPlan` | None | Derived dynamically from compiler image | Yes | `phalcom-core::modules::artifact` | Generated in `ProgramCompiler` from `CanonicalCompilerImageView`. | Connect to `CanonicalCompilerImageView`. | Low. |
| **21. Artifact Container** | None | None | `TeloscContainerBuilder`, `TeloscContainerView` | No (P7 only) | `phalcom-compiler-artifact::container` | Pure container subsystem; import nearly unchanged. | Create crate and add container module. | Very low. |
| **22. Section Table** | None | None | `SectionDirectoryEntryV1`, `SectionKind`, `SectionFlags` | No (P7 only) | `phalcom-compiler-artifact::section` | Pure directory subsystem; import intact. | Part of `phalcom-compiler-artifact`. | None. |
| **23. Schema Compatibility** | `NATIVE_SURFACE_SCHEMA_VERSION`, `TYPE_METADATA_SCHEMA_VERSION` | Same | `TeloscHeaderV2` compatibility fields | Yes | `TeloscHeaderV2` + `CanonicalCompatibilityIdentity` | Formalized compatibility vector in header; fail-closed validation. | Port header and validation checks. | Low. |
| **24. Digest** | `Fingerprint128` | Same | `ArtifactDigest` (BLAKE3-256) + domain separation | Yes | Artifact: `ArtifactDigest`; Metadata: `Fingerprint128` | Distinct roles: 256-bit for container/sections, 128-bit for metadata nodes. | Import `digest.rs` into `phalcom-compiler-artifact`. | None. |
| **25. Limits** | Ad-hoc limits | None | `ValidationLimits` | Yes | `phalcom-compiler-artifact::limits::ValidationLimits` | Unified fail-closed limits configuration. | Part of `phalcom-compiler-artifact`. | None. |
| **26. Allocation Accounting** | None | None | `AllocationBudget`, `ChildAllocationBudget`, `AllocationError` | No (P7 only) | `phalcom-type-meta::allocation` | Port allocation transaction ledger to `phalcom-type-meta` on `main`. | Add `allocation.rs` and `budgeted.rs` to `phalcom-type-meta`. | Low. |
| **27. Incremental Semantic Parent** | `SemanticWorkspaceSession` | None | `CanonicalUniverseFormalImage` | Yes | `SemanticWorkspaceSession::with_formal_image` | Formal image acts as immutable base; session compiles user modules into overlay DB without mutating parent. | Design takeover API on `SemanticWorkspaceSession`. | Medium. |
| **28. Local Revision Overlay** | `ProgramCompilationSession` | None | Same using formal image | Yes | `ProgramCompilationSession` | Backed by `SemanticWorkspaceSession` with immutable Universe formal image. | Adapt `ProgramCompilationSession` in P3/P4. | Low. |

---

## 5. Keep / Adapt / Replace Decisions

### 5.1 KEEP (Import Almost Intact from P7)
1. **Container substrate (`phalcom-compiler-artifact`)**: `container.rs`, `header.rs`, `section.rs`, `codec.rs`, `digest.rs`, `error.rs`, `identity.rs`, `limits.rs`.
2. **Allocation accounting (`phalcom-type-meta`)**: `allocation.rs` (`AllocationBudget`, `ChildAllocationBudget`, `AllocationError`), `budgeted.rs` (`BudgetedSortedMap`).
3. **Metadata envelope wrapper (`phalcom-compiler-artifact`)**: `metadata.rs`, `preflight.rs`.
4. **Container unit and allocation tests**: `container_test.rs`, `container_allocation_test.rs`, `metadata_test.rs`.

### 5.2 ADAPT (Survives Architecturally, Must Target Current-Main APIs)
1. **`module_world.rs`**: Adapt wire DTOs to serialize current-main `LinkedProgram`, `ModuleAuthorityMap`, and exposure structures.
2. **`semantic/world.rs`**: Adapt 22 table schemas to serialize current-main `TypeStore`, `StaticValueStore`, `FieldKeyRef`, `StaticDefinition`, and `StaticConstant`.
3. **`query.rs`**: Adapt `PortableQueryKindV1` to include query kinds added by current-main (e.g. `StaticDefinition`, `StaticConstant`, `CheckedStaticBody`).
4. **`phalcom-semantic/src/canonical_image.rs`**: Update `CanonicalUniverseFormalImage` to freeze current-main `SemanticSnapshot` and `SemanticDb`.
5. **`phalcom-semantic/src/artifact/`**: Adapt export/import modules (`export.rs`, `import.rs`, etc.) to translate between current-main live semantic structures and artifact wire DTOs.
6. **`phalcom-core/src/modules/canonical_artifact.rs`**: Adapt builder and loader to integrate with current-main `CompiledProgram` and `CanonicalUniverseProgram`.

### 5.3 REPLACE / UNIFY (Current-Main Implementation Supersedes P7)
1. **P7 StaticValue ontology**: P7 had separate static value types in `phalcom-compiler-artifact`. These are replaced/unified with current-main `phalcom-type-meta::static_value::StaticValueNode` and `phalcom-semantic::static_value::store::StaticValueData`. Wire DTOs strictly mirror `StaticValueData`.
2. **P7 FieldKey**: Unified with `phalcom-type-meta::field_key::FieldKeyRef`.

### 5.4 HISTORICAL (Historical Evidence Only)
1. **P7 exact allocation N = 373,069,634 bytes**: Historical evidence from old tree. Mainline artifact will measure its own exact allocation N.
2. **P7 container digest `b91ccc4e...`**: Historical evidence.
3. **P7 benchmark results in `docs/perf003-p7-audit/`**: Archived as reference baseline.

---

## 6. Future Universe Cutover Surface (P5)

During P1-P4, `phalcom-core/core/universe/` remains the sole production authority. When cutover occurs in P5, the following locations must be modified together:

1. **`phalcom-modules/src/builtin.rs`**:
   - `UNIVERSE_NODES` tree specification.
   - `UniverseSourceProvider::source_text()` implementation (currently contains 100+ `include_str!("../../phalcom-core/core/universe/...")` invocations). Must redirect to `packages/universe/src/...`.
2. **`phalcom-modules/src/builtin_interface.rs`**:
   - Builtin module paths, interface declarations, and root re-exports.
3. **`phalcom-semantic/src/prelude.rs`**:
   - Default imported symbols, type alias definitions, and prelude module paths.
4. **`phalcom-native-meta/src/universe.rs`**:
   - Known runtime class names, method selectors, and native descriptor mappings.
5. **`phalcom-core/src/native/source.rs`**:
   - `NativeSourceIndex::build()` and source anchor discovery scanning universe modules.
6. **`phalcom-core/src/modules/canonical_universe.rs`**:
   - `canonical_universe_program()` build routine and bootstrap module order.
7. **`phalcom-core/src/vm/bootstrap.rs`**:
   - Eager class initialization order, sacred kernel overrides, and class object wiring.
8. **LSP Builtin Source Mapping (`phalcom-lsp`)**:
   - Virtual URI mapping (`universe://...`) for "Go to Definition" on standard library symbols.
9. **Tests referencing `core/universe`**:
   - Any integration tests asserting on file paths or legacy Universe module names.

### Invariant for P5:
```text
physical path: packages/universe
semantic project identity: ProjectIdentity::Universe
```
`packages/universe` must retain builtin project identity; it must NEVER be loaded as an external `Resolved` project.

---

## 7. CLI and Cache Integration Surface (P6)

To avoid bolting caching independently onto different entry points in P6, the universal resolution point is identified:

### Centralized Authority: `canonical_universe_program()`
All compiler and runtime paths in the repository converge on:
```rust
phalcom_core::modules::canonical_universe_program() -> Result<&'static CanonicalUniverseProgram, &'static CanonicalUniverseBuildError>
```

Current consumers that automatically inherit artifact loading through this single point:
1. **CLI Run (`phalcom <path>`)**: Calls `ProgramCompiler::compile_entry_selection_with_mode()`, which invokes `canonical_universe_program()`, then runs `VM::new()`, which calls `VM::bootstrap_tier()` -> `canonical_universe_program()`.
2. **CLI Check (`phalcom check <path>`)**: Calls `ProgramAnalyzer::analyze_entry_selection()`, which invokes `canonical_universe_program()`.
3. **REPL (`phalcom-repl`)**: Constructs `VM::new()`, calling `canonical_universe_program()`.
4. **Fixture Compilation Session (`ProgramCompilationSession`)**: Uses `SemanticWorkspaceSession` with canonical universe state.
5. **Package Runtime Runners**: Uses `ProgramCompiler` and `VM::new()`.
6. **Native Installation**: Calls `canonical_universe_program()` to verify native contracts.

In P6, the artifact resolver will hook directly inside `canonical_universe.rs`:
```text
canonical_universe_program()
    -> Check for valid cached universe.telosc
    -> If valid: load source-free CanonicalCompilerImageView in ~300ms
    -> If missing or invalid: build from source, write universe.telosc, return compiler image
```
All consumers will automatically benefit without individual code changes.

---

## 8. Open Questions & Risks

1. **Table Volume in `SEMANTIC_WORLD`**: `SemanticWorldV3` has 22 distinct tables. During P2/P3, each table must be mapped precisely to current-main types. Scope should be addressed incrementally: container first (P2), then semantic tables (P3).
2. **Type Graph Schema Versioning**: As current-main has added static data and record row enhancements, the type graph codec schema version will be incremented to `TYPE_GRAPH_SCHEMA_V2` to distinguish it from the historical P7 schema.
3. **Exact Allocation Thresholds**: P7 required an exact byte limit N for whole-load allocation. In mainline integration, N will be re-baselined once current-main semantic products are serialized.

---

## 9. Recommended P2 Implementation Order

P2 should implement the smallest coherent slice that introduces the artifact crate and builds without altering compiler behavior:

1. **Step 1: Allocation Accounting in `phalcom-type-meta`**:
   - Add `src/allocation.rs` (`AllocationBudget`, `ChildAllocationBudget`, `AllocationError`).
   - Add `src/budgeted.rs` (`BudgetedSortedMap`).
   - Expose in `phalcom-type-meta/src/lib.rs`.
   - Port tests `tests/allocation_helpers.rs` and `tests/allocation_budgeted_map.rs`.

2. **Step 2: Create `phalcom-compiler-artifact` Crate**:
   - Create crate directory and `Cargo.toml`.
   - Add core container files: `error.rs`, `limits.rs`, `digest.rs`, `codec.rs`, `header.rs`, `section.rs`, `identity.rs`, `container.rs`.
   - Add metadata section: `metadata.rs`, `metadata/preflight.rs`.
   - Register in root workspace `Cargo.toml`.
   - Port unit tests: `container_test.rs`, `container_allocation_test.rs`, `metadata_test.rs`.

3. **Step 3: Verify Workspace Compilation**:
   - Verify `cargo check --workspace --all-targets`.
   - Verify `cargo test -p phalcom-compiler-artifact`.
   - Verify compiler behavior on `main` is completely untouched.
