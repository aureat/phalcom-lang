# Fresh-Session Handoff Prompt — PERF003.C2 P1–P9 Through `universe.telosc`

You are continuing the **Telos compiler/runtime performance program `PERF003`, checkpoint `PERF003.C2`** in repository **`aureat/phalcom-lang`** (also referred to locally as `phalcom`). This handoff is deliberately comprehensive. Its purpose is to prevent you from spending another session rediscovering architectural conclusions, already-audited source relationships, already-implemented work, and already-explained performance behavior.

Your task in the fresh session is to **continue planning and then, when asked, produce implementation-grade requirements/plans for the remaining checkpoint packages from the currently-running P4 through P9**, ending in a fully compiled, cached canonical Universe artifact named:

```text
universe.telosc
```

The final checkpoint objective is stronger than “warm compilation is fast”:

> **In an ordinary clean process, Telos must load a compatible `universe.telosc` and perform substantive compilation only for user-owned source. Universe source must not be parsed, semantically analyzed, metadata-exported, lowered, or bytecode-compiled during ordinary compiler/runtime startup.**

Do not casually weaken that target to an in-process cache.

---

# 1. Repository and current execution state

## Repository

```text
GitHub: aureat/phalcom-lang
working repository often called: phalcom
branch: perf/universe-characterization
```

Latest remote branch head verified at handoff creation:

```text
4474455c2042b66a6fccc0665948f67424f83c41
```

The most recent performance samples discussed below were explicitly taken from source commit:

```text
c34f9c3
```

The user reported that `4474455c` contains later documentation changes relative to that tested source. Therefore:

```text
Do NOT attribute the c34f9c3 timing samples to post-c34f9c3 P4 implementation.
Do NOT assume current local P4 edits are committed merely because the remote head is 4474455c.
```

P4 is being implemented concurrently in another application/worktree while this planning conversation continues.

---

# 2. User expectations

The user expects:

- compiler-architecture analysis grounded in actual repository code;
- exact crate/file/symbol ownership;
- patch-grade implementation sequencing;
- explicit invariants and non-goals;
- observability and work-cardinality gates;
- semantic/runtime parity gates;
- STOP/CONSULT conditions when architecture needs escalation;
- lifecycle/checkpoint integration;
- no expedient performance hacks that compromise architecture;
- no re-analysis of settled questions unless current source contradicts this handoff.

The user is comfortable with multi-turn investigation, but this handoff is intended to make the next session start from the established architecture rather than repeat P1–P4 discovery.

When implementation plans are requested, use dense, structured Markdown and exact source references.

---

# 3. Important planning-history clarification

The checkpoint numbering evolved during investigation.

The **old detailed implementation plan** was organized as five packages:

```text
Package A — canonical dependency image / one-shot overlay
Package B — consumer-specific publication
Package C — conformance/callable settlement
Package D — metadata reachability
Package E — backend/local complexity
```

The active checkpoint later inserted dedicated formal-image/query-root work as P2. The earlier Package B publication work was moved to **P7**, creating an awkward historical ledger with:

```text
P1
P2
P3
P4
P5
P7
```

and no active P6.

A later unified compiler/artifact specification expanded the conceptual program further into in-memory compiler packages plus artifact packages A1–A7.

**Do not preserve that historical numbering mechanically.**

The user explicitly approved the normalized forward program described in this handoff:

```text
P1  Canonical module / one-shot overlay boundary           committed
P2  Complete canonical formal image + immutable queries   committed
P3  Conformance/callable incrementalization               committed
P4  Root-driven metadata + cross-pool authority           IN IMPLEMENTATION
P5  Canonical publication + persistent image boundary     planned
P6  User-only backend/local complexity closure            planned
P7  .telosc schema + persisted semantic compiler image    planned
P8  Portable executable Universe image                    planned
P9  Artifact-first loader/cache/toolchain certification   planned
```

This is the authoritative **planning progression** to use going forward unless the user explicitly changes it.

---

# 4. Checkpoint success definition

The final `PERF003.C2` architecture should be:

```text
                     ┌───────────────────────────┐
                     │      universe.telosc      │
                     ├───────────────────────────┤
                     │ module dependency image   │
                     │ semantic compiler image   │
                     │ reusable query products   │
                     │ canonical metadata        │
                     │ portable executable image │
                     └─────────────┬─────────────┘
                                   │
                validate / relocate / materialize
                                   │
                                   ▼
                         canonical authority
                                   │
                                   ▼
user source ─► parse ─► analyze ─► lower ─► bytecode compile
                                   │
                                   ▼
                                  VM
```

Ordinary user compilation/runtime startup must **not** do:

```text
Universe source parsing
Universe interface rebuilding
Universe dependency discovery
Universe full semantic update
Universe callable-body analysis
Universe result settlement
Universe conformance/witness reconstruction
Universe metadata export
Universe backend lowering
Universe AST bytecode compilation
```

It may legitimately do:

```text
.telosc open/read
format/header validation
compatibility/digest validation
semantic arena relocation/materialization
canonical metadata-pool registration
VM-local object allocation
fresh closures/chunks/cache allocation from portable prototypes
Universe initializer EXECUTION
user-code compilation
```

Do not confuse **compiling an initializer** with **executing an initializer**. P8/P9 must eliminate Universe compilation, but initializer execution remains VM-local runtime work.

---

# 5. Established source architecture from P1–P3

The following architecture has already been investigated and should be treated as established unless the current source has materially changed.

## 5.1 Canonical module ownership

Canonical Universe dependencies are intended to be immutable external module products, not workspace-local source state.

Relevant owners include:

```text
phalcom-modules
  CanonicalModuleProducts
  UniverseSourceProvider
  WorkspaceModuleSession::with_external_modules
  ModuleLinker::with_external_modules
```

Core dependency discovery has been moved toward treating Universe targets as terminal external dependencies instead of recursively inserting all Universe source modules into local mutable state.

The architectural law is:

```text
Canonical Universe modules = immutable external dependency world
User/package modules        = mutable/local workspace world
```

Canonical modules must not appear as ordinary local components merely because user code imports them.

---

## 5.2 Canonical formal semantic image

Primary source:

```text
phalcom-semantic/src/canonical_image.rs
```

Important shape:

```rust
pub struct CanonicalUniverseFormalImage {
    provider: UniverseSourceProvider,
    world: CanonicalSemanticWorldId,
    modules: Arc<CanonicalModuleProducts>,
    snapshot: Arc<SemanticSnapshot>,
    queries: Arc<CanonicalSemanticQueryImage>,
    type_prefix: CanonicalTypePrefixId,
}
```

Process cache:

```rust
static IMAGE: OnceLock<Arc<CanonicalUniverseFormalImage>> = OnceLock::new();
static INITIALIZE: Mutex<()> = Mutex::new();
```

`canonical_universe_formal_image()`:

1. checks `IMAGE`;
2. uses a recursion guard;
3. serializes first construction through `INITIALIZE`;
4. builds the canonical image on cache miss;
5. publishes only a successful image;
6. subsequent consumers reuse the shared `Arc`.

The image build currently performs roughly:

```text
canonical_universe_semantic_baseline()
    ↓
CanonicalModuleProducts::capture(...)
    ↓
SemanticWorkspaceSession::from_foundation_for_image(...)
    ↓
SemanticWorkspaceInput containing canonical world
    ↓
session.update(...)
    ↓
complete SemanticSnapshot
    ↓
validate_publication(...)
    ↓
CanonicalSemanticQueryImage::freeze(...)
    ↓
CanonicalUniverseFormalImage
```

This source-built first construction is now the major cold-start bottleneck.

---

## 5.3 Ordinary sessions are image-backed

Primary source:

```text
phalcom-semantic/src/session.rs
```

Current architecture:

```rust
pub fn new() -> Self {
    Self::with_workspace(WorkspaceId::from_raw(1))
}

pub fn with_workspace(workspace: WorkspaceId) -> Self {
    Self::from_image(
        workspace,
        crate::canonical_image::canonical_universe_formal_image()
            .expect("canonical Universe image must build")
            .clone(),
    )
}
```

This is an important correctness decision:

```text
canonical image != local last_snapshot
```

`from_image(...)` creates a local workspace over immutable canonical authority. Local publication history begins empty:

```text
last_snapshot: None
last_known_good: None
```

Do not regress to:

```rust
last_snapshot = Some(canonical_snapshot)
```

Canonical and workspace histories are distinct lifetime domains.

---

## 5.4 Canonical SemanticDb read-through

The semantic DB gained canonical read-only parent behavior.

Conceptually:

```text
local query state
    +
canonical immutable query roots
```

Canonical products have a distinct origin such as:

```rust
SemanticProductOrigin::Canonical(CanonicalSemanticWorldId)
```

and may be read/reused without being inserted as mutable workspace query state.

Local invalidation/rollback must never mutate canonical roots.

P2 already established explicit freeze eligibility/rejection logic for canonical query products.

Do not serialize an arbitrary mutable `SemanticDb` later. P7 must persist only explicit immutable/relocatable query products.

---

## 5.5 Type-store lineage

The current in-memory model depends on a canonical type prefix plus workspace-local extension.

`TypeStore::fork_from_baseline()` preserves canonical prefix numeric IDs in a fresh local store lineage.

This works well in-process for one canonical parent.

It does **not** mean raw `TypeId`, `TypeStoreId`, `StaticValueStoreId`, etc. are safe durable identities.

P7 must introduce a relocatable on-disk semantic graph.

---

# 6. P3 accomplishments that should not be re-derived

The source/plan work for P3 established or implemented these directions:

```text
exact witness lookup
module-bounded callable/signature enumeration
partitioned ConformanceIndex
nominal-family + open-head indexing
brute-force parity oracle for indexed conformance queries
SCC/affected-result inferred-return settlement
module-value dependency sensitivity
shared/cached requirement views
existing GenericConformance SemanticDb query reuse
negative-dependency correctness
```

Important source fact:

```text
GenericConformanceKey is already a first-class SemanticDb query.
```

Do not invent a second global proof cache parallel to it.

Desired layering remains:

```text
SemanticDb GenericConformance query
    ↓
one proof-session evaluation
    ↓
ConformanceProofContext local recursive memo/cycle/budget logic
    ↓
published reusable product
```

P3's larger purpose was to make local semantic updates dependency-sensitive instead of repeatedly settling the entire canonical world.

The recent measurements strongly suggest that succeeded: local semantic-update time collapsed from roughly 1.4 s/fixture in the tested baseline to roughly 40 ms/fixture on the optimized branch.

---

# 7. Current P4 — Profile/Root-Driven Metadata Reachability

P4 is being implemented now in another worktree/application.

The implementation plan already established these responsibilities.

## 7.1 Problem

`MetadataProfile` exists in:

```text
phalcom-type-meta/src/header.rs
```

with:

```rust
RuntimeMinimal
RuntimePublic
ToolingDebug
Proof
```

but historically the exporter behaved as a whole-world exporter and largely treated the profile as descriptive metadata.

`phalcom-semantic/src/metadata/export.rs` had aggregate traversal and an O(declarations × callables) owner-discovery pattern.

Normal compiler metadata also had incomplete RuntimePublic field handling.

---

## 7.2 P4 target model

```text
MetadataProfile
    +
MetadataRootSet
    ↓
MetadataReachabilityClosure
    ↓
reachable local metadata records
    +
canonical metadata authority references
```

P4 must:

```text
make MetadataProfile operational
introduce explicit metadata roots
introduce deterministic reachability closure
index callable ownership
index field ownership where needed
wire RuntimePublic fields
eliminate production whole-world export/filter behavior
treat canonical metadata as external authority
add runtime stable-identity → pool-qualified record lookup
remove same-pool reflection assumptions
define duplicate stable-identity conflict policy
define canonical metadata compatibility contract
```

---

## 7.3 Existing runtime multi-pool architecture

Do not reinvent this.

Relevant source:

```text
phalcom-core/src/typing/handle.rs
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/loader.rs
phalcom-core/src/typing/inspect.rs
```

Existing concepts include:

```rust
MetadataPoolId
RuntimeTypeRef
RuntimeTypingRegistry
```

P4 should make existing pools genuinely compositional.

Critical rule:

```text
MetadataPoolId is VM-local load coordinate.
It is NOT persistent artifact identity.
```

Do not serialize “canonical Universe is pool 0”.

Cross-pool semantic authority should use stable semantic identities such as:

```text
StableDeclarationRef
StableCallableRef
StableFieldRef
```

and a persistent dependency/compatibility identity.

---

## 7.4 P4 artifact relevance

P4 is a prerequisite for clean `.telosc` composition because it establishes:

```text
canonical RuntimePublic metadata is owned by Universe
user bundle owns only local metadata
local type graphs may refer to canonical declarations by stable identity
runtime reflection can cross pool boundaries
```

At P4 closure, user compilation should not duplicate canonical declaration/callable/field metadata without explicit justification.

---

# 8. Critical performance evidence discovered after P1–P3

These measurements are extremely important. Do not repeat the basic diagnosis unless source behavior changed.

## 8.1 Single-fixture controlled sample

Exact first-fixture sample:

```text
baseline: b00d75b3
optimized tested source: c34f9c3
dev profile
one Cargo job
--test-threads=1
timing instrumentation enabled
```

Both runs failed on the first fixture before VM startup/user execution due metadata-export error.

Measured:

| Phase | Baseline | Optimized source |
|---|---:|---:|
| Semantic session initialization | 214 ms | 44.351 s |
| Semantic update | 1.446 s | 42.8 ms |
| Parse | 0.130 ms | 0.258 ms |
| Discovery/link | 0.766 ms | 3.735 ms |
| User projection | 21.6 ms | 33.0 ms |
| **Measured sum** | **1.682 s** | **44.430 s** |

Interpretation:

```text
semantic update became ~33.7× faster (~97% less time)
cold first-session startup grew by ~44.1 s
cold failed first-fixture measured sum became ~26× slower
```

This is not contradictory. The cost moved.

---

## 8.2 Ten-test sample

Tests:

```text
random
random_syntax
lexical
arithmetic
booleans
bindings
messages
dispatch
classes
fields
```

Results:

| Run | Baseline `b00d75b3` | Optimized source | Comparison |
|---|---:|---:|---:|
| 10 tests, 1 test thread | 17.13 s | 46.02 s | branch 2.69× slower |
| 10 tests, 10 test threads | 5.52 s | 42.29 s | branch 7.66× slower |

For the eight tests that reached semantic update in the serial batch:

```text
baseline semantic updates total: 11.696 s
optimized semantic updates total: 0.348 s
```

or approximately:

```text
33.6× faster
```

The optimized branch’s first test spent:

```text
44.973 s
```

initializing the semantic session.

Later sessions took roughly:

```text
9–12 ms each
```

With ten test threads, all threads wait behind the one-time canonical image build.

All ten tests failed in both tested sets; none reached successful VM execution. Most failed on metadata export; others on semantic type/dependency errors.

Therefore these samples establish phase behavior, **not successful end-to-end runtime parity**.

---

## 8.3 Concurrency diagnosis

This behavior follows directly from:

```rust
static IMAGE: OnceLock<Arc<CanonicalUniverseFormalImage>> = OnceLock::new();
static INITIALIZE: Mutex<()> = Mutex::new();
```

Cold concurrent consumers behave like:

```text
thread A → acquire initialization lock → build Universe for ~42–45 s
thread B → wait
thread C → wait
...
image published
all threads → cheap image-backed session creation
```

The branch has introduced a **serialized process-wide cold-start barrier**.

The baseline has expensive per-test semantic work that can parallelize across test threads; the optimized branch cannot exploit test parallelism until canonical bootstrap finishes.

---

## 8.4 Approximate amortization behavior

Using measured local semantic-update savings:

```text
baseline per-fixture update ≈ 1.46 s
optimized per-fixture update ≈ 0.043 s
saved per fixture ≈ 1.42 s
cold canonical cost ≈ 44–45 s
```

Crude breakeven:

```text
~32 fixtures
```

This is compatible with an earlier historical 50-fixture `corpus::classes` result around:

```text
~46 s optimized
vs
~793 s historical
≈ 17× faster
```

That old 50-fixture comparison was not a tightly controlled benchmark, so preserve it only as contextual evidence.

---

# 9. Primary performance conclusion

P1–P3 did **not** fail.

They changed the performance equation from:

```text
T ≈ N × expensive local/Universe-contaminated update
```

to:

```text
T ≈ expensive one-time canonical image construction
    + N × cheap local update
```

Current measured shape is approximately:

```text
T ≈ 45 s + N × ~40 ms
```

The desired final shape is:

```text
T ≈ .telosc validation/load
    + N × local user compilation
```

The major remaining cold-start issue is therefore not “incremental semantics are still slow”. It is:

> **The canonical dependency image is still built from Universe source on first use rather than loaded from a persisted compiled artifact.**

---

# 10. One known avoidable canonical-bootstrap issue

`ProgramCompilationSession::standalone()` currently does conceptually:

```rust
SemanticWorkspaceSession::new()
    .with_product_request(SemanticProductRequest::runtime_minimal())
```

But `SemanticWorkspaceSession::new()` must obtain/build the canonical image **before** the local session receives `runtime_minimal()`.

Inside `canonical_image.rs`, canonical bootstrap currently uses:

```rust
product_request: SemanticProductRequest::default()
```

and:

```rust
Default == full()
```

which means:

```text
PresentationLevel::Full
AdvisoryLevel::Full
```

during canonical image construction.

This deserves measurement and likely a later explicit request such as:

```text
SemanticProductRequest::canonical_compiler_image()
```

or equivalent.

Do **not** blindly change canonical bootstrap to `runtime_minimal()` without proving formal/query/tooling completeness. The key requirement is an explicit canonical-image product contract, not a convenient enum substitution.

This likely explains only part of the ~45 seconds; do not mistake it for the complete solution.

---

# 11. Another known post-bootstrap cost

`SemanticWorkspaceSession::from_image()` currently clones aggregate semantic tables, including conceptually:

```rust
base_declarations: (*snapshot.declarations).clone(),
base_hierarchy: (*snapshot.hierarchy).clone(),
base_dispatch: (*snapshot.dispatch).clone(),
base_callable_signatures: (*snapshot.callable_signatures).clone(),
...
```

and rebuilds some derived maps such as alias forms.

Warm session creation is currently only about 9–12 ms in the reported test, so this is not the 45-second bottleneck.

However, P5 should still move toward canonical immutable aggregate parents + local deltas rather than per-session canonical aggregate cloning.

---

# 12. VM-side fact that affects final checkpoint definition

Even if semantic `CanonicalUniverseFormalImage` becomes persistent, the VM still compiles Universe source/AST for execution.

Important source:

```text
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/interpret.rs
```

Universe bootstrap currently reaches:

```rust
compile_ast_as_with_bindings(...)
```

for source-authored Universe modules.

Generic `run_compiled()` can also call:

```rust
compile_program_module_closure(...)
```

when a materialized module lacks a closure.

Therefore a semantic-only artifact is **not** sufficient for the user's final wording:

```text
compiler only compiles user-code
```

The final `universe.telosc` must contain a **portable executable Universe image** whose prototypes can be materialized into fresh VM-local runtime objects without reparsing/recompiling Universe source.

---

# 13. Normalized P1–P9 program

This is the forward plan approved by the user.

---

# P1 — Canonical Module / One-Shot Overlay Boundary

## Status

```text
committed / architecture established
```

## Responsibility

Establish canonical Universe as an immutable external module world rather than user-workspace source.

Key effects:

```text
external canonical module products
canonical interface reuse
local discovery does not recursively absorb Universe
local workspace source state remains user/package-owned
one-shot and retained paths converge toward module-session ownership
```

## Architectural gate

For an ordinary user fixture:

```text
canonical modules in local source state = 0
canonical modules in local components = 0
canonical parsed discovery = 0
canonical local interface builds = 0
```

Do not redo P1 architecture unless current source contradicts this.

---

# P2 — Complete Canonical Formal Image + Immutable Query Roots

## Status

```text
committed
```

## Responsibility

Build one complete immutable formal Universe world and expose it as a read-only semantic parent.

Key types/symbols:

```text
CanonicalUniverseFormalImage
CanonicalSemanticQueryImage
CanonicalSemanticWorldId
SemanticDb::with_canonical_image
SemanticWorkspaceSession::from_image
SemanticWorkspaceSession::from_foundation_for_image
```

Key invariant:

```text
canonical image is not workspace last_snapshot
```

## Current limitation

First process access builds the full image from source, causing the ~45-second cold barrier.

P7/P9 eliminate that ordinary source build.

---

# P3 — Conformance / Callable Incrementalization

## Status

```text
committed
```

## Responsibility

Remove global/repeated callable/conformance settlement from local updates.

Established directions:

```text
partitioned ConformanceIndex
exact witness lookup
module-value dependency-sensitive analysis
SCC inferred-result propagation
structural/final witness separation direction
shared requirement views
reuse existing GenericConformance query layer
negative dependency correctness
```

## Evidence

Recent controlled samples show local semantic updates around ~40 ms versus ~1.4 s baseline in tested fixtures.

Treat this as strong evidence that local semantic architecture is much improved, while noting the sampled batches failed later and therefore are not full parity certification.

---

# P4 — Root-Driven Metadata + Cross-Pool Authority

## Status

```text
IN IMPLEMENTATION NOW
```

## Responsibility

Make metadata retention proportional to requested runtime/public roots and separate canonical metadata ownership from user metadata.

Target components:

```text
MetadataRootSet
MetadataRetentionRequest / MetadataRetentionPolicy
MetadataReachabilityClosure
callable owner index
field owner index if needed
closure-driven MetadataExporter
canonical external metadata authority
runtime stable-identity indexes
cross-pool reflection
canonical dependency compatibility
metadata cardinality observability
```

## P4 closure requirements

At minimum:

```text
RuntimeMinimal metadata work ∝ executable roots
RuntimePublic metadata work ∝ local public API + executable roots
no D×C callable owner scan
canonical declaration/callable/field records not copied into user bundle
RuntimePublic field metadata actually present
cross-pool reflection works
MetadataPoolId not persisted as stable identity
```

P4 is a prerequisite for clean `universe.telosc` metadata ownership.

---

# P5 — Canonical Publication + Persistent Image Boundary

## Purpose

Make the in-memory canonical compiler image explicitly serialization-ready and remove remaining full/tooling publication and aggregate-copy ambiguity.

P5 should absorb the historically displaced “consumer-specific publication / demand-driven products” work rather than preserving the stale current-P7 numbering.

## P5.1 Explicit canonical product request

Introduce/settle an explicit canonical image publication contract, conceptually:

```text
SemanticProductRequest::canonical_compiler_image()
```

or a richer request architecture.

Classify every product family:

```text
required for downstream type checking
required for generic/conformance queries
required for lowering
required for canonical runtime metadata
required for tooling only
optional presentation/source-map
nonpersistable/session-local
```

Do not let `Default == Full` implicitly define the canonical artifact.

---

## P5.2 Finish consumer-specific publication

Complete the earlier Package B/P7 work:

```text
runtime-minimal presentation exclusion
advisory exclusion
full-tooling compatibility
single-pass source-target traversal
source-index/presentation separation
explicit NotRequested/omission semantics as needed
```

Executable compiler mode should not reconstruct canonical editor/advisory products.

---

## P5.3 Persistent canonical aggregates

Move semantic session ownership toward:

```text
Arc<CanonicalUniverseFormalImage>
    +
local semantic deltas
```

instead of cloning canonical aggregate tables into each workspace.

Desired structural gate:

```text
zero canonical aggregate entries copied during warm session creation
```

where practical.

---

## P5.4 Formal serialization inventory

Produce the normative inventory P7 will encode:

```text
module products
declarations
aliases
hierarchy
dispatch
callable definitions/signatures
callable analyses required downstream
trait surfaces
associated surfaces
data/enum semantics
type members
generic signatures
conformance index
derived conformance rules
witness plans
static domains/values
eligible frozen query products
canonical metadata authority
compatibility/provenance data
```

Also explicitly classify:

```text
never persisted
process-local only
workspace-local only
debug/presentation optional
```

## P5 gate

Warm image-backed workspace creation must not cause:

```text
canonical semantic update
canonical source/advisory/presentation rebuild
canonical aggregate deep reconstruction
```

At this point the canonical in-memory image is ready for a portable projection.

---

# P6 — User-Only Backend & Local Complexity Closure

## Purpose

Prove that once an immutable canonical image exists, the remaining ordinary compiler pipeline scales with user-owned modules rather than whole-world aggregate tables.

This corresponds largely to the old Package E/current P5 backend work.

## Known source hotspot family

Especially:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

Whole aggregate scans currently/previously included patterns like:

```rust
for (callable, analysis) in snapshot.callable_analyses.iter() {
    if callable.owner.module() != module {
        continue;
    }
}
```

and trait requirement-slot discovery over all trait surfaces.

## P6 target indexes

Examples:

```text
module → callable analyses
module → lowering products
owner → callables
trait → requirement slots
declaration/family → inherent impl domain
module/source → source ranges
other measured hot lookups
```

Use observability before inventing indexes.

## P6 structural gate

Compiling one local user module against the canonical image must have:

```text
canonical declarations globally scanned       = 0
canonical callable analyses globally scanned  = 0
canonical trait surfaces globally scanned     = 0
canonical source modules traversed             = 0
```

Exact indexed reads of referenced canonical facts are allowed.

At P6 closure, the **in-memory compiler architecture is complete**:

```text
immutable dependency world
+
strictly local compilation overlay
```

Everything after P6 is persistence/loading/executable artifact work.

---

# P7 — `.telosc` Container + Persisted Semantic Compiler Image

## Purpose

Define a portable artifact format and create the first loadable `universe.telosc` containing enough module + semantic + query + metadata state to eliminate cold source semantic compilation.

The extension is explicitly:

```text
.telosc
```

Do not use `.telpkg` for this checkpoint.

The format should eventually be generic enough for:

```text
universe.telosc
dependency.telosc
application.telosc
```

but P7 should remain bounded to the canonical Universe use case.

---

## P7.1 Sectioned container from day one

Recommended conceptual layout:

```text
universe.telosc
│
├── HEADER
├── MODULE_WORLD
├── SEMANTIC_WORLD
├── QUERY_IMAGE
├── METADATA
├── EXECUTABLE        optional/absent in first P7 milestone
├── PRESENTATION      optional
├── SOURCE_MAPS       optional
└── DIGESTS
```

A sectioned format allows P8 to add/activate portable executable content without redesigning semantic serialization.

---

## P7.2 Header and compatibility identity

Conceptually include:

```text
format/schema version
semantic ABI/version
stable identity ABI/version
native surface ABI/version
compiler/code ABI where applicable
builtin package identity
Universe content/source digest
interface digest
semantic digest
capability bitmap
target ABI only for target-dependent executable sections
artifact content digest
```

Universe package identity should be a stable builtin coordinate, conceptually:

```text
builtin:universe
```

Do not persist transient IDs such as:

```text
WorkspaceId
ResolvedProjectId
SyntheticProjectId
MetadataPoolId
TypeStoreId
StaticValueStoreId
ClassId
ObjRef
```

---

## P7.3 Relocatable semantic graph

Do **not** `serde` the current `SemanticSnapshot` or `SemanticDb`.

Create explicit artifact DTOs.

Introduce artifact-local structural IDs as needed, conceptually:

```text
ArtifactTypeId
ArtifactKindId
ArtifactStaticValueId
ArtifactQueryProductId
```

plus stable semantic owners:

```text
StableDeclarationRef
StableCallableRef
StableFieldRef
StableImplRef
...
```

Loading performs:

```text
artifact-local IDs
    ↓
validated relocation/materialization
    ↓
new-process TypeStore / StaticValueStore / semantic aggregates
    ↓
CanonicalUniverseFormalImage
```

For Universe it may be efficient to recreate the canonical deterministic prefix, but disk correctness must not depend on yesterday’s raw process IDs.

---

## P7.4 Persist canonical module world

Persist enough `CanonicalModuleProducts` authority to avoid:

```text
Universe source parsing
BuiltinInterfaceBuilder work
canonical import resolution
canonical linking
topology reconstruction
```

Persist stable representations of:

```text
module identities
interfaces
imports/exports
linked namespace/read products
topology
module fingerprints/provenance
```

Do not require raw source merely to import/check user code.

---

## P7.5 Persist canonical semantic world

Persist authoritative downstream-required semantic facts, including the inventory settled in P5.

The artifact must support novel downstream user source without dependency source reanalysis.

---

## P7.6 Persist eligible immutable query products

Use P2 freeze eligibility as the conceptual starting point.

Persist only query products that are:

```text
immutable
dependency-closed
relocatable
semantically current under artifact identity
safe across processes
```

Do not persist workspace revisions/query states or mutable DB internals.

---

## P7.7 Persist canonical metadata

P4 provides the metadata authority and stable cross-pool identity contract.

Store canonical RuntimePublic metadata (or the profile explicitly chosen for canonical runtime authority) as an artifact section.

---

## P7 milestone

At P7 closure:

```text
Universe source
    ↓  artifact-build path only
universe.telosc
    ├── module image
    ├── semantic image
    ├── immutable query products
    └── canonical metadata
```

A fresh compiler process must be able to:

```text
load universe.telosc
    ↓
construct CanonicalUniverseFormalImage
    ↓
SemanticWorkspaceSession::from_image
    ↓
compile novel user source
```

with Universe source unavailable.

Mandatory counters:

```text
canonical source parses               0
canonical interface builds            0
canonical full semantic updates       0
canonical callable analyses           0
canonical result settlement           0
canonical witness rebuilds            0
canonical metadata export             0
```

This fixes **compiler semantic cold start**.

But do not claim final checkpoint completion yet: VM execution may still compile Universe AST unless P8 is complete.

---

# P8 — Portable Executable Universe Image

## Purpose

Eliminate VM-side Universe AST/source bytecode compilation.

Current repository fact:

```text
CanonicalUniverseProgram is VM-independent linked/lowered compiler state,
but runtime materialization/bytecode compilation remains VM-local.
```

Universe bootstrap reaches:

```rust
compile_ast_as_with_bindings(...)
```

This must disappear from artifact-backed ordinary startup.

---

## P8.1 Portable executable DTOs

Inventory all runtime compiler output containing VM-local handles/caches.

Do not serialize current VM objects directly.

Introduce VM-independent representations conceptually like:

```text
PortableModulePrototype
PortableCallablePrototype
PortableChunk
PortableInstructionStream
PortableConstant
PortableUpvalueSpec
PortableLinkedRead
PortableNativeRef
PortableRuntimeTypeRecipe
PortableRelocation
```

Current types such as `Chunk`, `Callable`, `Value`, caches, object refs, runtime class handles, etc. are not automatically safe artifact DTOs.

---

## P8.2 Separate code generation from VM materialization

Build-time path:

```text
Universe AST / checked lowering
    ↓
portable code emitter
    ↓
portable module/callable prototypes
    ↓
EXECUTABLE section of universe.telosc
```

VM startup path:

```text
EXECUTABLE section
    ↓
validate
    ↓
allocate fresh VM-local symbols/objects/chunks/closures/caches
    ↓
relocate stable/module/type/native references
    ↓
execute initializer closures
```

Never persist:

```text
heap
GC state
ObjRef
ClassId
live closures
inline caches
fibers
mutable module globals
```

---

## P8.3 Runtime semantics

Universe initializer **execution** remains per-VM and legitimate.

P8 removes only source parse / AST-to-bytecode compilation.

## P8 gate

Artifact-backed VM bootstrap:

```text
Universe AST compilations = 0
```

Existing test counter:

```text
UNIVERSE_AST_COMPILATIONS
```

is useful here.

At P8 closure, `universe.telosc` is a genuinely compiled Universe artifact.

---

# P9 — Artifact-First Loader, Cache, Toolchain Distribution & Final Certification

## Purpose

Make `universe.telosc` the ordinary source of canonical authority instead of an optional experiment.

P9 is where the final checkpoint becomes true operationally.

---

## P9.1 Canonical semantic loader cutover

Current:

```text
canonical_universe_formal_image()
    ↓
process OnceLock
    ↓
source build on first miss
```

Target:

```text
canonical_universe_formal_image()
    ↓
process OnceLock
    ↓
locate compatible universe.telosc
    ↓
validate
    ↓
load/relocate semantic image
    ↓
publish Arc<CanonicalUniverseFormalImage>
```

Source construction becomes:

```text
artifact generation
development fallback
explicit recovery path
```

not normal startup.

---

## P9.2 Canonical core program cutover

`canonical_universe_program()` should consume the artifact’s:

```text
module world
semantic world
metadata
portable executable prototypes
```

instead of running `ProgramCompiler` over Universe source.

---

## P9.3 VM bootstrap cutover

Replace:

```text
Universe parsed AST
    ↓
compile_ast_as_with_bindings
```

with:

```text
portable Universe executable prototype
    ↓
relocate/materialize
    ↓
fresh closure/chunk
    ↓
execute initializer
```

---

## P9.4 Physical artifact locations

Recommended production location:

```text
<Telos toolchain root>/lib/telos/artifacts/universe.telosc
```

Recommended content-addressed development/user cache:

```text
Linux:
  $XDG_CACHE_HOME/telos/artifacts/
  usually ~/.cache/telos/artifacts/

macOS:
  ~/Library/Caches/telos/artifacts/

Windows:
  %LOCALAPPDATA%\Telos\Cache\artifacts\
```

A useful internal layout:

```text
objects/
    <content-id>/
        universe.telosc

refs/
    universe/
        <compiler-or-semantic-abi> -> <content-id>
```

Cargo build staging may use:

```text
target/telos-artifacts/universe.telosc
```

but `target/` is not the runtime architectural location.

---

## P9.5 Lookup policy

Recommended order:

```text
1. explicit developer/toolchain override
2. artifact installed with active Telos toolchain
3. compatible content-addressed local cache
4. source rebuild only if current policy allows
5. atomically store successful rebuild in cache
```

Official releases should normally hit step 2.

---

## P9.6 Deterministic/atomic artifact production

Artifact generation must be deterministic.

Conceptual flow:

```text
build canonical world
    ↓
project to artifact DTOs
    ↓
stable-sort identity-indexed sections
    ↓
encode
    ↓
compute section/content digests
    ↓
validate encoded artifact
    ↓
write temp file
    ↓
atomic rename/install
```

Never publish partially-written `universe.telosc`.

---

## P9.7 Compatibility/invalidation

Do not key validity only by filename or compiler version string.

Compatibility/content identity must cover relevant inputs such as:

```text
.telosc format version
Universe source/content digest
semantic ABI
stable-identity ABI
native-surface ABI
compiler/code ABI
target/runtime ABI for target-specific executable image
other artifact capability/version inputs
```

A source-identical Universe may still require rebuild after semantic ABI or compiler executable-format changes.

---

# 14. Fundamental compiler optimization completion gate

P9 closes only when a **fresh process** compiling/running a novel user program with a compatible `universe.telosc` shows:

```text
canonical source parses                         0
canonical interface builds                      0
canonical source discovery                      0
canonical full semantic updates                 0
canonical callable-body analyses                0
canonical inferred-result settlement            0
canonical witness/conformance reconstruction    0
canonical metadata export                       0
canonical backend lowering                      0
Universe AST bytecode compilations              0
```

Allowed work:

```text
.telosc read/decode/validation
semantic relocation/materialization
canonical metadata registration
fresh VM heap/module/class/closure allocation
fresh mutable caches
Universe initializer execution
user-code compilation
```

---

# 15. Strongest acceptance test

Do not rely only on timing.

The strongest architectural test is:

```text
1. Build universe.telosc.

2. Make Universe .ph source unavailable.

3. Start a fresh compiler process.

4. Compile a novel user program that exercises:
   - multiple Universe modules
   - imports/re-exports
   - generics
   - traits
   - associated types
   - data/enums
   - static/type-level forms
   - conformance/evidence
   - reflection metadata where profile permits

5. Start a fresh VM.

6. Run the program.

7. Verify:
   - semantic parity
   - diagnostics parity where source-independent diagnostics apply
   - runtime parity
   - reflection/type identity parity
   - initializer behavior
   - zero canonical source semantic/compiler counters
   - zero Universe AST compilation
```

If this passes, the artifact is genuinely authoritative rather than a decorative cache.

---

# 16. `.telosc` format principles

Treat these as settled design constraints unless contradicted by implementation evidence.

## 16.1 Do not serialize in-memory Rust ownership graphs

Forbidden as the primary durable format:

```text
serde(CanonicalUniverseFormalImage)
serde(SemanticSnapshot)
serde(SemanticDb)
raw TypeStore dump with process IDs treated as stable
raw VM heap snapshot
```

The format is an explicit, versioned, portable projection.

---

## 16.2 Keep semantic and runtime identities separate

Persistent identity:

```text
stable declarations/callables/fields/impls/modules
artifact-local structural node IDs
content/ABI identities
```

Runtime-local identity:

```text
MetadataPoolId
TypeStoreId
StaticValueStoreId
ObjRef
ClassId
heap objects
mutable caches
```

Never conflate them.

---

## 16.3 Use section capabilities

The format should support:

```text
semantic-only artifact
semantic + metadata artifact
fully executable artifact
debug/presentation-enhanced artifact
```

P7 may initially produce no `EXECUTABLE` section.

P8 populates it.

This avoids a second artifact format.

---

## 16.4 Universe should use the ordinary artifact machinery where possible

Universe is special in:

```text
builtin identity
bootstrapping rules
toolchain distribution
canonical compatibility
```

but should not get a completely unrelated file format.

The future goal is that the same `.telosc` machinery can later support ordinary dependency artifacts.

General dependency/package caching can be a successor checkpoint; do not let it expand C2 indefinitely.

---

# 17. Source files/symbols the next agent should know

Do not re-locate these from scratch unless source moved.

## Modules / linking

```text
phalcom-modules/src/builtin.rs
  UniverseSourceProvider

phalcom-modules/src/builtin_interface.rs
  BuiltinInterfaceBuilder
  UniverseSourceDeclarationCatalog

phalcom-modules/src/linker.rs
  ModuleLinker
  with_external_modules
  LinkedProgram

phalcom-modules/src/session.rs
  WorkspaceModuleSession
  WorkspaceModuleUpdate

phalcom-modules/src/artifact.rs
  ResolvedPackageArtifact / package artifact boundary concepts

phalcom-modules/src/package_info.rs
  PackageArtifactIdentity / PackageInfoDescriptor
```

## Semantic canonical image

```text
phalcom-semantic/src/canonical_image.rs
  CanonicalUniverseFormalImage
  canonical_universe_formal_image
  canonical_universe_formal_image_with_provider
  CanonicalSemanticQueryImage
  CanonicalSemanticWorldId
  validate_publication

phalcom-semantic/src/session.rs
  SemanticWorkspaceSession::new
  ::with_workspace
  ::from_image
  ::from_foundation_for_image

phalcom-semantic/src/canonical_baseline.rs
  CanonicalUniverseSemanticBaseline

phalcom-semantic/src/db/mod.rs
  SemanticDb
  canonical read-through/origin

phalcom-semantic/src/request.rs
  SemanticProductRequest
  runtime_minimal
  full
  Default == full

phalcom-semantic/src/signature.rs
  CallableSignatureTable
  module/owner indexes

phalcom-semantic/src/impls.rs
  ConformanceIndex
  witness planning

phalcom-semantic/src/metadata/export.rs
  MetadataExporter
```

## Core compiler/program

```text
phalcom-core/src/modules/compile_session.rs
  ProgramCompilationSession::standalone
  compile_fixture

phalcom-core/src/modules/compile.rs
  ProgramCompiler
  CompiledProgram
  CompiledModule
  runtime metadata export roots

phalcom-core/src/modules/canonical_universe.rs
  CanonicalUniverseProgram
  canonical_universe_program
  build_canonical_universe_program

phalcom-core/src/modules/semantic_lowering.rs
  aggregate backend scans / P6 targets

phalcom-core/src/modules/artifact.rs
  ModuleMaterializationPlan and artifact-adjacent compiler DTOs

phalcom-core/src/modules/materialize.rs
  VM::materialize_program
  compile_program_module_closure
```

## VM/bootstrap

```text
phalcom-core/src/vm/bootstrap.rs
  canonical Universe bootstrap
  compile_universe_module
  UNIVERSE_AST_COMPILATIONS
  Universe initializer execution

phalcom-core/src/interpret.rs
  VM::run_compiled
  compile_closure_as_with_bindings
  compile_ast_as_with_bindings
```

## Runtime typing/metadata

```text
phalcom-core/src/typing/handle.rs
  MetadataPoolId
  RuntimeTypeRef

phalcom-core/src/typing/registry.rs
  RuntimeTypingRegistry

phalcom-core/src/typing/loader.rs

phalcom-core/src/typing/inspect.rs
phalcom-core/src/typing/reify.rs
phalcom-core/src/typing/identity.rs
phalcom-core/src/typing/environment.rs
```

---

# 18. Relevant checkpoint/spec documents

Current C2 documents live under:

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
```

Important files:

```text
CHECKPOINT.md
PERF003.C2-detailed-implementation-plan.md
PERF003-C2-Unified-Compiler-Artifact-Implementation-Spec.md
PERF003.C2-CORRECTED-IMPROVED-AUTHORITATIVE-ARCHITECTURE.md

PERF003.C2.P2-canonical-formal-image-and-immutable-query-roots-implementation-plan.md
PERF003.C2.P2-bootstrap-audit.md
PERF003.C2.P2-walkthrough.md
PERF003.C2.P2-handoff.md

PERF003.C2.P3-indexed-witness-lookup-implementation-plan.md
PERF003.C2.P3-walkthrough.md
PERF003.C2.P3-handoff.md
```

A P4 implementation plan was created during the preceding conversation titled:

```text
PERF003.C2.P4-profile-root-driven-metadata-reachability-implementation-plan.md
```

If the user provides/attaches it, treat it as the detailed P4 authority.

The current repository `CHECKPOINT.md` is historically stale in some plan-status/numbering details. Preserve evidence, but use the normalized P1–P9 progression in this handoff for forward planning unless the user directs otherwise.

The unified artifact specification is broad and valuable but was originally marked proposed. This handoff records the user's subsequent acceptance of the normalized `.telosc` path.

---

# 19. Recommended immediate work after P4

When P4 finishes, do **not** jump straight to serialization without one bounded source-state review.

The correct next sequence is:

```text
1. Audit P4 completion evidence.
2. Re-profile current HEAD.
3. Confirm current canonical cold-build subphase distribution.
4. Implement P5 canonical product/publication boundary.
5. Implement P6 user-only backend complexity closure.
6. Implement P7 semantic/compiler .telosc.
7. Implement P8 portable executable image.
8. Implement P9 artifact-first loader/cache/certification.
```

The P4 completion audit should be a **delta audit**, not a replay of the entire C2 investigation.

---

# 20. Cold canonical bootstrap measurements still needed

The current 44–45 s `SemanticSessionInit` measurement is too coarse for optimization decisions.

Break it into at least:

```text
CanonicalUniverseSemanticBaseline build
CanonicalModuleProducts::capture
topology construction
CanonicalImageBootstrap/session.update
declaration/signature publication
callable body analysis
result settlement
conformance/witness work
presentation
advisory
validate_publication
query freeze
from_image session attachment
```

Also measure in one process:

```text
A. first canonical_universe_formal_image()
B. second canonical_universe_formal_image()
C. SemanticWorkspaceSession::new() after image warm
D. another warm new()
```

Expected from current evidence:

```text
first image build      ~42–45 s
warm image lookup      tiny
warm session creation  ~9–12 ms
```

This confirms how much P5 can reduce versus how much P7 persistence must remove.

---

# 21. Performance methodology

Keep structural cardinality as the primary correctness/performance gate.

Wall-clock benchmarks alone are insufficient because:

```text
parallelism
hardware
build mode
debug/release
test failure position
cache state
```

can obscure the architecture.

Every package should define counters that prove unwanted work is absent.

Then measure wall time.

For current performance comparisons, always distinguish:

```text
cold artifact/image acquisition
warm process
session creation
semantic update
metadata export
backend lowering
VM bootstrap
initializer execution
user execution
```

Do not report a failed early test as a successful compile benchmark.

---

# 22. Important current test limitations

The recent timing samples are based on failing test sets.

Therefore:

```text
They prove phase timing behavior.
They do NOT prove successful end-to-end compilation/runtime speedup.
```

The next agent should preserve this distinction.

Current known failure areas in those samples:

```text
metadata-export errors
semantic type/dependency errors
```

P4 is expected to touch some metadata failures.

Do not silently classify failures as baseline or P4-owned without same-state evidence.

---

# 23. STOP / CONSULT conditions for P5–P9

Escalate rather than papering over these.

## Semantic artifact blockers

Stop if durable semantic reuse appears to require serializing raw process arena IDs without a proven relocation model.

Stop if downstream legal generic/conformance operations still require dependency raw source bodies and no portable typed-template contract exists.

Stop if stable semantic identities collide for legitimately distinct canonical/local entities.

## Executable artifact blockers

Stop if portable bytecode cannot represent some runtime compile product without serializing live VM handles.

Do not solve this by heap snapshotting.

Stop if generic/evidence specialization requires source AST at runtime; define a typed relocatable template/evidence ABI.

## Runtime bootstrap blockers

Stop if canonical initializers depend on process-global mutable compiler objects that cannot be reconstructed from portable prototypes.

Do not persist mutable caches or live closures.

## Coherence blockers

Stop if loading independently persisted semantic facts would allow conformance/coherence behavior different from source-backed canonical construction.

Artifact loading must recreate the same canonical semantic world, not bypass semantic invariants.

---

# 24. Non-goals for C2

Do not expand this checkpoint into all future package infrastructure.

C2 does **not** need to finish:

```text
general public package registry
remote dependency distribution
all project build caching
all third-party package artifact composition
full deployment bundling
VM heap snapshots
JIT/AOT native machine-code pipeline
new language semantics
reflection-policy redesign
```

It should establish `universe.telosc` as the first canonical compiled dependency.

Generalizing `.telosc` to ordinary dependency/package artifacts can be a successor checkpoint.

---

# 25. Final checkpoint narrative

The correct historical narrative is:

```text
P1–P3:
    separate immutable Universe dependency state from user work
    make local semantic updates cheap

P4:
    separate canonical metadata from local metadata
    make metadata root-driven

P5:
    finish canonical product profiles
    make canonical aggregates persistent/read-only parents
    define serialization authority

P6:
    remove residual whole-world backend scans
    prove user compilation is locally bounded

P7:
    persist module + semantic + query + metadata authority
    load universe.telosc instead of semantically compiling Universe

P8:
    persist portable executable Universe prototypes
    eliminate VM-side Universe AST compilation

P9:
    make artifact loading the normal toolchain/cache path
    certify clean-process source-free Universe boot
```

The final claim should be:

> **Universe is compiled once when producing the toolchain/cache artifact. Ordinary compiler processes load `universe.telosc`. Semantic analysis, metadata generation, lowering, and bytecode compilation are performed only for user-owned source; Universe startup performs only artifact validation/relocation, VM-local materialization, and initializer execution.**

That is the bounded endpoint of `PERF003.C2`.

---

# 26. What you should do in the fresh session

Start from this handoff.

Before proposing new architecture:

1. Verify the current branch/worktree revision.
2. Determine whether P4 has since been committed and inspect only its delta/walkthrough.
3. Read current `CHECKPOINT.md` for new evidence, but do not let historical plan-number drift override this normalized program.
4. Reuse all settled P1–P4 architecture above.
5. If asked to plan P5, begin with the serialization-readiness/publication boundary described here.
6. If asked for the whole rest-of-checkpoint plan, use P5–P9 exactly as normalized above, refining source references against current HEAD.
7. Do not re-run a broad “why is compilation 45 seconds?” investigation unless current evidence contradicts the established diagnosis.
8. Do not propose `.telpkg`; the user selected `.telosc`.
9. Do not declare the checkpoint complete at a semantic-only `universe.telosc`; VM-side Universe compilation must also be eliminated.
10. Keep all performance claims evidence-scoped and distinguish source-grounded fact, measured result, and architectural target.

Your job is to advance the program, not rediscover it.
