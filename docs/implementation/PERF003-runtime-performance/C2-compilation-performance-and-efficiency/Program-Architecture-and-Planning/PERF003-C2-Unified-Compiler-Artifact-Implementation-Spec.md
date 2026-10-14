# Telos — Unified Compiler, Canonical Images, and Compiled Package/Project Artifacts

## Architectural and Patch-Grade Technical Implementation Specification

| Field | Value |
|---|---|
| Program | `PERF003` — compiler/runtime performance and compiled artifact infrastructure |
| Checkpoint | `PERF003.C2` — canonical dependency ownership, fast compilation, and artifact-ready compilation architecture |
| Document status | **PROPOSED SPECIFICATION — architecture for review; implementation not claimed** |
| Repository | [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang) |
| Audited source revision | `c761de8d855993ea83e3597b016eec1f70d55d58` on `perf/universe-characterization` (revalidate immediately before execution) |
| Date | 2026-10-13 |
| Source inputs | `Pasted text.txt` (3,005-line unified investigation handoff); `PERF003.C2-detailed-implementation-plan.md`; `CHECKPOINT.md`; first architectural review; read-only repository investigation, Slices 1–12; follow-up GitHub source verification |
| Intended audience | Implementers, compiler architects, reviewers, performance engineers, artifact/VM owners |
| Change boundary | Multiple independently gated plans under **one** normative architecture; avoid a single unreviewable patch |

> **Normative language.** **MUST/MUST NOT** are correctness and acceptance conditions; **SHOULD** denotes preferred implementation shape; **MAY** denotes permitted alternatives subject to the same invariants. Rust types in sections labeled **proposed** are *illustrative target APIs*, not assertions that the types currently exist. Line references link to the audited source revision; verify that code still matches before editing.

> **Evidence separation.** `SOURCE-CONFIRMED` denotes observed source mechanisms; `MEASURED-HISTORICAL` is recorded performance at an earlier revision; `DESIGN-REQUIREMENT` is a target contract; `VERIFY-FIRST` is an unresolved technical decision requiring source experiments or proof. Do not present architectural expectations as measured improvements. This specification includes **new durable-artifact requirements** not yet established by the source handoff; those requirements are explicitly proposed rather than falsely attributed to the repository.

---

# 0. Decision Summary and Scope

## 0.1 Overall objective

Establish a **dependency-image compiler** in which canonical Universe, ordinary packages, and projects are independently compiled and reusable. In the immediate phase, canonical module/semantic/compiler products are built once per process; user code compiles against immutable external module and semantic images using workspace-local overlays. In subsequent phases, those products have well-defined, validated **serialized artifacts**, permitting package distribution, project build caching, cold-start image loading, and source-free execution of genuinely precompiled code.

The normative end state is:

```text
SOURCE / PROJECT MANIFEST                  ARTIFACT STORE / PACKAGE REGISTRY
          │                                           │
          ▼                                           ▼
Package graph + lock resolution  ◄──── manifest / content-digest validation
          │                                           │
          ├── source-owned modules                   ├── imported module image
          │                                           ├── semantic interface image
          │                                           ├── portable executable image
          │                                           └── typing/reflection metadata
          ▼                                           │
Local parse + import/link ◄────────────────────────────┘
          ▼
Formal local analysis using immutable external semantic facts
          ▼
Canonical/local coherence and proof queries
          ▼
Local compiler projection + portable code emission
          ▼
Versioned, deterministic, relocatable package artifact
          │
          ├── downstream compilation: imported module/semantic image only
          └── execution: verify → relocate → register → initialize
```

**Success is not merely faster tests.** A novel source program must compile without reconstructing immutable dependencies. A saved compiled package must be consumable without its source. A project must resolve, build, and optionally package its dependency graph deterministically without conflating project identity, package publication identity, and VM instance identity.

## 0.2 Three separable work horizons

| Horizon | Normative deliverable | Baseline/gate |
|---|---|---|
| **H1 — warm-process compilation** | Process-shared canonical module, formal-semantic, query, and compiled images; sound workspace overlays and external linking; no unnecessary canonical work per fixture | Historical ~45.56 s tiny-fixture median; zero forbidden work counters after cutover |
| **H2 — persisted dependency packages** | Durable package module/semantic interface and reusable compiler products; stable identifiers, dependency manifests, versioned schema, import relocation | Downstream packages compile without reparsing/rechecking precompiled dependencies |
| **H3 — source-free execution and cold startup** | Portable code section, VM-independent constants and relocation tables; validated loader; prebuilt Universe image; cold process loads rather than semantically builds/VM-compiles source | Loaded artifacts execute without parser, semantic checker, or VM source compiler; cold-start measurements |

H1 is necessary for H2/H3, but H1 **must not be blocked** on unrelated source-free code serialization or a frozen initialized VM heap. The published end-state contract is unified, while implementation is phased.

## 0.3 Distinguish four different things named “artifact”

1. **Module/package linked interface artifact** — existing [`ResolvedPackageArtifact`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/artifact.rs#L1-L64), an **in-memory** immutable linked package interface container; it is *not* a disk-format implementation.
2. **Semantic dependency image** — proposed in-memory immutable formal tables, type/static arena roots, eligible query products, rules/witnesses, with provenance and world identity.
3. **VM-independent compilation plan** — existing [`CompiledProgram` / `CompiledModule`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L197-L234), retaining linked data, source text, and lowering semantics; **currently not standalone compiled bytecode**.
4. **Saved precompiled package/project artifact** — **new**, versioned, content-verified, relocatable on-disk product with semantic dependency sections and actual portable executable prototypes; runtime VM loading MUST NOT recompile source.

Do not collapse these categories into a single Rust mega-struct, or conflate a compiled bundle with an initialized VM heap image.

## 0.4 Explicit non-goals for the first image cutover

No separate semantic analyzer; no change to language conformance/coherence; no weakening diagnostics; no `opt-level=3` workaround; no global TypeStore deep-copy rewrite; no frozen VM heap, GC, closure or mutable cache serialization; no on-disk schema forced into H1; no implicit semantic fallback to source at artifact execution time; no user-code cache keyed only on filename or package version.

---

# 1. Source-Grounded Repository Ownership and Exact Integration Map

The following are **existing** files and symbols verified at the pinned commit. New file suggestions are explicitly labeled later. Line anchors are starting points for implementers; they are not substitutes for call-site and contract audits.

| Layer | Existing file and precise symbol | Responsibility / required change |
|---|---|---|
| Project IDs | [`phalcom-modules/src/identity.rs:ProjectIdentity, ModuleId`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/identity.rs#L75-L85) and [`ModuleId`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/identity.rs#L266-L292) | Existing Universe/Resolved/Synthetic identities are distinct; `ResolvedProjectId` and synthetic IDs **are not durable global package IDs**. |
| Project graph | [`phalcom-modules/src/project.rs:ResolvedProject, ProjectUniverse`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/project.rs#L18-L35) and [`load_root_with_provider`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/project.rs#L122-L158) | Resolve project/package dependency graphs, import aliases, source revision fingerprints; add artifact-aware resolution and reproducible lock validation. |
| Manifest | [`phalcom-modules/src/manifest.rs:ProjectManifest, ValidatedProjectManifest, DependencySpec`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/manifest.rs#L13-L74) | Existing `project.toml` and path/package requirements; define lock/build mode and artifact identity without changing valid manifest semantics. |
| Package identity | [`phalcom-modules/src/package_info.rs:PackageArtifactIdentity, PackageInfoDescriptor`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/package_info.rs#L75-L165) | Existing serializable descriptive package IDs/provenance; **supplement** name/version with content and compatibility identities for caches/loaders. |
| Linked package boundary | [`phalcom-modules/src/artifact.rs:ResolvedPackageArtifact, PackageArtifactProvider`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/artifact.rs#L16-L64) | Extend provider abstraction for immutable external module catalogs and artifact-backed package resolution; not simply `serde` the present type. |
| Canonical sources | `phalcom-modules/src/builtin.rs:UniverseSourceProvider` and `UNIVERSE_NODES` | Own process-cached source/provider identity and canonical module image. |
| Canonical interfaces | `phalcom-modules/src/builtin_interface.rs:BuiltinInterfaceBuilder` / `UniverseSourceDeclarationCatalog` | Share interface/catalog computation at provider/image lifetime. |
| Resolver | [`phalcom-modules/src/resolver.rs:ModuleResolver::load_parsed/load_interface`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/resolver.rs#L665-L741) | Fix canonical cache bypass; resolve external image interfaces without local reparsing. |
| Module session | [`phalcom-modules/src/session.rs:WorkspaceSourceState`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L24-L36), [`new_discovered_sources` commit](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L1630-L1651) | Keep canonical dependencies out of mutable source state. |
| Transitive discovery | [`phalcom-modules/src/session.rs` discovery branch](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L2244-L2274) | Current canonical imports can load parsed source, build interface, add source state, and enqueue dependency. Make external targets terminal for local discovery. |
| Components | [`compute_connected_components`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L701-L742) and [`link_component_interfaces_tolerant` call](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L2313-L2400) | Local component population and relink should contain **local modules only**. |
| Linker | [`phalcom-modules/src/linker.rs:ModuleLinker, LinkedReadSpec, LinkedProgram`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/linker.rs#L25-L76); [`reachable_interfaces`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/linker.rs#L313-L372) | Change closed-interface assumption to local + immutable external module catalog. |
| Canonical base | `phalcom-semantic/src/canonical_baseline.rs:CanonicalUniverseSemanticBaseline` | Retain as bootstrap foundation; do not treat it as completed semantic authority. |
| Semantic input | `phalcom-semantic/src/workspace.rs:SemanticWorkspaceInput::new` | Stop rebuilding topology interfaces and discarding them. |
| Semantic session | `phalcom-semantic/src/session.rs:SemanticWorkspaceSession::from_baseline/update` | Add explicit `from_foundation` and `from_image` entrypoints; preserve local `last_snapshot = None`. |
| Queries | `phalcom-semantic/src/db/mod.rs:SemanticDb::product`, `record_dependency`, `is_reusable`, `validate_reuse`; `phalcom-semantic/src/db/query.rs` | Add canonical read-only query origin/currentness; keep mutable `query_state` workspace-local. |
| Type arena | `phalcom-semantic/src/types/store.rs:TypeStore::fork_from_baseline` | Preserve final canonical numeric prefix, give workspace fresh TypeStoreId, validate lineage. |
| Static arena | `phalcom-semantic/src/static_value/store.rs:StaticValueStore`, `StaticValueRef` | Preserve qualified static-store lineage across forks; define relocatable disk form separately. |
| Coherence | `phalcom-semantic/src/impls.rs:ConformanceIndex::query_exact/overlap_conflicts`, `conformance_is_authorized`; `phalcom-semantic/src/generic_evidence.rs` | Canonical/local candidates share one coherence domain; cache only closed proofs globally. |
| Canonical compiled program | [`phalcom-core/src/modules/canonical_universe.rs:CanonicalUniverseProgram/build_canonical_universe_program`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/canonical_universe.rs#L23-L42) and [`analyze_workspace`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/canonical_universe.rs#L203-L225) | Existing core analysis constructs full canonical semantic snapshot then does not retain it; later should consume `phalcom-semantic` image. |
| Compiler entry | [`phalcom-core/src/modules/compile.rs:EntrySelection`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L182-L195), [`ProgramAnalyzer`, `AnalyzedProgram`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L369-L389) | One-shot driver should use the production module-session model for Inline, Module, Package, Project. |
| Compiler projection | [`ProgramCompiler::compile_analyzed/project_analyzed`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L839-L865); [`compile_entry_selection`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L2000-L2008) | Compiler roots must drive only local lowering and required metadata; preserve errors. |
| Persistent compiler | `phalcom-core/src/modules/compile_session.rs:ProgramCompilationSession::standalone/compile_fixture` | Existing retained-session path is target ingestion architecture; expand beyond fixture-specific entry semantics. |
| Compiled program | [`phalcom-core/src/modules/compile.rs:CompiledModule/CompiledProgram`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L197-L234) | Current object holds source text and lowering: **not yet a source-free bytecode artifact**. |
| Declaration blueprints | [`phalcom-core/src/modules/artifact.rs:ModuleMaterializationPlan`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/artifact.rs#L5-L70) | Existing VM-independent module declaration representation; extend artifact DTO without serializing runtime handles. |
| VM materialization | [`VM::materialize_program`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/materialize.rs#L279-L313), [`compile_program_module_closure`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/materialize.rs#L695-L710) | Add executable-prototype loading; do not parse/source-compile on artifact load. |
| VM execution | [`VM::run_compiled`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/interpret.rs#L121-L145), [`compile_closure_as_with_bindings`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/interpret.rs#L103-L108) | Existing run path reparses/recompiles module source at VM execution; a genuine portable artifact must bypass it. |
| VM dependency gate | [`VM::initialize_single_module`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/initialize.rs#L20-L75) | Already accepts `ModuleState::Initialized` external deps through module registry; local topological order excludes them. |
| Runtime bytecode | [`Bytecode`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/bytecode.rs#L159-L166), [`Callable`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/callable.rs#L19-L38), [`Chunk`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/chunk.rs#L303-L358) | VM/runtime representations contain symbol IDs, runtime `Value`, `Cell` caches and semantic specs; **not safe raw on-disk DTOs**. |
| Metadata schema | [`SemanticMetadataBundle`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-type-meta/src/bundle.rs#L71-L93), [`validate_metadata_bundle`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-type-meta/src/validate.rs#L651-L660) | Existing serializable and validated type/reflection metadata schema; embed as a section with explicit dependency/pool identity. |
| Metadata load | [`VM::install_semantic_metadata_bundle`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/materialize.rs#L138-L148) and [bundle pool remapping](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/materialize.rs#L534-L562) | Move to symbolic relocation for artifacts; do not persist VM pool indices. |
| Stable semantic refs | [`StableIdentityContext` / `to_stable_*_with_context`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-semantic/src/metadata/stable_identity.rs#L11-L80) | Reuse structural identity concepts, but fix source-path/session-based instability for durable published artifacts. |
| CLI | [`phalcom-core/bin/phalcom/cli.rs:Commands`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/bin/phalcom/cli.rs#L125-L141) and [`cmd_run` path selection](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/bin/phalcom/cli.rs#L195-L220) | Existing CLI has no `build` artifact command. Add proposed build/inspect/execute commands without breaking source-run semantics. |

**File naming correction:** The original C2 draft referenced nonexistent or obsolete paths such as `phalcom-core/src/universe/canonical.rs`, `phalcom-semantic/src/db.rs`, `phalcom-semantic/src/conformance.rs`, and `phalcom-semantic/src/types/signatures.rs`. Use the verified paths above and search current source for the exact owner before editing.

---

# 2. Measured Baseline, Rejected Theories, and Target Complexity

## 2.1 Historical evidence (not a current performance guarantee)

The handoff records 14 core fixtures totaling approximately **765.24 s**, of which `ProgramCompiler::compile_entry_selection` consumed **675.82 s**, `VM::new()` **85.92 s**, and VM execution **3.31 s**. Tiny sources of 345 bytes to ~7.5 KB clustered near a **45.56 s median compile time**; a ~92.6 KB fixture compiled in **82.796 s**. These measurements support a large fixed cost plus a source-dependent component. They do **not** prove the exact time attributable to any particular source-confirmed mechanism. Record a new clean predecessor and both process-warm and process-cold results.

Falsified explanations: full TypeStore deep-copy on each fixture (`im` persistent storage); reparsing all Universe text (cached parsed ASTs exist); emitting bytecode for all 40 seeded semantic modules in every fixture (`runtime_modules` controls runtime lowering); and an unqualified claim that every semantic update relinks all 148 canonical modules. The real expensive mechanisms involve reprocessing cached canonical source and crossing ownership boundaries repeatedly.

## 2.2 Cost model

Let `U` be canonical Universe size, `L` local source size, `D` directly referenced external declarations, `R` reachable/runtime/reflection metadata roots, and `Δ` changed local semantic dependencies. The current path can contain broad `O(U)` and quadratic products regardless of `L`. The architectural target is:

\[
T_{\mathrm{warm\ one\!\mbox{-}\!shot}} = O(\mathrm{session\ setup}) + O(L + D + R + \mathrm{local\ proof\ work}),
\]

with **no mandatory traversal proportional to all `U`**. For retained sessions, recomputation should scale with `Δ` and its semantic reverse closure. For persisted artifact loading, expected work is proportional to compressed/decoded artifact bytes actually needed, relocations, runtime dependency checks, and initialization—not source AST traversal or semantic rechecking. The goal is architectural asymptotics and measured evidence, not a fabricated sub-200-ms acceptance claim.

## 2.3 Benchmark populations

MUST measure separately: (1) new process + first compile; (2) warm process + **fresh** one-shot session; (3) first retained-session compilation; (4) retained no-op; (5) body-only change; (6) signature change; (7) conformance change; (8) import/re-export change; (9) many canonical imports; (10) 345–500 B, ~7.5 KB, and ~92.6 KB sources; (11) saved package load/execute; (12) saved dependency package compile; (13) cold process with prebuilt Universe artifact. Report build profile, source hash, revision, samples, median, p95, peak RSS, disk I/O, loaded bytes and structural counters. `opt-level=0,1,2,3` A/B tests distinguish runtime multiplier from algorithmic work; compiler profile is not a substitute for the structural fix.

# 3. Fundamental Identity, Ownership, and Dependency Rules

## 3.1 Four distinct identity axes

The system MUST distinguish: **(a)** durable package/dependency identity; **(b)** exact artifact content identity; **(c)** in-process canonical/semantic world identity; and **(d)** VM instance/runtime module identity. Existing `ProjectIdentity::{Universe,Resolved,Synthetic}` is excellent for compiler-session ownership separation but `ResolvedProjectId` is an index into `ProjectUniverse`, not a persisted artifact key. Existing `PackageArtifactIdentity::{Builtin,Resolved,Standalone,Synthetic}` is serializable descriptive provenance, not proof that two `name@version` artifacts have identical content. Existing `StableProjectRef::SourceArtifact` includes a local source URI and revision fingerprint, while `StableProjectRef::Session` uses synthetic session identity; **these are unsuitable as the sole globally portable published identity** without explicit conversion or validation.

**Proposed identities** (not existing Rust APIs):

```rust
struct ArtifactContentId([u8; 32]);       // hash of canonical, encoded artifact bytes/sections
struct PackageCoordinate { name: String, version: Version, namespace: String }
struct PackageInstanceId { coordinate: PackageCoordinate, content: ArtifactContentId }
struct CompilerCompatibilityId { semantic_schema: u32, code_abi: u32, native_abi: Digest,
                                  target: TargetTriple, features: Digest }
struct SemanticWorldId(Digest);           // composition of root Universe + selected dependency images
struct WorkspaceRevision(u64);            // one mutable workspace session only
struct RuntimeArtifactInstanceId(u64);    // one VM loader instance only
```

The exact hash/encoding choice is a **VERIFY-FIRST** format decision (a cryptographic digest such as BLAKE3 or SHA-256 is preferred for durable content addressing). Structural and security validation MUST be independent of whether a digest matches. The same name/version MUST NOT silently accept different content in one resolved dependency graph; conflict is diagnosed or explicitly resolved by a documented multi-version namespace rule. A toolchain-native `universe` artifact gets a distinguished package coordinate, but uses the same fundamental image/ABI concepts as user packages.

## 3.2 Distinguish mutable local revisions from immutable parents

**Forbidden:** `session.last_snapshot = Some(canonical.snapshot.clone())`. The canonical snapshot has canonical-world source/module ownership, semantic generation and query history. A user's first workspace revision MUST start with `last_snapshot: None`, immutable parent image pointer(s), empty local source lifecycle, and its own `WorkspaceId`. Normal edits must never mutate, retire, or invalidate the canonical product. A change of canonical image identity constructs a **new semantic world**, not an in-place workspace revision.

## 3.3 One world, potentially many package images

**Critical multi-package extension:** the single-Universe `TypeStore::fork_from_baseline()` design does **not automatically permit combining N independently built immutable package TypeStores**: separately forked stores may allocate the same numeric `TypeId` for different local forms. Similarly `StaticValueRef` has store-qualified lineage and cannot be treated as a universal numeric ID. H1 SHOULD use the canonical final store prefix for one local overlay. H2/H3 MUST define a world assembly step that imports/remaps independently compiled package semantic nodes into a compilation-local arena or a qualified, multi-pool semantic reference representation. A simple concatenation or pointer-share of unrelated numeric TypeIds is prohibited.

Proposed sound strategies (choose and prove one before multi-package artifact integration):

- **Relocation/import strategy (preferred initial design):** each artifact stores stable structural/symbolic type IDs and a local compact type table. `SemanticWorldAssembler` maps each dependency's type node IDs, type parameters, kinds, static values and references into one checked world-local `TypeStore`/static-value environment, retaining per-package relocation tables. Canonical root prefix is installed first; packages are imported in stable dependency order. Validate that all body-derived and signature references are relocated consistently.
- **Qualified read-only pools:** preserve package-local type-node pools and represent dependency refs as `{package_image, local_node}`; local checker operations must support unified normalization/equality and generic substitution across pools. This avoids copying but requires a larger `TypeStore` abstraction change.

Neither alternative may infer equality of types from coincidentally identical numeric node IDs. A future serialized artifact's stable type references MUST be relocatable; raw process `TypeStoreId`, `StaticValueStoreId`, `ResolvedProjectId`, `SyntheticProjectId`, `Symbol`, `ObjRef`, or `ClassId` cannot be persisted as globally meaningful identities.

## 3.4 Canonical and local conformance are one coherence world

Canonical explicit heads, local explicit heads, canonical derived rules, and local derived rules participate in **one** proof candidate set. Storage layering MUST NOT imply fallback precedence. Validate canonical↔canonical conflicts at image build; local↔local and local↔canonical conflicts as local graph changes. For N independently built package images, world assembly must additionally validate any cross-package interaction not previously validated by their shared dependency world. A package-level “coherence passed” certificate by itself does not certify arbitrary unrelated package combinations.

---

# 4. Canonical Module Image — Construction and Read Protocol

## 4.1 Ownership and proposed shape

**Crate:** `phalcom-modules`. **Proposed new file:** `phalcom-modules/src/canonical_image.rs`. Reuse existing `UniverseSourceProvider`, `BuiltinInterfaceBuilder`, `ModuleResolver`, `ModuleLinker`, `ModuleTopology`, `ImportResolutionProduct`, `UniverseSourceDeclarationCatalog`, `ResolvedPackageArtifact` where semantically appropriate:

```rust
// PROPOSED, illustrate ownership rather than spelling.
pub struct CanonicalUniverseModuleImage {
    pub identity: CanonicalModuleWorldId,
    parsed: Arc<CanonicalParsedSourceCatalog>,
    interfaces: Arc<CanonicalInterfaceCatalog>,
    resolved_imports: Arc<CanonicalImportProducts>,
    linked: Arc<LinkedProgram>,
    topology: Arc<ModuleTopology>,
    declarations: Arc<UniverseSourceDeclarationCatalog>,
    public_fingerprints: Arc<ModuleFingerprintCatalog>,
    private_fingerprints: Arc<ModuleFingerprintCatalog>,
}
```

Build once per **canonical provider identity**; a custom/test `UniverseSourceProvider` source override MUST create isolated private state. `OnceLock`/`Arc` is an appropriate in-process ownership mechanism; never use leaked allocations (`Box::leak`) for test overrides. Errors must be cached/handled according to an explicit retry/fallback policy, not masked as a valid image. The module image MUST produce the same linked interface/exports/import targets/topology as the current full canonical source path.

## 4.2 Resolver cache correction (small independent patch)

**Existing source:** `ModuleResolver::load_interface()` at `phalcom-modules/src/resolver.rs:714–741` first loads parsed source and then uses `BuiltinInterfaceBuilder::build_from_parsed(&self.universe_source, &parsed)` on Universe. This bypasses the canonical provider's process-shared `load_interface` path and may reconstruct the all-node declaration catalog. In canonical-provider mode use the provider-owned cached interface. Keep provider-private handling for overridden Universe. Count `canonical_catalog_builds` and `canonical_interface_builds`: **zero per warm fixture** after initial image construction.

## 4.3 Module-product provenance and compatibility

Precomputed `interfaces`, import resolution, topology, and linked graph MUST belong to the same module-world revision and source/provider identity. Passing independently obtained maps without a shared revision token is not an acceptable long-term API. `SemanticWorkspaceInput::new` currently builds interfaces for topology and later exposes an empty interface map: remove this redundant producer/consumer mismatch, but do not make a lossy compatibility constructor the normative canonical image entry.

## 4.4 External module catalog

**Proposed read interface** to be supported eventually by both Universe and saved package artifacts:

```rust
trait ExternalModuleCatalog {
    fn module(&self, key: &StableModuleKey) -> Option<ExternalModuleView<'_>>;
    fn linked_interface(&self, key: &StableModuleKey) -> Option<&LinkedModuleInterface>;
    fn resolve_export(&self, key: &StableModuleKey, exported_name: &str)
        -> Result<LinkedExportRef, ExternalLookupError>;
    fn public_fingerprint(&self, key: &StableModuleKey) -> Option<InterfaceFingerprint>;
    fn package_identity(&self) -> &PackageInstanceId;
}
```

For the first cutover, a `ModuleId`-based adapter may avoid premature durable-identity changes. The portable artifact implementation MUST convert between stable package-qualified IDs and session `ModuleId`s explicitly, without aliasing or guessing.

## 4.5 Workspace module discovery is local-only

**Current failure:** [`phalcom-modules/src/session.rs:2244–2274`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-modules/src/session.rs#L2244-L2274) loads parsed imported targets, rebuilds interfaces, inserts `WorkspaceSourceState` and queues further discovery even for canonical targets. The repaired state must keep `sources_by_module`, source fingerprints, editable overlays, source revisions and local component membership **project-owned only**. A local import targeting an external image records an external edge and stops recursive source discovery. Canonical source remains addressable through immutable source/presentation products for IDE and diagnostics; virtual source locations do not confer mutable workspace ownership.

**Gate M1:** warm local import of Universe creates **zero** `WorkspaceSourceState` with `ProjectIdentity::Universe`, zero canonical import discovery queue entries, zero canonical reparses/interface rebuilds, and zero canonical source lifecycle mutations. Test first compile, body edit, import edit, removal and recovery.

---

# 5. Local + External Linking, Topology, and Runtime Dependencies

## 5.1 Replace the closed-interface assumption

**Existing:** `ModuleLinker::reachable_interfaces` in `phalcom-modules/src/linker.rs:313–372` accepts targets only if they appear in `self.interfaces`; `WorkspaceModuleSession::compute_connected_components` in `session.rs:701–742` puts all present interface keys into the component adjacency, with imported targets joining components. Later `session.rs:2364–2400` clones component interfaces and relinks them. Blindly removing Universe from `interfaces` breaks imports; retaining them as local interfaces causes redundant component work.

**Proposed:**

```rust
struct LinkEnvironment<'a> {
    local: &'a LocalInterfaceCatalog,
    external: &'a dyn ExternalModuleCatalog,
    project_roots: &'a ImportRootResolution,
}
```

All local import, re-export, selective import, import alias and exposed child resolution MUST operate through one semantic interface, obtaining exact canonical `SymbolId`/`ModuleId`/static target or durable equivalent. External target lookup is **terminal for local module graph discovery** but not opaque for symbol/export resolution. Preserve `ModuleDiagnostic`, import site, source span, blocked-module behavior, and tolerant-linker recovery.

## 5.2 Distinguish dependency edges from component membership

```text
local SCC/connected-component graph:  LocalA ─────► [External E] (leaf)
external canonical graph:            already linked inside image E
```

The local linker must not traverse E's internal imports, add E to local connected components, or create a local initialization unit for E. Local import-resolution products are still workspace-owned because **the import syntax and source diagnostic** belong to the local importer; the resulting external target depends on immutable external catalog identity/fingerprint. Local reverse-import indexes MAY retain keyed references to external targets for navigation/explanation but MUST NOT treat immutable external changes as ordinary workspace invalidation seeds.

**Gate M2:** `canonical_modules_in_local_components = canonical_modules_relinked = 0` in warmed local linking. Changing a local import recomputes only local module products and affected local components; validate linked symbol targets and diagnostics against the reference from-source linker.

## 5.3 Composed topology

Do not flatten every canonical topology node into each local `ModuleTopology`. Provide `ModuleTopologyView`/`ResolverTopologyView` using local and external roots to answer `contains_module`, children, exposed children, import roots and source location. A temporary flattened compatibility topology is allowed only as a measured migration step. Persisted package images must expose public topology/visibility without exposing private linked implementation modules as importable names.

## 5.4 Existing VM support for external module dependencies

`LinkedReadSpec::{Module(ModuleId),Binding(SymbolId)}` already encodes cross-module reads; `VM::initialize_single_module` looks up each `runtime_dependency` in `module_registry` and accepts `ModuleState::Initialized` (`phalcom-core/src/modules/initialize.rs:45–75`). This is the target runtime contract: local `LinkedProgram.modules` and `initialization_order` contain local units only, while local `LinkedModule.runtime_dependencies` can refer to installed dependency artifacts. A missing, failed, or wrong-version dependency MUST produce a structured failure; no silent source recompilation or duplicate Universe reinitialization.

## 5.5 Stable dependency identities at artifact load

In-memory `ModuleId::Resolved(ResolvedProjectId)` is session-local. Saved linked-read/module dependency records MUST use package-qualified stable keys. The artifact resolver maps them to VM/session `ModuleId` at load time using an explicit relocation map and collision checks. `StableModuleRef` in `phalcom-type-meta` and `PackageInfoDescriptor` offer precedents, but cannot be treated as self-verifying published object identities without content/ABI/dependency binding. Nested module paths, re-exports and aliases preserve canonical source-owned origin after relocation.

---

# 6. Complete Canonical Formal Image and Safe Bootstrap

## 6.1 Separate bootstrap foundation from finished image

`CanonicalUniverseSemanticBaseline` is a **partial** semantic bootstrap construct. It already contains declarations, hierarchy, dispatch, signatures, data/enum/associated products, alias source and diagnostics; it does not include all finalized trait/conformance/witness/body/query products. The existing [`build_canonical_universe_program`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/canonical_universe.rs#L154-L225) proves ordinary `analyze_workspace` can analyze the *complete* Universe. Retain the old baseline as initial `CanonicalSemanticFoundation` rather than rewriting fundamental semantics as part of the performance cutover.

**Explicit cycle-free sequence:**

```text
CanonicalUniverseModuleImage
         │
         ▼
CanonicalSemanticFoundation (existing partial baseline)
         │
         ▼
SemanticWorkspaceSession::from_foundation()  [never calls image-backed new()]
         │
         ▼
ordinary full canonical semantic update and successful commit
         │
         ▼
freeze final SemanticSnapshot + eligible canonical query image + arena roots
         │
         ▼
CanonicalUniverseFormalImage
         │
         ├── core canonical program compiler consumes this snapshot
         └── ordinary SemanticWorkspaceSession::from_image() consumes as parent
```

Never implement `from_foundation -> new -> canonical_image -> from_foundation` recursion. Use explicit constructors and image-build state guards; test concurrent first callers and error paths. Native/core descriptor contract verification remains core-owned.

## 6.2 Proposed image shape

```rust
struct CanonicalUniverseFormalImage {
    world: CanonicalSemanticWorldId,
    modules: Arc<CanonicalUniverseModuleImage>,
    reference_snapshot: Arc<SemanticSnapshot>, // authoritative completed canonical publication
    queries: Arc<CanonicalSemanticQueryImage>,
    type_prefix: CanonicalTypePrefixId,
    // StaticValueStore lineage resides in snapshot or an explicit immutable root.
}
```

Avoid 20+ duplicated `base_*` maps merely mirroring the completed snapshot. Verify the final snapshot includes complete trait headers/surfaces, conformance contributions/derived rules/witnesses/dispatch, callable signatures/analyses, hierarchy, static definitions/constants/domains, field/constructor/data/enum/associated products, native diagnostics and required presentation roots. **Do not freeze before transaction commit.** At freeze, `Computing`, `Cancelled`, `BudgetExceeded` products must never be advertised ready; classify `Blocked` and `Failed` cases against the tolerated canonical diagnostic contract.

## 6.3 Arena lineage

The user workspace MUST fork the **final** canonical `TypeStore` prefix produced by complete analysis; existing `TypeStore::fork_from_baseline()` preserves numeric IDs but assigns a fresh `TypeStoreId`. Introduce explicit `CanonicalTypePrefixId` validation for all shared products using raw `TypeId`, `TypeParameterId`, `KindId`, etc. `StaticValueRef` embeds `StaticValueStoreId`, and the relevant `StaticValueStore` clone preserves lineage; image-backed workspaces must not replace it with unrelated `StaticValueStore::new()`.

**Gate S1:** canonical image is immutable; canonical compiler uses it without repeating full semantic analysis; canonical/source diagnostics and compiled runtime behavior match the old complete-source bootstrap; foreign prefix/static lineage rejected; custom overridden `UniverseSourceProvider` uses a separate private semantic world.

---

# 7. Immutable Query Roots and Local Semantic Overlay

## 7.1 Do not clone a mutable SemanticDb

The current `SemanticDb` holds workspace revision, scheduler, query transaction state, mutable dependency indexes and metrics. The canonical image should freeze only **eligible, committed** semantic products with their fingerprints and dependency observations:

```rust
// PROPOSED
struct CanonicalQueryEntry {
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    product: Arc<SemanticProduct>,
    canonical_dependencies: Arc<[ImmutableQueryDependency]>,
}
struct CanonicalSemanticQueryImage {
    world: CanonicalSemanticWorldId,
    entries: Arc<CanonicalQueryMap>,
}
```

A successful `Ready` row alone is **not** proof of exportability: the product/dependency closure must contain no foreign workspace-local identity, mutable session reference, or nonrelocatable arena ref. Freeze and report eligible and ineligible categories with explicit reason codes.

## 7.2 Query origin and currentness API

Proposed API changes in `phalcom-semantic/src/db/mod.rs`, `db/query.rs`:

```rust
enum SemanticProductOrigin { Canonical(CanonicalSemanticWorldId), Workspace(WorkspaceId) }
fn product_origin(&self, key: &QueryKey) -> Result<SemanticProductOrigin, QueryError>;
fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>>;
fn ready_product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint>;
fn is_current(&self, key: &QueryKey) -> bool;
fn local_query_state(&self, key: &QueryKey) -> Option<&QueryState>;
```

**Keep `query_state` local-only.** Existing paths use `query_state(&key).is_some()` to decide `discard_for_recompute`; exposing immutable canonical facts as mutable `QueryState::Ready` would make them incorrectly discardable. Canonical `is_current` is established by image/world validity and immutable fingerprint, not `validated_revision == workspace_revision`. Local query currentness still uses revision validation. Replace all project-identity shortcuts asserting Universe dependencies are automatically ready with actual canonical-image lookup/currentness.

## 7.3 Dependency graph, negative facts, and invalidation

A local query may depend on an immutable canonical product; record the forward key and fingerprint without treating canonical products as mutable local reverse-invalidation seeds. Changing local source cannot mutate canonical facts. **Negative/candidate-absence dependencies need special treatment:** a proof that fails because no local conformance exists must become invalid if a local conformance is later added. Track conformance candidate index generations, import-set changes, evidence environments and negative/candidate-absence observations, not only the positive candidate IDs already found. A canonical generic *rule* is immutable; applying it to a local type is a **local proof**, not a globally shareable result.

## 7.4 Local lifecycle and snapshot boundary

```rust
// PROPOSED conceptual initialization
SemanticWorkspaceSession {
    canonical: image,
    db: SemanticDb::with_canonical_image(...),
    store: image.reference_snapshot.store().fork_from_baseline(),
    static_values: image.reference_snapshot.static_values.clone(),
    last_snapshot: None,        // previous publication of THIS workspace only
    local_sources: Default::default(),
    local_structure_shards: Default::default(),
    ...
}
```

Do **not** seed mutable local source fingerprints, alias/signature/generic dependency graphs, local module diagnostics, revision deltas or last-known-good from canonical modules. Read canonical facts through composite views. Preserve transactional rollback, cancellation, blocked/error states and local LKG after failures.

## 7.5 Transition from compatibility aggregate maps to persistent views

For the first parity cutover, clone selected canonical `DeclarationTypeTable`, `CallableSignatureTable`, hierarchy/dispatch/conformance/static aggregates if the checker requires concrete mutable APIs. **This is an explicitly measured temporary bridge**, not the final performance architecture. Subsequently adopt a persistent/module-sharded representation (existing `TypeStore`, `SourceSemanticIndex`, `FormalSemanticProjection` are precedents) with `Arc` shard sharing and local replacement. Target **zero canonical table entries copied per fresh workspace** after persistent-root migration. Distinguish `get`, `iter_local`, `iter_dependencies`, and explicitly expensive `iter_all` operations; avoid automatic flattening at snapshot/metadata boundaries.

## 7.6 Conformance, coherence and proof semantics

The logical candidate set MUST be:

```text
canonical explicit heads ∪ local explicit heads ∪ canonical derived rules ∪ local derived rules
```

No canonical/local precedence may be introduced. Reuse canonical↔canonical coherence checked once at image construction; check local↔local and local↔canonical interactions. Conformance bucket indexes must conservatively include every potentially overlapping pair; a bucket of size `k` still has `O(k²)` pairwise comparisons. Witness owner/selector indexes SHOULD return a candidate list initially until semantic uniqueness is proved. Structural witness-plan reuse across inferred-return changes is **VERIFY-FIRST**, not a premise. Generic proof caches must include goal, evidence environment, world identity and all relevant dependency/candidate-set changes; never publish workspace-dependent proof outcomes into a canonical global cache.

**Gate S2:** a fresh local session consumes immutable canonical query products with zero canonical query republication, does not copy canonical aggregates after final persistence cutover, and passes exact conformance/dispatch/generic proof parity under local edits, failure/recovery and concurrent sessions.

# 8. Consumer-Specific Publication and Compiler Projection

## 8.1 Formal core versus editor/advisory publication

Ordinary executable compilation MUST demand only formal semantic truth, required diagnostics, compiler lowering attachments and metadata roots. It MUST NOT build a whole canonical editor index or whole-workspace advisory fixed point as an implicit prerequisite. Existing cold publication had roughly 148 canonical presentation modules plus local source indexed; `resolve_type_reference_targets`, `resolve_type_member_projection_targets`, and `resolve_type_parameter_source_targets` repeatedly traversed the same parsed AST, with per-conformance reinvocations. This is a legitimate editor-algorithm problem, but the first performance fix is **not to request the editor product for an executable-only consumer**.

**Proposed:** validated product capabilities (not arbitrary booleans):

```rust
struct CompilationProductRequest {
    formal: FormalRequirement,               // always complete for compiling
    diagnostics: DiagnosticsRequirement,
    lowering: LoweringRequirement,
    metadata: MetadataProfile,               // runtime semantics remain normative
    source_presentation: ProductDemand,      // NotRequested / Requested
    formal_projection: ProductDemand,
    advisory: ProductDemand,
}
enum ProductState<T> { NotRequested, Available(Arc<T>) }
```

Do not fabricate `AdvisoryWorkspace::empty()` to mean not requested. Editor mode composes frozen canonical source/formal/advisory shards with changed local shards; executable mode omits unrequested products. A *checker-owned* `LoweringSourceAttachments` or equivalent keyed by stable AST/HIR/expression/binding identity MUST replace backend dependence on the entire editor `SourceSemanticIndex` for binding resolution. Remove duplicate snapshot-time `FormalSemanticProjection` construction when the session will replace it with the actual source-aware projection.

## 8.2 Residual semantic waves (after fixed work is removed)

Current semantic body analyses may pass through trait default bodies, class/inherent bodies, conformance witnesses, reverse dependent closures, inferred returns and final module-value revalidation. When module-value inputs transition `None → Some`, an input fingerprint can invalidate otherwise unrelated body checks. **Do not simply delete the fingerprint**. Profile fixture-local body rechecks after the canonical cutover. Then add dependency-sensitive module-value facts and SCC-style inferred-return processing only where their convergence/diagnostic semantics are proven. Witness selection that can depend on inferred return types MUST be correctly invalidated.

## 8.3 Metadata exporter requirements

`ProgramCompiler::project_analyzed_inner` determines runtime roots but currently supplies broad semantic tables to `phalcom-semantic/src/metadata/export.rs:MetadataExporter::build_bundle`. Global declaration/callable traversal and an explicit declaration × callable scan are residual costs. Define a `MetadataExportPlan` with executable roots, required runtime type/trait/generic roots, reflection/public API roots, dynamic lookup roots and external pool dependencies. For `RuntimePublic`, preserve every publicly observable reflection capability; do not switch the default to `RuntimeMinimal` merely to reduce bundle size. Build a `DeclarationId → callable candidate IDs` owner index; traverse **reachable semantic closure**, not all declarations unconditionally. Maintain coverage for aliases, associated statics, type forms, dynamic resolution, traits, conformance evidence, serialization and registration.

## 8.4 Runtime metadata pools

Existing `VM::install_semantic_metadata_bundle` assigns `MetadataPoolId(pool_count)` and validates bundles with `phalcom_type_meta::validate::ValidationLimits`. `materialize_program` remaps bundle-local lowering handles after assigning a VM pool. Persisted artifacts MUST carry **symbolic artifact-qualified metadata references**, not the incidental VM pool number; load-time relocation assigns VM-local handles and preserves cross-package type identity. Model `MetadataHandle { pool: MetadataPoolId, record: MetadataRecordId }` **only as a VM-local resolved handle**; the on-disk form is `{artifact_content_id, stable_record_key}` or a relocation index bound to the dependency manifest. Generalize beyond a special canonical/local two-pool enum, since future projects load multiple packages.

---

# 9. TRUE Ahead-of-Time Code: Portable Executable Artifact Requirements

## 9.1 Critical source-grounded fact: current CompiledProgram is not bytecode

Existing [`CompiledModule`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L197-L217) retains `source_text`, `ModuleMaterializationPlan` and `ModuleLoweringSemantics`. `ProgramCompiler::compile_module` calls [`ModuleMaterializationPlan::empty`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/modules/compile.rs#L2032-L2052) rather than emitting stored initializer bytecode. At execution, [`VM::run_compiled`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/interpret.rs#L121-L145) materializes modules, then if closure is absent invokes `compile_program_module_closure(id, source_text, program)`, whose [`compile_closure_as_with_bindings`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/interpret.rs#L103-L108) parses source and compiles it inside a mutable VM. **Saving the current CompiledProgram would cache semantic projection, not source-free executable code.**

**MUST:** A release-mode saved compiled package contains complete, valid, VM-independent executable prototypes for every emitted initializer and nested callable required at runtime. Execution from the saved artifact MUST NOT call source parsing, formal semantic analysis, or `compile_closure_as_with_bindings`. Source-only fallback may be exposed as a separately named developer mode, never silently presented as precompiled execution.

## 9.2 Current runtime objects cannot be serialized directly

[`Chunk`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/chunk.rs#L303-L358) contains `Vec<Bytecode>`, `Vec<Value>`, `Vec<SourceRange>`, `Cell<Option<InlineCache>>`, `Cell<Option<GlobalCache>>` and an `ExecutableSemanticPool` including mutable caches and runtime references. [`Callable`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-core/src/callable.rs#L19-L38) contains runtime `Symbol`, parameter shape and closure upvalue descriptors. Some bytecode operands refer to interned symbol IDs; constants can contain VM `Value` including heap references; late-bound class/metadata handles and executable semantic recipes require relocation. In particular, inline caches are **VM-local mutable runtime state**.

**MUST NOT** serialize raw `Chunk`, `Callable`, `Value`, `ObjRef`, `ClassId`, `Symbol`, `Cell`, GC root or VM-local metadata pool IDs as direct Rust binary memory images. Process pointer representation, memory layout, heap object indices and symbol intern order are not portable or semantically stable.

## 9.3 Introduce portable executable prototypes

**Proposed new code files**, exact path subject to crate review: `phalcom-core/src/compiler/portable.rs`, `phalcom-core/src/modules/portable_artifact.rs`, `phalcom-core/src/modules/code_loader.rs`. Prefer putting the stable wire DTO in an artifact-schema crate shared by compiler and loader to avoid cyclic dependencies.

```rust
// PROPOSED disk/transport DTO; NOT existing runtime Chunk.
struct PortableCallablePrototype {
    stable_callable: StableCallableKey,
    code: Vec<PortableInstruction>,
    constants: Vec<PortableConstant>,
    max_slots: u32,
    arity: u32,
    parameter_shape: PortableParameterShape,
    upvalues: Vec<PortableUpvalueDescriptor>,
    local_names: Vec<StableSymbolName>,
    semantic_pool: PortableExecutableSemanticPool,
    relocations: Vec<ExecutableRelocation>,
    debug_line_table: Option<PortableSourceMap>,
}
struct PortableModuleExecutable {
    stable_module: StableModuleKey,
    linked_reads: Vec<PortableLinkedRead>,
    initializer: Option<PortableCallablePrototype>,
    nested_prototypes: Vec<PortableCallablePrototype>,
    declarations: Vec<PortableDeclarationBlueprint>,
    runtime_dependencies: Vec<StableModuleKey>,
}
```

The representation MUST preserve executable semantics including: method/selector bindings; closures and upvalue capture descriptors; default/named/pack parameters; constant values; trait/evidence invocation recipes; conditional/associated invocation; data/enum/anonymous product forms; static product recipes; contract/debug modes; and compiler-internal opcode semantics. The **existing exhaustive `Bytecode::index` and opcode names** provide a useful inventory, not a serialized opcode ABI. Introduce a versioned portable opcode encoding with explicit operand types, stable ordering, overflow and feature checks.

## 9.4 Decouple code emission from a live VM

The current compiler allocates a closure through `Compiler::new_with_bindings_and_mode(self, module, source_id, ...)` where `self` is a mutable `VM` (`phalcom-core/src/interpret.rs:70–83`). Two viable migration strategies must be evaluated:

1. **Preferred target — VM-independent emitter.** Extract parsing/formal-lowering → portable function/code prototype generation into a compiler context with an abstract constant/symbol builder. The VM loader then interns strings, allocates fresh runtime `Callable`/`Chunk`/closure instances, assigns runtime handles and creates caches. Both source execution and persisted artifact execution share the same emitted code semantics.
2. **Interim offline compilation adapter.** Use a private compilation-only VM/compiler context to obtain code prototypes and immediately **lower them into portable symbolic representations**, then prove no VM-local handles, caches, closures or heap values escape. This is allowed only as a measurable compatibility stage and requires complete relocation auditing.

Neither approach may require creating a fully bootstrapped source-runtime VM for every package compilation. Artifact load must not perform the compiler-emission stage. A precise **CodeEmission API** and a distinct **CodeMaterialization API** should be introduced. Avoid calling artifact emission “serialization of CompiledProgram” until it actually contains portable executable code.

## 9.5 Executable relocation model

At build time, persist references to stable semantic entities and relocation slots, not allocated VM identities. At load time, resolve relocations against the checked package graph and VM context:

| Portable reference | Load-time target |
|---|---|
| Stable selector/field name | VM `Symbol` interner ID |
| Stable module key | VM-local `ModuleId`, registry handle |
| Stable declaration/callable/field | Runtime canonical object/definition or metadata binding |
| Stable external package key | Loaded artifact instance and dependency namespace |
| Portable numeric/string/structural constant | Fresh VM `Value`/immutable constant object |
| Prototype constant | Fresh runtime Callable/closure prototype (appropriate capture semantics) |
| Qualified metadata record | VM-local `(MetadataPoolId, record index)` |
| Type/trait/generic recipe | Validated runtime type/evidence environment handle |
| Class/enum/data shape | VM registry descriptor and class/variant binding |
| Source span/debug source | Relocated source map or stripped debug sentinel |
| Branch target/instruction operand | Validated portable bytecode position/index |

**Proposed relocations:** `Symbol`, `Module`, `Export`, `Callable`, `Type`, `Metadata`, `StaticValue`, `ClassDescriptor`, `NestedPrototype`, `NativePrimitive`, `ExecutableSemanticOperand`. Exact variants should be derived from an exhaustive codegen and bytecode operand audit. All relocations MUST be bounds-checked and type-checked; unresolved mandatory relocation is a load error. Avoid eager relocation work for prototypes proven unreachable when lazy loading is implemented, but preserve identical behavior.

## 9.6 Mutable runtime state remains per-VM

The loader allocates fresh `Chunk.caches`, `Chunk.gcaches`, `ExecutableSemanticPool` mutable cache cells, closures/upvalues, runtime module objects, class bindings and GC roots. All are initially in valid empty/unresolved states and are owned by the loading VM. No user-visible mutable objects or runtime aliases are shared merely because the executable artifact bytes are process-shared. Immutable decoded code/metadata pages MAY be shared after a safety/ownership audit; cache and closure state MUST remain isolated. This is a **compiled-code artifact**, not a snapshot of initialized VM heap.

## 9.7 Source-free execution contract

For artifact-only execution, assertions MUST include:

```text
source_parser_calls                    == 0
semantic_checker_calls                 == 0
vm_compile_closure_as_with_bindings    == 0
portable_prototypes_loaded            > 0  (for nonempty executable)
portable_relocations_resolved          == required_relocations
bytecode_source_compilation_fallback   == 0
```

Test nested closures, upvalues, calls across package boundaries, static constants, generic specialization/trait evidence, runtime reflection, exceptions/diagnostics and contract modes. Re-run from an artifact after deleting/renaming build-source directories (with optional debug source handling specified) to prove source-independent execution.

---

# 10. Unified Saved Package/Project Artifact Format — Proposed `TPA` v1

## 10.1 Product taxonomy and naming

**Proposed naming only** (not current CLI/file formats):

- `.telpkg` — **atomic compiled package artifact**, suitable for publication, caching, importing and optional execution when an entry exists. Every project source package with public interfaces gets a distinct artifact identity.
- `.telproj` — **project/deployment bundle** containing a root entry descriptor, an artifact dependency lock/index and either embedded package artifacts or content-addressed references; it does **not** flatten all package semantic namespaces into one fictitious package.
- `.telcache/` (or user-configurable cache) — build cache and content-addressed local image store; not a published format guarantee.
- Precompiled `universe` — distinguished `.telpkg`-equivalent built-in artifact plus optional presentation/debug sections; runtime may embed its bytes or discover an authenticated toolchain installation.

The **real persisted format name and extension are a reviewed UX decision**. `TPA v1` here is a specification placeholder, not a claim that a parser/serializer exists. Preserve the current distinction that a *project* is the development container and a *package* is the atomic publication/distribution unit (`phalcom-modules/src/artifact.rs:1–5`).

## 10.2 Package contents

A package artifact MUST include enough information for **two independent consumers**: (a) downstream compiler/type checker without dependency sources; (b) VM executor without source compilation. Optional debug/IDE sections do not affect correctness of the required consumer.

```text
TPA Package Artifact
  Header
    magic / container schema / min reader
    producer compiler + semantic model + bytecode ABI
    target / feature / contract / native ABI compatibility
    artifact role / package coordinate / root namespace
    content digest / section directory / resource limits
  Package Manifest
    verified package descriptor / exposed module paths
    dependency list / exact artifact digests / aliases
    resolved entry/default entry / build configuration
  Module Image
    stable module table / public linked interfaces / exports
    import resolution and external dependency references
    local module dependency/init graph / topology summary
    public/private module fingerprint summaries
  Semantic Interface Image
    exported type/kind/parameter/type-member representations
    declarations, signatures, fields, constructors, aliases
    hierarchy / traits / associated surfaces / static domains
    conformance heads, witness rules and derived rules
    relevant finalized callable contracts and static constants
    normalized symbolic references / relocation index
    eligible canonical or package-scoped query product summaries
  Executable Image (when built with executable profile)
    module initializers and callable prototypes
    opcode streams / portable constant pools
    closure/upvalue/parameter shapes
    executable semantic recipes / relocation tables
    runtime dependency graph / declaration blueprints
  Runtime Metadata
    validated SemanticMetadataBundle (or versioned successor)
    reflection roots / external pool dependencies
  Optional Sections
    source provenance and source maps
    debug symbols / inlay/reference/presentation shards
    diagnostics for development builds
    signatures / supply-chain attestations
```

The package MUST declare **artifact capabilities**, e.g. `InterfaceOnly`, `Executable`, `Reflectable`, `DebugInfo`, rather than forcing every build to emit all sections. `InterfaceOnly` is valid as a compiler dependency but MUST fail clearly when asked to execute. An executable artifact MUST either include a valid entry/init code graph or explicitly describe itself as a library with no executable entry.

## 10.3 Container header and section directory

**Proposed illustrative header:**

```rust
struct ArtifactHeaderV1 {
    magic: [u8; 8],              // fixed, versioned format identifier
    container_version: u32,
    min_reader_version: u32,
    compiler_compatibility: CompilerCompatibilityId,
    package_key: PackageCoordinate,
    declared_features: FeatureBitmap,
    section_directory_offset: u64,
    section_directory_len: u64,
    artifact_digest: ArtifactContentId,
}
struct SectionDirectoryEntry {
    kind: SectionKind,
    schema_version: u32,
    required: bool,
    compression: CompressionKind,
    offset: u64,
    stored_size: u64,
    decoded_size: u64,
    digest: Digest,
}
```

Use an explicitly specified canonical encoding (fixed integer widths, chosen byte order, bounded lengths, sorted map order, discriminant versioning, no native struct padding). Specify the digest domain/coverage and self-hash exclusion precisely to avoid circularity. Reject overlapping/out-of-file section ranges, duplicate mandatory sections, unrecognized required section, inconsistent feature declarations, oversized decoded lengths and hash mismatches. Optional unknown sections may be skipped only when their feature dependency contract allows it. Never run arbitrary code during artifact parsing.

## 10.4 Determinism and reproducibility

Same normalized sources, manifests, lock graph, compiler/ABI/features/target and build mode MUST produce byte-for-byte identical required sections (and artifact content ID) independent of hash-map enumeration, file system walk order, workspace absolute path or `ResolvedProjectId` allocation sequence. Sort canonical module/semantic tables deterministically. Normalize paths and prohibit local absolute paths from entering required semantic identities. Strip timestamps/hostnames/random nonces from required content. Source/debug sections MAY preserve paths under an explicit reproducibility policy (`relative`, remapped, or deliberately nonreproducible debug build), without changing the required semantic/executable digest policy. Test building identical packages from different checkout paths and separate processes.

`ResolvedProject::revision_fingerprint()` in `phalcom-modules/src/project.rs:49–69` already hashes sorted source paths/content but is **not** a complete artifact compatibility identity: manifest options, compiler flags, dependency exact artifact IDs, schema and native ABI must also participate. Stable semantic project refs must not rely solely on a local absolute `logical_uri` or synthetic session ID (`phalcom-semantic/src/metadata/stable_identity.rs:21–44`). Define durable published stable-module refs against package content identity + namespace/module path.

## 10.5 Package ABI, implementation and artifact digests

Maintain distinct fingerprints:

- **Interface/semantic ABI fingerprint:** externally observable declarations/signatures, trait surfaces, generic constraints, exported conformance rules/witness layouts, associated/statics, visibility, reflection/public metadata commitments and runtime calling/evidence layouts as applicable.
- **Implementation fingerprint:** private bodies, private metadata/runtime code and local implementation details.
- **Full artifact content digest:** exact serialized bytes/sections (subject to canonical encoding), including executable and optional content as defined.

A dependency's **public ABI unchanged** may allow downstream semantic cache reuse, but execution/deployment must still bind to the exact compatible dependency artifact content and required native/runtime ABI. A private dependency change cannot be ignored if linked code inlines/specializes the private implementation or imports private compiler contracts; record those dependency modes explicitly.

## 10.6 Artifact dependency manifest and lock

Every dependency edge MUST include a normalized import alias, exact selected package identity/content ID, required ABI/feature constraints and expected exported root. A resolved package lock is authoritative for reproducible builds; unresolved version requirements from `PackageInfoDescriptor` are insufficient for deterministic artifact load. Existing `DependencySpec::{Path,Package}` and `PackageRequirementDescriptor` should be normalized through the current project resolver and a new lock/build-graph boundary; do not silently replace legitimate path dependencies with network retrieval. Detect missing, duplicate, cyclic (where current project rules disallow them), conflicting or incompatible dependency artifacts before publishing a runnable root.

## 10.7 Serialization of semantic dependencies: interface, not arbitrary snapshot

Do **not** attempt to `serde` the entire `SemanticSnapshot` or `SemanticDb`. These own in-process `Arc`s, persistent collections, arena identities, session-local references and query state. Define an explicit **SemanticInterfaceArtifact** with exported source-independent semantic DTOs, stable declaration/callable/trait/impl keys, structural type graphs and optional relocatable query products. The in-memory canonical reference snapshot MAY be retained for tools and canonical compilation, but its on-disk form is a schema-versioned projection. Query cache entries containing workspace-specific spans, `QueryState` revision, unsupported compiler pointers, mutable cache cells or nonrelocatable IDs MUST be omitted or encoded through a supported stable adapter. Downstream semantic checking **must** have enough exported facts to answer valid import/type/generic/conformance queries without dependency source; if a required operation cannot be answered from the artifact, reject the artifact profile at build or import time rather than silently reanalyzing unavailable source.

## 10.8 Generic callable bodies and specialization boundary — VERIFY-FIRST

Telos runtime uses generic/evidence specialization and conditional trait selection. Determine whether downstream compilation requires source bodies or specialized typed lowering from dependencies for any legal call. **Executable package artifacts MUST export either** (a) sufficient compiled generic templates + evidence recipes usable by downstream calls with new type parameters, or (b) a stable, typed, relocatable intermediate representation with a safe downstream specialization/instantiation protocol. The default dependency compiler must not depend on raw upstream source AST for generic specialization. `CallableAnalysis` and `ModuleLoweringSemantics` currently provide many checked ingredients, but are not automatically a portable public generic-template ABI. Define the exact template/evidence/associated-type contract before declaring interface-only artifacts sufficient for all downstream operations.

## 10.9 Separate development/presentation sections

IDE source navigation, formal projection, advisory facts and original source text are optional artifact sections. They MAY be shipped for developer builds or lazy symbol-download workflows. Executable loading must not depend on them. A debug-source-map entry must refer to the corresponding stable module/source identity and mapping version; removing sources must not alter runtime semantic behavior (only diagnostic richness).

# 11. Saved Project Artifacts, Package Builds, Dependency Graphs, and CLI

## 11.1 Project versus package semantics

`ProjectUniverse` and `ResolvedProject` in `phalcom-modules/src/project.rs` model a resolved development graph with project paths, import roots, namespace and dependency aliases. `phalcom-modules/src/artifact.rs` states that a **package is the atomic unit of publication/distribution** and a project is its development/workspace container. Preserve this distinction:

- Building **one package** produces one immutable `.telpkg` with its own stable namespace, public module interfaces, semantic image and optionally executable section.
- Building **one project** may produce multiple package artifacts, one or more executable roots, and a reproducible `.telproj` deployment/index bundle; it must not conflate the graph into one synthetic package. A project build result can be a directory/index referencing content-addressed package artifacts, plus an optional self-contained deployment archive.
- A package built only as a library has no required executable entry; attempting to execute it without selecting an explicit supported entry is a typed diagnostic, not fallback to some source file.
- Standalone/inline compilation retains synthetic in-process identity for sessions, but persisted artifacts need a newly generated deterministic **published standalone identity** derived from normalized source/config instead of serializing the process synthetic counter.

## 11.2 Project build and package build pipelines

```text
build project.toml / package.ph / source module
  1  Parse/validate manifest and compile options
  2  Resolve exact package versions, aliases and paths with dependency provider
  3  Construct/update lock graph; validate package identity and cycles
  4  Load selected dependency artifact headers and published module images
  5  Assemble semantic world (canonical prefix + relocated package dependencies)
  6  Parse/update only root source-owned modules; resolve imports against external catalog
  7  Formal local semantics, conformance/coherence and local queries
  8  Determine public/semantic ABI, runtime/reflection roots and codegen roots
  9  Compile local module initializers/callables to portable executable prototypes
 10  Export semantic interface, metadata, typed generic templates if needed
 11  Validate complete artifact in memory, compute deterministic fingerprints
 12  Atomically publish to content-addressed cache and requested output path
 13  Record build manifest/reproducibility stats; optionally create project bundle
```

Steps 4–6 MUST NOT materialize dependency source as local `WorkspaceSourceState`. Package modules may depend on Universe and on independently published package images. The current project dependency cycle policy (`ProjectUniverse::resolve_project_recursive` reports `ProjectDependencyCycle`) must be preserved unless a separate language/package-resolution change deliberately broadens it.

## 11.3 Developer command interface — PROPOSED

The current CLI `phalcom-core/bin/phalcom/cli.rs:Commands` has `Tokenize`, `Parse`, `Disasm`, `Check`, `Version` and a default run path; there is **no existing persisted-artifact `build` implementation**. Propose (spelling subject to UX review):

```text
phalcom build <project-dir|package-dir|module.ph> \
    [--profile debug|release|interface-only] [--locked] [--offline] \
    [--target TARGET] [--output PATH] [--emit package|project]
phalcom run <source|package.telpkg|project.telproj> [--entry MODULE]
phalcom artifact inspect <file> [--json] [--sections] [--dependencies]
phalcom artifact verify <file> [--full] [--offline]
phalcom artifact disasm <file> [--module MODULE] [--callable SYMBOL]
phalcom cache stats|prune|verify
```

Do not break existing default source execution, project/package directory detection or `--release`/`--unchecked` contract semantics. Compile-mode, reflection policy, target and feature flags MUST participate in compatibility/build keys. `--locked` rejects dependency graph drift. `--offline` MUST perform no network retrieval and fail explicitly for missing required artifacts. `--output` writes atomically only after validating complete artifact bytes. `artifact verify` MUST validate without executing any initializers.

## 11.4 Package registry/provider interface

The existing `PackageArtifactProvider::get_package_artifact(ResolvedPackageId)` returns a reference to an in-memory linked artifact. An artifact-aware provider should resolve durable selected versions/content digests, inspect headers, and retrieve semantic/executable sections as needed. Consider a new trait or evolving adapter rather than overloading the old method ambiguously:

```rust
trait CompiledArtifactProvider {
    fn resolve(&self, request: &PackageRequirement, lock: &ArtifactLock)
        -> Result<ArtifactDescriptor, ResolutionError>;
    fn open(&self, id: &ArtifactContentId) -> Result<Arc<ArtifactReader>, ArtifactIoError>;
    fn external_catalog(&self, id: &ArtifactContentId)
        -> Result<Arc<dyn ExternalModuleCatalog>, ArtifactError>;
}
```

Providers MAY be local cache, embedded Universe, filesystem/path, vendored, or registry sources (existing `PackageOrigin` vocabulary). All must enforce identical identity/integrity and ABI validation. Remote access/security policy is an independent product decision; no implicit package download inside semantic query evaluation.

## 11.5 Incremental build granularity

A project build MUST reuse unchanged dependency artifacts by exact key and avoid recompiling all packages when one local body changes. Differentiate:

- public interface/type/conformance/metadata ABI changes → invalidate downstream semantic consumers that depend on changed facts;
- private body changes without observable ABI change → recompile/repackage owner, normally preserve downstream semantic query reuse;
- generic template/inlining implementation changes → invalidate any downstream executable specialization that consumes that template/implementation;
- dependency resolution/feature/ABI change → invalidate affected package world assembly;
- local source changes → only owning package source shard and semantic reverse closure;
- optional debug/source map changes → avoid unnecessary semantic/bytecode recompilation when debug profile policy permits.

Content-addressing is not sufficient by itself: the **build graph** must record exact semantic and codegen dependencies to avoid over-invalidation and unsound under-invalidation. A package implementation change can leave semantic ABI stable while still requiring re-link/re-deploy because its executable artifact content changed.

## 11.6 Project deployment bundling

A `.telproj` bundle should specify root project identity and executable entry selection, exact package artifact content IDs, dependency graph, installation order, compile/native compatibility and optional embedded `.telpkg` blobs. It MUST preserve package module ownership and avoid rebinding two distinct packages to the same runtime `ModuleId`. A self-contained bundle is a convenient transport; the loader still verifies and resolves each distinct package image, including `universe` compatibility. Project artifact paths are not semantic identity and can be relocated to another machine without recompilation.

---

# 12. Source-Free Artifact Loader — Verification, Relocation, Installation, Initialization

## 12.1 Two distinct loading consumers

1. **Compiler dependency image loader**: decodes/validates module and semantic-interface sections; assembles a semantic world with type/static/evidence relocations; registers immutable external catalogs and query roots; **does not execute code**.
2. **VM executable loader**: decodes/validates required executable and runtime metadata sections; resolves dependencies and relocations; allocates fresh VM runtime objects and callable code; initializes modules in their specified order. It may use compiler-dependency products for validation/debugging but MUST NOT run semantic analysis merely to execute a built package.

These loaders SHOULD share the artifact reader/header/identity verifier and stable symbol mapping, but MUST NOT make the runtime loader instantiate a mutable semantic workspace unnecessarily.

## 12.2 Required loader state machine

```text
UNOPENED
  → HEADER_VERIFIED
  → SECTION_DIRECTORY_VERIFIED
  → REQUIRED_SECTIONS_PARSED
  → DEPENDENCY_GRAPH_VERIFIED
  → CODE_AND_METADATA_VALIDATED
  → RELOCATION_PLAN_BUILT
  → VM_INSTALL_PREPARED
  → RELOCATED_AND_REGISTERED
  → INITIALIZING
  → INITIALIZED
```

Failures before `VM_INSTALL_PREPARED` have **no VM side effects**. Plan registration and pool assignment deterministically before mutating live VM state. Pre-initialization failures MUST rollback staged registration/objects where existing runtime transactional capabilities permit it; where rollback is impossible, fail closed and record a failed artifact/module instance that cannot masquerade as initialized. Once a top-level initializer executes user code, its effects are not generally reversible: preserve existing sticky `ModuleState::Failed` semantics and document that distinction. Never publish a partially loaded package as a successful dependency.

## 12.3 Detailed load algorithm

**Phase L0 — Open and resource budget.** Check magic, version, file length and bounded section directory without allocating attacker-declared unbounded memory. Validate target, native ABI, bytecode schema, semantic schema, feature requirements and package coordinate.

**Phase L1 — Verify integrity and provenance.** Compute section/content digest in the specified canonical domain; validate dependency exact content IDs and selected package coordinates against the lock/bundle. Optional signature/trust verification is additional to mandatory structural/digest validation, not a replacement.

**Phase L2 — Resolve dependencies.** Resolve topological artifact dependencies and `universe` first. Detect missing/incompatible/cyclic/conflicting artifact refs. Do not silently load source for unresolved required binary dependencies. Bind import aliases to stable package instances.

**Phase L3 — Validate semantic/code payloads.** Validate metadata using existing [`validate_metadata_bundle`](https://github.com/aureat/phalcom-lang/blob/c761de8d855993ea83e3597b016eec1f70d55d58/phalcom-type-meta/src/validate.rs#L651-L660) and its limits. Validate portable opcode discriminants, operand schemas, constants, prototype trees, upvalue descriptors, parameter shapes, branch/jump targets, instruction/constant/semantic pool indexes, stack/slot bounds, relocation indices and all stable imported symbol identities. Unknown required opcode/feature is fatal. Use iterative or depth-budgeted traversals.

**Phase L4 — Build relocation tables.** Map stable package/module/symbol/callable/type/metadata/native keys to VM-local IDs and checked handles. Resolve references against previously validated dependency export catalogs; disallow private/exposure violations. Create a single mapping for repeated symbol identity so all imports and re-exports converge to the same canonical entity.

**Phase L5 — Reserve runtime modules/pools.** Allocate/stage `ModuleRecord`, runtime module objects, static constant registration and required class/enum/data descriptor rows. Assign metadata pools and module runtime IDs with complete collision checks. Artifact load idempotence is keyed by **(VM instance, selected artifact content ID, semantic world/dependency set)**, not a global hardcoded `RuntimeProgramId(1)` or `ModulePlanFingerprint(0)` (both currently appear in ordinary materialization at `materialize.rs:300–307`).

**Phase L6 — Materialize code without compiling source.** Decode/instantiate `PortableCallablePrototype` into fresh `Callable` and `Chunk` values, intern symbols, allocate immutable runtime constants appropriately, allocate empty inline/global/associated caches, install typed executable semantic pools, bind top-level initializer closure and nested function prototypes. Preserve `Chunk.source_id` mapping and debug info if present; absent optional source maps must not break execution.

**Phase L7 — Link runtime reads, exports, metadata.** Materialize `LinkedReadSpec`-equivalent symbolic dependencies, source-owned re-exports and static constants, map artifact-qualified metadata records into assigned VM pools, install reflection/declaration and method bindings, and verify all runtime dependencies are registered. Reuse `VM::materialize_linked_exports_for_module` behavior where possible instead of implementing a second inconsistent export projection.

**Phase L8 — Execute initializers.** Initialize dependency packages first and then only local modules in deterministic topological order. External modules already `Initialized` in module registry satisfy dependencies; do not re-run them. Detect cycles/reentrancy, missing and failed deps using the same semantic error behavior as `VM::initialize_single_module`. Mark each module Initialized or Failed accurately.

**Phase L9 — Commit and expose.** Publish the root artifact instance and entry module only after successful validation/installation; successful initialization for executable mode. Expose loaded modules to future artifacts through stable symbol/package maps. Track cache hits, relocations and import sharing.

## 12.4 Bootstrap of precompiled Universe

The current VM bootstrap (`phalcom-core/src/vm/bootstrap.rs`, around lines 193–237) constructs a native-floor VM, installs canonical runtime metadata and native primitives, and executes source-authored Universe modules so their class reopens and wrappers attach to kernel objects. The precompiled Universe loader MUST preserve **the same installation and initialization order**, replacing source parsing/compilation of Universe modules with portable precompiled initializer/code prototypes. It does **not** freeze the initialized heap. Each VM still creates fresh runtime classes/module objects/closures/caches and runs required initialization once for that VM. The process may share immutable artifact bytes and verified decoded prototype tables; it must not share runtime object aliases or mutable runtime callables.

Boot fallback policy MUST be explicit: if the embedded/preinstalled Universe artifact is missing or incompatible, either (a) fail with a typed compatibility error or (b) use a source-build fallback explicitly permitted by developer/toolchain policy, with a recorded counter. A production “artifact fast path” acceptance test must reject silent source fallbacks.

## 12.5 Lazy vs eager loading

A minimal first loader MAY decode/relocate all required prototypes eagerly for simplicity, but must not execute initializers prematurely. Later lazy prototypes/metadata sections may load on demand via verified indices and memoized immutable reads. The invariant is that lazy loading cannot change side-effect order, reflection visibility, class identity, exception source locations or failure semantics. Lazy decoding is a performance optimization, not permission to omit required static validation of executable boundaries.

## 12.6 Load-time error taxonomy

Proposed typed failures: `UnsupportedContainerVersion`, `UnsupportedBytecodeAbi`, `UnsupportedSemanticModel`, `NativeAbiMismatch`, `TargetMismatch`, `FeatureMismatch`, `DigestMismatch`, `MalformedSection`, `ResourceLimitExceeded`, `InvalidOpcode`, `InvalidRelocation`, `MissingDependencyArtifact`, `DependencyVersionConflict`, `MissingExport`, `VisibilityViolation`, `PackageIdentityCollision`, `MetadataValidationFailed`, `InvalidTypeRelocation`, `RuntimeRegistrationConflict`, `InitializerFailed`. Maintain error context (artifact key, section, module, callable, source span where available). Never panic on malformed external artifact bytes.

---

# 13. Safety, Integrity, Supply Chain, and Resource Limits

## 13.1 Untrusted artifact assumption

Treat artifact input as untrusted even when retrieved from a local cache. Validate before runtime binding; do not invoke `serde` on unrestricted recursively nested structures without explicit size/depth budgets. `phalcom-type-meta` already has `ValidationLimits`; extend comparable constraints to artifact section sizes, counts of modules/types/callables, prototype nesting, constant pool size, number of relocations, source map length, package depth and decoded byte limits. Support cancellation and memory budgets for compilation/loading where practical.

## 13.2 Content-addressed cache robustness

Use atomic temp-write → flush/fsync as required by durability policy → validate → rename. Concurrent builders of the same digest must not publish different bytes; on cache hit revalidate identity and schema. Handle interrupted writes, corrupt cache entries, races, symlinks/path traversal (especially archive extraction) and mixed compiler versions. Do not mix 'cache key produced from known inputs' with 'artifact digest of actual bytes' without verifying both. Support eviction by reachability/quota, not only file age; an artifact referenced by a current project lock must be recoverable.

## 13.3 Trust, signatures, and native ABI

Content hashes provide integrity and reproducibility, not publisher authentication. If registry distribution needs authentication, define optional signatures/attestations with an explicit trust-store and canonical signed payload; never assume authenticated just because bytes hash correctly. Native ABI digest MUST cover the semantic shape of intrinsic/native declarations and the runtime/primitive contract actually used by compiled code; an incompatible native primitive surface must fail validation instead of yielding delayed VM corruption. No artifact may install arbitrary unapproved native host functions merely by naming an intrinsic.

## 13.4 Artifact version migration

Container format, semantic schema, metadata schema and portable bytecode ABI MUST be versioned independently. Define a compatibility table and maximum forward/minimum backward accepted versions, with converters only when representation migration is provably sound. Unknown optional sections may be skipped, unknown required sections cannot. A new opcode, executable semantic pool tag, native ABI change or metadata representation change must invalidate or require conversion of affected artifacts. Version refusal is preferable to speculative reinterpretation of old bytes.

---

# 14. Incrementality, Reuse, and Complexity Budgets

## 14.1 Separate invalidation domains

| Change | Source/module work | Semantic work | Executable artifact work |
|---|---|---|---|
| Local function body only, exported contract stable | Changed local body/source shard | Changed body and exact relevant dependents | Re-emit affected prototypes/modules; package digest changes |
| Local callable signature/export | Affected local interface/importers | Relevant signatures, call sites, proof dependents | Recompute semantic ABI; dependent packages invalidate if observed ABI changes |
| Local conformances/derived rules | Local module update | Candidate-set coherence, witness and proof dependents | Rebuild affected evidence recipes, exported semantic interface if public |
| Local import/re-export | Import-site/local linker graph | Local dependency closure | Relocations/export table and affected initializers |
| Dependency implementation changed, public ABI unchanged | Artifact lock/content update | Normally preserve semantic consumers unless inlining/private template use | Relink/redeploy exact dependency instance; invalidate specialized consumers as needed |
| Dependency public interface/semantic ABI changed | External image descriptor changes | Revalidate all observed exported facts and conformance candidate sets | Rebuild affected importers and callable prototypes |
| Canonical Universe image changed | New world identity | New semantic world assembly; reject stale lineage | Rebuild/revalidate all incompatible compiled artifacts |
| Debug metadata/source map only | Optional presentation update | No formal reanalysis if semantic inputs unchanged | Debug section may change; executable digest policy explicit |

## 14.2 Required persistent compiler dependency graph

Maintain semantic input fingerprints at the smallest sound level: source parse/interface, callable signature/body, trait/conformance candidate set, type normalization, alias/static domains, generic evidence environment, import/export resolution and codegen dependencies. A query's correctness must not depend merely on checking that its current positive dependencies remain unchanged when an absent candidate or newly added conformance could change the answer. Fingerprint invalidation MUST be deterministic and transactionally published.

## 14.3 Scaling target

Warm compilation work should be approximately:

```text
T = canonical image lookup (constant relative to Universe size)
  + local module resolution and affected graph closure
  + local semantic queries and referenced external facts
  + candidate proof/coherence work
  + required metadata/reflection closure
  + local portable code emission
  + deterministic artifact writing
```

Do not enforce a false strict `O(L)` bound when a local program legitimately references `D` external declarations or emits a large `R` reflective closure. Do enforce absence of unconditional global scanning over all canonical modules/callables/declarations. Persistent no-op compilation should amortize to fingerprint/validation work without body, conformance or codegen recomputation, subject to deterministic no-op check policy.

## 14.4 Current algorithmic debt to address after the architectural cutover

1. Combine three source type-target AST traversals into one per-module `ResolvedSourceTypeTargets` product; eliminate per-conformance whole-program AST walks.
2. Replace full-snapshot formal attachment searches with `SourceSiteId`/module indexes; use interval-aware queries for containment/overlap, not a `BTreeMap<SourceRange>` when it only supports exact keys.
3. Group advisory callables by local module, index source occurrences and track local fixed-point deltas; do not run advisory in executable-minimal mode.
4. Replace per-requirement global callable signature scans with a witness-owner/side/selector candidate index; retain exact semantic candidate matching.
5. Index conformance by trait/target family **with conservative generic/blanket fallback**; avoid canonical↔canonical recomputation each workspace.
6. Memoize exact generic evidence proofs only with complete goal/context/dependency closure; guard negative/ambiguous results.
7. Make module-value environment body inputs dependency-sensitive; avoid wholesale second recheck when no relevant fact changed, but never skip semantic validation on a mere fingerprint hack.
8. Index lowering `(ModuleId, CallableId, ExpressionId)` and trait slot / conditional impl lookups; avoid expression × reference and global conditional-member scans.
9. Replace metadata declaration × callable scan with owner indexes; root-driven traversal according to normative reflection profile.
10. Move flat semantic aggregate snapshots to shared persistent/module-sharded roots after initial parity cutover.

---

# 15. Implementation Program — Plans, Dependency Order, Write Sets, and Gates

This is **one unified specification**, implemented through separately reviewable, evidence-gated work packages. A package/task's test gate must pass before its consumers assume that capability exists. Avoid broad concurrent edits in `phalcom-semantic/src/session.rs` and `phalcom-core/src/modules/compile.rs` without a defined integration owner.

## 15.1 High-level dependency DAG

```text
P0 evidence/observability
      │
      ▼
P1 resolver/catalog/input hygiene
      │
      ▼
P2 canonical module image ───────────────────┐
      │                                      │
      ▼                                      ▼
P3 canonical formal semantic image       P5 external module linker
      │                                      │
      ▼                                      │
P4 immutable query base + local overlay      │
      │                                      │
      └─────────────────┬────────────────────┘
                        ▼
                 P6 remove 40-source seed
                        │
               ┌────────┴─────────┐
               ▼                  ▼
       P7 consumer profiles   P8 persistent aggregates
               └────────┬─────────┘
                        ▼
              P9 metadata closure/pools
                        ▼
              P10 local algorithm fixes
                        │
                        ▼
A1 stable package identities / wire format / build graph
                        │
              ┌─────────┴──────────┐
              ▼                    ▼
A2 exported semantic        A3 portable codegen
   package images                  │
              └─────────┬──────────┘
                        ▼
A4 package artifact serializer + project bundle builder
                        │
                        ▼
A5 dependency compiler loader and multi-package world assembly
                        │
                        ▼
A6 VM artifact verifier/relocator/loader
                        │
                        ▼
A7 precompiled Universe cold-start / CLI / publishing certification
```

Some artifact-format/API design (`A1`) can be undertaken during P2/P3, and `A2` and `A3` can be prototyped concurrently. However **no A5/A6 production cutover** should occur before the module/semantic external-dependency contracts and portable code invariants are validated. P9 metadata identity is a dependency of source-free cross-pool execution. A single giant implementation PR is prohibited.

## 15.2 P0 — Baseline and observability (required first)

**Read:** `telos-observability/src/{phase,metric,context,recorder,report}.rs`; compiler spans in `phalcom-core/src/modules/compile.rs` and `canonical_universe.rs`; module counters in `phalcom-modules/src/session.rs`. **Tasks:** pin commit/toolchain/profile; reproduce tiny/large/warm/cold workloads; record source/module/conformance/count populations; capture `opt-level` controls; add counters with negligible feature-disabled and Off-mode overhead; establish reference output/diagnostic fixtures. **Gate:** comparable before/after machine-readable baselines, verified test selector counts and declared instrumentation overhead. Failure to reproduce historical 45 s is not permission to skip cardinality analysis—update the measured baseline.

## 15.3 P1 — Immediate reconstruction hygiene

**Write set:** `phalcom-modules/src/resolver.rs`, `builtin_interface.rs`, perhaps `builtin.rs`; `phalcom-semantic/src/workspace.rs`, `snapshot.rs`/`session.rs` as needed. **Tasks:** canonical interface-cache routing; provider-owned declaration catalog; coherent interface forwarding instead of rebuild/discard; remove duplicate formal projection construction. **Gate:** zero warm canonical interface/catalog rebuilds per fixture; no changed source diagnostics, import resolution or source locations. Keep patches small and independent.

## 15.4 P2 — Canonical module image

**Write set:** proposed `phalcom-modules/src/canonical_image.rs`; existing `builtin.rs`, `builtin_interface.rs`, `resolver.rs`, `linker.rs`, `project.rs`, `topology.rs`, `lib.rs`, selected module tests. **Tasks:** build process-shared module image from consistent provider, interfaces, resolved imports, linked graph and topology; record provenance and public/private fingerprints; support private overridden provider world; expose read-only external catalog. **Gate:** one canonical build per world; exact parity with current source-complete canonical linking and export identities. **Do not** yet remove local discovery/semantic seeds.

## 15.5 P3 — Canonical formal image

**Write set:** proposed `phalcom-semantic/src/canonical_image.rs`; `canonical_baseline.rs`, `session.rs`, `workspace.rs`, `snapshot.rs`, `lib.rs`; `phalcom-core/src/modules/canonical_universe.rs`; focused semantic/core tests. **Tasks:** preserve baseline as foundation; explicit `from_foundation`; complete ordinary analysis using P2 module image; freeze committed snapshot + final type/static arena roots + query eligibility census; rewire core canonical program projection to consume the image; document current tolerated Universe diagnostic contract. **Gate:** exactly one full canonical semantic analysis per new world; no second full canonical analysis in core; VM bootstrap behavior/bytecode projection parity.

## 15.6 P4 — Canonical-aware SemanticDb and session overlay

**Write set:** `phalcom-semantic/src/db/{mod,query,dependency,state,key}.rs` as applicable; `session.rs`; type/static lineage code; `impls.rs` and `generic_evidence.rs` only for composition interfaces. **Tasks:** immutable query image, namespace/origin, canonical product/fingerprint/currentness fallback, local-only `query_state`, immutable leaf dependency handling, reject canonical writes/discard, local transaction failure handling, `from_image`. Temporary table clones allowed with explicit `canonical_entries_copied` counter. **Gate:** zero canonical query republications in fresh image-backed local sessions; wrong-world fail-closed; local/canonical coherence parity, negative proof invalidation and no-op revisions sound.

## 15.7 P5 — External module-world linker cutover

**Write set:** `phalcom-modules/src/{session,linker,resolver,topology,artifact,project}.rs`, new catalog adapter; tests `workspace_component_linking.rs`, `workspace_session.rs`, `linker.rs`, `import_resolution_reuse.rs`. **Tasks:** local/external target classification, stop transitive canonical source discovery, local-only connected components, external export/linked interface lookup, composed topology, external runtime dependency leaves, tolerant diagnostics. Integrate with `ProgramCompilationSession` and standalone/project/package entry; ensure no fixture-only bypass. **Gate:** zero canonical sources in local lifecycle, components and relink counts; exact import/re-export/runtime symbol and diagnostic parity; no canonical runtime reinitialization.

## 15.8 P6 — Remove 40-module semantic source workaround

**Write set:** `phalcom-core/src/modules/compile.rs` (`ProgramAnalyzer`, `EntrySelection`, `discover_reachable_dependencies`, `compile_entry_selection`), `compile_session.rs`, `phalcom-semantic/src/session.rs`, focused tests. **Tasks:** compare source-seeded reference mode to external-image mode for inline/standalone/package/project; preserve entry/source identity and semantic products; then remove `collections/scalar/text/option.unit` semantic seed paths; compile runtime-module-local programs. **Gate:** zero canonical deep analysis/bodies/trait surfaces/witness rebuilds for warmed normal user compilation; semantic diagnostics, lowering and execution exact parity. Keep reference path behind tests until gate passes.

## 15.9 P7 — Consumer-specific publication

**Write set:** `phalcom-semantic/src/{session,snapshot,editor}.rs`, `source_index/` and advisory/formal projection modules; `phalcom-core/src/modules/semantic_lowering.rs`; compiler request APIs. **Tasks:** validated product profiles, explicit `NotRequested`, frozen canonical source/advisory/formal shards, local deltas, checker-owned lowering attachments independent of full editor index. **Gate:** zero canonical source/advisory/formal reconstruction in executable mode; editor navigation and diagnostics parity in full mode. Do not equate omitted editor products with omitted runtime reflection metadata.

## 15.10 P8 — Persistent semantic aggregates

**Write set:** `phalcom-semantic/src/{declarations,signature,dispatch,impls,derived_conformance,session,snapshot}.rs`, relevant hierarchy/data/enum/static tables. **Tasks:** replace repeated deep map clones/flattening with `Arc` canonical shards and persistent module-keyed local deltas; retain read views and indexed owner/candidate lookup. **Gate:** zero canonical aggregate entries copied during warm fresh-session creation, no full-world snapshot materialization except explicitly requested full-world consumers; transaction/parity tests.

## 15.11 P9 — Metadata roots and pool references

**Write set:** `phalcom-semantic/src/metadata/{export,stable_identity}.rs`; `phalcom-type-meta/src/{bundle,identity,validate}.rs` as justified; `phalcom-core/src/{modules/materialize.rs,typing/loader.rs,typing/handle.rs}`. **Tasks:** normative root taxonomy; owner indexes; root-driven closure with public reflection; stable identity and general cross-artifact references; VM pool remapping/validation. **Gate:** reflection/dynamic/serialized type behavior unchanged, no unconditional whole-Universe metadata sweep, validated metadata cross-pool identities. No arbitrary percentage reduction is a correctness criterion.

## 15.12 P10 — Local algorithmic optimization

**Write set:** checker body/query implementations, `source_index/`, `generic_evidence.rs`, `impls.rs`, lowering/callable lookup modules. **Tasks:** combine repeated AST collectors, index attachments, witness and conformance candidates, track module-value reads, return dependency SCCs where justified, memoize proofs safely, index lowering selections and trait slots. **Gate:** same semantics under edits and cycles; improved local work-count slopes and wall-time profiles, no new over-invalidation.

## 15.13 A1 — Portable package identity, schema and build graph

**Proposed write set:** `phalcom-modules/src/{artifact,package_info,project,manifest}.rs`; new artifact-schema crate or cohesive shared module, deterministic encoder/decoder, lockfile/build cache interfaces; CLI arguments. **Tasks:** settle package coordinate/content identities, compiler ABI matrix, stable refs/relocation, artifact sections and capabilities, content digests, manifest+dependency exact locks, deterministic encoding, limits and error taxonomy. **Gate:** golden format fixtures, structural round-trip, unknown-required section rejection, checksum and invalid-identity refusal, deterministic cross-directory builds; no source compiler change yet.

## 15.14 A2 — Serialized semantic interface/package image

**Write set:** new semantic-interface DTO adapter in `phalcom-semantic`, stable identity code, `phalcom-modules/src/artifact.rs`, type/static relocation adapter, query export eligibility. **Tasks:** export downstream-required declarations, signatures, generic/trait/conformance/static facts without source; create compiler world assembler for multiple package images; prove type/store/static relocation and cross-package coherence. **Gate:** compile a consumer from package artifact alone after dependency source removal; exact type/diagnostics/proof semantics; incompatible package/world rejection. Interface-only artifacts explicitly reject execution.

## 15.15 A3 — Portable code emitter and runtime-independent prototypes

**Write set:** `phalcom-core/src/{compiler/*,bytecode.rs,chunk.rs,callable.rs,modules/compile.rs,modules/artifact.rs}`; proposed `compiler/portable.rs`, runtime prototype loader adapter. **Tasks:** exhaustive operand/constant/cache inventory; extract VM-independent emitter or isolated compilation adapter; emit top-level and nested prototypes; portable constants, upvalue and semantic recipes; build relocations; validate emitted prototypes. **Gate:** code prototype parity vs VM compiler for fixture corpus; no runtime handles in DTO; reproducible code bytes; source-free executing compiled module prototype in a test VM.

## 15.16 A4 — Package writer and project bundle builder

**Write set:** package artifact encoder, `phalcom-core` build coordinator, project dependency graph/store, CLI. **Tasks:** deterministic package sections; verified manifest/lock; atomic writes; `.telpkg` library/executable variants; `.telproj` root graph index and optional embedded dependencies; audit metadata/reflective closure. **Gate:** reproducible builds across directories/processes; verify/inspect commands; no incomplete published outputs; project with multiple nested/dependent packages preserves distinct identities.

## 15.17 A5 — Imported artifact compiler world

**Write set:** external module catalog provider, `SemanticWorldAssembler`, type/static relocation, SemanticDb foreign image references, conformance proof/coherence index; `ProgramAnalyzer` project path. **Tasks:** load exact dependencies from artifact store, merge into one coherent semantic world with stable module refs, compile local source-only owner against them, reuse ABI/inlining dependency fingerprints. **Gate:** source-free downstream compilation with multiple packages, generic derived rule over user type, local↔external and external↔external coherence, no accidental TypeId collisions, no broad dependency source work.

## 15.18 A6 — Source-free VM artifact loader

**Write set:** proposed `phalcom-core/src/modules/{portable_artifact,code_loader}.rs`, `materialize.rs`, `initialize.rs`, `registry.rs`, `interpret.rs`, `typing/loader.rs`, `vm/bootstrap.rs`; artifact format verifier. **Tasks:** open/validate/dependency DAG, code/metadata relocation, staged module registry assignment, fresh callable/chunk/cache allocation, exact export materialization, deterministic initialization, failure handling. **Gate:** execute artifact with all source files absent and `compile_closure_as_with_bindings = 0`; module identity/idempotence/rollback/failure tests; metadata and runtime parity.

## 15.19 A7 — Precompiled Universe, cold-start, and release certification

**Write set:** canonical image builder/loading integration, toolchain packaging, CLI distribution/config, benchmark harness, docs. **Tasks:** build versioned Universe package artifact with the ordinary package pipeline (special builtin identity), embed/install toolchain artifact, skip canonical source analysis and VM source compilation on compatible cold start, preserve native bootstrap order, reproducible toolchain packaging, diagnostic fallback policy. **Gate:** clean-process artifact boot (no canonical source semantic construction), fresh per-VM heap/caches, correct canonical class reopens/identity and reflection, statistically measured cold/warm/resource improvements.

---

# 16. Tests, Differential Oracles, and Acceptance Matrix

## 16.1 Testing principle

Keep a **source-complete reference path** until each migration's differential gate is green. Test both source-backed and image-backed compilation for the *same source*, comparing more than execution output. Normalize only legitimately session/world-specific IDs under a documented mapping; keep stable public `SymbolId`, qualified module/reflection identity and export identity exact where the contract requires it. No test failure may be silently weakened or reclassified as baseline without same-state clean-predecessor evidence.

## 16.2 Repository test owners and registration

Existing modules test files include `phalcom-modules/tests/{builtin_catalog,builtin_provider,parse_cache,linker,workspace_component_linking,workspace_session,import_resolution_reuse,deterministic_module_graph,package_info_semantics,package_semantic_contract}.rs`. The primary semantic integration harness is `phalcom-semantic/tests/semantic.rs` (many subordinate modules); core module cases include `phalcom-core/tests/core/modules/{compile,linking,parity,runtime,reflection,universe,package_info,project_reflection}.rs`, plus existing `canonical_bootstrap_stack.rs` and `phalcom-type-meta/tests/schema_compat.rs`. **Check actual Cargo harness registration and test selection**; a passing command selecting zero tests is not evidence. Add new artifact format tests as dedicated registered integration tests where appropriate.

## 16.3 Mandatory cross-boundary test cases

| Test ID | Scenario | Expected invariant |
|---|---|---|
| MOD-01 | Local whole-module `universe.*` import | Same exact target; canonical module excluded from local source/component |
| MOD-02 | Selective import, alias, re-export | Stable original `SymbolId`, visibility and source diagnostics |
| MOD-03 | Nested package and exposed child import | Canonical topology viewed, not flattened/relinked |
| MOD-04 | Missing canonical export/import typo | Identical local range, error code/message/severity |
| MOD-05 | Persistent import added, removed, restored | Only local graph delta; no canonical source lifecycle events |
| MOD-06 | Two local importers share canonical target | One external dependency entity; no source/module duplication |
| MOD-07 | Two distinct published package versions | Correct identity isolation or explicit conflict according to policy |
| SEM-01 | Local type conforms to Universe trait | Same witness and trait evidence |
| SEM-02 | Canonical derived rule matches local type | Local proof uses immutable rule correctly |
| SEM-03 | Local head overlaps canonical blanket head | Same coherence conflict, no accidental precedence |
| SEM-04 | Two imported package conformances conflict | Detect conflict at world composition |
| SEM-05 | Local subclass of canonical generic class | Same hierarchy, substitution, dispatch |
| SEM-06 | Alias/associated static/generic type across package | Stable normalization and static-value semantics |
| SEM-07 | Failed local update followed by fix | Canonical image unchanged, local LKG/rollback correct |
| SEM-08 | Add local conformance after cached negative proof | Cached failure invalidated |
| SEM-09 | Foreign type prefix or static store lineage | Fail closed, never reinterpret raw IDs |
| SEM-10 | Two package semantic images share numeric local TypeId | Explicit relocation/qualification prevents collision |
| AOT-01 | Build and execute package after deleting sources | No parse/check/VM source compilation on load |
| AOT-02 | Nested closures, captures/upvalues | Identical behavior, fresh VM-local captures and caches |
| AOT-03 | Trait/default/conditional/associated invocation | Identical evidence, code recipe and dispatch |
| AOT-04 | Static/data/enum/anonymous products | Portable constants, layouts, case identities correctly relocated |
| AOT-05 | Linked import from pre-initialized canonical module | No duplicate initialization; correct runtime binding |
| AOT-06 | Load same artifact twice into one VM | Idempotence or typed duplicate error per documented policy |
| AOT-07 | Load same artifact into two VMs | Shared bytes allowed; runtime object identity/caches isolated |
| AOT-08 | Missing/failed dependency | Typed failure with no partial successful publication |
| AOT-09 | Reflection on local and external declarations | Metadata pool identity and public discoverability parity |
| AOT-10 | Change compiler/native/bytecode ABI | Incompatible artifact rejected before execution |
| AOT-11 | Corrupt/malicious bytecode/section/relocation | Bounded typed validation error, never panic/UB |
| AOT-12 | Identical build in distinct source checkout paths | Identical required artifact bytes/content hash |
| ART-01 | Interface-only package imported | Consumer compiles without dependency source |
| ART-02 | Interface-only package executed | Explicit unsupported-capability error |
| ART-03 | Project with nested packages | Distinct package artifacts, correct root entry and lock |
| ART-04 | Library private body edit | Public ABI stable, owner implementation/code digest changes |
| ART-05 | Public trait/evidence change | Downstream semantic dependents invalidated precisely |
| ART-06 | Concurrent artifact builds/loads | Atomic cache writes and thread/VM isolation |
| PERF-01 | Warm fresh sessions A/B/C | No canonical reconstruction in any fresh session |
| PERF-02 | Persistent no-op/body/signature/import edit | Only required local invalidation/recompilation |
| PERF-03 | Precompiled Universe first-process bootstrap | No source semantic analysis and no VM source compile |

## 16.4 Source/image differential comparison

Use both paths:

```text
REFERENCE: current canonical-source-seeded, source-complete compilation
IMAGE:     completed canonical module/formal images + local workspace overlay
```

Compare declaration IDs/forms, signatures, type results, inferred returns, hierarchy, dispatch, trait surfaces, explicit/derived conformance rules, witness plans, generic proof outcomes, static definitions/values, import/export identity, runtime dependencies, diagnostics, formal facts and lowering selections. After A3, compare emitted opcode semantic shape, constant/relocation identity, VM results, errors and reflective outputs. `SnapshotId`, `WorkspaceId`, `TypeStoreId` are not ordinarily literally identical between sessions; require a proven semantic lineage mapping rather than deleting those assertions wholesale.

## 16.5 Persistence and failure corpus

Maintain fixtures for truncation at every header/section boundary; size overflow; invalid UTF-8 in symbolic names; duplicate section entries; forward-unknown optional/required sections; digest mismatch; unsupported semantic/native/opcode version; cyclic/deep graphs; out-of-range branch/constant/upvalue; invalid static-value/type relocation; nonexistent private export; circular project dependency; invalid runtime initializer order; interrupted atomic write and corrupt cache recovery. Fuzz decoding/validation of *nonexecuting* reader and relocation planning. Protect CI resource budgets to avoid denial of service.

---

# 17. telos-observability and Performance Acceptance

## 17.1 Required phase spans

Add or extend phase spans and counters in `telos-observability` and owning crates. Minimum spans:

```text
compile.total
module_session.discovery
module_session.interface
module_session.import_resolution
module_session.link_local
canonical_module_image.build/load
canonical_semantic_image.build/load
semantic.session_init
semantic.signatures / bodies / conformance / static
semantic.source_index / advisory / formal_projection / snapshot
semantic.query_image.hit / miss / publication
compiler_projection.lowering / metadata / evidence
portable_codegen.emit / validate / serialize
artifact.reader.open / header / sections / dependencies
artifact.compiler_world.import / type_relocation / coherence
artifact.vm.relocate / materialize / metadata / initialize
canonical_universe.artifact_load / source_fallback
```

Emit wall time, counts, bytes, reuse hit rates and failure outcome; avoid hot-path instrumentation overhead when disabled/Off. Where the existing metric enums differ, extend intentionally and write a metric contract; do not rename existing counters silently and break dashboards/benchmarks.

## 17.2 Structural counters and exact scopes

| Counter | Warm-one-shot target after corresponding gate |
|---|---:|
| `module.canonical_catalog_builds` | 0 |
| `module.canonical_interface_builds` | 0 |
| `module.canonical_sources_promoted_to_workspace` | 0 |
| `module.canonical_modules_discovered_transitively` | 0 |
| `module.canonical_modules_in_local_components` | 0 |
| `module.canonical_modules_relinked` | 0 |
| `semantic.canonical_modules_deep_analyzed` | 0 |
| `semantic.canonical_callable_bodies_analyzed` | 0 |
| `semantic.canonical_witness_plans_built` | 0 |
| `semantic.canonical_query_republications` | 0 |
| `semantic.canonical_canonical_overlap_checks` | 0 |
| `source_index.canonical_modules_rebuilt` | 0 |
| `formal_projection.canonical_modules_rebuilt` | 0 |
| `advisory.canonical_modules_processed` | 0 |
| `snapshot.canonical_entries_copied` | 0 **after P8**, not immediately after P4 |
| `metadata.unconditional_canonical_full_scans` | 0 **after P9** |
| `artifact.vm.source_compilations` | 0 for release saved-artifact execution **after A6** |
| `artifact.load.semantic_checker_calls` | 0 for saved executable load **after A6** |
| `canonical_universe.artifact_source_fallbacks` | 0 for prebuilt Universe fast-path acceptance **after A7** |

`local_canonical_overlap_checks`, local proofs using canonical rules, referenced external declaration lookups, required metadata closure traversal and actual local codegen are **allowed**. The zero counters refer to *unnecessary canonical reconstruction*, not every interaction with canonical semantics.

## 17.3 Benchmark acceptance without invented thresholds

Mandatory H1 benchmark success: all structural zero gates at relevant plan boundaries, semantic/runtime parity, reproducible reduced median/p95 against same-profile predecessor, no catastrophic large-source or retained-session regression and bounded memory overhead. A 10× or 100× warm tiny-compilation speedup and sub-200-ms optimized compilation may be tracked as **stretch goals**, not falsely promised guarantees. Separate compiler wall time from VM canonical bootstrap and VM execution. Always report cold-vs-warm numbers and whether the process image was already initialized.

Mandatory H3 benchmark success: source-free package load has zero parser/analyzer/VM source compiler invocations and measurably lower startup work than the matched source path; precompiled Universe cold boot skips canonical source semantic/code emission. Report disk read/decode/verify/relocation/metadata/initialization separately, not one undifferentiated `VM::new()` duration. Report package size/section sizes, peak RSS, code/metadata cache sharing and per-VM allocated bytes.

---

# 18. Architectural Decisions That MUST Be Resolved Before Specific Cutovers

| ID | Decision / unresolved risk | Required outcome / owning gate |
|---|---|---|
| D-01 | Module external dependency interface | `ExternalModuleCatalog` supports linked imports/exports/visibility without local linked interfaces; P2/P5 |
| D-02 | Canonical module-world identity | Consistent provider/source/native ABI provenance and override isolation; P2 |
| D-03 | Bootstrap constructor cycle | `from_foundation` does not recursively call image-backed `new`; P3 |
| D-04 | Query freeze eligibility | Only committed, closed and relocatable canonical products; P3/P4 |
| D-05 | Immutable `query_state` semantics | Local-only query rows; canonical currentness/fingerprint separate; P4 |
| D-06 | TypeStore/static arena lineage | Final prefix sharing for one world; world-import relocation for packages; P4/A2/A5 |
| D-07 | Conformance candidate completeness | No local-first precedence, conservative generic-head candidate coverage; P4/P10/A5 |
| D-08 | Source-index/editor product omission | Explicit `NotRequested` and lowering-specific attachment; P7 |
| D-09 | Reflection root contract | RuntimePublic preserved, dynamic/public closure normative; P9/A4 |
| D-10 | Durable package identity | Distinguish coordinate, content hash, semantic ABI and VM instance; A1 |
| D-11 | Stable semantic project/module refs | No absolute checkout path or synthetic process counter in published identities; A1/A2 |
| D-12 | Multiple immutable package TypeStores | Relocation/import or qualified pool representation, proof of noncolliding identity; A2/A5 |
| D-13 | Generic templates exported | Downstream generic specialization without upstream source; A2/A3 |
| D-14 | Portable code emitter boundary | Code generation independent of live mutable VM or validated interim adapter; A3 |
| D-15 | Executable constant/relocation format | No raw `Value`/`ObjRef`/`Symbol`/`Cell` serialization; A3 |
| D-16 | Artifact bytecode ABI | Explicit opcode feature/version validation and branch/operand checks; A1/A3 |
| D-17 | Build manifest + lock semantics | Exact dependencies, cross-version namespace rules and deterministic source identity; A1/A4 |
| D-18 | Loader atomicity and partial initializer effects | Verify before publication; sticky failed state after side-effecting execution; A6 |
| D-19 | Runtime module identity collision/idempotence | VM artifact instance key, program ID and plan fingerprint, no implicit overwrite; A6 |
| D-20 | Universe prebuilt fallback policy | Source fallback explicit and measurable, not hidden in release artifact fast path; A7 |

**Stop-and-consult** if an implementer needs to violate any invariant above; provide a minimal reproducer, actual function/call path, failing counter/test, options and recommended decision. The spec is intentionally not permission to invent unreviewed language or artifact ABI semantics.

---

# 19. Change Control, Checkpoint Lifecycle, and Deliverables

## 19.1 Current lifecycle

Treat the earlier one-piece C2 draft as **superseded for execution**, retaining it and its first architectural review in history. The live checkpoint is now `IN_PROGRESS / PARTIAL / UNVERIFIED`; `CHECKPOINT.md` is the lifecycle source of truth and records prior focused evidence separately from the current implementation increment. Commit `a7b75980` extends runtime product filtering and adds canonical-image test scaffolding, but remains unverified; `phalcom-semantic/src/canonical_image.rs` is not yet present, so no canonical formal image is claimed as implemented. This document remains a **PROPOSED specification**, not an approved plan amendment and not evidence that its proposed architecture is implemented. Reconcile proposed decisions through the repository consultation/amendment process before treating them as execution authority. The execution baseline is `99b84ecc`; the source audit revision above remains historical, so verify live symbols and mechanisms before each implementation slice.

## 19.2 Per-plan evidence contract

Each plan must have: task-by-task source ownership, explicit implementation sequence, feature/compatibility seam, unit/integration tests, canonical/provider/rollback parity, performance counter baseline, accepted counter target, commands with **selected nonzero tests**, failure ledger, docs updates and end-of-plan walkthrough. A clean predecessor comparison is mandatory before calling failures “baseline.” `cargo check` success alone is not certification. If full-suite failures are known, classify by same-state A/B rather than weakening tests.

## 19.3 Proposed milestone sequence

1. **Image foundation:** immutable canonical module and semantic images built once; core canonical compilation consumes semantic image without full duplicate analysis.
2. **Sound overlay:** fresh workspace reads immutable canonical query products, no canonical source/revision pollution, no query republication, cross-layer proofs correct.
3. **External linker:** local imports into Universe/package images are terminal leaves; exports/bindings retain exact identity; local components and runtime order exclude external modules.
4. **Fixed-floor collapse:** historical 40-source seed removed with differential semantic/runtime parity; unrequested canonical presentation/advisory/formal publication absent.
5. **Persistent roots + metadata closure:** no canonical map-copy floor or unconditional metadata global sweep.
6. **Saved semantic packages:** deterministic package identity and interface/query/generic artifact; dependency consumer compiles with no upstream source.
7. **Saved executable packages:** portable initializer/nested callable code, VM loader with validation/relocation, no source compilation.
8. **Saved projects and precompiled Universe:** lock/bundle graph, CLI build/run/inspect, cold process artifact load with fresh VM runtime heap and canonical bootstrap parity.

## 19.4 Hard prohibitions

- Never treat canonical image as local `last_snapshot` or populate local sources with canonical modules.
- Never use `opt-level=3` to conceal unnecessary work.
- Never collapse package publication, project workspace and VM instance into one ID.
- Never change coherence to local-first lookup or skip ambiguous/generic candidate heads.
- Never publish local-dependent proof results into immutable canonical cache.
- Never combine independent package TypeStores by bare numeric `TypeId` equality.
- Never persist VM `Value`, `ObjRef`, `Symbol`, `ClassId`, mutable cache cell or raw `Chunk` as executable bytes.
- Never report `CompiledProgram` source-text serialization as precompiled source-free execution.
- Never use implicit upstream source compilation as artifact loader fallback in accepted release profile.
- Never silently narrow RuntimePublic reflection to executable-call-only metadata.
- Never weaken diagnostics, runtime semantics, source ownership or accepted tests to hit a timing threshold.
- Never freeze/restore VM heap state as part of this compiled artifact program.
- Never finalize a persisted format before explicit ABI/schema/version/relocation validation exists.

---

# 20. Final Acceptance and Architecture Summary

The architecture must satisfy four independent propositions:

**(1) Immutable dependency ownership:** canonical and saved-package module, semantic and executable products are calculated or loaded once per matching world/artifact identity; source and query ownership remains with the correct layer. No consumer reconstructs entire canonical products simply because it crossed a crate or lifecycle API boundary.

**(2) Correct local semantics:** local source/imports, signatures, conformance candidates, witness selection, generic proofs, inference, static values, diagnostics and runtime lowering have exact parity with source-complete semantics. Immutable dependency base and mutable previous workspace revision are explicitly different concepts.

**(3) Real compiled artifact independence:** a package is a versioned, portable, deterministic and verifiable distribution unit with published module and semantic interfaces and optionally **complete source-free executable code**. A project is a reproducible dependency/deployment bundle of package artifacts, not a flattened package. Source-free execution loads verified portable code with VM-local relocation and initialization; it never triggers the VM source compiler.

**(4) Measured structural performance:** after warm canonical image construction, fresh one-shot compilation performs zero unnecessary canonical discovery, relinking, body analysis, witness reconstruction, editor/advisory publication and query republication. Later aggregate/metadata plans remove residual whole-world scans, and persisted artifacts eliminate cold-process semantic/code generation when compatible precompiled products are available.

**Architecture, in one sentence:** **Build immutable module → formal semantic → portable executable package images at their owning layers; expose them as validated dependencies; compile only workspace-local semantic/code deltas; and load persisted artifacts by stable identity, verification and relocation into fresh VM-owned runtime state.**

---

# Appendix A — Immediate Current-Revision Audit Checklist

Before source edits, search current repository for these symbols and record whether the documented behavior still exists:

```text
ProgramCompiler::compile_entry_selection
ProgramAnalyzer::analyze_entry_selection
ProgramCompilationSession::compile_fixture
WorkspaceModuleSession::apply_batch
WorkspaceSourceState / new_discovered_sources
ModuleResolver::load_interface
BuiltinInterfaceBuilder::build_from_parsed
UniverseSourceDeclarationCatalog
ModuleLinker::reachable_interfaces / link_component_interfaces_tolerant
compute_connected_components
CanonicalUniverseSemanticBaseline
canonical_universe_program / build_canonical_universe_program
SemanticWorkspaceSession::from_baseline / update
SemanticWorkspaceInput::new
SemanticDb::product / query_state / is_reusable / record_dependency
TypeStore::fork_from_baseline / StaticValueStore / StaticValueRef
ConformanceIndex::query_exact / overlap_conflicts
build_source_semantic_index / build_advisory_workspace
FormalSemanticProjection
MetadataExporter::build_bundle / MetadataProfile::RuntimePublic
CompiledModule / CompiledProgram / ModuleMaterializationPlan::empty
VM::run_compiled / compile_program_module_closure
VM::compile_closure_as_with_bindings
Chunk / Callable / ExecutableSemanticPool / Bytecode
VM::materialize_program / initialize_single_module
SemanticMetadataBundle / validate_metadata_bundle
ResolvedPackageArtifact / PackageArtifactProvider
PackageInfoDescriptor / PackageArtifactIdentity / StableIdentityContext
```

If a mechanism has changed since the pinned SHA, update its evidence classification and planned write set. Do not blindly apply an outdated patch or use nonexistent old-draft filenames.

# Appendix B — Glossary

| Term | Definition |
|---|---|
| Canonical module image | Immutable parsed/interface/import/linked/topology products for Universe or a chosen dependency module world |
| Formal semantic image | Immutable committed semantic facts, arenas and eligible query products for a dependency world |
| Query image | Read-only exportable query products/fingerprints with closed immutable dependency provenance |
| Workspace overlay | Mutable local modules, sources, queries, declarations and revision state depending on immutable images |
| External module leaf | Dependency module resolvable for imports/runtime without becoming a local linked/source component |
| Package artifact | Atomic published unit containing stable module/semantic interface and optional executable image |
| Project artifact | Root deployment/build description of a graph of package artifacts with entry/lock policy |
| Compiler ABI | Compatibility requirements for semantic schema, generated code recipes, bytecode, native semantics and target |
| Artifact content ID | Digest of deterministic serialized artifact bytes according to the defined hash domain |
| Semantic ABI | Exported semantic contracts on which downstream type checking and generic/evidence resolution depend |
| Portable executable | Versioned instructions, constants, semantic recipes and relocations requiring no source compilation |
| VM materialization | Allocation/binding of fresh runtime objects, symbols, metadata pool handles and closures from verified portable code |
| World assembly | Validated combination/relocation of multiple immutable dependency type/evidence domains into one compilation context |
| Golden/negative dependency | A semantic query observation that a requested candidate does not exist; requires invalidation if candidate set changes |
| Structural performance gate | Count-based assertion that an unwanted class of work is absent; distinct from wall-time targets |

# Appendix C — Source Index and Provenance

**Primary handoff:** `Pasted text.txt`, “Telos Analyzer and Compiler Performance — Unified Investigation Handoff, Architectural Resolution, and Implementation Program”, sections 1–81. The handoff's original historical 45-second measurements and 40/148/149 source inventories are **historical**, not a current run at the pinned source revision. The source-grounded table in §1 links directly to committed repository files and inspected functions.

**Earlier implementation artifacts:** `PERF003.C2-detailed-implementation-plan.md` and `CHECKPOINT.md` were reviewed as precursors; the monolithic task order and revision-ancestry shape are superseded by this specification. The separate first architectural review supplied additional safety/ownership objections. This document deliberately extends their in-memory image program with **new proposed** persisted package/project and VM-independent bytecode/loader architecture; those new requirements must not be misclassified as existing implementation.

**End of unified specification.**
