# PERF003.C2.P7 — `.telosc` Persisted Semantic Compiler Image
## Detailed Patch-Grade Implementation Plan

**Program:** `PERF003`  
**Checkpoint:** `PERF003.C2`  
**Normalized plan:** `PERF003.C2.P7`  
**Repository:** `aureat/phalcom-lang` (`phalcom`)  
**Target branch:** `perf/universe-characterization`  
**Planning baseline audited:** `92dfde0bbe4932e8543c1228fc3bd564476f4d8f`  
**Predecessors:** `PERF003.C2.P5` — Canonical Publication + Persistent Image Boundary; `PERF003.C2.P6` — User-Only Backend / Local Complexity Closure  
**Successor:** `PERF003.C2.P8` — Portable Executable Universe Image  
**Prepared:** 2026-10-13  
**Primary objective:** persist the canonical Universe compiler authority as a deterministic, versioned, relocatable `universe.telosc` artifact that a fresh compiler process can load without parsing, linking, semantically analyzing, or metadata-exporting Universe source.

## Adopted native-catalog compatibility amendment — 2026-10-15

The Level 2 adviser decision adds a fixed `[u8; 16] native_catalog_fingerprint`
to `CanonicalCompatibilityIdentity` and the container header. The header codec
is V2 and rejects V1 explicitly; section schemas remain unchanged. The
source-built compiler-image view captures the trusted catalog fingerprint, the
writer copies it into the header, and the loader compares it against the
current trusted catalog immediately after container parsing and before section
materialization. Loaded portable identity retains and compares the field.
Exact native key and transported symbolic payload checks remain required.
`catalog_fingerprint()` must not allocate during load; its implementation
streams the historical stable-key-ordered bytes without allocating a sorted
vector or formatted row strings.

Focused discriminators: V1 rejection; V2 matching-catalog round-trip; correctly
rehashed foreign fingerprint rejection before materialization; exact native-row
payload mismatch; portable compatibility comparison. The amendment changes no
semantic owner, identity, or native metadata representation.

## Adopted callable-analysis owner-graph amendment — 2026-10-15

The Level 2 adviser decision confirms that the full compiler-required
`CallableAnalysis` graph is mandatory. Execution summaries or signature-only
reconstruction do not satisfy T7.B/T8. Persist every owned expression and
binding product; iteration, expansion, sequence, associated, callable-reference,
family-application, match, and pattern-site resolution; CFG nodes/edges; entry
and exit flow; return validation; dependencies; constructor selection;
diagnostics; and the owned explanation graph. Preserve every semantic
judgment and reference. A complete top-level body may still contain unsuccessful
expression judgments or source diagnostics; do not turn them into successful
facts or require every expression to be Ready.

Portable coordinates are deterministic artifact-local ordinals qualified by
the stable callable owner, never source ranges or globally collected numeric
IDs. Export enumerates each owner and records relocation maps; import validates
owner-qualified definition/reference closure before publishing and explicitly
relocates all references into fresh process-local handles. Recompute
`callable_body_product_fingerprint` after relocation, then rebuild query
products and fingerprints from relocated owners. Preserve causal-diagnostic
sharing and exact statuses/reports. Canonical images reject nonempty internal
incidents and incident-bearing failure states. They preserve genuine empty
call-resolution/predicate optionals and reject nonempty handles until those
families have an actual canonical owner.

The compiler-required binding/site/target attachments used by lowering are
part of this graph's required reader contract. Rebuild one source-independent
formal reader from the exact body-owned binding/expression facts or persist an
equivalent owner-held attachment. Do not recover lexical resolution from names
or leave those binding maps empty. `source_index` remains omitted as a broad
source/tooling index; its compiler-required subset must be served by the
validated callable-analysis owner graph and reconstructed formal projection.

Required callable-analysis discriminators cover source/import field parity
modulo relocation; two-owner isolation with overlapping numeric IDs/ranges;
relocation under different fresh-store numbering; diagnostics, causal sharing,
explanations and blocked judgments; malformed cross-owner closure and
unowned-handle rejection; deterministic wire output and measured allocation;
and source-denied compilation through imported binding/target attachments.
T7.B, T8, T9, T12, the field manifest, and the remaining execution evidence
matrix are amended by this decision. No P7 acceptance gate is reduced.

Because this changes the `SEMANTIC_WORLD` record layout, advance its payload
schema from 1 to 2 and reject schema 1 explicitly. This is independent of the
already-adopted container header V2 and does not imply backward decoding of
incomplete callable-analysis records.

## Adopted callable binding storage-origin amendment — 2026-10-15

The Level 2 adviser decision requires explicit provenance for ambient module
binding handles retained in each callable analysis. At the existing
`CheckingContext::seed_linked_module_import_values` seam, preserve each
`BindingState` and record an owner-held storage origin: `BodyLocal`, or
`ModuleValue { symbol: SymbolId, import provenance }`. `SymbolId` is the
canonical storage authority; the fresh body-local `BindingId` remains only the
analysis handle. Parameters retain canonical parameter identity and the
constructor body/declaration mapping already adopted above remains unchanged.
Missing required origins fail closed.

The compiler's source-independent projection distinguishes
`BodyLocal { callable, binding }` from `ModuleValue { symbol }`. Source ranges
locate executable sites but do not establish identity. Module declarations
and authored import aliases enter lowering once through module-owned authority;
copied ambient states never become callable-local declarations. Exact reads
and writes retain their source occurrences and canonical global target.
Product-planner candidates, active products, and conflict checks compare the
qualified identity, so repeated module-symbol copies agree while equal numeric
`BindingId`s from different callables remain distinct. No name/range heuristic
or silent row omission is permitted.

Extend T7.B/T8/T9/T12 and the field/evidence manifests to encode and relocate
the origin with its callable owner. Required regressions cover ambient reuse,
unused copies, local shadowing, exact source/import projection parity,
cross-callable numeric-ID collisions, and malformed origin/conflict rejection.
This amendment adds no binding identity and reduces no P7 acceptance gate.

## Adopted `<main>` source-owner and attachment amendment — 2026-10-15

The same Level 2 adviser selected option A for top-level lexical declarations
whose checker facts belong to the synthetic module `<main>` analysis. Keep the
analysis `CallableId` unchanged and explicitly register
`CallableId → SourceOwner::Module(module)` at its producer. Generalize the
body-to-source-owner relation: ordinary bodies map to
`SourceOwner::Callable(source_callable)` and constructors retain the approved
declaration-owner mapping. Include registered module-body analyses when
constructing formal source attachments, keyed by the original analysis
`CallableId`.

A top-level lambda parameter such as `text` remains `BodyLocal` under its
`(<main> CallableId, BindingId)` and attaches to the exact module-owned
`SourceSiteId`. `SourceOwner::Module` denotes the source namespace; it does not
change the binding's storage origin. Keep storage origin, source owner,
lexical scope, body-fact identity, and declaration-site identity distinct.
Correct `<main>`'s body range from the executable module statement/root extent,
not `program.preamble.range`; it must cover module expressions and nested
bodies, including empty-preamble modules. Transport and relocate binding,
expression, and source-site references together through callable-analysis and
compiler-facts artifacts. Do not emit a normal callable declaration/signature
row for synthetic `<main>`.

Do not select the relation later by a `<main>` selector-name test, merge
formal-expression and module AST sites by matching ranges, or use count/order
rebasing for module bodies. Exact declaration attachment resolves to the
source index's `SourceSiteId`; ranges remain lookup locations. Required narrow
discriminators: two top-level lambdas each have a `text` parameter, distinct
exact module-owned sites, and reads resolving to their own parameter; ordinary
method module-value reads remain `ModuleValue`; source/import owner parity; and
successful lowering with empty and documentation-bearing preambles. The exact
Universe artifact test observed before this amendment fails at the
`universe:json` `<main>` `text` binding with
`MissingFormalBindingAttachment`; no later loader or source-denied evidence
follows.

## Adopted explicit Dynamic call-environment amendment — 2026-10-15

The focused `Box<Dynamic>.ping()` versus `Box<Int>.ping()` discriminator and
Level 2 adviser decision confirm that the receiver environment is the
canonical argument authority. The Dynamic case selects the same exact
`Box.ping` callable as the Int control, and its owner parameter is bound as
`StaticArgument::DynamicType`; the defect is the TypeId-only read in
`finalize_owner_call_specialization`. Preserve concrete, Dynamic, and row
categories distinctly through `CallSpecialization` (reuse `StaticArgument`
where it fits), checked `GenericInstantiation`, generic conformance discharge,
callable-analysis DTOs, compiler projections, and runtime call environments.
Exactly one category may be present for a parameter. A genuinely absent
required binding remains an error.

The generic type materializer must distinguish a proper resulting term from a
Dynamic boundary and from an absent/invalid binding. `List<T>` under
`T := Dynamic` remains the proper applied term `List<Dynamic>`; bare Dynamic
does not become a `TypeId`. A Dynamic argument supplies no static conformance,
subtype, or equivalence proof. Obligations independent of Dynamic may still be
discharged; no witness slot or runtime conformance check is synthesized from
Dynamic. New Dynamic inference, automatic runtime witness acquisition, and
unrelated native erasure behavior are out of scope and require separate
consultation if they become necessary.

Reuse the runtime's existing `RuntimeStaticArgument::{Term, RecordRow,
DynamicType}` meaning. Runtime generic call recipes wrap the proper-type
`RuntimeTypeRecipe` or carry Dynamic/row directly. Update call-root discovery,
covering-body closure, generic-instantiation/conformance-slot projection,
derived conditional environments, VM dispatch/call entry, bound/family call
recipes, environment interning, and recipe pool remapping. Explicit Dynamic
must remain distinct from a concrete type and from a missing entry. A request
for a nominal class from Dynamic returns a typed Dynamic boundary/failure; do
not use the runtime's existing missing-parameter `Special("Dynamic")`
fallback, Object, or another nominal substitute.

Persist the argument discriminant/payload in the existing semantic-world v2
call-specialization owner; term and row payloads use the existing relocation,
while Dynamic registers no type root. Validate category/kind agreement,
duplicate/conflicting entries, and owner closure. The call-specialization
argument must contribute to `callable_body_product_fingerprint`; update query
fingerprint ABI ingredients where needed. Do not change the already-versioned
container header again. Keep the exporter's canonical internal-incident guard.

Focused acceptance must prove: `Box<Int>` and `Box<Dynamic>` select the same
method and publish distinct exact arguments without incidents; cold/source-
import parity; forwarded `T := Dynamic` and nested `List<T>` preservation;
missing-required-binding rejection; no fabricated Dynamic conformance;
runtime environment interning distinguishes concrete, Dynamic, and missing;
fingerprint sensitivity to the call argument; and all four real Universe
incident sites are incident-free before rerunning the exact P7 artifact gate.
This is a bounded P7 prerequisite to R3/T7/T12, not a new checkpoint objective
or language semantic. General row-environment execution remains outside this
amendment; preserve existing supported row behavior and escalate any expansion.

### Typed call-argument transport progress — 2026-10-15

The wire slice now represents callable-analysis static call arguments as
owner-local expression ordinals with relocated type-parameter references and
tagged `Term`, `RecordRow`, or `DynamicType` payloads. Root registration skips
proper-type roots for Dynamic; the semantic-world validator checks ordinal
closure, unique parameters, selected-callable owner compatibility, and
kind/category agreement. A budgeted importer helper relocates the argument
tags. Focused semantic export/import-helper tests and the artifact codec tests
pass. The helper is not yet connected to a complete `CallableAnalysis`
reconstruction path, and nonempty conformance bindings are rejected because
their proof owner payload is not implemented. The whole callable-analysis
export guard and import rejection remain active; no source/import parity or
P7 completion is claimed.

---

# 0. Executive implementation decision

`PERF003.C2.P7` is **not** a serialization pass over `SemanticSnapshot`, `SemanticDb`, or `CanonicalUniverseFormalImage`.

P7 must introduce a real compiled semantic artifact boundary:

```text
artifact build process
────────────────────────────────────────────────────────────────────

Universe source
    ↓
P1–P5 canonical module + semantic authority
    ↓
P6 module/identity-indexed local-consumption authority
    ↓
explicit artifact projection
    ↓
portable MODULE_WORLD
+ portable SEMANTIC_WORLD
+ portable QUERY_IMAGE
+ canonical RuntimePublic METADATA
    ↓
versioned sectioned binary container
    ↓
universe.telosc


fresh compiler process
────────────────────────────────────────────────────────────────────

universe.telosc
    ↓
container validation
    ↓
compatibility + digest validation
    ↓
portable ID relocation/materialization
    ↓
source-free external module authority
+ fresh-process TypeStore
+ fresh-process StaticValueStore
+ semantic primary aggregates
+ reconstructed P6 secondary indexes
+ immutable canonical query image
+ canonical RuntimePublic metadata
    ↓
CanonicalUniverseFormalImage / compiler-image view
    ↓
SemanticWorkspaceSession::from_image(...)
    ↓
compile novel user source
```

At P7 closure, the explicit artifact-backed compiler path must perform **zero**:

```text
Universe source parses
Universe BuiltinInterfaceBuilder builds
Universe canonical link construction
Universe semantic workspace updates
Universe callable semantic analyses
Universe conformance/witness reconstruction
Universe metadata export
```

P7 deliberately stops before executable persistence. VM-local Universe AST/bytecode compilation remains P8. Default artifact discovery, cache installation, fallback policy, toolchain shipping, and automatic artifact-first startup remain P9.

---

# 1. Repository synchronization and activation gate

The planning baseline is not the implementation start revision. P5 and P6 are expected to advance after this plan was prepared.

Before changing source, the implementer MUST run:

```sh
git fetch origin
git status --short
git rev-parse HEAD
git rev-parse origin/perf/universe-characterization
git log --oneline --decorate -n 40 origin/perf/universe-characterization
```

Record in `PERF003.C2.P7-walkthrough.md`:

```text
P7_START_SHA
P5_CLOSE_SHA
P6_CLOSE_SHA
P4_METADATA_CLOSE_SHA
P5 persistence-inventory type/symbol
P5 canonical compiler-image metadata composition type/symbol
P6 final indexed table shapes
P6 final structural counter baseline
```

Then perform a **bounded predecessor delta audit** only over the P7-facing boundaries:

```text
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/*
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/static_value/*
phalcom-semantic semantic aggregate owners modified by P5/P6

phalcom-modules/src/canonical_products.rs
phalcom-modules/src/session.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/package_info.rs

phalcom-core/src/modules/canonical_universe.rs
phalcom-core/src/modules/compile.rs
phalcom-core metadata composition path

phalcom-type-meta/src/header.rs
phalcom-type-meta/src/identity.rs
phalcom-type-meta/src/bundle.rs
```

Do not repeat P1–P6 architectural investigations unless current source invalidates an explicit predecessor contract in this plan.

## 1.1 P7 entry ruling

P7 implementation may begin in slices before all predecessor certification is finished, but ownership is constrained:

```text
May begin early:
    T1 container/wire crate
    T2 portable compatibility identity
    module DTO design that does not touch active P5/P6 table representations

Must wait for P5.T8/T10 closure:
    final SEMANTIC_WORLD persistence census
    final METADATA composition
    final compiler-image capability manifest

Must wait for stable P6 table shapes:
    T7 semantic aggregate load constructors
    T8 P6 index reconstruction

Final P7 certification:
    requires P5 + P6 closure gates
```

If P5/P6 are actively rewriting the exact same table representation in the same worktree, stop and coordinate rather than land parallel abstractions.

---

# 2. Fixed predecessor contract

## 2.1 P5 contract consumed by P7

P7 assumes P5 closes the following:

```text
immutable canonical semantic parent authority
local overlay ownership for canonical-aware tables
zero canonical entry copies during image-backed local session creation
explicit local / merged / module iteration semantics
formal persistence inventory / capability classification
one query persistence classification authority
canonical RuntimePublic metadata composition boundary
```

P7 MUST preserve:

```text
canonical facts = immutable parent authority
local facts     = workspace publication
```

An artifact-loaded canonical image must behave like the source-built canonical parent, not like a local snapshot copied into a workspace.

## 2.2 P6 contract consumed by P7

P7 assumes P6 closes backend locality:

```text
callable analyses module-indexed
callable definitions module-indexed
declarations module-indexed
data semantics module-indexed
enum semantics module-indexed
trait requirement layouts trait-local
ImplId → inherent domain exact lookup
binding/source reverse-target lookup
local lowering does not linearly scan unrelated canonical authority
```

P7 must reconstruct or validate these properties after loading. Loading a semantic image quickly is not sufficient if the compiler then performs whole-Universe scans on every user compile.

## 2.3 P4 metadata contract consumed by P7

P7 assumes the canonical metadata authority is already:

```text
root/reachability driven
RuntimePublic (or the final approved canonical profile)
stable-identity based
cross-pool capable
validated for duplicate/conflicting stable identity
not dependent on a fixed MetadataPoolId
```

P7 transports this authority. P7 does not redesign metadata reachability or reflection semantics.

---

# 3. P7 deliverables

P7 closes only when the repository contains all of the following.

## 3.1 Wire/container deliverable

A new compiler-artifact boundary, preferably:

```text
phalcom-compiler-artifact/
```

owning:

```text
.telosc fixed header
section directory
section identifiers
section/version flags
portable artifact-local IDs
portable compatibility identity
artifact digest type
validation limits
binary codec
structural decode/validation errors
MODULE_WORLD DTOs
SEMANTIC_WORLD DTOs
QUERY_IMAGE DTOs
METADATA section wrapper
reserved EXECUTABLE section identity
```

## 3.2 Projection deliverables

```text
phalcom-modules
    source compiler objects → MODULE_WORLD
    MODULE_WORLD → source-free external canonical module authority

phalcom-semantic
    TypeStore ↔ portable type graph
    StaticValueStore ↔ portable static-value graph
    semantic primary aggregates ↔ SEMANTIC_WORLD
    canonical query image ↔ QUERY_IMAGE
    reconstructed current-process query fingerprints/dependencies
    loaded canonical image construction

phalcom-core
    canonical semantic image + canonical metadata composition
    explicit artifact writer
    explicit artifact loader
    source-free compiler integration test path
```

## 3.3 Generated artifact deliverable

A deterministic test/toolchain-build artifact:

```text
universe.telosc
```

P7 need not commit the generated binary to source control unless repository policy explicitly requires it. The build API/command and artifact summary must be recorded.

## 3.4 Certification deliverable

A fresh-process test where Universe source access is fail-fast/unavailable and novel user source still compiles successfully using the loaded artifact.

---

# 4. Fixed architectural invariants

## INV-P7-01 — `.telosc` contains portable DTOs, not runtime/compiler snapshots

Forbidden persistent roots:

```text
SemanticSnapshot
SemanticDb
CanonicalUniverseFormalImage
WorkspaceModuleSession
ProgramCompilationSession
TypeStore as raw serde state
StaticValueStore as raw serde state
```

Artifact DTOs must represent semantic meaning independently of mutable implementation ownership.

## INV-P7-02 — Process allocation coordinates are never persistent identity

Never encode as durable identity:

```text
WorkspaceId
SemanticRevision
SnapshotId
TypeStoreId
StaticValueStoreId
ResolvedProjectId
SyntheticProjectId
MetadataPoolId
RuntimeOverlayTypeId
RuntimeOverlayKindId
ClassId
ObjRef
pointer addresses
```

These are recreated in the receiving process.

## INV-P7-03 — Artifact-local graph IDs are explicit types

Use explicit names such as:

```rust
ArtifactTypeId
ArtifactKindId
ArtifactTypeParameterId
ArtifactRowId
ArtifactFamilyTypeId
ArtifactTypeLambdaId
ArtifactStaticDomainId
ArtifactStaticConstructorId
ArtifactStaticValueId
ArtifactQueryId
```

Do not expose raw `u32` as an ambiguous cross-table ID.

## INV-P7-04 — Stable semantic ownership uses stable refs

Reuse the existing stable semantic identity vocabulary from `phalcom-type-meta` where semantically equivalent:

```text
StableProjectRef
StableModuleRef
StableDeclarationRef
StableCallableRef
StableFieldRef
StableImplRef
StableTypeMemberRef
stable static-domain/member identities
```

Do not invent a third public Universe/declaration identity scheme.

## INV-P7-05 — Source is not compiler authority after load

An artifact-backed compiler must not require raw Universe source, parsed AST, or `UniverseSourceProvider` in order to:

```text
resolve imports into Universe
consume Universe interfaces
perform user semantic checking
lookup canonical declarations/callables/fields
evaluate supported canonical semantic products
lower user modules
consume canonical RuntimePublic metadata
```

## INV-P7-06 — Optional tooling capabilities are explicit

If source text, source maps, presentation products, or diagnostics are retained, expose them through capability bits/optional sections. Their absence must not make the compiler semantic image unusable.

## INV-P7-07 — P6 secondary indexes are semantic derivations

Do not serialize every in-memory secondary index merely because it exists. Persist primary authority; rebuild deterministic indexes during load unless an index is itself semantic identity/ordering authority.

## INV-P7-08 — Query persistence has exactly one eligibility authority

`canonical_query_persistence(kind)` remains the semantic eligibility authority. P7 may further decide whether an eligible family is present in the selected artifact capability profile, but it must not maintain an independent contradictory classification of open-world/session-local query semantics.

## INV-P7-09 — Canonical metadata remains `phalcom-type-meta`

The `METADATA` section stores the existing `SemanticMetadataBundle` model or its explicitly versioned binary representation. Do not create a second compiler metadata schema.

## INV-P7-10 — One `.telosc` artifact represents one semantic world

Header, module section, semantic section, query section, and metadata section must all cross-check the same canonical compatibility identity/digests.

## INV-P7-11 — Artifact construction is deterministic

Given the same canonical inputs and artifact schema/ABI versions:

```text
same section ordering
same entity ordering
same encoded bytes
same section digests
same whole-artifact digest
```

Host path, process IDs, allocation order, pointer address, thread scheduling, wall-clock time, or hash-map iteration must not affect bytes.

## INV-P7-12 — P7 semantic artifact is target-independent

P7 contains no target-specific executable machine/VM state. `EXECUTABLE` is reserved and absent. Target ABI enters P8 when executable sections become target-sensitive.

## INV-P7-13 — Decode is fail-closed and resource-bounded

Malformed, incompatible, oversized, overlapping, referentially invalid, or digest-invalid artifacts are rejected before publication.

## INV-P7-14 — Artifact load performs no semantic analysis

The loader may:

```text
parse container bytes
validate schemas/digests/capabilities
allocate stores/tables
relocate IDs
rebuild deterministic secondary indexes
validate referential/coherence invariants
recompute current-process fingerprints from loaded products
```

The loader must not:

```text
parse Telos source
run SemanticWorkspaceSession::update over Universe
resolve Universe imports from source
rebuild callable bodies semantically
recompute witness plans from source
re-export canonical metadata
```

## INV-P7-15 — P7 exposes an explicit loader; P9 owns default cache policy

Do not make the current `canonical_universe_formal_image()` silently search arbitrary filesystem locations in P7. Add an explicit artifact load path suitable for tests/build tooling. P9 owns production discovery/fallback/distribution policy.

---

# 5. Preferred layering and dependency architecture

Create a new workspace crate unless predecessor implementation establishes an equivalent dedicated wire crate:

```text
phalcom-compiler-artifact
```

Preferred dependency direction:

```text
phalcom-type-meta
        │
        ▼
phalcom-compiler-artifact
        │
        ├───────────────┐
        ▼               ▼
phalcom-modules    phalcom-semantic
        │               │
        └───────┬───────┘
                ▼
           phalcom-core
```

The precise Cargo graph may require the artifact crate to depend only on lower-level common/stable identity crates while `phalcom-modules` and `phalcom-semantic` own conversion implementations. Preserve the no-cycle invariant even if the arrows above need a concrete adjustment.

## 5.1 `phalcom-compiler-artifact` owns

```text
wire primitives
container header/directory
section schemas
portable IDs
portable compatibility identity
artifact digest
codec
validation limits
structural validation
portable DTOs
```

It MUST NOT own:

```text
SemanticWorkspaceSession
SemanticDb mutation
TypeStore mutation logic
StaticValueStore mutation logic
ModuleResolver
ProgramCompiler
RuntimeTypingRegistry
VM
```

## 5.2 `phalcom-modules` owns

```text
ModuleId / interface / linked-product projection into MODULE_WORLD
stable Universe module mapping
source-free external canonical module authority
module-world import validation
WorkspaceModuleSession / resolver consumption of external canonical authority
```

## 5.3 `phalcom-semantic` owns

```text
TypeStore artifact projection/import
StaticValueStore artifact projection/import
semantic aggregate projection/import
P5 persistence inventory enforcement
P6 index reconstruction on semantic tables
QUERY_IMAGE projection/import
CanonicalUniverseFormalImage construction from loaded authorities
```

## 5.4 `phalcom-core` owns

```text
formal semantic image + canonical metadata composition
artifact-build orchestration
explicit artifact load orchestration
compiler-facing loaded-image handle
fresh-process source-free compiler proof
```

This preserves the P5 layering rule:

```text
phalcom-semantic ─X→ phalcom-core
```

---

# 6. `.telosc` container schema v2 specification to implement

P7 must freeze a sectioned container before semantic DTO volume is added.

## 6.1 Required section identities

```text
MODULE_WORLD     required
SEMANTIC_WORLD   required
QUERY_IMAGE      required for CanonicalCompilerImageV1 capability
METADATA         required for canonical RuntimePublic capability
PRESENTATION     optional
SOURCE_MAPS      optional
EXECUTABLE       reserved, absent in P7
```

Recommended fixed IDs are stable integers/4-byte tags defined explicitly in the wire crate. Never derive IDs from Rust enum discriminants.

## 6.2 Fixed header

Implement an explicit wire header conceptually equivalent to:

```rust
pub struct TeloscHeaderV2 {
    pub container_schema: u32,
    pub minimum_reader_schema: u32,
    pub package_identity: TeloscPackageIdentity,

    pub semantic_model_version: u32,
    pub stable_identity_schema: u32,
    pub module_semantic_abi: u32,
    pub semantic_artifact_abi: u32,
    pub query_fingerprint_abi: u32,

    pub native_surface_schema_version: u32,
    pub native_catalog_fingerprint: [u8; 16],
    pub type_metadata_schema_version: u32,

    pub capabilities: TeloscCapabilities,

    pub source_digest: ArtifactDigest,
    pub interface_digest: ArtifactDigest,
    pub semantic_digest: ArtifactDigest,
    pub query_digest: ArtifactDigest,
    pub metadata_digest: ArtifactDigest,
    pub artifact_digest: ArtifactDigest,

    pub section_count: u32,
}
```

Exact field names may adapt to repository conventions, but every semantic purpose above must remain explicit.

## 6.3 Wire primitive rules

Specify and test:

```text
magic bytes: fixed 8-byte sequence
endianness: fixed little-endian or fixed big-endian; choose once and test
integer widths: u8/u16/u32/u64 only as specified
length prefixes: explicit bounded u32/u64
strings: UTF-8, length-delimited
booleans: explicit 0/1
optional values: explicit tag
variant tags: explicit stable integer/tag table
alignment: explicit; no host struct padding
```

Never serialize:

```text
usize
isize
native endian integer bytes
Rust enum discriminants
raw struct memory
```

## 6.4 Section directory

Implement:

```rust
pub struct SectionDirectoryEntryV1 {
    pub kind: SectionKind,
    pub schema_version: u32,
    pub flags: SectionFlags,
    pub offset: u64,
    pub encoded_len: u64,
    pub decoded_len: u64,
    pub digest: ArtifactDigest,
}
```

Decode validation order:

```text
1. header/magic/schema bounds
2. section_count bounds
3. directory range bounds
4. duplicate required section detection
5. offset + length overflow
6. out-of-file range rejection
7. overlapping section rejection
8. required/optional unknown-section rule
9. per-section encoded-size budget
10. section digest verification
11. section schema support
12. section DTO decode
13. section semantic/referential validation
14. cross-section digest/identity validation
```

## 6.5 Unknown-section rule

```text
unknown optional section → skip
unknown required section → reject
```

## 6.6 EXECUTABLE reservation

Define a stable section identity and capability bit now, but P7 writer must emit:

```text
EXECUTABLE absent
TeloscCapabilities::PORTABLE_EXECUTABLE == false
```

P7 must not persist current `Chunk`, `Callable`, `Value`, live closures, heap objects, inline caches, `ClassId`, `ObjRef`, or runtime registries.

---

# 7. Artifact digest and compatibility design

## 7.1 Distinguish semantic fingerprints from artifact integrity

Existing `phalcom-type-meta::Fingerprint128` and compiler `InputFingerprint`/`ProductFingerprint` remain semantic/internal fingerprints.

Introduce a separate:

```rust
pub struct ArtifactDigest([u8; 32]);
```

with a versioned digest algorithm. Preferred v1 choice:

```text
BLAKE3-256, domain-separated by container/section purpose
```

If repository policy chooses another 256-bit digest before schema v2 freezes, record the decision in the format spec and tests. Do not silently change the digest algorithm after v2 artifacts exist.

## 7.2 Required digest domains

At minimum:

```text
source/content digest
interface/module-world digest
semantic-world digest
query-image digest
metadata digest
whole-artifact digest
```

Recommended domain separators:

```text
telos.telosc.v1.source
telos.telosc.v1.module-world
telos.telosc.v1.semantic-world
telos.telosc.v1.query-image
telos.telosc.v1.metadata
telos.telosc.v1.artifact
```

## 7.3 Whole-artifact digest rule

Do not self-hash the mutable digest field naively. Define one canonical algorithm, for example:

```text
artifact_digest = digest(
    fixed header with artifact_digest bytes zeroed
    + canonical section directory
    + section bytes in directory order
)
```

Test exact reproducibility.

## 7.4 Portable compatibility identity

Introduce a semantic identity suitable for loaded and source-built images, conceptually:

```rust
pub struct CanonicalCompatibilityIdentity {
    pub package: TeloscPackageIdentity,
    pub source_digest: ArtifactDigest,
    pub interface_digest: ArtifactDigest,
    pub semantic_digest: ArtifactDigest,
    pub semantic_model_version: u32,
    pub stable_identity_schema: u32,
    pub module_semantic_abi: u32,
    pub semantic_artifact_abi: u32,
    pub native_surface_schema_version: u32,
    pub native_catalog_fingerprint: [u8; 16],
    pub type_metadata_schema_version: u32,
}
```

`CanonicalSemanticWorldId` may continue to exist as an in-process provenance handle, but artifact-loaded compatibility must not require retained source-text maps or `CARGO_PKG_VERSION` equality alone.

Required compatibility tests:

```text
same source-built world → same portable compatibility identity
private provider source mutation → source/semantic identity changes
same .telosc loaded twice → compatible world identity
semantic digest difference → incompatible world
native surface schema mismatch → reject
semantic model mismatch → reject
metadata schema unsupported → reject
producer version match alone → insufficient
```

---

# 8. Stable package/module identity plan

Reuse existing durable vocabulary.

## 8.1 Universe package mapping

Canonical Universe must map consistently between:

```text
PackageArtifactIdentity::Builtin("universe")
StableProjectRef::Builtin { namespace: "universe", version: ... }
ProjectIdentity::Universe            // runtime/compiler-local materialization
```

Do not introduce another externally visible Universe identity.

## 8.2 Portable module identity

Wire module identity is:

```text
stable package/project identity
+
logical ModulePath components
```

For P7 Universe loading:

```text
StableProjectRef::Builtin(universe) → ProjectIdentity::Universe
```

is a deterministic relocation.

## 8.3 Never persist graph-node project IDs

Do not encode `ResolvedProjectId` or `SyntheticProjectId` as durable module identity. Future dependency/application `.telosc` support will map stable package identity into a receiving process's project graph independently.

---

# 9. P7 task graph

```text
P7.T0   Bind to P5/P6 closing revisions + persistence census
   ↓
P7.T1   Artifact crate + binary container/header/directory/limits
   ↓
P7.T2   Portable compatibility + stable package/module identity
   ↓
P7.T3   Split source-free external canonical module authority
   ↓
P7.T4   MODULE_WORLD DTO projection/import
   ↓
P7.T5   TypeStore portable graph + relocation
   ↓
P7.T6   StaticValueStore portable graph + relocation
   ↓
P7.T7   SEMANTIC_WORLD DTO projection/import from P5 inventory
   ↓
P7.T8   Rebuild/validate P6 indexes + trait layouts
   ↓
P7.T9   Portable QUERY_IMAGE projection/import using P5.T9
   ↓
P7.T10  METADATA section from P4/P5 canonical authority
   ↓
P7.T11  Deterministic writer / artifact-build path
   ↓
P7.T12  Explicit loader → canonical compiler image
   ↓
P7.T13  Fresh-process source-unavailable user compile
   ↓
P7.T14  Corruption/compatibility/determinism/cardinality/performance gates
   ↓
P7.T15  Lifecycle closure + P8 handoff
```

Independent implementation branches are permissible only when they do not duplicate DTO/identity ownership. T1/T2 can proceed in parallel with late P6 work; T7/T8 cannot safely freeze until predecessor table shapes are final.

---

# 10. P7.T0 — Bind to predecessor closures and freeze the persistence census

## Purpose

Prevent P7 from encoding stale P5/P6 intermediate structures.

## Primary files

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md
    PERF003.C2.P5-handoff.md
    PERF003.C2.P6-handoff.md
    PERF003.C2.P7-walkthrough.md

phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/snapshot.rs
P5/P6 aggregate owners
```

## Required actions

1. Pin `P7_START_SHA`, `P5_CLOSE_SHA`, `P6_CLOSE_SHA`, `P4_METADATA_CLOSE_SHA`.
2. Read the final P5 persistence inventory/capabilities.
3. Produce a machine-checked or reviewable exhaustive table over every `SemanticSnapshot` field and every canonical compiler-image product family.
4. Record the final P6 primary/secondary table representations and module-index APIs.
5. Record P5/P6 structural acceptance counter baselines.
6. Confirm canonical metadata is constructible and accessible without a semantic rebuild.

## Mandatory census columns

```text
family
owner file/type
P5 persistence class
compiler-required after load? yes/no
wire authority DTO
stable owner identity
artifact-local IDs referenced
load constructor
secondary indexes rebuilt
optional capability dependency
query products depending on family
verification test
```

## STOP conditions

Stop if any compiler-required canonical family has no P5 persistence classification.

Stop if P6 still changes the owning primary table shape in a way that would change the DTO/load contract.

## Exit gate G7.T0

There is no unclassified compiler-required `SemanticSnapshot`/canonical-image authority entering the artifact design.

---

# 11. P7.T1 — Introduce `phalcom-compiler-artifact` and container v2

## Purpose

Land the wire boundary independently of semantic content.

## Files

```text
Cargo.toml
phalcom-compiler-artifact/Cargo.toml
phalcom-compiler-artifact/src/lib.rs
phalcom-compiler-artifact/src/container.rs
phalcom-compiler-artifact/src/header.rs
phalcom-compiler-artifact/src/section.rs
phalcom-compiler-artifact/src/digest.rs
phalcom-compiler-artifact/src/codec.rs
phalcom-compiler-artifact/src/limits.rs
phalcom-compiler-artifact/src/error.rs
phalcom-compiler-artifact/src/identity.rs
```

Names may adapt, but keep container/codec/DTO concerns separate enough for review.

## T1.A — Fixed schema constants

Define:

```rust
TELOSC_MAGIC
TELOSC_CONTAINER_SCHEMA_V2
TELOSC_MIN_READER_SCHEMA_V2
```

Define stable `SectionKind` numeric values and required/optional flags.

## T1.B — Binary reader/writer primitives

Implement bounded helpers for:

```text
u8/u16/u32/u64
fixed byte arrays
length-delimited bytes
UTF-8 strings
arrays/vectors with count budgets
explicit tagged enums
optional values
```

Every allocation from decoded length must check `ValidationLimits` first.

## T1.C — Header + directory codec

Implement encode/decode independent of Rust struct layout.

## T1.D — Structural validation

Reject:

```text
bad magic
unsupported container schema
section count overflow
missing required section
duplicate required section
duplicate singleton section
offset overflow
length overflow
out-of-file section
overlapping sections
unknown required section
section budget violation
section digest mismatch
```

## T1.E — Minimal round-trip fixture

Construct a container with mock required sections and prove deterministic byte round trip.

## T1 tests

```text
minimal_container_round_trips
header_is_fixed_endian
unknown_optional_section_is_skipped
unknown_required_section_is_rejected
duplicate_required_section_is_rejected
overlapping_sections_are_rejected
offset_overflow_is_rejected
section_size_limit_is_enforced
section_digest_mismatch_is_rejected
whole_artifact_digest_is_deterministic
```

## T1 exit

No semantic crates need to compile against DTOs yet; container format and safety rules are independently tested.

---

# 12. P7.T2 — Portable identity, compatibility, and digest domains

## Purpose

Create the cross-process identity layer before semantic graphs depend on it.

## Primary files

```text
phalcom-compiler-artifact/src/identity.rs
phalcom-compiler-artifact/src/header.rs
phalcom-compiler-artifact/src/digest.rs
phalcom-type-meta/src/identity.rs          // only if a missing stable identity is proven
phalcom-modules/src/package_info.rs        // mapping helpers only
phalcom-semantic/src/canonical_image.rs
```

## T2.A — Portable package identity

Represent builtin Universe using the existing package/stable-project semantics. Add conversion helpers rather than duplicating identity models.

## T2.B — Portable compatibility object

Add the cross-process compatibility structure and make source-built canonical-image construction derive it. Include `native_catalog_fingerprint: [u8; 16]` and compare it in portable compatibility checks. Capture it from the trusted catalog when building the source-backed compiler-image view; loaded identity retains the value after the early reader check.

Do not remove the in-process safety checks used by current readers; layer portable compatibility underneath/alongside them.

## T2.C — Canonical world refactor

Refactor `CanonicalSemanticWorldId` so equality/origin of an artifact-loaded world does not require full retained source text.

Recommended separation:

```text
CanonicalCompatibilityIdentity   portable semantic compatibility
CanonicalSemanticWorldId         current-process provenance/origin token
```

A loaded world may create a new `CanonicalSemanticWorldId` that embeds/references the portable compatibility identity.

## T2.D — Stable owner conversion

Provide exhaustive conversion helpers for every stable semantic owner required by P5 inventory/P7 DTOs:

```text
ModuleId ↔ StableModuleRef
DeclarationId ↔ StableDeclarationRef
CallableId ↔ StableCallableRef
FieldId ↔ StableFieldRef
ImplId ↔ StableImplRef
Variant/static-domain/type-member/etc. equivalents
```

If a current semantic identity has no faithful stable representation, STOP instead of encoding a process-local surrogate.

## T2 tests

```text
universe_module_stable_round_trip
stable_declaration_round_trip
stable_callable_owner_variants_round_trip
stable_impl_round_trip
same_world_same_compatibility_identity
source_change_changes_identity
abi_change_rejects_identity
artifact_loaded_twice_is_compatible
```

---

# 13. P7.T3 — Split source-free external module authority from source-backed build products

## Purpose

Remove the current architectural requirement that an external canonical module image also owns parsed source and a `UniverseSourceProvider`.

## Current coupling to remove

At the planning baseline:

```text
CanonicalUniverseFormalImage
    provider: UniverseSourceProvider
    modules: Arc<CanonicalModuleProducts>

CanonicalModuleProducts
    linked
    parsed sources
    unlinked interfaces
    source fingerprint

SemanticWorkspaceSession::from_image
    → WorkspaceModuleSession::with_external_modules(provider, products)
```

This cannot be the artifact-loaded dependency model.

## Primary files

```text
phalcom-modules/src/canonical_products.rs
phalcom-modules/src/session.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/session.rs
phalcom-core/src/modules/canonical_universe.rs
```

## T3.A — Introduce source-free module authority

Preferred conceptual shape:

```rust
pub struct CanonicalExternalModuleWorld {
    linked: Arc<LinkedProgram>,
    interfaces: BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>,
    topology: Arc<ModuleTopology>,
    // final P5/P6/P7-required module fingerprints/products
}
```

Exact fields must be driven by local compiler consumers and T0 census.

Keep source-backed build provenance separate, for example:

```text
CanonicalModuleBuildProducts
    source-free external world
    parsed sources
    source provenance
```

or an equivalent explicit split.

## T3.B — Refactor module session construction

Target API should no longer require a source provider to consume completed external modules:

```rust
WorkspaceModuleSession::with_external_modules(
    external: Arc<CanonicalExternalModuleWorld>,
)
```

If a provider remains for local/workspace sources, it must not be the canonical Universe source authority.

## T3.C — Resolver precedence

When an import resolves to an external canonical module:

```text
1. consult external module authority
2. return completed external interface/module identity
3. do not call UniverseSourceProvider
4. do not run BuiltinInterfaceBuilder
5. do not recursively discover source
```

## T3.D — Preserve source-built core bootstrap temporarily

P8 has not yet removed VM-side source/AST compilation. Existing `canonical_universe_program()` may still require source-backed build material in its source-build path.

Do not force P7 artifact-loaded semantic compiler mode to provide that source. Keep source-backed P8 preparation as a separate capability/path until P8 replaces it.

## T3 tests

```text
external_universe_module_is_terminal_without_source_provider
external_interface_lookup_does_not_build_builtin_interface
external_import_does_not_discover_parsed_source
source_backed_canonical_build_still_operates
artifact_semantic_path_has_no_source_capability
```

## T3 exit gate

`SemanticWorkspaceSession::from_image()` can consume a canonical external module world without receiving `UniverseSourceProvider`.

---

# 14. P7.T4 — Define, project, and load `MODULE_WORLD`

## Purpose

Persist everything local module discovery/linking needs from canonical Universe without source reconstruction.

## DTO ownership

```text
phalcom-compiler-artifact/src/module_world.rs
```

Conversion ownership:

```text
phalcom-modules/src/compiler_artifact.rs
```

or equivalent module-local file.

## Required payload families

At minimum preserve, after T0 proof:

```text
stable module identity
module kind
public/exposed child surface
unlinked interface declaration/import/export surface
linked interface
linked reads/binding identities required by local linking/lowering
resolved canonical import targets if required as external authority
canonical module topology/runtime graph information needed by local compiler
public interface fingerprint
private linkage/dependency fingerprint when it affects local link validity
module order where semantically observable
```

Raw source ranges may be omitted unless required by an optional source/presentation capability.

## Projection algorithm

```text
CanonicalExternalModuleWorld
    ↓ stable identity conversion
sort by StableModuleRef
    ↓ explicit DTO projection
ModuleWorldV1
    ↓ deterministic validation
encode
```

Never iterate a `HashMap` directly into wire order.

## Load algorithm

```text
ModuleWorldV1
    ↓ structural validation
stable builtin project → ProjectIdentity::Universe
stable module path → ModuleId
    ↓ interface/link DTO validation
construct CanonicalExternalModuleWorld
```

Do not invoke `ModuleLinker` to recompute the canonical linked world from source interfaces on load. Decode the already-authoritative linked products and validate referential consistency.

## Required validation

```text
unique stable module IDs
root Universe module present
all import/export module refs resolve
linked read targets resolve
module kind/interface invariants hold
interface fingerprints match DTO content or validated persisted fingerprint ABI
runtime/topology references resolve
no duplicate symbol identities inside a module surface
```

## T4 acceptance counter gate

Artifact-backed user compile:

```text
canonical source parses          == 0
canonical interface builds       == 0
canonical link builds            == 0
```

---

# 15. P7.T5 — Portable `TypeStore` graph and relocation

## Purpose

Recreate canonical type/kind semantics in a fresh process without persisting `TypeStoreId`.

## Primary files

```text
phalcom-compiler-artifact/src/semantic/type_store.rs
phalcom-semantic/src/types/artifact.rs          // preferred conversion owner
phalcom-semantic/src/types/store.rs             // narrow import/export APIs only
```

## T5.A — Artifact-local IDs

Define distinct IDs for all indexed arenas referenced by canonical type semantics, including at least:

```text
ArtifactTypeId
ArtifactKindId
ArtifactTypeParameterId
ArtifactScopedTypeId / ArtifactTypeLambdaId as required
ArtifactRecordRowId
ArtifactFamilyTypeId
ArtifactVariantTypeId
ArtifactStaticDomainId
ArtifactStaticConstructorId
```

## T5.B — Portable node DTOs

Mirror semantic meaning, not `TypeStore` private storage layout. Cover all `TypeData`, `KindData`, lambda/scoped type, row, family, parameter, variance, variant, static-domain and static-constructor cases present in the final canonical store.

Use stable owner refs for semantic owners and artifact-local IDs for graph edges.

## T5.C — Export context

Provide an exporter that walks the finalized canonical prefix and converts every store-local reference through explicit maps.

Record counts:

```text
types
kinds
parameters
lambdas/scoped nodes
rows
families
variants
static domains
static constructors
```

## T5.D — Import context

Create a fresh `TypeStore` with a new process-local `TypeStoreId` and build an explicit relocation map:

```text
ArtifactTypeId              → TypeId
ArtifactKindId              → KindId
ArtifactTypeParameterId     → TypeParameterId
...
```

Do not rely on `ArtifactTypeId(n) == TypeId(n)` even if v1 construction order currently makes this true. Relocation must remain explicit and tested.

## T5.E — Interner restoration

Use normal validated interning/bulk-import APIs so:

```text
by-content lookup works after load
canonical prefix can be forked by TypeStore::fork_from_baseline
P5 reader prefix checks remain meaningful
new local types can append without collision
```

If performance requires a dedicated bulk importer, it must validate the same semantic uniqueness/coherence rules as normal insertion.

## T5.F — Fingerprint/parity oracle

For every artifact type/kind node, compare a stable portable semantic projection rather than raw process IDs.

## T5 tests

```text
canonical_type_store_round_trip_semantic_parity
loaded_type_store_has_fresh_store_id
artifact_contains_no_type_store_id
all_artifact_type_refs_relocate
all_artifact_kind_refs_relocate
interner_lookup_returns_loaded_nodes
fork_from_loaded_baseline_preserves_prefix
malformed_type_ref_is_rejected
duplicate/incoherent_parameter_identity_is_rejected
```

## T5 STOP condition

If load correctness requires restoring yesterday's raw `TypeId`/`TypeStoreId` as identity rather than using relocation, stop and redesign the DTO/import boundary.

---

# 16. P7.T6 — Portable `StaticValueStore` graph and relocation

## Purpose

Recreate canonical static compile-time values in a fresh store lineage.

## Primary files

```text
phalcom-compiler-artifact/src/semantic/static_value.rs
phalcom-semantic/src/static_value/artifact.rs
phalcom-semantic/src/static_value/store.rs
```

## Required DTO families

Cover all final canonical `StaticValueData`/structural arenas, including:

```text
primitive values
strings
symbols
tuples
homogeneous sequences
records
data values
variant values
pack backings
packs
static terms/types as applicable
```

Use:

```text
ArtifactStaticValueId
ArtifactStringId
ArtifactSymbolId
ArtifactTupleId
ArtifactSequenceId
ArtifactRecordId
ArtifactDataValueId
ArtifactVariantValueId
ArtifactPackBackingId
ArtifactPackId
```

as explicit wire identities.

## Relocation rules

```text
ArtifactTypeId       → loaded TypeId from T5
stable declaration   → loaded semantic owner identity
ArtifactStaticValueId → fresh StaticValueRef in new StaticValueStoreId lineage
```

Never encode `StaticValueStoreId`.

## Interner rebuild

Loaded static values must participate in ordinary semantic equality/hash/interning exactly as source-built values.

## T6 tests

```text
canonical_static_value_store_round_trip
loaded_store_has_fresh_lineage_id
artifact_contains_no_static_value_store_id
recursive_static_refs_relocate
static_type_refs_use_type_relocation
semantic_eq_hash_parity
materialization_descriptor_parity
malformed_static_ref_is_rejected
```

---

# 17. P7.T7 — Project/load `SEMANTIC_WORLD`

## Purpose

Persist the compiler-required canonical semantic authority defined by final P5.T8.

## Hard rule

Do not serialize `SemanticSnapshot`.

Construct an explicit root such as:

```rust
pub struct SemanticWorldV1 {
    pub type_store: TypeStoreImageV1,
    pub static_values: StaticValueStoreImageV1,
    pub aggregates: SemanticAggregatesV1,
    pub capabilities: SemanticWorldCapabilities,
}
```

Names are illustrative.

## Primary files

```text
phalcom-compiler-artifact/src/semantic/mod.rs
phalcom-compiler-artifact/src/semantic/*.rs
phalcom-semantic/src/artifact/mod.rs
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/snapshot.rs          // constructors/read APIs only
aggregate owner files as needed for bulk construction
```

## T7.A — Exhaustive persistence mapping

For every final P5 family, add one explicit artifact decision. No compiler-required snapshot field may disappear because it was inconvenient to encode.

Representative required semantic families, subject to final P5.T8:

```text
declaration shells/type bindings
hierarchy
aliases
declaration/associated surfaces
dispatch primary semantic facts
callable signatures
callable accepted definitions
field signatures
constructor initialization products
trait headers
trait surfaces
conformance heads
witness plans
resolved/derived conformance rules
trait dispatch semantic authority when primary rather than purely derived
data semantics
enum semantics
enum requirements
type-member declarations
associated static-domain declarations
static case catalogs/signatures
static definitions
static constants
module static bindings where compiler-required
callable analyses required by backend P6/local lowering
semantic graph/formal projection only if P5 marks required rather than rebuildable
source semantic index only to the degree compiler-required; optional source/presentation data must be capability-gated
```

## T7.B — DTO design rule

For each product:

```text
stable semantic IDs → stable refs
TypeId/KindId/etc. → artifact-local IDs
StaticValueRef → ArtifactStaticValueId
source paths/ranges → omitted or optional source-map IDs unless compiler-required
Arc/Rc → value/reference DTO, never pointer semantics
HashMap → deterministic sorted sequence/map on wire
```

For callable-analysis nested identities, qualify every ordinal by its stable
callable owner: expression, binding, explanation, flow node/edge, diagnostic
cause, and attachment references. Same-range expressions remain distinct.
Reject a dangling or wrong-owner reference before publication. Reuse the shared
type/signature/evidence codecs and relocation contexts; do not add another
binding authority. The `CallableAnalysis` field disposition is exhaustive and
tracks the live owner fields listed in the adopted owner-graph amendment.
The body-to-source-owner relation is owner-held callable-analysis transport:
ordinary bodies reference `SourceOwner::Callable`, module-body `<main>` is
explicitly registered as `SourceOwner::Module`, and constructors retain their
approved declaration-owner relationship. Keep attachment owner keys as the
analysis `CallableId`; relocate exact binding, expression, and `SourceSiteId`
references together. Synthetic `<main>` does not acquire a callable signature
or declaration record.

## T7.C — Bulk-load ordering

Define one explicit dependency order. Recommended broad order:

```text
1. stable module/project mapping
2. TypeStore
3. StaticValueStore
4. declaration identity shells
5. hierarchy + aliases
6. signatures/fields/constructors
7. surfaces/dispatch primary entries
8. traits/conformance primary entries
9. data/enum/type-member/static-domain products
10. static definitions/constants/module-binding products
11. callable analyses / backend-required products
12. derived secondary indexes
13. query image
```

Do not hide cyclic construction behind unsafe/unvalidated partially initialized objects. If primary semantic products form a legitimate cycle, introduce two-phase IDs/shells then fill validated payloads.

## T7.D — Snapshot materialization

Construct a new immutable `SemanticSnapshot` representing the loaded canonical world with fresh process-local:

```text
WorkspaceId/SnapshotId/SemanticRevision as internal load provenance only
TypeStoreId
StaticValueStoreId
```

These local coordinates are never compared against artifact bytes for durable identity.

## T7.E — Source/presentation capability split

Loaded compiler profile should not require:

```text
sources: parsed AST map
presentation_sources
advisory editor products
full diagnostics/source maps
semantic_structure_shards
```

unless P5.T8 explicitly classifies a subset as compiler-required. Optional sections/capabilities may restore tooling functionality later.

## T7 tests

For each family:

```text
source-built canonical stable projection == loaded canonical stable projection
```

Add an exhaustive test/census that fails when a new required persistence family is added without DTO/import support.

---

# 18. P7.T8 — Reconstruct and certify P6 indexes after load

## Purpose

Guarantee that artifact loading does not undo P6's asymptotic complexity work.

## Primary owners

Use final P6 owning tables rather than a central P7 index database.

Expected final families include:

```text
callable analyses by module
callable definitions by module
declarations by module
data semantics by module
enum semantics by module
trait requirement layouts
ImplId → inherent impl domain
existing derived conformance get_rule index
source target reverse lookup if compiler-required
```

## T8.A — Rebuild during primary insertion

Preferred:

```text
for each decoded primary row
    validate stable identity
    insert through owner table constructor
    owner table updates secondary index
```

Avoid:

```text
decode flat global table
then scan whole table once per module/index family
```

A single O(N) validation/index-build pass is acceptable; repeated O(M×N) reconstruction is not.

The compiler-required formal attachment projection is part of this rebuild.
Construct it from relocated callable owner facts plus the explicit
body-to-source-owner relation. In particular, attach registered `<main>`
lexical bindings to module-owned declaration sites while retaining their
`BodyLocal { callable, binding }` identity. Do not infer ownership from the
selector name, coalesce site identities because ranges match, or extend
count/order rebasing from callable shards to module shards.

## T8.B — Trait requirement layouts

Either:

```text
encode authoritative layout + validate against trait member ordering
```

or:

```text
deterministically rebuild layout from members + validate persisted layout fingerprint
```

Do not allow unrelated trait order to alter slots.

## T8.C — P6 counter parity

Run the same local compile workload against:

```text
source-built canonical image
artifact-loaded canonical image
```

Assert equivalent structural counts and P6 gates.

Reconstruct `FormalSemanticProjection` and the compiler's exact binding/site/
target reader from the relocated callable-analysis owners. The reader must
serve every lowering attachment currently obtained from `CallableSourceAttachment`
and source-index target sets, without scanning names or requiring source-index
occurrence data. Source-built and imported compilations must consume equivalent
canonical binding IDs and source ranges.

## T8 acceptance

```text
CompilerBackendEntitiesRejectedByModule == 0
CompilerBindingOccurrenceFullScans == 0
CompilerTraitSurfacesVisitedForSlotLookup <= 1
CompilerConditionalMembersScannedForImplDomain == 0
broad canonical backend entities visited == 0 except exact referenced dependency lookups
```

---

# 19. P7.T9 — Portable canonical `QUERY_IMAGE`

## Purpose

Load immutable reusable canonical query roots without persisting `SemanticDb` state.

## Primary files

```text
phalcom-compiler-artifact/src/query.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/artifact/query.rs
```

## T9.A — Eligibility

The semantic eligibility authority remains:

```rust
canonical_query_persistence(kind)
```

P7 artifact profile support must be exhaustively paired with it.

Add a compile/test census over every `QueryKind`:

```text
QueryKind
canonical persistence class
artifact profile support: required/optional/omitted
portable key DTO
portable product reconstruction source
reason
```

A newly added `QueryKind` must force an explicit artifact decision.

## T9.B — Parsed-module conflict

`ParsedModule` may be `EligibleStructural` in P5 but raw parsed source is not required P7 compiler authority.

Preferred P7 compiler profile:

```text
omit ParsedModule products
prune dependent query roots whose closure requires them
or replace the dependency with artifact-level immutable content authority only if that preserves query semantics exactly
```

Do not make source/AST mandatory merely to preserve the in-memory query graph shape.

## T9.C — Portable query keys

Map process-local semantic IDs inside `QueryKey` to stable semantic refs/artifact-local IDs.

## T9.D — Avoid persisting current `u64` query fingerprints as eternal ABI

Current `InputFingerprint`/`ProductFingerprint` use compiler-internal hashing over current-process semantic forms.

Preferred load behavior:

```text
persist portable query key
persist portable product reference/DTO
persist stable dependency keys
    ↓
load/relocate product
    ↓
recompute current-process ProductFingerprint from loaded semantic object
    ↓
rebuild DependencyEdge.observed_fingerprint from loaded dependency products
```

In particular, never copy the source `callable_body_product_fingerprint` as a
loaded result: its expression, binding and type coordinates are process-local.
Recompute it only after callable-analysis relocation, then construct dependent
query products and fingerprints from the same relocated owners.

If any input fingerprint is persisted, header `query_fingerprint_abi` must match exactly before it is trusted.

## T9.E — Avoid duplicate semantic payloads

Where a query product is exactly the final aggregate product, prefer a stable reference to the corresponding `SEMANTIC_WORLD` authority and reconstruct `Arc<SemanticProduct>` from the loaded aggregate rather than encoding a second independent semantic copy.

Query-only payloads may use dedicated DTOs.

## T9.F — Preserve freeze invariants

After load validate:

```text
canonical origin
supported product kind
final aggregate agreement
dependency closure
dependency fingerprint agreement
acyclic dependency graph
no open-world negative proof persistence
```

## T9 acceptance

Loaded `SemanticDb` canonical reads return immutable query products without creating local query rows and without semantic recomputation.

---

# 20. P7.T10 — Persist canonical RuntimePublic `METADATA`

## Purpose

Transport P4/P5 canonical metadata exactly once as part of `universe.telosc`.

## Primary files

```text
phalcom-compiler-artifact/src/metadata.rs
phalcom-type-meta/*                // existing schema/validator reused
phalcom-core canonical compiler-image composition owner
```

## T10.A — Use existing authority

Input must be the already-constructed canonical `Arc<SemanticMetadataBundle>` from P4/P5 composition.

Do not rerun `MetadataExporter` merely to write `.telosc` if the canonical compiler-image view already owns the bundle.

## T10.B — Required metadata profile

Use:

```text
RuntimePublic
```

or the exact P4/P5-approved canonical profile recorded at T0.

Artifact identity scheme must be portable (`V1Standard` or final equivalent), not `SessionLocal`.

## T10.C — Cross-header validation

Cross-check outer `.telosc` header against inner metadata header:

```text
semantic_model_version
native_surface_schema_version
type_metadata_schema_version
source/interface fingerprint/digest relationship as defined
package/project stable identity
metadata profile/capabilities
```

## T10.D — Runtime pool rule

Artifact does not contain a persistent `MetadataPoolId`.

Later runtime load:

```text
SemanticMetadataBundle
    ↓
RuntimeTypingRegistry assigns fresh MetadataPoolId
```

P7 loader should retain the bundle as immutable `Arc<SemanticMetadataBundle>` until runtime consumption.

## T10 tests

```text
metadata_section_round_trip
metadata_validator_runs_on_load
session_local_identity_metadata_rejected
outer_inner_schema_mismatch_rejected
metadata_digest_mismatch_rejected
loaded_metadata_stable_identity_parity
no_metadata_export_on_artifact_load
```

---

# 21. P7.T11 — Deterministic artifact writer

## Purpose

Produce `universe.telosc` from the finalized source-built canonical compiler image.

## Primary file ownership

```text
phalcom-core/src/modules/canonical_artifact.rs       // preferred orchestration
or a narrowly named equivalent
```

Keep generic container encoding in `phalcom-compiler-artifact`.

## T11.A — Writer input

Use one nonduplicating compiler-image view containing:

```text
canonical formal semantic image
source-free canonical module authority
canonical persistence inventory/capabilities
canonical RuntimePublic metadata
portable compatibility identity
```

## T11.B — Projection sequence

```text
1. validate source-built canonical image completeness
2. project MODULE_WORLD
3. project TypeStore
4. project StaticValueStore
5. project semantic aggregates
6. reconstruct/project selected QUERY_IMAGE
7. encode METADATA
8. encode optional sections only when capability requests them
9. validate all DTOs before bytes are finalized
10. encode each section independently
11. compute section digests
12. build fixed header + section directory
13. compute whole-artifact digest
14. final self-validation using reader
15. atomically publish target file
```

Copy the already captured source-view native catalog fingerprint into the V2
header. Do not independently derive a second compatibility value in the writer.

## T11.C — Atomic output

For filesystem utility path:

```text
write temporary file in target directory
fsync/flush according to repository policy
read/validate final bytes
rename to target only after success
```

Failed builds must not replace a previously valid artifact.

## T11.D — Artifact build summary

Return/write a structured summary:

```text
container schema
section schemas
capabilities
package identity
source/interface/semantic/query/metadata/artifact digests
section byte sizes
entity counts per section
metadata profile
artifact total bytes
```

## T11.E — Determinism test

In independent build invocations with identical input:

```text
bytes A == bytes B
artifact digest A == B
section digests A == B
```

Prefer a child-process determinism test so process-global allocator differences cannot be accidentally hidden.

---

# 22. P7.T12 — Explicit `.telosc` loader and canonical image materialization

## Purpose

Create a fresh-process compiler authority without invoking canonical source construction.

## Primary files

```text
phalcom-core/src/modules/canonical_artifact.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/canonical_image.rs
phalcom-modules/src compiler-artifact import owner
```

## T12.A — Loader result

Preferred core-level result:

```rust
pub struct LoadedCanonicalCompilerImage {
    pub formal: Arc<CanonicalUniverseFormalImage>,
    pub metadata: Arc<SemanticMetadataBundle>,
    pub identity: Arc<CanonicalCompatibilityIdentity>,
    pub capabilities: TeloscCapabilities,
}
```

Names may adapt.

## T12.B — Load pipeline

```text
open bytes
    ↓
container header/directory validation
    ↓
section digests / whole digest
    ↓
compatibility checks
    ↓
trusted native catalog fingerprint check before section materialization
    ↓
MODULE_WORLD decode/materialize
    ↓
TypeStore decode/materialize + relocation
    ↓
StaticValueStore decode/materialize + relocation
    ↓
SEMANTIC_WORLD decode/materialize
    ↓
P6 index reconstruction/validation
    ↓
QUERY_IMAGE decode/reconstruct current fingerprints
    ↓
METADATA decode/validate
    ↓
cross-section compatibility validation
    ↓
construct CanonicalUniverseFormalImage
    ↓
construct LoadedCanonicalCompilerImage
```

## T12.C — Formal-image constructor

Add a distinct constructor for trusted validated artifact components. Do not route loaded data through `build(provider)` or `SemanticWorkspaceSession::from_foundation_for_image`.

The reconstructed portable identity preserves the V2 header fingerprint,
already matched to the trusted current catalog before semantic materialization.

Conceptually:

```rust
CanonicalUniverseFormalImage::from_loaded_artifact(parts)
```

It must still enforce image publication completeness and reader-prefix invariants.

## T12.D — Current-process provenance

Allocate fresh process-local store/workspace/snapshot identities as needed. Bind the canonical query image to the freshly loaded TypeStore/StaticValueStore roots.
The loaded compiler-facts projection must preserve and relocate the explicit
`<main>` to module-source-owner relation, exact binding/site attachments, and
the corrected executable body range. Imported binding/expression/site
references must be validated against their relocated owners before compiler
publication.

## T12.E — Error taxonomy

Expose errors that distinguish:

```text
I/O error
bad container
unsupported schema
unsupported required section
resource budget exceeded
digest mismatch
compatibility mismatch
module-world validation error
type relocation error
static-value relocation error
semantic-world validation error
query-image validation error
metadata validation error
missing required capability
```

Do not collapse to a generic `bad artifact` string.

## T12 non-goal

Do not yet change normal production startup to discover/cache this artifact automatically. Provide explicit API/path used by P7 tests and later P9.

---

# 23. P7.T13 — Fresh-process source-unavailable proof

## Purpose

Demonstrate that P7 removed the 40–45 second canonical semantic source-build barrier for artifact-backed compiler use, not merely added an in-process round trip.

## Required two-process test

### Process A — artifact build

```text
normal canonical source available
build canonical compiler image
write universe.telosc
exit process
```

### Process B — source-free compile

```text
start fresh process
install fail-fast canonical source/interface build guards
load universe.telosc explicitly
create semantic/module sessions from loaded image
compile novel user source that imports/uses Universe semantics
verify output/diagnostics parity
assert zero forbidden canonical build counters
exit successfully
```

## Source-unavailable mechanism

Use a test-only fail-fast provider/guard or isolated process mode that causes an immediate explicit failure if any of the following occur:

```text
UniverseSourceProvider::load_parsed
Universe canonical source read
BuiltinInterfaceBuilder::build for Universe
canonical semantic source build
SemanticWorkspaceSession::from_foundation_for_image for Universe
canonical MetadataExporter execution
```

Do not merely inspect counters after a path that could silently recover from source.

## Novel user fixture requirements

The fixture should exercise enough canonical authority to prove the image is not trivial:

```text
Universe imports
nominal type lookup
callable/member signature lookup
at least one trait/conformance path
at least one data/enum/static semantic path when stable fixtures exist
metadata/reflection root consumption where compiler-side applicable
```

Keep fixture deterministic and small enough for frequent regression use.

## Mandatory zero counters

On Process B:

```text
canonical source parses               0
canonical interface builds            0
canonical links                       0
canonical semantic full updates       0
canonical callable analyses           0
canonical witness/conformance rebuild 0
canonical metadata exports            0
```

Initializer **execution** and VM-side source/bytecode work are not claimed by P7 and must be measured separately if the test enters the VM.

---

# 24. P7.T14 — Final validation, parity, determinism, and performance certification

## Purpose

Turn the artifact path into a certified compiler-image implementation rather than an experimental serializer.

## 24.1 Corruption suite

Test at minimum:

```text
bad magic
truncated fixed header
truncated directory
duplicate section
missing required section
unknown required section
overlapping sections
offset overflow
length overflow
section over configured budget
invalid UTF-8
invalid enum tag
section digest mismatch
whole-artifact digest mismatch
out-of-range ArtifactTypeId
out-of-range ArtifactStaticValueId
unresolved stable declaration/callable/field/impl ref
cross-section identity mismatch
metadata schema mismatch
query dependency to missing root
query cycle injected
trait layout inconsistency
module reference to missing module
```

Every case must fail before publishing a partially valid canonical image.

## 24.2 Semantic parity suite

Compare source-built and artifact-loaded canonical authorities through stable projections for:

```text
module count/interfaces/linked reads
type/kind/static-store content
declarations/hierarchy/aliases
callable + field signatures
surfaces/dispatch
trait headers/surfaces/slots
conformance/witness/derived rules
data/enum products
static domains/cases/definitions/constants
module bindings
backend-required callable analyses/definitions
selected query roots
RuntimePublic metadata
```

Do not compare process-local IDs directly.

## 24.3 P5 regression gates

Artifact-backed local session creation must preserve:

```text
CanonicalAggregateEntriesCopied == 0
SnapshotFullMapMaterializations == P5 closing baseline or lower
canonical query rows examined during local session creation == 0
canonical semantic updates during local session creation == 0
```

Use final actual counter names from P5.

## 24.4 P6 regression gates

Run P6 backend locality tests against loaded image and preserve all G6 structural counters.

## 24.5 Determinism gate

At least two independent artifact-build processes from the same revision/input must produce byte-identical files.

Also prove process allocator perturbation does not change bytes by allocating unrelated workspaces/stores before one build if practical.

## 24.6 Performance measurements

Record separately:

```text
source-built canonical image time        historical/current baseline
artifact open/read time
container validation time
MODULE_WORLD decode/materialization
TypeStore decode/relocation
StaticValueStore decode/relocation
SEMANTIC_WORLD decode/materialization
P6 index rebuild time
QUERY_IMAGE load/rebuild
METADATA decode/validation
full canonical artifact load time
SemanticWorkspaceSession::from_image time after load
novel user compile semantic time
novel user compiler projection/lowering time
artifact byte size
peak load allocations / RSS if harness supports it
```

Primary P7 performance requirement:

> **Fresh-process canonical compiler-image load must be materially smaller than source-built canonical semantic construction and must not contain hidden semantic recomputation.**

The architectural zero-work counters are the hard gate; wall-clock is the performance result.

Do not choose a weak percentage target before measuring the first correct binary implementation. If correct artifact loading is unexpectedly close to source-build time, stop and profile rather than deleting validation/invariants.

## 24.7 Build/test commands

Use actual repository test names after implementation and verify nonzero selected tests. Minimum crate checks:

```sh
cargo check -p phalcom-compiler-artifact
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-type-meta
cargo check -p phalcom-core --features perf-observability
cargo check -p telos-observability

cargo fmt --all -- --check
```

Add focused test commands for the new artifact crate, module artifact tests, semantic round-trip tests, query tests, and core fresh-process test. Record selected/pass/fail/ignored counts in the walkthrough.

---

# 25. P7.T15 — Lifecycle closure and P8 handoff

## Required lifecycle files

At P7 activation/closure update:

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md
    PROGRAM.md / STATUS.md where used by current lifecycle
    PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md
    PERF003.C2.P7-walkthrough.md
    PERF003.C2.P7-handoff.md
```

## P7 walkthrough must record

```text
P7_START_SHA / close SHA
P5/P6/P4 predecessor SHAs
final persistence census
wire schema/container version
section schema versions
capability bits
stable identity schema version
semantic/module/query ABI versions
digest algorithm/version
artifact build API/command
artifact load API/command
artifact byte size + section sizes
entity counts
section + whole digests
all compatibility validation rules
all fresh-process zero counters
P5/P6 counter parity
load timings
user compile timings
all test commands and selected counts
all deferred P8/P9 concerns
```

## Correct P7 completion statement

Use language equivalent to:

> **`PERF003.C2.P7` defines and implements the sectioned `.telosc` compiler-image format. The canonical Universe module world, relocatable semantic/type/static-value authority, eligible immutable query roots, and canonical RuntimePublic metadata can be written deterministically and loaded in a fresh process into a valid `CanonicalUniverseFormalImage`/compiler-image authority. A source-disabled fresh process compiles novel user source without Universe parsing, interface construction, linking, semantic analysis, witness reconstruction, or metadata export. Loaded authority preserves P5 zero-copy ownership and P6 local-complexity gates. Executable Universe bytecode/prototypes remain P8; artifact-first discovery/cache/toolchain distribution remain P9.**

Do not claim:

```text
Universe VM bootstrap performs zero AST/bytecode compilation
ordinary production startup automatically uses universe.telosc
user/dependency artifacts are generically shipped/cached
```

until P8/P9.

---

# 26. Primary anticipated write set

The exact write set must be re-bound at T0, but the implementation should expect the following.

## 26.1 Workspace / new crate

```text
Cargo.toml
phalcom-compiler-artifact/Cargo.toml
phalcom-compiler-artifact/src/lib.rs
phalcom-compiler-artifact/src/container.rs
phalcom-compiler-artifact/src/header.rs
phalcom-compiler-artifact/src/section.rs
phalcom-compiler-artifact/src/codec.rs
phalcom-compiler-artifact/src/digest.rs
phalcom-compiler-artifact/src/limits.rs
phalcom-compiler-artifact/src/error.rs
phalcom-compiler-artifact/src/identity.rs
phalcom-compiler-artifact/src/module_world.rs
phalcom-compiler-artifact/src/semantic/*
phalcom-compiler-artifact/src/query.rs
phalcom-compiler-artifact/src/metadata.rs
phalcom-compiler-artifact/tests/*
```

## 26.2 `phalcom-modules`

```text
phalcom-modules/Cargo.toml
phalcom-modules/src/canonical_products.rs
phalcom-modules/src/session.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/package_info.rs
phalcom-modules/src/compiler_artifact.rs      // likely new
module-focused tests
```

## 26.3 `phalcom-semantic`

```text
phalcom-semantic/Cargo.toml
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/artifact.rs        // likely new
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_value/artifact.rs // likely new
phalcom-semantic/src/artifact/mod.rs           // likely new
phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/query.rs
aggregate owner files requiring validated bulk constructors
semantic integration tests
```

## 26.4 `phalcom-core`

```text
phalcom-core/Cargo.toml
phalcom-core/src/modules/mod.rs
phalcom-core/src/modules/canonical_artifact.rs   // likely new
phalcom-core/src/modules/canonical_universe.rs   // preserve P8-facing source path
phalcom-core/src/modules/compile.rs              // explicit loaded-image injection only if needed
phalcom-core/tests/core/*
possibly a dedicated child-process integration test binary/fixture
```

## 26.5 `phalcom-type-meta`

Prefer no semantic redesign. Touch only for:

```text
missing stable identity proven by T2
binary section wrapper if appropriate
validator reuse/export
compatibility helpers
```

## 26.6 `telos-observability`

```text
telos-observability/src/metric.rs
telos-observability/src/phase.rs only if existing phases cannot express load/build stages
```

---

# 27. Observability plan

P7 needs explicit build/load cardinality, not just elapsed time.

## 27.1 Recommended phases

Prefer existing phase taxonomy where possible; add narrowly named phases for:

```text
CanonicalArtifactBuild
CanonicalArtifactEncodeModuleWorld
CanonicalArtifactEncodeSemanticWorld
CanonicalArtifactEncodeQueryImage
CanonicalArtifactEncodeMetadata
CanonicalArtifactWrite

CanonicalArtifactLoad
CanonicalArtifactValidateContainer
CanonicalArtifactDecodeModuleWorld
CanonicalArtifactDecodeTypeStore
CanonicalArtifactDecodeStaticValues
CanonicalArtifactDecodeSemanticWorld
CanonicalArtifactRebuildIndexes
CanonicalArtifactDecodeQueryImage
CanonicalArtifactDecodeMetadata
CanonicalArtifactValidateCrossSection
```

## 27.2 Required counters

Recommended semantic counters:

```text
CanonicalArtifactBuilds
CanonicalArtifactLoads
CanonicalArtifactLoadFailures
CanonicalArtifactBytesRead
CanonicalArtifactBytesWritten
CanonicalArtifactSectionsDecoded
CanonicalArtifactTypeNodesDecoded
CanonicalArtifactStaticValuesDecoded
CanonicalArtifactSemanticEntriesDecoded
CanonicalArtifactQueryRootsDecoded
CanonicalArtifactMetadataRecordsDecoded
CanonicalArtifactSecondaryIndexesRebuilt
CanonicalArtifactRelocationsApplied

CanonicalSourceParsesDuringArtifactLoad
CanonicalInterfaceBuildsDuringArtifactLoad
CanonicalLinksDuringArtifactLoad
CanonicalSemanticUpdatesDuringArtifactLoad
CanonicalCallableAnalysesDuringArtifactLoad
CanonicalWitnessRebuildsDuringArtifactLoad
CanonicalMetadataExportsDuringArtifactLoad
```

The final seven counters must be zero on a correct loaded path.

Reuse P5/P6 counters rather than duplicating their semantic meaning.

---

# 28. Resource limits and validation budgets

Artifact decode is an untrusted-byte boundary even for toolchain-shipped files.

Define `ValidationLimits` for at least:

```text
max total artifact bytes
max section count
max encoded section bytes
max decoded section bytes
max string bytes
max modules
max type nodes
max kind nodes
max parameters/lambdas/rows/families
max static values/structural nodes
max semantic entries per family
max query roots
max query dependencies per root
max metadata bytes/records
max recursion/depth where any recursive validation occurs
```

Defaults should comfortably exceed canonical Universe production sizes while preventing allocation bombs.

The writer should emit counts that make limit headroom visible in the walkthrough.

---

# 29. Deterministic ordering rules

Every wire collection must define an ordering.

Preferred rules:

```text
modules                    StableModuleRef order
declarations               StableDeclarationRef order
callables                  StableCallableRef order
fields                     StableFieldRef order
impls                      StableImplRef order
artifact-local arena nodes explicit ID order
query roots                stable portable QueryKey order
query dependencies         stable key order
metadata                   existing phalcom-type-meta deterministic order
section directory          SectionKind order
```

If source declaration order is semantically observable, encode an explicit ordinal and sort by owner + ordinal rather than relying on insertion order.

No artifact byte sequence may depend on `HashMap` iteration order.

---

# 30. Commit slicing

Land P7 as reviewable commits. Avoid one large serializer patch.

## P7.A — Activation + artifact crate skeleton

```text
CHECKPOINT activation
workspace crate registration
container/header/section/limits/errors
mock round-trip tests
```

No semantic behavior change.

## P7.B — Portable digest + identity/compatibility

```text
ArtifactDigest
compatibility header fields
Universe stable identity mapping
CanonicalSemanticWorldId portable-compatibility refactor
identity tests
```

## P7.C — Source-free external module authority

```text
CanonicalExternalModuleWorld split
WorkspaceModuleSession external construction
resolver/linker cutover
source-free module tests
```

No `.telosc` semantic payload yet.

## P7.D — MODULE_WORLD DTO + codec

```text
portable module DTO
projection/import
referential validation
round-trip tests
```

## P7.E — TypeStore artifact graph

```text
portable type/kind graph
relocation
fresh-store import
semantic parity tests
```

Keep separate because this is a high-risk persistence boundary.

## P7.F — StaticValueStore artifact graph

```text
portable static graph
TypeId relocation integration
fresh lineage import
parity tests
```

## P7.G — Semantic primary aggregate DTO/import slice 1

Start with foundational products:

```text
declarations
hierarchy
aliases
callable/field signatures
surfaces/dispatch primary state
```

## P7.H — Semantic primary aggregate DTO/import slice 2

```text
traits/conformance
data/enum
static/type-member/associated products
callable analyses/definitions
remaining P5 required families
```

Split further if review size warrants.

## P7.I — P6 index reconstruction certification

```text
module indexes
trait layouts
ImplId direct lookup
P6 structural test parity
```

## P7.J — QUERY_IMAGE

```text
portable query keys/products/dependencies
current fingerprint reconstruction
parsed-source omission policy
query closure tests
```

## P7.K — METADATA

```text
canonical bundle section
outer/inner cross-validation
no fixed pool ID
metadata tests
```

## P7.L — Deterministic writer

```text
source-build projection
section encoding/digests
atomic file publish
artifact summary
determinism test
```

## P7.M — Explicit loader

```text
full section materialization
CanonicalUniverseFormalImage loaded constructor
LoadedCanonicalCompilerImage
structured errors
```

## P7.N — Fresh-process source-free proof

```text
child process fixture
fail-fast source guards
novel user compile
zero-work counters
```

## P7.O — Final certification/lifecycle

```text
corruption suite
parity suite
P5/P6 gates
performance measurements
walkthrough/handoff/checkpoint close
```

---

# 31. Test matrix

## 31.1 Artifact crate

| Area | Required proof |
|---|---|
| header | fixed endian/width, schema rejection |
| directory | bounds, overlap, duplicate/missing required sections |
| digest | per-section and whole-file mismatch rejection |
| limits | allocation/resource budget enforcement |
| unknown section | optional skip / required reject |
| determinism | identical DTO → identical bytes |

## 31.2 Module world

| Area | Required proof |
|---|---|
| stable identity | builtin Universe paths round-trip |
| import target | external canonical target resolves without source |
| interface | stable/public surface parity |
| linked reads | target references survive load |
| source independence | no provider/interface builder calls |

## 31.3 Type/static stores

| Area | Required proof |
|---|---|
| fresh identity | new TypeStoreId/StaticValueStoreId |
| relocation | every artifact-local edge maps |
| interning | post-load lookup/intern parity |
| prefix | local fork extends loaded canonical prefix |
| malformed refs | fail closed |

## 31.4 Semantic world

For every P5 required family:

```text
stable entity count parity
stable semantic projection parity
lookup parity
module/owner index parity where P6 owns index
remove/lookup semantics on local fork unchanged
```

## 31.5 Queries

```text
eligible family persistence
unsupported/open-world omission
closed dependency graph
aggregate-product parity
loaded immutable hit
local query cannot shadow canonical query
new QueryKind requires explicit artifact decision
```

## 31.6 Metadata

```text
bundle byte/model parity
stable identity parity
validation limits
outer/inner compatibility
no metadata export during load
fresh runtime pool assignment remains later/local
```

## 31.7 End-to-end

```text
build artifact in process A
load in process B
canonical source disabled
compile novel user source
P5 zero-copy gates
P6 locality gates
all forbidden canonical source-build counters == 0
```

---

# 32. Failure atomicity requirements

## 32.1 Writer atomicity

A failed projection/encode/digest/final-validation must not replace the prior artifact.

## 32.2 Loader atomicity

Do not publish any global/process canonical image until all required sections and cross-section invariants pass.

Use local construction variables through validation; only construct/publish final `Arc<CanonicalUniverseFormalImage>` after success.

## 32.3 No poisoned global cache

If explicit artifact loading is wrapped in a cache for tests, a failed load must not permanently poison the process unless current canonical-image error policy explicitly requires it. Preserve retry semantics comparable to existing canonical image initialization where practical.

## 32.4 No semantic repair on failure

If decoded authority is incomplete/inconsistent:

```text
reject artifact
```

Do not fall back inside the loader to source parsing or semantic rebuild. P9 may later own an explicit policy that chooses a source rebuild outside the loader.

---

# 33. Forbidden implementation shortcuts

Do NOT:

```text
serde/bincode the raw SemanticSnapshot
serde the raw TypeStore private struct
serde the raw StaticValueStore private struct
serialize SemanticDb or QueryState
serialize WorkspaceId/SnapshotId/SemanticRevision
serialize TypeStoreId/StaticValueStoreId
serialize MetadataPoolId
serialize current VM Chunk/Callable/Value/ObjRef/ClassId
persist current hash-map iteration order
use CARGO_PKG_VERSION as sole compatibility identity
use current DefaultHasher ProductFingerprint as unversioned durable cache identity
make raw Universe source mandatory to satisfy ParsedModule query persistence
re-run semantic analysis during artifact load
re-run metadata export during artifact load
rebuild canonical linked program from source interfaces during load
materialize one giant merged semantic map to rebuild P6 indexes
introduce a second metadata schema
introduce a second stable package/declaration identity scheme
silently skip an unknown compiler-required semantic family
silently accept unknown required sections
silently repair malformed cross-section references
switch ordinary production startup/cache policy in P7
claim executable Universe persistence before P8
```

---

# 34. STOP / CONSULT triggers

The implementer MUST stop and produce an incident/architecture consultation packet if any of the following is encountered.

## 34.1 P5 persistence inventory is incomplete

A compiler-required canonical product cannot be classified as required/rebuildable/process-local/optional.

Do not invent P7-local classification silently.

## 34.2 P6 primary table ownership is still changing

The DTO/load constructor would encode an intermediate representation that P6 is replacing.

Coordinate first.

## 34.3 Compiler dependency still requires raw source

If `SemanticWorkspaceSession::from_image` or local import/link logic genuinely cannot operate without `UniverseSourceProvider`/parsed source, stop and settle the external module authority before encoding around the coupling.

## 34.4 Stable semantic identity is insufficient

A required owner cannot be represented faithfully by existing stable refs plus an explicitly versioned artifact-local ordinal.

Do not persist process-local IDs as a workaround.

## 34.5 Type/static loading requires restoring process IDs

If correctness seems to require keeping the old `TypeStoreId` or `StaticValueStoreId`, the relocation model is wrong.

## 34.6 Query reuse requires an unversioned process hash

If loaded canonical query correctness depends on persisting `DefaultHasher` values with no ABI boundary, stop and redesign/recompute the fingerprint.

## 34.7 Metadata requires fixed pool numbering

`MetadataPoolId` must remain receiving-runtime-local.

## 34.8 Loader needs semantic analysis to repair missing authority

Missing compiler-required artifact facts are an artifact construction/schema defect. Do not hide them with semantic recomputation.

## 34.9 P7 needs VM-local handles

Any need to encode runtime classes, closures, chunks, heap objects, inline caches, or VM object IDs crosses into P8. Split the requirement instead of contaminating P7.

## 34.10 Determinism cannot be achieved because semantic meaning depends on insertion/hash iteration order

Treat this as a compiler semantic determinism defect and resolve it explicitly before freezing a wire format.

## 34.11 Loaded semantic parity only works by comparing raw numeric IDs

Introduce stable/portable semantic projections; do not make numeric equality the public artifact contract.

---

# 35. Structural acceptance gates

## G7.0 — Predecessor contract

```text
P5 required persistence inventory closed
P5 canonical metadata composition closed
P5 canonical copy gate passing
P6 backend locality/index gates passing
```

## G7.1 — Portable identity

```text
no process-local allocation ID in persistent semantic identity
Universe package/module stable mapping exact
portable compatibility identity cross-checks all required ABIs
```

## G7.2 — Container

```text
sectioned v1 format
bounded validation
required/optional section semantics
per-section digests
whole-artifact digest
EXECUTABLE reserved/absent
```

## G7.3 — Module world

```text
source-free external canonical module authority exists
artifact-backed local import resolution uses it
Universe source parse/interface/link work during loaded compile == 0
```

## G7.4 — Type/static relocation

```text
fresh TypeStoreId on load
fresh StaticValueStoreId on load
all artifact-local refs relocate
semantic equality/parity passes
local TypeStore fork extends loaded prefix
```

## G7.5 — Semantic world

```text
all P5 RequiredSemantic families represented
all Rebuildable families deterministically reconstructed
no required family silently omitted
stable semantic parity source-built vs loaded
```

## G7.6 — Query image

```text
P5.T9 remains sole semantic eligibility authority
no open-world/session-local query persisted
loaded graph dependency-closed/acyclic
final aggregate agreement validated
no semantic recomputation to restore queries
```

## G7.7 — Metadata

```text
canonical RuntimePublic authority persisted
portable identity scheme
outer/inner compatibility matches
metadata exporter runs during loaded path == 0
MetadataPoolId not persisted
```

## G7.8 — Fresh-process compiler proof

In a new process with canonical source access fail-fast:

```text
load universe.telosc
create canonical semantic dependency
compile novel user source successfully
```

and:

```text
canonical source parses               == 0
canonical interface builds            == 0
canonical links                       == 0
canonical semantic updates            == 0
canonical callable analyses           == 0
canonical witness rebuilds            == 0
canonical metadata exports            == 0
```

## G7.9 — P5/P6 non-regression

```text
canonical aggregate entries copied == 0
no hidden merged canonical materialization
P6 backend visit counters remain local-only
binding full scans == 0
trait global slot scans == 0
conditional-member exact-Impl scans == 0
```

## G7.10 — Determinism

```text
same input/schema/ABI → byte-identical universe.telosc across fresh processes
```

## G7.11 — Performance

```text
artifact load is materially faster than source semantic rebuild
load phases show decode/relocation/index construction, not semantic analysis
local compile remains at P6 locality baseline
```

---

# 36. Final acceptance checklist

P7 is complete only when every applicable item is true.

## Container and identity

- [ ] `.telosc` magic/header/directory schema v2 implemented, with explicit V1 rejection.
- [ ] Wire integer widths/endianness/string/enum encoding are explicit.
- [ ] Required/optional section behavior is tested.
- [ ] `ArtifactDigest` is distinct from semantic fingerprints.
- [ ] Stable Universe package/module mapping is canonical.
- [ ] No process allocation ID is used as artifact identity.
- [ ] `EXECUTABLE` section identity is reserved and absent.

## Module world

- [ ] Canonical external module authority no longer requires parsed source/provider.
- [ ] `MODULE_WORLD` round-trips stable interfaces/link products.
- [ ] Local imports into Universe terminate at loaded external authority.
- [ ] Loaded path performs zero Universe interface/source discovery.

## Type/static stores

- [ ] Portable type/kind graph covers every canonical node family.
- [ ] Fresh `TypeStore` materializes through explicit relocation maps.
- [ ] Portable static-value graph covers every canonical value family.
- [ ] Fresh `StaticValueStore` materializes through explicit relocation maps.
- [ ] Interner semantics work after load.
- [ ] Local store fork preserves loaded canonical prefix semantics.

## Semantic world

- [ ] Final P5 persistence census is exhaustively implemented.
- [ ] All compiler-required aggregates have explicit DTO/load paths.
- [ ] Optional presentation/source products are capability-separated.
- [ ] Loaded stable semantic projection equals source-built projection.
- [ ] P6 secondary indexes are rebuilt/validated without broad rescans.

## Query image

- [ ] Artifact support is exhaustively mapped against `canonical_query_persistence`.
- [ ] Parsed-source query tension resolved without requiring source in compiler profile.
- [ ] Portable query keys use stable/artifact-local IDs.
- [ ] Current-process fingerprints/dependency edges are reconstructed safely.
- [ ] Loaded canonical query reader works without `SemanticDb` replay.

## Metadata

- [ ] Canonical P4/P5 RuntimePublic bundle is persisted.
- [ ] Metadata validator runs on load.
- [ ] Outer/inner schema and identity fields cross-check.
- [ ] No persistent `MetadataPoolId`.
- [ ] No canonical metadata exporter invocation during load.

## Writer/loader

- [ ] Writer is deterministic.
- [ ] Writer publishes atomically.
- [ ] Loader is resource-bounded and fail-closed.
- [ ] Loader never semantically repairs missing authority.
- [ ] Loaded formal image is constructed directly from validated components.
- [ ] Structured errors distinguish corruption/compatibility/semantic failures.

## End-to-end

- [ ] `universe.telosc` generated from source-built canonical image.
- [ ] Fresh process loads it with canonical source access disabled.
- [ ] Novel user source compiles successfully.
- [ ] All canonical source-build counters are zero.
- [ ] P5 zero-copy gates pass.
- [ ] P6 locality gates pass.
- [ ] Two independent builds are byte-identical.
- [ ] Load timing and artifact size are recorded.
- [ ] Required crate checks and formatting pass.
- [ ] P7 walkthrough/handoff/checkpoint updated.

---

# 37. P7 → P8 handoff contract

P8 receives a **portable semantic compiler image**, not an executable runtime image.

At P7 closure, `universe.telosc` contains:

```text
FIXED_HEADER
SECTION_DIRECTORY
MODULE_WORLD
SEMANTIC_WORLD
QUERY_IMAGE
METADATA
optional PRESENTATION/SOURCE_MAPS
reserved-but-absent EXECUTABLE
DIGESTS/section integrity
```

P8 may assume:

```text
canonical stable package/module/declaration/callable identity already exists
artifact-local relocation infrastructure is established
sectioned container can add EXECUTABLE without redesign
canonical RuntimePublic metadata is already persisted
fresh-process semantic compiler bootstrap is source-free
P5/P6 canonical semantic consumption is zero-copy/local-complexity safe
```

P8 must add:

```text
portable compiler executable prototypes
portable instruction/chunk representation
portable constant/runtime-type/native references
portable relocation records
fresh VM-local callable/chunk/closure/cache materialization
zero Universe AST/bytecode compilation on artifact-backed VM bootstrap
```

P8 must continue to forbid persistence of:

```text
heap objects
GC state
live closures
inline caches
fibers
mutable module globals
ObjRef
ClassId
VM-local MetadataPoolId
```

P7's correct handoff sentence is:

> **The compiler no longer needs Universe source to reconstruct canonical module, semantic, query, or metadata authority. The `.telosc` container and relocation framework are stable. P8 now owns the remaining VM-independent executable prototype projection and fresh-VM materialization required to remove Universe AST/bytecode compilation.**

---

# 38. P7 → P9 deferred operational contract

P7 intentionally does not settle:

```text
where production toolchains install universe.telosc
cache directory layout
artifact discovery precedence
source fallback policy
cache eviction
remote cache protocol
package registry transport
user-facing rebuild command UX
compiler upgrade invalidation UX
automatic `canonical_universe_formal_image()` cutover
```

P9 receives from P7:

```text
explicit writer API
explicit loader API
portable compatibility identity
artifact digest
validation/failure taxonomy
source-free semantic compile proof
artifact performance measurements
```

P9 turns these into the ordinary toolchain/runtime operational path.

---

# 39. Recommended implementation sequence relative to active P6 work

## Can begin before P6 closes

```text
1. P7.T0 lifecycle activation + live synchronization record
2. P7.T1 artifact crate/container/limits/errors
3. P7.T2 digest + stable package/module identity + compatibility shell
4. T3 source-free module authority design if it does not conflict with P5/P6 writes
```

## Wait for P5.T8/T10 and stable P6 table shapes

```text
5. finalize semantic persistence census
6. freeze SEMANTIC_WORLD DTO families
7. implement semantic load constructors
8. reconstruct P6 indexes
9. freeze METADATA composition
```

## Then close the persistence path

```text
10. QUERY_IMAGE
11. deterministic writer
12. explicit loader
13. fresh-process source-free proof
14. corruption/determinism/performance certification
15. lifecycle closure + P8 handoff
```

Do not block independent container work on late P6 tests, but do not freeze semantic DTOs against intermediate aggregate representations merely to keep P7 moving.

---

# 40. Code-review checklist for every P7 patch

Every implementation patch must answer:

```text
What semantic authority is being persisted or reconstructed?
Why is this authority compiler-required rather than optional/rebuildable?
What identity survives across processes?
What process-local IDs are deliberately discarded/recreated?
What artifact-local IDs are introduced?
How are all references validated before publication?
What deterministic ordering governs the bytes?
What size/resource budget bounds decoding?
Does the loaded representation preserve P5 parent/local ownership?
Does it preserve P6 module/exact-index complexity?
Does the path invoke any parser/linker/semantic analyzer/metadata exporter?
What test proves source-built vs loaded semantic parity?
What corruption test proves fail-closed behavior?
What counter proves no hidden source/semantic reconstruction occurred?
Does this patch accidentally absorb P8 VM/executable concerns or P9 cache policy?
```

If a patch cannot answer these questions, it is not ready to merge.

---

# 41. Final implementation outcome

The implementation target for `PERF003.C2.P7` is:

```text
source-built canonical authority
        │
        ▼
deterministic portable projection
        │
        ▼
   universe.telosc
        │
        ▼
fresh-process validation + relocation
        │
        ├── source-free canonical module world
        ├── fresh canonical TypeStore prefix
        ├── fresh canonical StaticValueStore lineage
        ├── canonical semantic primary aggregates
        ├── reconstructed P6 local/exact indexes
        ├── immutable canonical query roots
        └── canonical RuntimePublic metadata
        │
        ▼
CanonicalUniverseFormalImage / compiler image
        │
        ▼
compile user code only at the semantic/compiler-dependency layer
```

P7 is finished when this pipeline is proven in a **fresh process with Universe source unavailable**, with P5 zero-copy gates and P6 local-complexity gates intact, deterministic artifact bytes, bounded fail-closed decoding, and no Universe source parsing/linking/semantic analysis/metadata export on the loaded path.

P7 is **not** finished merely because an in-process round trip succeeds, and the overall PERF003.C2 program is **not** finished until P8 removes VM-side Universe compilation and P9 makes the artifact-first path operationally canonical.
## Adopted budgeted flow-graph owner amendment — 2026-10-15

After the Level 2 adviser confirmed the importer must reconstruct canonical
`FlowGraph` while its public `BTreeMap` backing cannot be covered by the
artifact allocation budget, use dual storage inside the existing graph owner.
Source graphs retain mutable map storage; imported graphs hold shared frozen
`BudgetedSortedMap` node/edge storage, entry, ordered exits, and adjacency
vectors. Preserve `CallableAnalysis.flow_graph: Arc<FlowGraph>` and expose one
allocation-free read API across both forms. Frozen mutation returns a typed
error; there is no thawing or hidden BTree reconstruction. Clone of frozen
storage shares an inner Arc, while source clone retains ordinary isolated
mutation semantics.

On import, validate graph closure and adjacency orientation, use charged
scratch to count in/out degrees, reserve every adjacency vector exactly, and
append edge IDs in ascending owner-local edge ordinal. Preserve the persisted
exit sequence without sorting or normalization. Charge node/edge buffers,
adjacency and exit buffers, relocation/count scratch, and publication wrappers;
do not call `reachable_nodes()` in the charged path. A source adjacency ordering
that conflicts with the exporter's established sorted-order invariant blocks
the slice and requires consultation rather than normalization.

Focused acceptance before removing callable-analysis fail-closed guards:
source/frozen read equality and fingerprints; malformed graph rejection;
frozen clone sharing and typed mutation rejection; source clone isolation; and
independent allocator-measured success plus high-degree adjacency N/N-1 budget
rollback. This ruling changes only the graph representation seam. Remaining
callable-analysis owner fields and compiler projection, other P7 families,
source-denied compilation, determinism, parity, and P5/P6 gates remain required.

## Adopted denotation owner-transport amendment — 2026-10-15

The existing GPT-6.1 Sol High Effort adviser reviewed the live
SemanticDenotation owner model and returned PLAN NEEDS CLARIFICATION plus a
bounded Level 2 amendment. Implementation may resume. This amendment applies
to T7.B, T8, T9, T12, the CallableAnalysis field manifest, and R3 evidence. It
does not change accepted language semantics or reduce P7 acceptance.

Use a hybrid transport:

| Denotation | Portable owner |
|---|---|
| TypeForm(TypeId) | ArtifactTypeId reference through the shared type graph and TypeRelocation |
| Kind(KindId) | ArtifactKindId reference through the shared kind/type graph |
| StaticDomain(StaticDomainId) | Artifact domain reference plus declaration-owner validation |
| StaticTerm(TypeId) | ArtifactTypeId reference; it is not a StaticValueStore value |
| StaticApplication(StaticApplicationId) | Reference to a validated checked-application row qualified by its actual checked-body owner; keep fail-closed until producer retention is fixed |
| AssociatedValue Exact/Family | Inline context-owned captured payload with stable owner/member/target refs and relocated type refs |
| BehavioralFamily | Inline context-owned captured payload with stable target refs, relocated type refs, behavior spec, and conditional selections |

The inline capture payload preserves the selected members, optional targets,
operation shapes, and conditional-selection meanings that the checker recorded.
Import must not rerun lookup or dispatch, synthesize capability, or rebuild the
capture from declaration tables. Reuse shared codecs for
AssociatedMemberId, AssociatedFamilyId, InvocationTargetId, operation shapes,
BehavioralFamilySpec, and ConditionalDispatchSelection. A body-owned
denotation pool with artifact-local IDs is permissible only if deduplication is
later needed; no global family-selection registry is introduced.

StaticApplication is an explicit producer prerequisite. The live
CheckingContext::finalize_with_normal_returns does not retain
checked_static_applications in CallableAnalysis. In
analyze_checked_static_application, record_checked_static_application receives
the target in its body-owner slot, although the row identity is defined by its
checked-body owner. Fix and test that owner relation before transporting
applications. Do not resolve through the callee static definition or compare
unqualified ordinals across bodies.

Required construction order:

1. Register denotation-owned type, kind, and domain roots before shared graph closure.
2. Validate declaration, associated-member, constructor, and implementation authorities.
3. Retain checked-body application rows and establish owner-qualified identity maps.
4. Materialize callable and checked-body graphs through the shared relocation.
5. Validate every reference and allocation charge before publishing the snapshot.

Focused discriminators: both associated variants with optional targets; a
captured member subset that differs from declaration authority; behavioral
captures with and without conditional selections; roots reachable only through
denotations; overlapping application ordinals under two body owners; missing
and wrong-owner references reject before publication; and a restored capture
works in a source-denied process without lookup. Keep the existing callable
analysis export/import completeness guards until the complete owner graph,
compiler projection, and these discriminators pass.

## StaticApplication owner clarification — 2026-10-15

The user resolved the owner choice: `CheckedStaticBody` is the sole owner of
checked static-application rows. Live source already assigns the enclosing
`StaticBodyId + ordinal` and stores the checked rows on that body. The older
producer-prerequisite wording above is superseded: do not duplicate rows into
`CallableAnalysis` or treat regular callable finalization as their owner.

Root registration now walks the checked-body rows and includes their signature
domains, generic parameters, and kinded argument types. The remaining gap is
source-free transport. A native row contains source-local expression IDs and
ranges, while the current operational DTO has target and arguments but no
owner-qualified checked-row reference. A bounded architecture comparison
recommends a portable row form nested under the checked-body owner, with
explicit portable links to operation operands; it avoids reconstructing or
fabricating source expression IDs. Preserve fail-closed denotation and
CallableAnalysis guards until row existence, target/signature equality, and
operand mapping are cross-validated by focused discriminators.
