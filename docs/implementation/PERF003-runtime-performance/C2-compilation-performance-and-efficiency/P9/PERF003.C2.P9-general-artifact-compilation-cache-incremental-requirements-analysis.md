---
id: PERF003.C2.P9.REQUIREMENTS
program: PERF003
checkpoint: PERF003.C2
phase: PERF003.C2.P9
kind: architectural-requirements-analysis
status: PROPOSED
prepared_on: 2026-10-14
predecessors: [PERF003.C2.P7, PERF003.C2.P8]
source_basis: [PERF003.C2.P7-remaining-execution-plan.md, PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md, CHECKPOINT.md, PROGRAM.md, STATUS.md]
verification: DOCUMENT_REVIEW_ONLY
---

# PERF003.C2.P9 — General Compiler Artifacts, Deterministic Caching, Incremental Reuse, and Universe Distribution

**Requirements analysis and architectural decision proposal · 14 October 2026**

> **Evidence boundary.** This is a P9 **requirements analysis**, not an adopted implementation plan or a repository implementation audit. The five attached planning/lifecycle artifacts were reviewed. Concrete symbol/file names below are grounded in those documents; proposed new identifiers are explicitly labeled. No live repository checkout, source execution, timing benchmark, or CI result was obtained as part of this analysis. In particular, subsequent changes beyond the attached `3ca30fca` execution-tree baseline are not certified. The branch may have progressed further.

## 1. Executive architectural conclusion

**P9 should turn `.telosc` into the ordinary, general, versioned unit of compiler-authority reuse, with `universe.telosc` as the first high-value prebuilt package artifact.** It should not encode the current embedded Universe tree as a permanent compiler concept, should not create a second module compiler, and should not equate caching with loading an entire already-executed VM. General package compilation produces immutable portable compiler sections and, when requested, independently compatible executable sections; loading those sections creates fresh process/VM state; the cache maps fully described compilation inputs and dependency identities to validated immutable artifacts.

The critical optimization is **avoided work**, not merely faster serialization. A valid warm hit must avoid parsing, linking, semantic checking, witness solving, metadata export, lowering and bytecode emission for the unchanged dependency. A user-only update must retain the P5 immutable-parent/P6 indexed-local-work design. A cache miss may compile exactly the missing/invalidated dependency closure. All reuse is conditional on **compatibility, content provenance and dependency closure**, not filesystem timestamps or package names alone.

**Recommended P9 scope:** (a) general package/project artifact build and validated install/load; (b) content-addressed cache and safe automatic Universe discovery; (c) dependency-aware artifact DAG and incremental rebuild/reuse at meaningful boundaries; (d) compiler/CLI/build-tool/test-runner integration; (e) deterministic publication, diagnostics and cache lifecycle; (f) performance correctness and closure of the C2 objectives. Defer a remote registry, cross-host cache distribution, machine-native object emission, a daemon, and universal fine-grained cross-process query persistence unless separately authorized. Build interfaces should make those natural extensions without making them necessary for first completion.

### 1.1 Nonnegotiable semantic invariant

A cached compilation must be *observationally equivalent* to a clean compilation using the same language/compiler/Universe/dependency configuration. Cache policy is an optimization that cannot change lookup, overload resolution, conformance, generic inference, type/effect identity, method dispatch, metadata/reflection, diagnostics semantics, module initialization order, user-visible runtime values, or error behavior.

### 1.2 P9 is not P7 or P8 repeated

| Predecessor | Contract supplied | P9 must add | P9 must not redo |
|---|---|---|---|
| P5 | Immutable canonical publishing and zero-copy overlays; exact structural dependency evidence | Preserve immutable attachment under external artifacts | Copy parent products into local mutable tables |
| P6 | Owner-scoped backend indexes, lookup and bounded local complexity | Use the same indexed lookup over relocated artifact authority | Global scans of full imported world on each user compile |
| P7 | Bounded `.telosc` container; MODULE_WORLD, SEMANTIC_WORLD, QUERY_IMAGE, RuntimePublic METADATA; explicit build/load/injection | Generalize Universe-specific public path to package compiler artifacts; caching/selection | Reparse or semantically reconstruct imported Universe from source |
| P8 | PORTABLE_EXECUTABLE, portable initializer prototypes, native ABI compatibility, fresh VM bootstrap and later imports | Reuse executable dependency artifacts where compatible; production default path | Serialize live heaps, call native source provider on a hit, or create a second bytecode compiler |
| P9 | **Proposed:** packaging policy, graph-based cache resolution, incremental artifact build and distribution/read-only consumption | Productize the optimized default workflow | Weaken validation for trusted or local artifacts |

Source: P7 remaining plan §§Contract, T9–T12; P8 plan §§Architecture, T1–T8; C2 checkpoint §§1–3.

## 2. Baseline, predecessor gates, and trustworthy status

The checkpoint records a historical Universe-dominated ~45–47-second floor for tiny one-shot source programs, including 148 rebuilt interfaces, 40 injected Universe modules and 1,238 canonical callable bodies; its desired optimized ~500-byte-program target is **under 200 ms**. This is a target—not a measurement of `.telosc` startup. It also documents focused P3/P4/P5/P6 evidence, with P5 and P6 `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. P1 and P2 lifecycle records remain partial, so P9 must assess their relevant functional effects rather than infer whole-checkpoint completion.

The attached checkpoint places P7 `IN_PROGRESS / PARTIAL / UNVERIFIED`, with T1–T5 code partially present and remaining semantic families, static relocation, full writer/loader/injection and source-free acceptance not completed. The attached P8 plan places P8 `PLANNED / NOT_STARTED / UNVERIFIED`, with build entry conditional on completed P7 interfaces. The P7/P8 test-execution prohibition is explicit: compiler-only checks were authorized under that previous execution policy; source-free tests and runtime probes were deferred. **P9 cannot inherit a PASS from predecessor build checks.** P9 implementation cannot be called complete until P7/P8's source-free semantic and executable gates are actually exercised under an updated authorized verification policy.

### 2.1 Required P9 entry contracts

1. **P7 semantic image:** exported, validated, deterministic artifact; fresh TypeStore and StaticValueStore relocation; complete semantic primary products, checked attachments, immutable query/dependency authority, module world and canonical metadata; explicit `ProgramCompiler` loaded-artifact entry with source provider absent.
2. **P8 executable image:** complete initializer prototypes for eager and later imports, fully checked bytecode, independent executable/emitter/native compatibility, rooted fresh runtime allocation, native-floor source independence, and a shared loaded `Arc` for compiler and VM.
3. **P5/P6 behavior:** stable overlay lineage and exact dependency invalidation, owner indexes retained after import, no parent table flattening, no canonical query Ready rows inserted locally.
4. **Validated format authority:** `docs/specs/compiler-artifacts/telosc.md` and P7/P8 ABI constants are the normative wire source of truth. The cache may not substitute weaker structural validation for the loader.
5. **Explicit errors:** corrupted/missing/incompatible explicit artifact requests fail without silent source fallback; automatic mode may choose an authorized compile-on-miss policy and must report it.

P9 can design and implement modules incrementally before every behavioral predecessor gate finishes, but **C2 closure cannot be certified** while these prerequisites remain unproved.

## 3. The real unit of reuse: package artifact, not embedded Universe singleton

### 3.1 Terminology and ownership (proposed)

- **Source project:** editable root with a package manifest, source tree, toolchain and declared dependencies. The *project* owns a graph of source packages and target configurations.
- **Package identity:** stable logical identity (namespace/name and declared version/source-coordinate when applicable), distinct from filesystem location and separate from artifact content digest.
- **Compilation unit:** independently addressable package or smaller strongly connected compilation component whose external dependencies are finalized. Boundaries must follow existing module/link/semantic ownership, not arbitrary files.
- **Compiler artifact (`.telosc`):** immutable validated compilation product with canonical module and semantic authority, optional portable executable image, metadata, provenance and dependency manifest. One package artifact can depend on other `.telosc` artifacts.
- **Artifact instance:** a particular compilation of a package for a compiler/ABI/profile/dependency closure and content fingerprint. Multiple instances may share logical package identity.
- **Artifact store:** content-addressed bytes, safe publication and admission checks.
- **Artifact index/cache:** untrusted, rebuildable mapping from a normalized input key to an artifact digest (not itself semantic authority).
- **Loaded artifact graph:** immutable validated imported parent authorities, with a dedicated local compilation overlay and separate per-VM materialization.

**Important distinction:** logical package identity is not sufficient to identify an artifact revision; `universe` is a logical package, while `universe.telosc` is a selected *instance*. Treating a filename as canonical identity would silently reuse outdated Universe across branch, compiler, dependency or source changes.

### 3.2 Architectural owners and layering

```text
source project / package manifest + dependency resolution
                         │
                         ▼
package graph and source/ABI fingerprints (resolver owner)
                         │
           ┌─────────────┴───────────────┐
           ▼                             ▼
    cache resolver                 compile-on-miss
           │                             │
           ▼                             ▼
 validated artifact store   existing module → semantic → emitter pipeline
           │                             │
           └──────────────┬──────────────┘
                          ▼
     generic artifact loader + dependency graph linker
                          │
           immutable package compiler authorities
                          │
        ┌─────────────────┴─────────────────┐
        ▼                                   ▼
 user-local incremental overlay    fresh per-VM executable recipes
        │                                   │
 semantic/backend compilation       native/kernel setup + initializers
```

**Proposed ownership rule:** artifact crate owns stable wire DTOs, framed codec, budgets and content validation; modules/package resolver owns dependency/package topology; semantic owns import/export/relocation, exact query provenance and parent integration; core orchestrates build/load, native runtime and CLI; a dedicated cache/storage layer owns keys, atomic files, locking, eviction and diagnostics. Do **not** put network or filesystem policy inside SemanticDb, runtime execution, or wire DTO types.

The existing P7 `CanonicalExternalModuleWorld` can be generalized through an adapter, but should not simply be renamed or made to accept an arbitrary graph before validating assumptions about canonical Universe IDs, owner keys and bootstrap roles. Distinguish *canonical built-in authority* from ordinary dependency packages: Universe retains privileged native/primordial roles, whereas ordinary dependencies may be opaque compiler authorities without bootstrap privilege.

### 3.3 Composition and symbol identity

A package artifact must have stable package-local owner IDs, stable exported symbol identities, and references to **dependency artifact identities**, not process type IDs or physical paths. On load, form a graph of immutable package authorities and disjoint fresh-store namespaces. Any user-local type relation/lookup must see the same external canonical identities as the clean build. Handle aliasing, trait/conformance coherence and generic constraints across package edges deliberately; reject conflicting versions only according to the language's package/version identity model, not by accidental interner collision.

Package dependencies need a canonical, ordered manifest with declared edges, resolved artifact digest, interface/semantic compatibility digest and capability expectations. Detect cycles and record a valid compilation scheduling unit; never accept cyclic `.telosc` references merely because file references exist. If the current language permits module cycles **inside** a package, maintain that semantics within the artifact; cross-package cycles require an explicit SCC interface/finalization design rather than recursive loaders.

### 3.4 Immutable graph attachment complexity

Do not flatten every imported dependency into a single enormous cloned snapshot. Prefer layered indexed views, package-scoped selection and memoized, validated graph access. For `k` dependencies and `N` total canonical products, loading work can be proportional to artifact bytes/necessary relocation, but compiling a local delta should tend toward the affected local nodes plus their exact dependency edges, not `N × k` or full-world scans. Track cross-package owner conflicts once during graph composition. Retain `Arc` ownership of loaded sections and images across multiple sessions while preventing local mutation.

## 4. Future Universe migration to `packages/universe`

**Hard requirement:** design P9 around the *future* `packages/universe` package tree. The current embedded `phalcom-core/src/core/universe/` (or equivalent) is a transitional source-acquisition adapter, not part of artifact identity, required cache paths or public package ABI. Do not hardcode one physical directory, 40 source module list, 148 interface count, Git checkout root or Cargo workspace layout into production cache logic.

### 4.1 Source discovery through a provider-independent package model

Define a source package abstraction (proposed `PackageSource`, `SourcePackageGraph` or existing resolver extension), able to enumerate stable logical module URIs, package manifest, dependency declarations, source content digests and compilation profile. Implement **two adapters** as necessary: transitional embedded Universe corpus and future `packages/universe`. Both produce the *same logical package identity and public semantic products* when their contents are semantically equivalent; they need not produce identical artifact hashes if their source bytes or compilation inputs differ. Canonical artifact serialization already promises root-path independence; preserve that across adapters.

Do **not** force migrated filesystem URIs into semantic package identity. Source locations in diagnostics may use logical module URIs, with an optional environment-specific source map outside canonical hashed identities. Preserve the distinction between `packages/universe` as a source package, a published `universe` dependency, and a precompiled cache entry.

### 4.2 Transitional states and migration gate

| Stage | Input | Correct behavior |
|---|---|---|
| A — present | Embedded Universe source adapter | Build `universe.telosc` using ordinary package artifact builder; semantic/VM loading never requests embedded source after hit |
| B — dual-source migration | Embedded source and `packages/universe` exist | Explicit provider/profile selection; parity tests compare exported APIs, semantic products, code behavior and dependency fingerprints; no ambiguous automatic preference |
| C — migrated | `packages/universe` is authoritative | Build from package manifest and source resolver; cache discovery uses same logical Universe package identity; embedded adapter can be removed without changing artifact codec/loader |
| D — installed distribution | Only validated `universe.telosc` available | Compiler and VM function without either physical source tree; package source may be absent permanently |

Migration correctness cannot be established by content-hash equality alone because source representations may change while semantics remain compatible; use both clean-compile differential tests and exported interface/ABI comparisons. Native ABI anchors remain compiled-in and checked against the artifact; moving files must not change primordial identity or installation order.

## 5. General build pipeline and artifact capabilities

### 5.1 Proposed API shape (conceptual, not established current signatures)

```rust
// Names are architectural examples; preserve repository owners and existing error enums.
struct ArtifactBuildRequest {
    root_package: ResolvedPackage,
    target: ArtifactTarget,            // CompilerOnly | PortableExecutable
    dependencies: Vec<ResolvedArtifactDependency>,
    compilation_profile: CompilationProfile,
    reproducibility: ReproducibilityPolicy,
}

struct ArtifactBuildResult {
    bytes_or_staged_artifact: BuiltArtifact,
    identity: ArtifactIdentity,
    dependency_manifest: ArtifactDependencyManifest,
    statistics: ArtifactBuildStatistics,
}

fn compile_package_artifact(request: &ArtifactBuildRequest)
    -> Result<ArtifactBuildResult, ArtifactBuildError>;

fn load_package_artifact_graph(
    root: &ArtifactReference,
    store: &dyn ArtifactStore,
    limits: &ArtifactLoadLimits,
) -> Result<Arc<LoadedArtifactGraph>, ArtifactLoadError>;
```

Avoid making a public Rust caller pass redundant mutable stores or raw P7/P8 DTO maps. A compile pipeline consumes validated source graph + dependency compiler authorities; a loader consumes bytes and validates format/ABI/cross-package references; a VM consumes only executable-capable loaded authorities. Universe convenience APIs (`build_universe_telosc`, `load_universe_telosc`, `VM::try_new_with_universe_artifact`) should delegate to these general layers while retaining their stricter native-bootstrap semantics.

### 5.2 Capability profiles

Keep P7's `MODULE_WORLD`, `SEMANTIC_WORLD`, `QUERY_IMAGE`, `METADATA` and P8's `PORTABLE_EXECUTABLE` as independent, validated payload contracts. Compiler-only packages and full executable packages are different legitimate artifacts. Cache lookup must request **minimum required capabilities**, rather than always inflating everything to a runtime image. Runtime metadata may be required for execution/reflection but irrelevant to some semantic-only tasks. Do not invent another image format for incremental builds; add versioned additive sections only when cross-process reuse demonstrably requires them.

### 5.3 General package, Universe, and user compilation

- **Universe build:** source-backed *at build time only*, finalized semantic compiler authority, executable recipe capture in isolated build VM when requested, then deterministic publication.
- **Ordinary dependency build:** compile against validated parent package artifact graph. Do not install native/primordial Universe bootstrapping for arbitrary package artifacts. Produce public compiler interfaces and executable module prototypes as required.
- **Root user package:** compile sources against cached dependencies using the same module/semantic pipeline; save its `.telosc` only when requested or when build-system policy chooses to cache it.
- **Ephemeral inline/tests:** retain fast user-only overlay; avoid forcing serialization or cache disk writes for every tiny revision.
- **Compiler tooling/LSP:** import compiler-capable dependency authority without executing initializer bytecode; retain editable local source revisions and diagnostics.

## 6. Cache identity: content, dependencies and compatibility

### 6.1 Two distinct hashes are required

1. **Artifact content digest (`H_artifact`)**: P7's domain-separated BLAKE3 validation of framed artifact sections/whole artifact; addresses immutable serialized bytes and detects tampering/corruption. Existing masked-whole-digest semantics are preserved.
2. **Derivation/cache key (`K_build`)**: domain-separated digest of all compilation-relevant *inputs* and resolved dependency identities, computed **before** compilation. It answers whether an artifact can be reused. This is not the same as the output digest.

Suggested conceptual key:

```text
K_build = BLAKE3(
  "telos.artifact.build-key.v1" ||
  canonical_package_identity || canonical_source_manifest ||
  resolved_dependency_artifact_identities ||
  compiler_semantic_abi || artifact_schema_policy || target_profile ||
  applicable_language_flags || metadata_profile ||
  executable_emitter_abi_if_requested || native_abi_if_requested ||
  canonical_builtin_configuration || build_environment_contract
)
```

**Canonical source manifest** includes every source input and relevant generated input by logical package-relative URI and exact normalized bytes/content digest, with an explicit symlink/case policy. Exclude physical checkout root, modification time, transient inode, thread scheduling, wall clock and random hash-map iteration. Avoid hashing irrelevant user environment variables, but include any environment setting that can alter emitted semantics or code. Resource/file reads used by compile-time evaluation or generated code must either be explicitly tracked and fingerprinted or make the build uncacheable under the initial cache contract.

### 6.2 Compatibility matrix

| Dimension | Semantic/compiler load | Executable/VM load | Cache key effect |
|---|---|---|---|
| Container and section schemas | Mandatory supported schema | Mandatory | Include accepted writer policy; check reader separately |
| Stable owner/package identity ABI | Mandatory | Mandatory | Include canonical identity version |
| Compiler semantic ABI | Mandatory | Mandatory | Include compiler fingerprint/ABI |
| Query/product fingerprint rules | Mandatory for QUERY_IMAGE | Mandatory if compiler authority used | Include fingerprint-schema version |
| Metadata schema/RuntimePublic | Conditional to capability | Mandatory when used | Include metadata profile/schema |
| Bytecode/emitter ABI | Not required for compiler-only | Mandatory | Include for executable target |
| Native ABI and primordial layout | Not required for compiler-only | Mandatory | Include native registry digest |
| Language feature flags / target semantics | Mandatory if behavior-affecting | Mandatory | Include relevant flags |
| Host OS/architecture | Must not be intrinsically needed for portable `.telosc` | Same for portable bytecode if VM ABI permits | Include only if actual platform semantics enter output |
| User local source edits | Do not invalidate unchanged dependency artifacts | Do not invalidate unchanged dependency artifacts | Re-key local/root artifact only |

Version strings alone are too coarse for correct and efficient reuse: require explicit schema/semantic/emitter/native compatibility authority. Conversely, exact compiler binary hash is a conservative safe fallback for cache invalidation until narrower compatibility fingerprints are proven; do not claim cross-version reuse by comparing display versions only.

### 6.3 Dependency fingerprints and invalidation

A package artifact has at least three different digests: **source input**, **public interface/semantic ABI**, and **complete implementation artifact**. A dependency's private implementation change can leave downstream *semantic* products valid if its exported contract remains unchanged, but downstream executable linkage/inline templates/compile-time evaluation may still depend on implementation. Therefore record **typed dependency edges** (interface, body/inline implementation, metadata, executable/native, build-script input, etc.) and invalidate the smallest safe set; where precision is unimplemented, conservatively depend on the whole dependency artifact digest.

Never apply an interface-only optimization where checked static operational templates, macro/compile-time evaluation or link behavior depend on implementation. The P7 query image distinguishes reusable roots from dependency-only evidence and prohibits open-world negative proof persistence; P9 must preserve that fail-closed behavior across package graphs. Existing P5 exact dependency fingerprints should be the source of truth rather than another global hash oracle invented in the cache layer.

## 7. Incremental compilation and incremental artifact design

### 7.1 Three different increments

| Layer | Current/reusable foundation | P9 required behavior |
|---|---|---|
| **In-process source revision** | P5 canonical parent and P6 user-only indexes; SemanticWorkspaceSession transaction | Recompile affected user/local products only, exact dependency edges and rollback intact |
| **Across processes: package-level** | P7/P8 complete immutable `.telosc` | Reuse dependency package artifact wholly; compile changed package from sources and cached dependencies |
| **Across processes: finer-grained compiler products** | P7 QUERY_IMAGE plus dependency evidence | Plan extension only where complete, validated transport/owner stability exists; avoid prematurely freezing mutable local query DB state |

**Minimum P9 deliverable:** robust package-level incremental reuse, plus in-process granular incremental edits. A fine-grained *persisted per-query delta cache* is a potentially valuable future capability but is not required to land the major performance benefit; it is considerably harder because it needs stable query keys, epoch lineage, input tracking, dependency closure, exact invalidation and portable row payloads. If P9 is explicitly meant to finish cross-process incremental artifacts, include a narrow validated subset described below, not a claim to serialise the entire SemanticDb.

### 7.2 Proposed package incremental algorithm

1. Resolve the dependency DAG and normalize package inputs without invoking canonical Universe parser/semantic builder.
2. Resolve each dependency bottom-up: find digest by `K_build`; read/check content and compatibility; reject stale index entries; load and validate, or rebuild on cache miss according to policy.
3. For the editable/root project, attach validated dependency authorities as immutable parents. Preserve local incremental session state for repeated edits.
4. Form exactly the required local semantic products, preserving P5/P6 owner-local complexity; compile local executable changes only when requested.
5. Emit a new immutable `.telosc` on a durable-build boundary; publish atomically; update cache index only after successful content validation and fsync policy.
6. If a dependency public interface changes, invalidate downstream consumers that depend on it. If only implementation changes, use edge classes to prove safe reuse or conservatively rebuild.
7. If any cache load/validation/compatibility fails, quarantine/reject corrupt bytes and take **explicit** configured policy: fail in strict/offline/explicit mode; rebuild in automatic trusted-source mode.

### 7.3 Optional but architecturally prepared fine-grained persistence

Define a versioned `IncrementalProductIndex` **only after** a consumer census. An entry must have stable package/module owner, product kind, source epoch or input fingerprint, complete transitive dependency observations, portable product reference or reconstructible payload, compiler schema, and authoritative validation state. Never persist process-local `TypeStoreId`, `StaticValueStoreId`, session revision numbers, raw `QueryKey` allocated IDs, mutable DB Ready rows, raw ParsedModule units or heap refs. For each persisted query, P7's exhaustive `canonical_query_persistence(QueryKind)` is authoritative, and any supplemental cross-package query kinds require explicit addition plus negative dependency rules.

An incremental artifact should be a *fully validated immutable snapshot or immutable base+delta chain*. Prefer the former for first release: fixed-size/linear decode, bounded graph closure, trivial atomic installation, and no unbounded chain compaction. If incremental delta artifacts are introduced, require immutable base digest, deterministic delta canonicalization, bounded chain depth, transactional compaction, cross-section reference validation, garbage-collection reachability of bases, and immediate fallback to a full rebuild. **No delta may be made visible if any referenced base or dependency fails validation.**

### 7.4 Correctness pitfalls

- Cached negative lookup proofs become stale when newly added exports/impls make a previously missing symbol resolvable. Track absence against exact namespace/module/version evidence or avoid persisting it.
- Inferred-return settlement and conformance coherence must invalidate dependents transitively, not only directly named products.
- Generics and parameter versions require the P7 stable owner-slot mapping; cached references cannot assume same numeric `TypeStore` IDs on another process.
- Source editing while compilation is underway requires a snapshot of the source manifest and a double-check before publication; never cache bytes built from mixed file revisions.
- Build configuration changes must not preserve incompatible metadata, emission, overflow/check behavior or native ABI references.
- Package cycles, aliases, imported static values and checked templates need explicit closure and final validation before committing an artifact.

## 8. Artifact store and discovery policy

### 8.1 Store structure (suggested, not normative)

```text
<cache-root>/
  objects/blake3/<prefix>/<full-digest>.telosc  # immutable validated object
  indexes/v1/<build-key>                         # key -> digest/metadata
  locks/<build-key>.lock                          # per-key coordination
  tmp/<unique-staging-id>                         # same filesystem staging
  quarantine/<digest-or-ticket>                   # optional diagnostics
```

The exact path schema is private and changeable. Use content digests for data and build keys for lookup, and keep indexes rebuildable/disposable. Platform cache root selection follows the OS convention or explicit `TELOS_CACHE_DIR`-style policy (actual env var naming must be audited); avoid writing into the checkout by default unless there is an intentional project-local build directory. Install/distribution artifacts are separate from evictable transient cache entries.

### 8.2 Atomicity and concurrency

- Single writer per derivation key across processes (lock with abandoned-owner recovery); readers may concurrently read an existing immutable committed object.
- Stage to a unique same-filesystem temporary path, bounded encode, flush, validate bytes and references, fsync file and parent directory where promised, then atomic rename/commit. Publish index pointer **last**.
- Never hold locks while running user-defined compilation/static evaluation if that creates deadlock across dependency builds; define lock ordering and deduplicate build work without recursive lock inversion.
- Concurrent processes computing the same key may race safely: losing writer verifies the existing digest and discards its temp. Corrupt or incompatible committed objects never poison future attempts.
- Untrusted cache indexes, unexpected symlinks, path traversal, hardlinks, malicious length fields, denial-of-service graphs and partial rename/crash recovery are all part of the security model.
- Eviction must not delete objects needed by active readers, pinned installed artifacts or base+delta dependencies. Reference lifetime is explicit; atomic removal is decoupled from loaded `Arc` lifetime.

### 8.3 Discovery precedence and fallback contract

Recommend the following explicit precedence: (1) caller-provided artifact path/handle; (2) project manifest or lockfile-pinned artifact; (3) installed toolchain-bundled canonical Universe artifact with validated identity; (4) content-addressed local cache; (5) source build if policy permits and valid source exists; (6) typed failure. Resolve actual package identity and compile profile **before** selecting a candidate. A downloaded prebuilt artifact must not silently outrank an explicitly chosen local source override. An external Universe provider or development override must not reuse the built-in canonical artifact unless its provider identity and content are proven equal.

Offer cache modes conceptually equivalent to `auto`, `off`, `readonly`, `refresh`, `strict`/`artifact-only`, and explicit `--universe-artifact` behavior. Precise CLI syntax must be reconciled with P8's established flags. **Explicit means no fallback**; automatic may rebuild, and should explain cache miss reason under diagnostics. Source-unavailable mode must remain fully operational with validated installed `.telosc`.

### 8.4 Installed Universe distribution

A release/toolchain may ship a validated precompiled `universe.telosc` built for that exact compiler semantic/bytecode/native ABI. The executable is portable at the language VM level, not necessarily interchangeable across incompatible binary releases. Fresh installations should avoid a mandatory ~45-second first-launch source build; a toolchain release process should generate and test the image. A content-addressed cache remains useful for local development and custom Universe source builds. Release manifests pin artifact digest and ABI; no network is required for standard local compilation.

## 9. CLI, developer ergonomics and build/test integration

Proposed user-facing workflow (names shown as an **interface recommendation**, not claims of implemented commands):

```bash
# Canonical package root; toolchain-supplied universe cache can be reused.
phalcom artifact build --package packages/universe --output universe.telosc
phalcom artifact build --package packages/mylib --profile compiler
phalcom artifact inspect path/to/package.telosc
phalcom artifact verify path/to/package.telosc
phalcom artifact cache status
phalcom artifact cache prune

# P8 explicitly planned workflows remain supported.
phalcom artifact build-universe --output universe.telosc
phalcom --universe-artifact universe.telosc app.ph

# Normal transparent mode should resolve the toolchain/manifest/cache artifact.
phalcom app.ph
```

A normal `phalcom app.ph` should take the fast artifact path automatically where compatible; no extra flags should be required for the common case. Keep opt-out and force-rebuild available for correctness diagnosis. Build logs default to quiet, with structured diagnostics for `HIT`, `MISS`, `INCOMPATIBLE`, `INVALID`, `REBUILT`, `SOURCE_UNAVAILABLE` and why; never report a cache hit before full validation.

The test runner is a major beneficiary: compile/test fixture source as small overlays on one immutable Universe image per process; share loaded package graphs across test cases; avoid repeatedly rebuilding all Universe semantics. Do not over-share mutable user workspace state or runtime VM heaps across independently specified tests. For test harnesses needing isolation, share immutable artifact bytes and create fresh semantic overlays/VMs. Parallel tests must avoid global bootstrap locks that serialize all tests after the first immutable image load.

Compiler/LSP integration should retain developer source overlays and diagnostics; `CompilerOnly` capability suffices where VM execution is not required. Per-file edits should neither rewrite package artifacts nor invalidate unchanged dependency authorities. Project typechecking and test compilation should be able to reuse a background process-level loaded `Arc` without a daemon requirement.

## 10. Performance requirements and quantitative acceptance

### 10.1 Structural speedup budget

The existing checkpoint sets a **sub-200 ms optimized tiny-user-program compile target** and a preoptimization ~45–47 s baseline. P9 should adopt that as a conditional end-to-end target, **after** obtaining a reproducible release-mode hardware profile, rather than silently announcing attainment. Define separate targets for source-free image discovery + read + decode + semantic relocation, first user-only compilation, warm compilation and fresh-VM bootstrap. Loading a full executable image may cost more than compiling a tiny user overlay; measure end-to-end cold and warm separately.

Suggested non-negotiable structural counters on a valid cached Universe hit:

| Work family | Required count for cached Universe on user-only compile |
|---|---:|
| Universe source provider accesses / parsed modules | 0 |
| Universe interface/link rebuilds | 0 |
| Canonical declaration catalog construction | 0 |
| Canonical callable-body analysis/reanalysis | 0 |
| Canonical witness solving/conformance reconstruction | 0 |
| Canonical metadata exporter passes | 0 |
| Universe AST lowering / bytecode emission (executable hit) | 0 |
| Canonical aggregate entries copied into mutable local tables | 0 |
| Full Universe presentation/advisory rebuilds in runtime-minimal compile | 0 |
| User-source semantic/product work | proportional to local change and exact affected closure |

P7 loads do need bounded validation and relocation; they are **not** permitted to repeat source semantics. P8 VMs execute stored initializers once per fresh VM: **zero Universe compilation** is required, but **not zero runtime initialization**. Do not conflate semantic compile, artifact load, VM allocation and initializer execution times.

### 10.2 Required metrics

Instrument in `telos-observability` or its current owner: cache lookup/index/verification; object read bytes, mapped/decoded bytes, section validation, graph relocation and immutable composition; semantic/image attach; user-only analysis/backend; emitter work; VM recipe allocations; native-floor setup; initializer execution; cache miss/rebuild cause; package counts and reusable products. Record p50/p95 (and worst outlier) on pinned test hardware, plus peak RSS/allocations and disk size. Distinguish clean cold OS-page-cache, warm OS-page-cache, fresh process with reused artifact, warm retained process, local incremental update, unchanged rebuild, dependency interface change and implementation-only dependency change.

### 10.3 Avoiding hidden performance regressions

No unconditional artifact write on every ephemeral compilation, metadata re-export on hit, global type graph flattening, duplicate semantic/module import, repeated digest of unchanged byte buffers, repeated per-module O(N) scans, or full artifact graph reload when the loaded identity is already shared. Lazy on-demand section initialization may help cold-start latency but is only safe if full structural validation and publication semantics remain intact; consider memory mapping as an **optional optimization** after bounded decoding and lifetime rules are correct, not a P9 architectural premise. Bench both compressed and uncompressed section policies before adding compression support (P7 rejects unsupported compression).

## 11. Verification matrix and adverse-case coverage

All gates below are **proposed P9 gates**, not evidence of PASS. They supplement—not replace—P7 G7.0–G7.11 and P8 G1–G7.

| Gate | Acceptance proof | Representative fixtures |
|---|---|---|
| P9.G0 predecessor | Actual P7 source-free compiler + P8 executable/fresh-VM/late-import evidence | Cross-process no-provider process B |
| P9.G1 generality | Same generic compiler artifact builder works for Universe, ordinary library, app with transitive dependencies | 1 / 2 / 10 package DAGs; compiler-only/full |
| P9.G2 migration | Embedded adapter and future `packages/universe` provider produce equivalent semantics where inputs match; removal of embedded path has no wire/loader impact | Source-layout relocation, source-free installed test |
| P9.G3 identity | Fully described build keys; deterministic artifact bytes independent of root/path/alloc history | Two fresh hosts/processes with equivalent ABI/profile |
| P9.G4 cache correctness | Correct hits/misses, no stale reuse across semantic/emitter/native/metadata/schema/profile changes | Mutate flags, ABI, deps, provider, compiler version |
| P9.G5 incremental reuse | Unchanged dependency reused; interface/body-only changes invalidate appropriate consumers; exact open-world invalidation | Add export, add impl, change inferred return, static definition |
| P9.G6 startup | Cached source-unavailable user compiler works with no canonical source work and P5/P6 counters | Small, medium, large user modules; retained session |
| P9.G7 executable | One loaded graph compiles user module + initializes two independent VMs + later imports without source | Native/class/trait/closure/ADT/reflection/exception fixtures |
| P9.G8 corrupted artifact | Every malformed/stale/hostile object fails typed before authority publication or initializer side effects | Truncation, overflows, false section digest, forward refs, cycles |
| P9.G9 atomicity | Parallel writers/readers, crash interruption, stale lock recovery and safe retry | Multi-process barrier/fault injection |
| P9.G10 cache lifecycle | Offline, read-only, explicit strict, pruning/pinning semantics work | No sources, revoked object, symlink race, cache eviction |
| P9.G11 performance | Release-mode medians/p95 + cardinality; sub-200 ms checkpoint ambition measured, not inferred | Cold and warm independent processes; test harness |
| P9.G12 differential correctness | Source build vs artifact compiler/VM equivalence and diagnostics/provenance | Cross-package generics, coherence, reflection, late imports |
| P9.G13 downstream compatibility | CLI, test runner, LSP retain ordinary flags, errors and source override behavior | no-option, artifact-only, local override, migration |

### 11.1 Hostile format and security tests

Generate bound-checked malformed frames: huge declared lengths, duplicate tags, missing required sections, mismatched content hash, unsupported compression/flags/schema, semantically incompatible dependency digest, cyclic dependency manifest, dangling owner and type refs, invalid relocated static ADT, query product mismatch, opcode target/register/upvalue invalidity, incorrect native selector/arity, metadata graph overflow. Test memory and CPU budgets, not just error values. Fuzz the framed decoder and semantic/executable structural validator with seed corpus; place fuzz runs in separately authorized CI tiers.

### 11.2 Test isolation and source-free proof

A child-process test must deny *every* known source acquisition route: embedded provider, filesystem `packages/universe`, canonical singleton initialization and fallback provider. Process A builds and validates artifact, exits. Process B starts with unavailable sources, resolves/loads the pinned artifact, semantically compiles **novel** user code not present during build, initializes fresh VM, accesses both eager and non-eager imports, and checks zero canonical parse/interface/link/semantic/lowering counters. Run corresponding clean-source comparison with matching public metadata and observable outputs. A trivial `--help`/CLI parser test cannot establish source-free semantics.

### 11.3 Incremental testing protocol

Use initial build `A`, local-only change `B`, dependency implementation-only change `C`, public interface change `D`, missing-name-to-new-export change `E`, trait/conformance/new generic contract change `F`, and revert `G`; compare each cached/incremental result against a fresh clean build. Check diagnostics and signature/type identity, not only emitted bytes. A cache hit in `C` is valid only where dependency edge classes prove its safety. Test failed incremental updates and transactional rollback leaving prior immutable artifacts intact.

### 11.4 Test workload/cadence

Separate quick focused library tests, cross-crate compilation/codec tests, source-free child-process integration, executable parity, LSP/test-runner smoke, malicious-input/fuzz, and benchmark suites. Avoid running expensive ~45-second canonical source compilation for every small unit test; build a validated pinned test artifact once per compatible test process/suite or use a CI artifact fixture with format/ABI pinning. Nevertheless, regularly execute a source-built oracle to catch drift. Never treat a bundled fixture built by an old compiler as current compatibility proof.

## 12. Proposed implementation slices and dependency order

P9 should be a **bounded integration/optimization program**, not an indefinite rewrite of the incremental compiler. Suggested ordered slices (not a task-by-task final implementation plan):

| Slice | Dependencies | Deliverable | Key risk |
|---|---|---|---|
| P9.S0 — census & contracts | P7/P8 finalized APIs | Inventory package source resolution, normal entry paths, query/ABI inputs, all source-provider fallbacks; freeze capability matrix | Accidental Universe specificity |
| P9.S1 — general artifact producer | S0, P7 writer, P8 projection | `compile_package_artifact`, stable package+dependency manifest, compiler-only/full targets | Wrong artifact composition ownership |
| P9.S2 — loaded graph | S1, P7 loaded authority | Validated dependency graph with immutable parent interfaces, type/static/query relocation and collision policy | Duplicated world tables; identity misbinding |
| P9.S3 — derivation identity | S0–S2 | Complete build key, source content closure and compatibility invalidation | Hidden ambient inputs or stale cache hits |
| P9.S4 — atomic store | S3 | Immutable object store, index, locks, crash recovery, quarantine, pruning/pinning | Publication corruption and racing writes |
| P9.S5 — automatic Universe | S4, P8 VM API | Toolchain/install/cache discovery and default semantic/VM path, explicit precedence and opt-out | Hidden source fallback; source override collision |
| P9.S6 — incremental package builds | S2–S5 | Dependency-aware reusable package compiler authority; changed-root recompilation | Interface/body dependencies and open-world changes |
| P9.S7 — migration abstraction | S1/S5 | Dual adapter for embedded source and `packages/universe`; no path baked into identity | Native roles, module URI drift |
| P9.S8 — tooling and observability | S5–S7 | CLI/build/test/LSP integration, cache explain/inspect/verify, stage counters | Different entry points quietly bypass cache |
| P9.S9 — certification | all | Source-free + differential + adversarial + performance matrix and checkpoint closeout | Missing predecessor certification |

**Scheduling advice:** implement one complete end-to-end tracer early: deterministic generic package artifact → validated cache store → fresh-process cache hit → novel dependent user module semantic compile. Only then scale the graph and integrate default tooling. Follow with executable dependency loading and the automated Universe path, not a toy serialization demo disconnected from real owners. Preserve the P7/P8 contract that no partial authority is published. Distinguish a planning slice being coded from an acceptance gate passing.

## 13. Code-owner inventory and required audit before detailed P9 planning

The following are **references from attached artifacts**, not line-verified observations of the latest branch. A detailed P9 implementation plan must revisit live source and identify actual signatures and call paths.

| Owner / file or symbol named in attachments | P9 audit question |
|---|---|
| `phalcom-compiler-artifact/src/{query.rs,metadata.rs,semantic/,executable/}` | Container/DTO extensibility, stable schemas, shareable sections, bounded validation, imported package refs |
| `phalcom-semantic/src/{canonical_image.rs,snapshot.rs,artifact/,types/artifact.rs,static_value/}` | General loaded parent graph, fresh relocation, exact dependency/provenance, source-free compiler attachments |
| `phalcom-core/src/modules/{canonical_artifact.rs,canonical_universe.rs,compile.rs}` | Existing Universe-specific builders, loader, `ProgramCompiler` injection and source-backed `OnceLock` bypass |
| `phalcom-core/src/modules/{materialize.rs,builtin_materialize.rs}` | Runtime/module materialization and source provider coupling |
| `phalcom-core/src/vm/bootstrap.rs`, `native/install.rs`, `interpret.rs` | Native ABI/fresh-VM initialization/later imports and source fallback elimination |
| `phalcom-core/bin/phalcom/{main.rs,cli.rs}` | P8 commands, normal no-option compile path and default discovery seam |
| `phalcom-type-meta/src/{bundle,encode,validate}.rs` | RuntimePublic schema compatibility, source-free metadata import and bounded decoding |
| `phalcom-semantic/src/db/{key,product,fingerprint}.rs` | Stable query keys, replayable exact dependency evidence and no mutable canonical publication |
| `telos-observability` | Required stage counters and cardinality protections across automatic path |
| `docs/specs/compiler-artifacts/telosc.md` | Normative generic artifact manifest, ABI, capability and storage extension |
| Package source resolver, package manifest, test runner, LSP entry points | Discover exact code owners in live repository; **not established by attached plans** |

New cache/store directory, generic package builder APIs and keys named in this document are recommendations and need source-guided location selection. Do not reallocate existing P7/P8 owners to a new catch-all `artifact` module without justified encapsulation. The attached `PROGRAM.md` and `STATUS.md` still contain historical P5-state prose while `CHECKPOINT.md` reports P7: clean these lifecycle inconsistencies as records, **not** as evidence that P7/P8 actually completed.

## 14. Architectural decisions to fix before implementation

**D1 — Generality:** `.telosc` represents a versioned package compilation product; Universe is a privileged logical package using the same codec and compiler payload foundation. **Recommended: adopt.**

**D2 — Cache key:** separate derivation key from content-addressed output digest; use exact inputs, ABI and dependency manifests. **Recommended: adopt.**

**D3 — Cache trust:** all local/installed artifacts undergo common bounded loader validation; index hints are untrusted. **Required for correctness.**

**D4 — Dependency granularity:** first ship whole-package cross-process reuse plus P5/P6 local fine-grained incremental compilation; enable interface-vs-implementation invalidation only with proven typed edges. **Recommended to avoid an unfinishable P9.**

**D5 — Source precedence:** explicit artifact never falls back; automatic policy rebuilds from authorized source only on miss/rejection. Source provider overrides cannot be shadowed by canonical cache. **Required.**

**D6 — Universe relocation:** source root is a replaceable input provider; no physical embedded path is in wire identity or permanent cache keys. **Required.**

**D7 — Runtime state:** cached `.telosc` stores executable recipes, not executed heap/global state; P8 VM materializes fresh mutable state each time. **Already fixed by P8.**

**D8 — Delta artifact scope:** do not require a general persisted SemanticDb query-row/delta-log mechanism for P9's first complete result. Define later extension rules and a possible bounded subset. **Recommended.**

**D9 — Distributed cache:** defer remote fetching/registry/signing/attestation to a later program; keep stable digests and installed artifact manifest hooks. **Recommended.**

**D10 — Test policy:** predecessor compiler-only authorization does not satisfy new actual performance/behavior gates; seek explicit permission before executing tests/benchmarks in the implementation environment if restriction remains active. **Required for honest completion.**

## 15. Stop conditions and difficult open questions

1. **Semantic identity composition:** If TypeStore, static interning, conformance and canonical owner identity cannot be safely merged across package artifacts without rerunning semantic solving, halt graph-generalization and design the cross-package authority model first.
2. **Static/checked template dependencies:** P7 explicitly warns that checked static bodies currently retain syntax attachments. If package reuse would need source parsing or evaluator rechecking, do not omit the affected legal family; amend its portable checked-template contract.
3. **Complete input tracing:** If compiler behavior depends on files/environment/time/network not represented in `K_build`, mark build uncacheable or track all inputs; never approximate with `mtime`.
4. **Package boundary availability:** It is not proven from attachments alone that all ordinary packages currently have fully isolated cross-package semantic/link products. Audit actual package module resolution and cyclic imports before asserting general artifact support.
5. **Native authority separation:** Universe-native bootstrap roles are distinct from ordinary user packages; never let arbitrary packages claim primordial anchors through artifact metadata.
6. **Determinism vs historical IDs:** P7 preserves distinct historical parameter identities and ordered scoped unions; output canonicalization must not merge semantically distinct equal-looking nodes.
7. **Executable constants:** If legitimate emitter constants cannot be projected into P8 recipes without live heap capture, resolve at P8 owner before P9 stores arbitrary compiled packages.
8. **Incremental negative proof:** Cross-package missing symbol/trait candidates can become valid after dependency updates; invalidate exactly or conservatively rebuild.
9. **Memory/time tradeoff:** If importing all packages causes enormous eager relocation cost, evaluate validated lazy section access and sharing rather than converting to unsafe raw runtime snapshot.
10. **Migration schedule:** Do not make P9 depend on immediate `packages/universe` migration; require compatibility architecture now, and a separate migration validation gate when files actually move.

## 16. Definition of done and final end state

The intended finished C2 developer experience is:

```text
Toolchain build/distribution:
  sources for universe package (eventually packages/universe)
      → ordinary package compile + isolated executable recipe capture
      → deterministic validated universe.telosc
      → installed/bundled and/or content-addressed cached object

User's first compilation in a new process:
  resolve exact Universe/package identities
      → find compatible validated immutable .telosc graph
      → bounded decode/relocation + parent attachment
      → compile only user/local sources and affected products
      → run fresh VM initializer recipes when execution requested

Next compile or test in same process:
  share immutable artifact graph and cached dependencies
      → update only local exact dependency closure
      → no Universe source/semantic/executable reconstruction

After a dependency edit:
  re-key changed source package
      → reuse unaffected artifact graph
      → recompute only required downstream consumer work
      → atomically publish newly complete package image

On incompatible, missing, or corrupt artifact:
  typed reason + strict rejection or explicit automatic source rebuild
      → never silently accept stale semantics or source-fallback in strict mode
```

**P9 complete only when** generic package artifacts and automatic cache discovery are working on ordinary compilation/test/tooling paths; installed source-free Universe works from the future-proof package identity model; incremental reuse and invalidation preserve clean-build semantics; cache storage survives errors and concurrent writers; all P7/P8 source-free and fresh-VM gates pass; and release-mode timing/counter reports establish that the ~45-second canonical floor is removed with no compensating whole-world work. The original sub-200 ms tiny-compile ambition remains an explicit measured target; if not attained, document the measured residual bottleneck and do **not** claim that all C2 performance goals were met.

The correct success criterion is not merely **“we can write and read `universe.telosc`.”** It is **“the ordinary compiler, semantic engine, runtime and test infrastructure treat validated compiled dependency authority as the default reusable input—and compile only what has actually changed.”**

---

## Source grounding and qualifications

**User-provided primary sources:** `PERF003.C2.P7-remaining-execution-plan.md` (notably contract, waves and T9–T15); `PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md` (architecture, executable schema, T1–T10 and gates); `CHECKPOINT.md` (goal/targets, P3–P8 ledger, P5/P6 facts, P7 progression, P7/P8 research handoff); `PROGRAM.md`; `STATUS.md` (older lifecycle summaries). The proposed generalized store, incremental package DAG, cache key, discovery precedence, migration adapters, suggested CLI and P9 gates are **architectural recommendations, not statements that these interfaces exist today**. Reconcile this analysis against live repository state and actual package source/manifest owners before converting it into a patch-grade implementation plan.
