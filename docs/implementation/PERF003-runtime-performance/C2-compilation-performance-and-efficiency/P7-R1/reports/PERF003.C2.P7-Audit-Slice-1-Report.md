# PERF003.C2.P7 — Audit Slice 1 Report

**Audit date:** 2026-10-15  
**Audited branch:** `perf/universe-characterization`  
**Remote HEAD audited:** `2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a` — `feat(perf): extend artifact loading foundations`  
**Repository:** `aureat/phalcom-lang`  
**Plan:** `PERF003.C2.P7` — persisted semantic compiler image (`universe.telosc`)  
**Audit slice:** 1 of a multi-turn comprehensive audit  
**Status after this slice:** **P7 remains IN_PROGRESS / PARTIAL / UNVERIFIED. Closure is not currently reachable without substantial semantic-owner and loader work.**

---

## 1. Executive verdict

The current P7 branch has made real progress on the mechanics required for a portable compiler image: bounded container parsing, compatibility identities, native-catalog fingerprinting, type/static export infrastructure, partial semantic DTOs, partial callable-analysis transport, frozen/budgeted CFG storage, query-image infrastructure, atomic writer plumbing, image-aware compiler entry points, and source-free external-module resolution seams. Those are useful foundations.

However, **the branch does not yet contain a usable canonical Universe artifact pipeline**. The live implementation has two unconditional hard stops:

1. the writer rejects every real Universe snapshot before `SEMANTIC_WORLD` can be emitted because non-empty `CallableAnalysis` and several other required primary families remain unsupported; and
2. the budget-aware semantic importer unconditionally returns `AllocationError::UnaccountedBoundary` before semantic validation/materialization begins.

More importantly, these guards are **not merely stale blockers**. The code behind them is still semantically unsafe for P7 closure. If the guards were deleted today, the importer would fabricate or erase compiler authority in several places: callable definitions are synthesized as primary declarations, dispatch callable knowledge is replaced with assumed `TypeId(0)`, trait source/visibility/domain facts are defaulted, witness-plan failure and conditional-selection evidence is collapsed, callable analyses are discarded, derived conformance and trait-dispatch indexes are empty, static-case/static-definition/static-constant/module-binding owners are empty, and the formal/compiler attachment structures are defaulted. The current `SemanticWorldV2` schema also does not carry all families that the persistence manifest declares as `Wire`.

Therefore the principal conclusion of Slice 1 is:

> **Do not remove the fail-closed writer/importer guards to make the artifact tests green. The guards are currently protecting correctness. P7 closure requires completing the owner-authoritative wire contracts, exact reconstruction, allocation accounting, and end-to-end acceptance underneath those guards.**

The current closure objective stated by the user — compile the Universe once, persist it, load it to accelerate ordinary compilation, Universe work, and workspace testing, and support a standalone package exercising real Telos features such as `parse`, `similarity`, and `slotmap` — is **architecturally plausible with the existing branch direction**, but the implementation is still materially short of that end state.

### Severity summary

| ID | Severity | Finding | Closure impact |
|---|---|---|---|
| S1-01 | **CRITICAL** | Canonical artifact construction is deliberately impossible for a real Universe snapshot | No `universe.telosc` can currently be built |
| S1-02 | **CRITICAL** | Budget-aware semantic loading is unconditionally disabled | No semantically loaded artifact can currently succeed |
| S1-03 | **CRITICAL** | Importer behind the guards fabricates/drops required semantic authority | Removing guards would produce an incorrect compiler image |
| S1-04 | **HIGH** | Persistence manifest and live wire schema materially disagree | Coverage bookkeeping can claim contracts the format cannot encode |
| S1-05 | **HIGH** | Source-free completeness predicate is cardinality-oriented rather than parity-oriented | It is insufficient as the final correctness gate and is Universe-specific |
| S1-06 | **HIGH** | Core loader re-stringifies typed allocation failures | Error handling can allocate precisely when the allocation ledger is exhausted |
| S1-07 | **HIGH** | Required static/derived/compiler owners are detached, defaulted, or never installed | Real lowering/static evaluation/trait resolution cannot rely on the loaded image |
| S1-08 | **HIGH** | Most end-to-end artifact tests are blocked in fixture setup; HEAD has no CI evidence | Test presence substantially overstates currently exercised behavior |
| S1-09 | **MEDIUM** | Writer clones all four encoded sections before container assembly | Avoidable peak memory and artifact-build overhead |
| S1-10 | **MEDIUM** | Atomic writer is coupled to the full loader and hard-coded default validation limits | Writer cannot succeed today and may reject artifacts built under valid custom limits |
| S1-11 | **HIGH** | Loaded-image package APIs exist, but standalone-package acceptance is absent | User-defined P7 closure behavior (`parse`/`similarity`/`slotmap`) is unproven |
| S1-12 | **MEDIUM** | Lifecycle records are stale relative to remote HEAD; generated rustc ICE file is committed; branch is 7 commits behind `main` | Takeover/integration risk and repository hygiene debt |

---

## 2. Audit scope and method

This slice intentionally audits the **artifact critical path**, not every one of the 51 branch commits. The branch currently diverges from `main` by **51 commits ahead / 7 commits behind**, so a broad historical diff would dilute the closure analysis with already-established P1–P6 and unrelated performance work.

The live path inspected in this slice is:

```text
CanonicalUniverseFormalImage / CanonicalCompilerImageView
        |
        v
build_universe_telosc
        |
        +--> MODULE_WORLD
        +--> TypeExportContext / StaticExportContext
        +--> export_semantic_world      <-- CURRENT HARD STOP #1
        +--> QUERY_IMAGE
        +--> METADATA
        v
write_universe_telosc
        |
        +--> load_universe_telosc_from_bytes (self-validation)
        v
load_universe_telosc[_from_bytes]
        |
        +--> container + reader compatibility
        +--> module world
        +--> SemanticWorldV2 decode
        +--> validate_and_import_semantic_world_with_allocation_budget
                        ^
                        +-- CURRENT HARD STOP #2
        +--> query reconstruction
        +--> metadata
        +--> CanonicalUniverseFormalImage::from_loaded_artifact
        v
ProgramCompiler::compile_entry_selection_with_image
        |
        +--> Project / Package / Module / Inline user overlay
```

Primary files inspected:

- `phalcom-core/src/modules/canonical_artifact.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/tests/core/modules/canonical_artifact.rs`
- `phalcom-semantic/src/artifact/export.rs`
- `phalcom-semantic/src/artifact/import.rs`
- `phalcom-semantic/src/artifact/manifest.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-compiler-artifact/src/semantic/world.rs`
- current P7 `CHECKPOINT.md`, handoff and walkthrough at remote HEAD

The supplied planning records were used as the acceptance contract and historical evidence, but every substantive closure conclusion below was checked against the live branch implementation at `2939e9d4`.

No GitHub combined-status checks or PR-triggered workflow runs are recorded for the audited remote HEAD. Accordingly, focused commands recorded in project documentation are evidence for those exact revisions/commands only; there is no independent remote CI certification for the current push.

---

# 3. Detailed findings

## S1-01 — CRITICAL — Real Universe artifact construction is still structurally impossible

### Evidence

`phalcom-semantic/src/artifact/export.rs:2186-2238` contains the current coverage gate:

```rust
fn validate_semantic_world_export_coverage(snapshot: &SemanticSnapshot) -> Result<(), ArtifactError> {
    ...
    if !snapshot.callable_analyses.is_empty() {
        return Err(referential(
            "semantic artifact export does not yet encode complete callable analysis facts and cannot publish a complete image",
        ));
    }
    if !snapshot.static_case_catalogs.is_empty()
        || !snapshot.static_case_signatures.is_empty()
        || snapshot.static_definitions.iter().next().is_some()
        || !snapshot.static_constants.is_empty()
        || snapshot.module_static_bindings.iter().next().is_some()
        || !snapshot.derived_conformance_rules.is_empty()
        || !snapshot.trait_dispatch_index.is_empty()
    {
        return Err(referential(
            "semantic artifact export omits required primary owner families and cannot publish a complete image",
        ));
    }
    ...
}
```

The gate also rejects legitimate witness plans containing fields the current DTO cannot preserve, including requirement views, invalid explicit members, non-`Complete` completeness states, associated-type failures, conditional inherent selections, and applicability evidence.

`build_universe_telosc` invokes `export_semantic_world` after module-world encoding and type/static graph-root closure (`phalcom-core/src/modules/canonical_artifact.rs:128-245`). Therefore every real canonical Universe snapshot with callable analyses stops here.

The current P7 walkthrough records exactly this observed result: the canonical integration test reaches the builder and fails at the callable-analysis export coverage guard. That means the failure is not speculative or a dead branch; it is the live end-to-end behavior.

### Impact

This alone prevents the headline P7 deliverable:

- no complete `universe.telosc` bytes;
- no deterministic real-World artifact comparison;
- no successful atomic publication;
- no cold-process load;
- no source-denied compilation;
- no measured artifact-load speedup;
- no package execution against a loaded image.

### Audit judgment

**The guard is correct and must stay.** It is exposing genuine format/owner incompleteness rather than obstructing a complete implementation.

### Required correction

Complete the semantic owner inventory *before* weakening this guard. The minimum missing families include:

- full `CallableAnalysis` owner graph and compiler projection;
- real accepted callable definitions, not inferred copies from signatures;
- static-case catalogs and signatures;
- static definitions and constants installed into their actual semantic owners;
- module static bindings;
- derived conformance rules;
- trait dispatch authority or an exact, proven derivation from transported primary data;
- lossless witness-plan views/failure/completeness/conditional evidence;
- every type/static root reachable from those rows.

Only remove an individual coverage clause when a source-built/imported parity discriminator for that exact owner family passes.

---

## S1-02 — CRITICAL — The semantic loader is unconditionally disabled

### Evidence

`phalcom-semantic/src/artifact/import.rs:1798-1819`:

```rust
pub fn validate_and_import_semantic_world_with_allocation_budget(
    world: SemanticWorldV2,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    // TypeStore relocation/interning and the primary/static owner imports do
    // not yet expose every allocator request to this transaction. Reject
    // before validation or materialization can allocate outside its ledger.
    require_semantic_import_allocation_accounting()?;
    ...
}

fn require_semantic_import_allocation_accounting() -> Result<(), ArtifactError> {
    Err(AllocationError::UnaccountedBoundary.into())
}
```

This is called directly by the core loader at `phalcom-core/src/modules/canonical_artifact.rs:346-349`.

### Impact

Even a manually produced or future corrected semantic section cannot currently be loaded through the supported budget-aware P7 path. Consequently:

- `load_universe_telosc_from_bytes` cannot construct a loaded canonical semantic image;
- `load_universe_telosc` cannot succeed from disk;
- `write_universe_telosc` cannot self-validate a valid artifact, because it calls the loader before publication;
- image-aware package/compiler APIs cannot be reached from persisted bytes.

### Why the guard exists

The comment accurately identifies remaining unaccounted allocations. The inner path calls APIs such as `TypeStore::import_type_graph` and multiple primary/static owner constructors without threading the shared load ledger through every requested allocation. Therefore simply deleting `require_semantic_import_allocation_accounting()` would violate the adopted R1/R7 contract.

### Required correction

Move the allocation boundary downward rather than deleting it:

1. make type graph reconstruction fallible and budget-aware;
2. make structural primary reconstruction budget-aware;
3. make static-value reconstruction budget-aware end-to-end;
4. account owned strings, maps/vectors, `Arc`/`Box`, relocation maps, index builders, scratch, and publication wrappers;
5. ensure child-ledger commit happens only after a complete stage succeeds;
6. independently compare ledger debits to allocator-requested bytes for a real Universe load;
7. verify `N` exact fit, `N-1` rejection, parent rollback, and retry.

Only after those pass should `UnaccountedBoundary` be removed.

---

## S1-03 — CRITICAL — The importer behind the guards is not semantically safe to publish

This is the most important finding of Slice 1. The remaining work is not merely “finish serialization.” The existing inner importer contains deliberate placeholders/defaults that would corrupt semantic authority if publication were enabled.

### 3.1 Accepted callable definitions are fabricated from signatures

`phalcom-semantic/src/artifact/import.rs:1858-1885` reconstructs every signature and synthesizes:

```rust
EffectiveCallableDefinition {
    callable: callable.clone(),
    origin: CallableDefinitionOrigin::PrimaryDeclaration,
    source_member_index: 0,
    signature: sig.clone(),
    inherent_body_context: None,
}
```

This loses the real accepted-definition origin, member index, provenance, and inherent body context.

A `CallableSignature` does not imply `PrimaryDeclaration`. Definitions can be accepted through different semantic paths, and the compiler consumes definition provenance. This is a primary-authority violation, not a cosmetic metadata loss.

### 3.2 Callable analyses are discarded entirely

Immediately afterward, the importer creates an empty `CallableAnalysisTable`. The existing partial DTOs — flow graph, source owner, binding origins, denotations, typed call arguments — are therefore not integrated into the actual snapshot.

### 3.3 Dispatch surface callable knowledge is replaced with `TypeId(0)`

`phalcom-semantic/src/artifact/import.rs:1980-1994` inserts both instance and class callables using:

```rust
TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::Flow)
```

This is a particularly dangerous placeholder. It gives a valid-looking type-evidence object backed by an arbitrary relocated store coordinate. `TypeId(0)` is not a portable semantic statement and cannot stand in for each callable's actual receiver/result knowledge.

If the coverage guards were removed, dispatch/type-checking behavior could silently diverge rather than simply fail.

### 3.4 Trait surfaces/headers default semantic fields

The importer currently assigns or erases fields such as:

- associated-type domain => `KindId(0)`;
- source spans => default range/module-derived placeholder;
- member visibility => always `Public`;
- `default_source` => `None`;
- trait header source => synthesized default span.

Those substitutions contradict the owner-preserving artifact design. They also make imported/source-built diagnostic and generic-resolution behavior non-equivalent.

### 3.5 Witness-plan semantics are collapsed

The current DTO/import path reconstructs several completeness tags with invented generic payloads:

```text
Incomplete        -> failures = []
Unknown           -> NoTypeEvidence
Blocked           -> RecursiveFixpoint
Dynamic           -> reason "dynamic"
BudgetExceeded    -> BudgetReport(Steps, 0, 0)
InternalFailure   -> "restored from artifact"
```

It also forces:

```text
requirement_views = {}
invalid_explicit_members = false
conditional_impl = None
applicability = None
associated-plan failures = omitted
plan diagnostics = []
```

This is not parity-preserving reconstruction. Fortunately the writer currently rejects many of these cases, but that only proves the guard is necessary.

### 3.6 Multiple required snapshot owners remain explicitly empty/default

The eventual snapshot assembly currently uses empty/default values for:

- `derived_conformance_rules`;
- `trait_dispatch_index`;
- `semantic_graph`;
- `formal_projection`;
- `source_index`;
- `module_products`;
- `static_case_catalogs`;
- `static_case_signatures`;
- `static_definitions`;
- `static_constants`;
- `module_static_bindings`;
- callable analyses.

Several of these are mandatory for the branch's own `has_source_free_artifact_contents()` gate, so the inner importer cannot currently promote a realistic snapshot even after the allocation guard is removed.

### Required correction

Treat every placeholder above as a closure blocker. The importer needs owner-specific adapters that preserve source-built fields modulo relocated IDs. A safe rule is:

> **If the source snapshot contains semantic information consumed by compilation, the loader must either transport it losslessly or reconstruct it from explicitly transported canonical primary authority with a proved equivalence. “Populate something structurally plausible” is not acceptable.**

Tests must compare real source-built and imported owners field-by-field/behavior-by-behavior, not merely assert non-empty tables.

---

## S1-04 — HIGH — The persistence manifest overstates the live wire contract

### Evidence

`phalcom-semantic/src/artifact/manifest.rs` declares the callable-analysis fields below as `Wire`:

- expressions;
- bindings;
- for-iteration lanes;
- positional expansions;
- known-invalid positional expansions;
- sequence literals;
- associated resolutions;
- callable-reference resolutions;
- family applications;
- match resolutions;
- pattern-site resolutions;
- flow graph;
- entry flow;
- exits;
- return validation;
- diagnostics;
- explanations and return explanation;
- dependencies and semantic dependencies;
- constructor-super initializer;
- status.

But `phalcom-compiler-artifact/src/semantic/world.rs:298-312` defines the current `CallableAnalysisRecordV1` as only:

```rust
record!(CallableAnalysisRecordV1 {
    callable: StableCallableRef,
    body_source_owner: Option<CallableAnalysisSourceOwnerV1>,
    body_range: SpanV1,
    status: CallableAnalysisStatusV1,
    exits_unreachable: bool,
    flow_graph: CallableFlowGraphV1,
    constructor_super_initializer: Option<ConstructorSuperInitializerV1>,
    expression_count: u32,
    expression_denotations: Vec<CallableExpressionDenotationV1>,
    binding_denotations: Vec<CallableBindingDenotationV1>,
    call_specializations: Vec<CallableCallSpecializationV1>,
    formal_bindings: Vec<CallableBindingSourceAttachmentV1>,
    binding_origins: Vec<CallableBindingOriginEntryV1>,
});
```

The same mismatch exists at the snapshot-family level. The manifest marks families such as callable definitions, derived conformance rules, static case catalogs/signatures, static definitions/constants, and module static bindings as wire authority, while `SemanticWorldV2` does not yet carry all of them as complete owner records.

### Impact

The manifest currently behaves as a design target, not an executable coverage proof. That is dangerous because future reviewers can read `Wire` and infer completion even though the schema cannot encode the field.

### Required correction

Introduce a mechanical relationship between the manifest and the codecs. Options include:

1. per-family compile-time/test assertions that every `Wire` manifest row has an exporter, DTO and importer adapter registered;
2. an explicit implementation-status column distinct from architectural disposition (`RequiredWire` vs `ImplementedWire`);
3. a canonical field census test against `CallableAnalysis` and `SemanticSnapshot` that fails whenever a required field lacks a completed disposition.

The important requirement is that a manifest row cannot be marked operationally complete merely because the intended final architecture says it should be wire-backed.

---

## S1-05 — HIGH — The final source-free completeness predicate is too weak for correctness and too Universe-specific for general artifact use

### Evidence

`phalcom-semantic/src/snapshot.rs:777-822` checks source-free contents primarily through non-emptiness and broad status conditions. Examples:

```rust
let has_callable_body_facts = self.callable_analyses.values().any(|analysis| {
    !analysis.expressions.is_empty()
        || !analysis.bindings.is_empty()
        || !analysis.associated_resolutions.is_empty()
        ...
});

self.sources.is_empty()
    && self.presentation_sources.is_empty()
    && !self.callable_analyses.is_empty()
    && callable_analyses_are_complete
    && has_callable_body_facts
    && !self.callable_signatures.is_empty()
    && !self.callable_definitions.is_empty()
    ...
```

### Problems

#### A. It proves population, not parity

Only **one** callable analysis needs body facts for `has_callable_body_facts`. The predicate does not prove that every analysis has every required owner field, that binding/expression references are closed, or that compiler attachments agree with the source-built image.

Similarly, non-empty witness/static/trait tables do not prove correct membership or exact owner relationships.

#### B. It can be fooled by fabricated rows

A table filled with semantically wrong but non-empty rows can pass this class of predicate. This is precisely why the current `TypeId(0)` and synthesized callable definitions are high-risk.

#### C. It is not a generic artifact completeness contract

The predicate requires many Universe-specific families to be non-empty (e.g. enums, aliases, traits, static cases, conformance structures). A perfectly valid standalone package may legitimately have no enum, no trait, no static constant, etc. Such a package should not be considered “incomplete” merely because a category is empty.

This matters for the user's stated end state: P7 should lead to a system that can run a standalone package and, later, general package artifacts rather than encoding “Universe cardinalities” as semantic validity.

### Required correction

Split the concept into two layers:

1. **Artifact structural completeness:** every required family has an explicit disposition and every persisted/derived owner relation validates, regardless of cardinality.
2. **Canonical Universe acceptance profile:** expected Universe-specific invariants/cardinalities may be checked as a separate regression profile.

For callable analyses, completeness should be per-owner and field-manifest driven, not “at least one analysis has facts.”

---

## S1-06 — HIGH — Core error wrapping can reintroduce allocations on allocation-exhaustion paths

### Evidence

The semantic layer recently added a typed allocation error specifically to avoid formatting a `String` after the budget is exhausted. However, the core loader maps many lower-level failures like this:

```rust
.map_err(|e| {
    TeloscArtifactError::CanonicalImage(
        CanonicalImageError::IncompletePublication(e.to_string())
    )
})?
```

This pattern is used around module decode/materialization, semantic decode/import and metadata decode in `phalcom-core/src/modules/canonical_artifact.rs:320-386`.

There is a separate typed `allocation_error(...)` helper, but it is used only for selected explicit allocation helpers such as publication `Arc`s.

### Impact

On `ArtifactError::Allocation(Exhausted|Failed|Overflow|UnaccountedBoundary)`, `e.to_string()` may itself allocate. This undermines two current P7 goals:

- a load budget should bound every requested owned allocation; and
- budget-exhaustion reporting itself must not create an untracked allocation.

It can also destroy typed error classification, making tests distinguish only “incomplete publication” rather than the actual allocation cause.

### Required correction

Add typed propagation from lower artifact/module/semantic errors into `TeloscArtifactError`. Preserve `ArtifactError::Allocation` directly and only allocate diagnostic strings where the allocation contract permits it. Then add an independent allocator test at the **core loader boundary**, not only semantic helper tests.

A useful discriminator:

```text
Given zero remaining artifact budget,
when SemanticWorld import returns ArtifactError::Allocation(...),
then load_universe_telosc_from_bytes returns the corresponding typed error
and global allocator requested bytes do not increase while constructing the error.
```

---

## S1-07 — HIGH — Required compiler owners are still detached or absent even where DTO work exists

Several implemented pieces currently stop one layer short of the actual semantic owner used by the compiler.

### Examples

#### Operational templates

`import_checked_static_definitions` produces `operational_templates`, and `ValidatedSemanticParts` carries them separately. But the current snapshot is built with an empty `StaticDefinitionTable`, empty `StaticConstantTable`, empty static-case tables and empty module-static-binding catalog.

A detached decoded template is not equivalent to a restored semantic owner.

#### Reconstructed owner indexes

`ReconstructedModuleOwnerIndexes` is returned separately from the snapshot and is keyed by `String`. The importer constructs keys using:

```rust
format!("{:?}", module)
```

This has three problems:

1. it allocates unbudgeted strings;
2. debug formatting is not an appropriate canonical identity representation;
3. the maps are detached from the P6 module-indexed structures that actual consumers read.

The final image needs P6's real typed owner indexes installed or rebuilt through their owning APIs, not a parallel debug-key catalog.

#### Formal projection/compiler attachments

The imported snapshot currently installs `FormalSemanticProjection::default()`. Yet lowering/compiler APIs explicitly read binding/site/target facts. These facts must be reconstructed from the transported callable owner graph or an equivalent canonical attachment, as the adopted P7 amendment already requires.

### Required correction

For every detached return value in `ValidatedSemanticParts`, answer:

- Which real compiler owner consumes this authority?
- Is the data installed there before `SnapshotStatus::Complete`?
- Does the consumer execute against the restored owner in a source-denied test?

If the answer is no, the family remains incomplete regardless of codec coverage.

---

## S1-08 — HIGH — End-to-end test coverage is largely blocked before the behavior it names

### Evidence

The core test file contains good-looking tests for:

- build/load/source-free user compilation;
- corruption rejection;
- deterministic rebuilding;
- atomic writer behavior;
- load budgets/timings;
- cumulative transaction budget;
- compatibility header rejection;
- etc.

But many begin by calling `build_universe_telosc(&view, ...)`. The canonical build currently fails at the callable-analysis export coverage guard, so these tests cannot reach the downstream behavior named by the test.

The current walkthrough explicitly records the primary end-to-end test as failing during artifact construction.

Some useful tests do bypass the builder and exercise local behavior independently, including synthetic foreign-native-catalog rejection, oversized-file acquisition, and invalid-byte output preservation. These are valid focused tests, but they do not prove the real artifact path.

### Remote verification gap

For audited HEAD `2939e9d4`, GitHub reports:

- no combined commit statuses;
- no associated PR-triggered workflow runs.

Therefore there is no remote automation backstop for the current branch tip.

### Required correction

Classify tests by the deepest phase they actually reach:

| Phase | Required evidence |
|---|---|
| Writer projection | real canonical Universe reaches completed section assembly |
| Byte validation | complete bytes are reparsed and cross-section validated |
| Semantic materialization | fresh stores/owners reconstructed |
| Query reconstruction | real roots/edges/fingerprints reconstructed |
| Source-denied compiler | child process compiles novel program/import with Universe source access forbidden |
| Package closure | standalone package using real supported features compiles/runs against loaded image |

The project records should report “blocked at build” for tests that never execute their intended loader assertion.

---

## S1-09 — MEDIUM — Writer doubles section-buffer ownership unnecessarily

### Evidence

`build_universe_telosc` encodes four complete section vectors, then passes clones to `TeloscContainerBuilder`:

```rust
builder.add_section(..., module_world_bytes.clone());
builder.add_section(..., semantic_world_bytes.clone());
builder.add_section(..., query_image_bytes.clone());
builder.add_section(..., metadata_bytes.clone());
```

The originals are retained only so their lengths can later be placed in the summary and their digests were already computed for the header.

### Impact

At build peak, the process can simultaneously own:

- all four original section vectors;
- all four cloned section vectors in the builder;
- the final container vector while assembling output;
- graph/export scratch.

For the canonical Universe this can become a material memory spike and works against P7's compilation-efficiency objective.

### Required correction

Capture lengths before moving the vectors into the builder:

```rust
let semantic_world_len = semantic_world_bytes.len();
...
builder.add_section(..., semantic_world_bytes);
```

If `TeloscContainerBuilder::build` itself re-copies all payloads into a final vector, consider a capacity-exact builder/streaming assembly after correctness is closed.

This is not a reason to reorder current correctness priorities, but it should be fixed before performance certification.

---

## S1-10 — MEDIUM — Atomic writer is over-coupled to the full loader and ignores caller limits

### Evidence

Before touching the destination, `write_universe_telosc` invokes:

```rust
load_universe_telosc_from_bytes(&built.bytes, &ValidationLimits::default())?;
```

### Problems

1. **Today it can never validate a legitimate artifact** because the semantic loader is hard-disabled by `UnaccountedBoundary`.
2. The writer discards any limits under which the artifact was legitimately built and uses `ValidationLimits::default()` instead.
3. A writer's publication validation should prove actual byte/container/owner integrity, but coupling it to all runtime materialization allocations makes writing dependent on a potentially expensive full semantic reconstruction.

### Required correction

Keep the strong “validate actual bytes before replacing prior output” invariant, but separate:

- **artifact self-validation / semantic-owner validation** required for publication; from
- **consumer materialization under a particular runtime allocation budget**.

At minimum, pass explicit validation limits rather than hard-coding defaults. The exact API design can remain mechanically flexible, but a valid build should not become unwriteable solely because a default runtime load budget differs from its configured build/verification environment.

---

## S1-11 — HIGH — Standalone-package closure is architecturally wired but completely unproven from persisted bytes

### Positive finding

`phalcom-core/src/modules/compile.rs` already has a useful shape for the requested end state:

```rust
pub enum EntrySelection {
    ModuleId(ModuleId),
    Project(PathBuf),
    Package(PathBuf),
    Module(PathBuf),
    Inline(Arc<str>),
}
```

The image-aware analyzer supports `EntrySelection::Package`, loading `package.ph` / `main.ph`, then using `ModuleResolver::with_external_modules`, an external-module-aware linker, and `SemanticWorkspaceSession::from_image` with a runtime-minimal product request.

This is the correct architectural direction: local package source remains local, while canonical Universe authority is supplied from the immutable image.

### Gap

Because persisted semantic loading cannot currently succeed, no test demonstrates:

```text
universe.telosc -> load -> EntrySelection::Package -> real package -> compiler/lowering/runtime
```

The current artifact test only intends to compile the tiny inline program `print(42);`, and it never reaches loading.

Even after that test is repaired, `print(42)` is too weak for the user's P7 closure criterion. It does not exercise the package model or demanding library/compiler features such as parsing, similarity, slot maps, generic dispatch, static definitions, trait/conformance resolution, imported module values, etc.

### Required acceptance expansion

Before P7 closure, add at least these source-denied cases against **loaded** artifact authority:

1. a minimal standalone package with `package.ph` + `main.ph`;
2. a package exercising `parse` in an actually compiled/run code path;
3. a package exercising `similarity`;
4. a package exercising `slotmap`;
5. a multi-module package with explicit Universe imports;
6. at least one generic/trait/static-definition-heavy package to ensure persisted compiler attachments are not accidentally bypassed.

These should use real package discovery/compilation APIs, not a special artifact-only harness.

---

## S1-12 — MEDIUM — Branch/lifecycle hygiene is inconsistent at the current remote tip

### Stale handoff coordinates

At remote HEAD `2939e9d4`, the committed P7 handoff still says:

```text
branch perf/universe-characterization; HEAD remains c64dfc23;
changes are uncommitted and unstaged
```

That description is stale relative to the actual remote branch. The checkpoint header also retains older execution/test revision fields. Historical lifecycle fields may intentionally point at prior evidence, but an explicit “HEAD remains …” takeover instruction must match the actual branch state.

This matters because a future implementer could check out or reason from the wrong revision and incorrectly infer that current changes are only local/uncommitted.

### Committed rustc ICE artifact

The latest commit includes `rustc-ice-2026-10-15T09_58_51-58574.txt` at repository root. This appears to be a generated compiler crash report rather than product source or deliberate test fixture. It should normally be removed and ignored unless intentionally retained as an incident artifact under documentation.

### Branch integration risk

The P7 branch is 7 commits behind `main`. P7 should not be certified in isolation if those commits alter module, semantic, compiler, package or test behavior. Rebase/merge strategy need not interrupt current owner completion, but a final closure candidate must be re-audited against the consolidation target.

---

# 4. Additional correctness observations

## 4.1 `StaticApplication` remains a real owner-model blocker

The wire enum already has a `StaticApplication` denotation variant, but export deliberately rejects `SemanticDenotation::StaticApplication`. The current project record correctly notes that checked static applications live under `CheckedStaticBody` in the live producer path, while the callable-analysis plan previously described different retention.

This should remain fail-closed until the actual owner is settled. A portable identity must include the real checked-body owner plus application ordinal/signature required by consumers; resolving through a callee static definition would be semantically wrong when multiple bodies have overlapping ordinals.

## 4.2 Conditional behavioral dispatch cannot currently validate

`SemanticWorldV2::validate_conditional_dispatch_selection` explicitly returns an error because the semantic world does not serialize the implementation-member authority needed to prove `(impl, owner, callable)` selection.

This means behavioral-family denotations with conditional selections are not merely “untested”; they are structurally unrepresentable in a valid current artifact. That authority needs to be included in the conformance/dispatch owner work rather than bypassed in the denotation validator.

## 4.3 Partial callable-analysis helpers are useful but not yet compositional proof

The branch has meaningful pieces:

- flow graph DTO + budgeted frozen importer;
- source-owner/body range/status;
- binding origins;
- formal binding attachments;
- expression/binding denotations;
- typed `Term` / `RecordRow` / `DynamicType` call-specialization arguments;
- partial root registration.

These are valuable and should be preserved. The problem is not their local correctness; it is that the full graph still lacks required expressions, bindings, flow facts, match/pattern, diagnostics/explanations, dependencies, compiler attachment reconstruction, and a complete materializer. A synthetic helper round trip cannot substitute for source-built/imported `CallableAnalysis` parity.

---

# 5. Performance and memory assessment from Slice 1

P7's performance goal is not just “loading works”; it is to remove repeated canonical Universe reconstruction from normal compilation and testing. The current architecture has good foundations for that goal, but performance certification should not start yet because the working load path does not exist.

### Positive architecture already present

- immutable image-aware `SemanticWorkspaceSession::from_image`;
- external-module-aware resolver/linker path;
- runtime-minimal semantic product request;
- precomputed/module-indexed owner work inherited from P6;
- budgeted frozen CFG representation suitable for imported immutable data;
- shared type/static graph relocation concepts;
- bounded file/container decode work;
- native catalog compatibility fingerprinting.

### Current inefficiencies/gaps visible now

1. **Writer section cloning** — avoidable full-payload copies (S1-09).
2. **Debug-string owner index keys** — needless allocations and slower comparisons; use typed module IDs.
3. **Current failed build does nontrivial work before the coverage guard** — module world and shared graph root/closure work happens before the inevitable semantic coverage failure. This is acceptable during implementation but makes repeatedly running broad build tests expensive; focused tests should target owners until the guard is realistically close to removable.
4. **Full loader used for writer publication validation** — potentially duplicates all materialization work every time an artifact is written. Strong semantic byte validation is necessary, but the final writer may benefit from a validation mode that proves owner integrity without retaining a complete loaded image.
5. **No current end-to-end measurements** can demonstrate artifact load/compile speed because the load path is disabled. Existing earlier performance evidence must not be promoted to P7 closure evidence.

---

# 6. Reanalysis of remaining work toward P7 closure

The active plan's broad categories remain valid, but the live tree suggests a more concrete implementation order. The main risk is doing allocation/query/polish work around a semantic image that still cannot represent the compiler's canonical owners.

## Closure Phase A — Make the semantic contract truthful and exhaustive

**Priority: immediate / blocking everything else.**

1. Reconcile `SNAPSHOT_FIELD_MANIFEST` and `CALLABLE_ANALYSIS_FIELD_MANIFEST` with the real current structs and readers.
2. For every `Wire` row, identify the exact DTO field, export helper, root registration, validation, import helper and final owning table.
3. Add missing owner DTOs rather than synthesizing authority from adjacent tables.
4. Finish full `CallableAnalysis` transport:
   - expression records and type knowledge;
   - binding states/contracts/current knowledge;
   - all iteration/sequence/expansion products;
   - call/resolution products;
   - match/pattern products;
   - CFG/entry/exits/return validation;
   - diagnostics and explanations with causal sharing;
   - dependencies / semantic dependencies;
   - fingerprint reconstruction after relocation;
   - constructor products;
   - owner-qualified source/formal attachments;
   - StaticApplication after the real checked-body owner decision.
5. Add complete accepted callable-definition transport.
6. Complete trait/dispatch/witness/conformance primary authority without defaults.
7. Complete static case/definition/constant/module-binding owners.
8. Complete the type/static root census for every newly persisted field before graph closure.

**Exit condition:** exporter no longer needs to reject a valid canonical Universe owner family, and a source-built world can be encoded without dropping information.

## Closure Phase B — Make import exact before making it available

1. Eliminate all `TypeId(0)` / `KindId(0)` semantic placeholders.
2. Eliminate default visibility/source/provenance substitutions where source authority exists.
3. Preserve all witness completeness/failure/requirement-view/conditional fields exactly.
4. Materialize callable analyses into `CallableAnalysisTable`.
5. Install checked operational products into real `StaticDefinitionTable` / `StaticConstantTable` / module binding owners.
6. Rebuild typed P6 owner indexes through actual owning APIs; delete detached debug-string maps once redundant.
7. Rebuild formal compiler attachments from owner facts and exercise the real lowering readers.
8. Replace cardinality-only completion with owner/field/referential completeness validation.

**Exit condition:** full source-built/imported semantic parity passes modulo intentional process-local IDs and relocated store coordinates.

## Closure Phase C — Close the allocation transaction

Only now should the unconditional `UnaccountedBoundary` be removed.

1. Budget TypeStore graph import and interners.
2. Budget every structural/static/primary owner and index.
3. Budget query reconstruction outputs.
4. Budget formal-image and publication wrappers.
5. Preserve typed allocation errors through core without string formatting.
6. Run a counting allocator around a real full Universe load.
7. Test exact fit, one-byte-short failure, rollback and retry.

**Exit condition:** independently observed allocator requests equal the transaction ledger for the full load path within the adopted accounting definition.

## Closure Phase D — End-to-end artifact acceptance

1. `build_universe_telosc` succeeds on the canonical Universe.
2. Two independent builds are byte-identical.
3. Writer validates actual bytes and atomically replaces output.
4. Disk and byte loaders create fresh stores and complete loaded image.
5. Child process has Universe source acquisition actively denied.
6. Compile a novel user program with explicit Universe imports.
7. Compile/run standalone package fixtures exercising `parse`, `similarity`, `slotmap` and other current Telos features.
8. Confirm zero canonical parsing/linking/body analysis/presentation/advisory counters in the loaded-image path.
9. Run P5/P6 locality/non-regression gates.
10. Integrate latest `main`/consolidation changes and repeat the closure discriminators.

## Closure Phase E — Performance and memory certification

After correctness:

- remove section buffer cloning;
- measure artifact build time, write time, load time, first user overlay compile and warm compile;
- compare workspace/corpus testing with source-built vs artifact-loaded Universe;
- ensure no new O(N×M) reconstruction has reappeared in the loader or compiler projection;
- confirm immutable canonical structures are shared rather than copied into each user overlay.

---

# 7. Immediate implementer guidance from this slice

The next implementer should **not** begin by trying to make `build_and_load_universe_telosc_compiles_source_free_inline_user_entry` green directly. That would create pressure to remove the very guards preventing invalid publication.

Recommended immediate sequence:

1. **Freeze the current guards.** Add comments/tests making it explicit that a guard may be removed only after its owner-family parity test passes.
2. **Build an executable field-to-wire matrix for `CallableAnalysis`.** The existing manifest is the target; mark each row Implemented/Partial/Missing and name the live DTO/importer.
3. **Complete one vertical owner slice at a time.** Prefer expressions + bindings + storage/source attachments first because compiler lowering depends on them and many downstream records reference their ordinals.
4. **Then complete resolutions/flow/dependencies/diagnostics**, ensuring every owner-local ordinal is validated and relocatable.
5. **Resolve StaticApplication under its real checked-body owner** before declaring denotation coverage complete.
6. **In parallel with final callable graph work, replace fabricated callable/trait/dispatch/witness reconstruction** with owner-backed data.
7. **Only after source-built/import parity passes should the callable-analysis export/import gates be relaxed.**
8. **Then close the global allocation ledger** and remove `UnaccountedBoundary`.
9. **Only then run the real build/load/package acceptance matrix.**

### Tests that should be added or strengthened before guard removal

| Test | Original defect it must detect |
|---|---|
| Source-built vs imported full `CallableAnalysis` equality modulo relocation | Partial DTO treated as complete |
| Two callables with equal numeric local expression/binding IDs | Bare local IDs accidentally compared cross-owner |
| Dispatch surface real callable `TypeKnowledge` parity | `TypeId(0)` assumed placeholder |
| Callable-definition origin/member-index/inherent-context parity | Signature-derived fabricated definition |
| Trait associated-type domain/visibility/source/default parity | `KindId(0)` / `Public` / default span substitutions |
| Witness plan every completeness variant with non-empty failure payload | Generic fabricated failure states |
| Conditional inherent selection/applicability parity | `None` substitutions |
| Static definition/constant/module binding consumer test | Detached operational templates |
| Formal binding/site/target lowering against imported image | Default formal projection |
| Full-load counting allocator exact-fit/N-1 | Unaccounted TypeStore/owner/index allocations |
| Core typed allocation-error zero-extra-allocation test | `e.to_string()` error allocation |
| Standalone package source-denied tests for `parse`, `similarity`, `slotmap` | Tiny inline fixture hiding package/compiler attachment gaps |

---

# 8. What is already worth preserving

This audit should not be read as a recommendation to restart P7. Several recent implementation choices are aligned with the closure architecture and should be retained:

- fail-closed publication/import while required authority is missing;
- native catalog fingerprint in compatibility identity;
- explicit `Term` / `RecordRow` / `DynamicType` call-static arguments;
- inline owner-held enum requirement signatures;
- budget-aware immutable flow-graph backing for loaded analyses;
- stable owner-local ordinals rather than process-native IDs;
- shared type/static relocation contexts;
- external-module-aware resolver/linker path;
- explicit image injection into compiler/analyzer;
- atomic same-directory temporary publication;
- source-free model that excludes raw parsed Universe syntax;
- runtime-minimal user overlay request.

The correct closure strategy is to finish these designs through the actual compiler owners, not replace them with shortcuts.

---

# 9. Slice 1 closure assessment

### Can P7 currently compile and persist the Universe?

**No.** Real artifact construction stops at semantic export coverage.

### Can a complete `universe.telosc` currently be loaded?

**No.** Budget-aware semantic import unconditionally returns `UnaccountedBoundary`.

### Would deleting those two guards make P7 functional?

**No.** The inner importer currently fabricates/drops required authority and would fail its own source-free completeness check.

### Is the source-free compiler injection architecture present?

**Yes, substantially.** `EntrySelection::Project/Package/Module/Inline`, external module resolver/linker integration, and `SemanticWorkspaceSession::from_image` provide the right integration seam. It is not yet reachable from a valid persisted image.

### Is standalone-package support proven?

**No.** The API path exists; artifact-backed package acceptance has not executed, and feature-dense cases such as `parse`, `similarity`, and `slotmap` are not yet part of P7 closure evidence.

### Is current P7 allocation safety complete?

**No.** Several subpaths are budgeted and measured, but semantic import remains intentionally disabled because the full owner/materialization transaction is not accounted.

### Is P8 takeover appropriate?

**No.** P7's compiler-image interface is still incomplete and unsafe to consume as a stable predecessor contract.

---

# 10. Recommended scope for Audit Slice 2

The next audit turn should go deeper into the **full `CallableAnalysis` owner graph and compiler-reader contract**, because this is the first dominant correctness dependency after the two hard guards. Specifically:

1. enumerate every live `CallableAnalysis` field and compare it to `CALLABLE_ANALYSIS_FIELD_MANIFEST` and `CallableAnalysisRecordV1`;
2. inspect every compiler/lowering/static-eval reader of those fields;
3. audit `register_callable_analysis_roots` for complete graph-root reachability;
4. audit owner-local ordinal generation for determinism and collision safety;
5. trace `StaticApplication` production/ownership through `CheckedStaticBody` and operational lowering;
6. inspect formal source/binding/site reconstruction for `<main>`, constructors, imported module bindings and ordinary methods;
7. enumerate exact missing DTOs/materializers and produce the implementation order needed to remove the callable-analysis guards safely;
8. identify memory/allocation consequences of the complete body graph before the importer shape becomes entrenched.

A later slice should separately audit conformance/witness/trait/dispatch/static owners, then query reconstruction/allocation, then end-to-end package/performance acceptance.

---

## Appendix A — Live branch state used by this audit

```text
branch:       perf/universe-characterization
remote HEAD:  2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a
commit:       feat(perf): extend artifact loading foundations
vs main:      diverged, 51 ahead / 7 behind
merge base:   e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
CI statuses:  none reported for HEAD
workflow runs associated with HEAD: none reported
```

## Appendix B — Highest-value code references

| Concern | Live reference |
|---|---|
| Export hard stop | `phalcom-semantic/src/artifact/export.rs:2186-2238` |
| `StaticApplication` export rejection | `phalcom-semantic/src/artifact/export.rs:1683-1692` |
| Budgeted semantic import hard stop | `phalcom-semantic/src/artifact/import.rs:1798-1819` |
| Callable-analysis import completeness guard | `phalcom-semantic/src/artifact/import.rs:630-636`, `1833-1835` |
| Fabricated callable definitions | `phalcom-semantic/src/artifact/import.rs:1858-1885` |
| Trait defaults/placeholders | `phalcom-semantic/src/artifact/import.rs:1890-1970` |
| Dispatch `TypeId(0)` placeholder | `phalcom-semantic/src/artifact/import.rs:1980-1994` |
| Lossy witness reconstruction | `phalcom-semantic/src/artifact/import.rs:2026-2160` |
| Empty/default required owners in reconstructed snapshot | `phalcom-semantic/src/artifact/import.rs:2215-2282` |
| Detached debug-string owner indexes | `phalcom-semantic/src/artifact/import.rs:2284-2308` |
| Callable analysis wire record | `phalcom-compiler-artifact/src/semantic/world.rs:298-312` |
| Conditional dispatch validator hard failure | `phalcom-compiler-artifact/src/semantic/world.rs:1212-1221` |
| Field manifests | `phalcom-semantic/src/artifact/manifest.rs` |
| Completeness predicate | `phalcom-semantic/src/snapshot.rs:777-822` |
| Writer/build path | `phalcom-core/src/modules/canonical_artifact.rs:128-245` |
| Loader path | `phalcom-core/src/modules/canonical_artifact.rs:314-410` |
| Atomic writer self-validation | `phalcom-core/src/modules/canonical_artifact.rs:251-263` |
| Image-aware package/compiler entry | `phalcom-core/src/modules/compile.rs:2304-2635` |
| End-to-end artifact tests | `phalcom-core/tests/core/modules/canonical_artifact.rs` |

---

**Audit Slice 1 verdict:** The implementation has substantial infrastructure, but **P7 is still an incomplete compiler-state persistence system, not yet a working Universe artifact cache**. The immediate priority is exact semantic-owner transport and materialization. Allocation closure and source-denied package acceptance should follow that semantic closure; performance polish follows functional correctness.
