# 02 — Runtime Reification, Semantic Metadata, and Artifact Contract

**Date:** 2026-08-22
**Revision:** post-Spec-01.5 canonical generic semantic model
**Status:** Ratified metadata/reification specification; implementation is dependency-gated on the relevant Spec 01 and Spec 01.5 semantic products
**Authority:** durable semantic metadata schema, artifact-retention contract, semantic export boundary, runtime metadata loading, runtime typing-context storage, lazy reification, descriptor identity/lifetime, GC/cache policy, native-metadata convergence, and metadata security/performance requirements
**Depends on:** [01 — Compiler-Owned Typing Implementation Architecture](01-implementation-architecture.md) and [01.5 — Canonical Generic Type Semantics and Declaration Model](01.5-canonical-generic-type-semantics-and-declaration-model.md)
**Consumed by:** [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), explicit runtime type-form values from [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md), compiler artifacts, native surfaces, runtime validation, and future advanced-effect/proof extensions from Spec 05
**Primary owners:** new VM-independent `phalcom-type-meta` crate; `phalcom-semantic` exporter; `phalcom-core` artifact loader/runtime typing registry and heap bridge; `phalcom-native-meta` producer/adaptor
**Repository snapshot inspected:** `aureat/phalcom-lang@a43f26e0ddd6b1d6e37ddf7a0b9588769bb41f3e` (`main`, 2026-08-22)
**Scope:** stable store-independent type/kind/signature metadata; generic parameters/constraints/variance; type lambdas; `Self`; generic superclass templates; callable and field signatures; aliases; metadata profiles and required roots; deterministic fingerprints; artifact carriage; lazy runtime descriptors; runtime overlay composition; weak descriptor caching; runtime method-to-semantic-signature indexing; world-sensitive cache invalidation; native metadata convergence; validation limits; migration and verification
**Non-goals:** redefining canonical generic semantics from 01.5; defining source grammar from 04; defining public reflection selector spelling from 03; inventing effect/proof semantics before 05; serializing raw `TypeId`/`KindId`/solver IDs; per-instance generic tokens; specialized runtime classes/metaclasses; type-directed selector identity; permanent package-file binary encoding before package/reproducibility policy is ratified

---

## 0. Revision contract

This document replaces the previous revision of **02 — Runtime Reification, Semantic Metadata, and Artifact Contract** wherever that revision conflicts with Spec 01.5 or with the runtime-performance decisions ratified after the original document was written.

The previous revision got the central direction right: compiler semantics remain authoritative; persisted metadata must be store-independent and versioned; nominal type forms reuse ordinary runtime class objects; synthetic forms are reified lazily; erased generic applications do not alter object layout or dispatch; raw semantic IDs do not cross artifact boundaries; unbounded synthetic descriptors must not be strongly rooted.

The following changes are normative in this revision:

| Previous design | Revised decision |
|---|---|
| Spec 02 depended only on Spec 01 | Spec 02 depends on both 01 and 01.5; metadata cannot freeze before the canonical generic calculus exists |
| `TypeParameterRecord` carried `upper_bound`, `constraints`, and `default` | parameter record carries identity/name/kind/variance/provenance; **generic signature owns constraints**; no generic default field exists yet |
| no type-lambda schema | core schema includes alpha-normalized type-lambda metadata and a scoped bound-variable graph |
| generic superclass represented only as nominal declaration relation | declaration metadata carries the canonical generic superclass/supertype template |
| callable record primarily modeled a return type plus future contracts | metadata carries canonical `CallableId`-equivalent signatures and canonical field signatures |
| `Self` not fully represented | metadata can encode owner-relative `Self` recursively inside signature terms |
| broad `Arc<RwLock<RuntimeTypeArena>>` attached through typing context/descriptors | immutable loaded metadata is read directly; synchronization/mutation is confined to a small bounded overlay/cache; no lock is required for ordinary descriptor property reads |
| weak cache algorithm used `heap.get(obj_ref)` | weak cache **must** use non-panicking `heap.try_get(obj_ref)` and treat stale generational handles as expected misses |
| monolithic reflection profile could imply metadata absent | `RuntimeMinimal` retains executable semantic roots even when public reflection is stripped; public reflection is a separate retention level |
| proof/effect record shapes were partially frozen in core schema | core metadata defines versioned extension carriage; exact advanced effect/proof payloads remain owned by revised Spec 05 |
| every descriptor retained an arena `Arc` | descriptors retain a compact context handle; the context roots immutable metadata and the bounded overlay |
| runtime semantic signature might be copied into method objects | `MethodObject` remains compact; VM-owned side metadata maps live method handles to stable callable metadata when retained |
| durable graph contained only global type nodes | schema distinguishes canonical/global type nodes from alpha-normalized scoped lambda bodies |

The following prior decisions remain ratified:

- static semantic authority flows one way into metadata/reflection;
- `reify(Int) === Int` and `reify(List) === List` for nominal forms;
- applied/union/tuple/record/callable/lambda synthetic forms do not create runtime classes;
- runtime `value.class` never reconstructs an erased static generic type;
- synthetic descriptor identity is context-local and weakly cached;
- structural equivalence, not runtime `===`, is the cross-context/cross-process semantic notion;
- metadata decoding is bounded, iterative, deterministic, and hostile-input aware;
- the persisted format never contains raw `TypeId`, `KindId`, `InferVarId`, query IDs, generation-local pointers, `ObjRef`, or `ClassId`;
- `Unknown`, `Dynamic`, invalid annotation, cancellation, and budget exhaustion are statuses/results, not canonical type nodes;
- runtime reflection never mutates or upgrades the compiler semantic snapshot;
- open-world method-table mutation invalidates world-sensitive runtime typing queries, not structural type identity.

---

# 1. Purpose and authority

Phalcom's type system spans several lifetimes and representations:

```text
source/native/generated declarations
          |
          v
compiler-owned canonical semantics              Spec 01 + 01.5
          |
          | publish/export
          v
stable VM-independent semantic metadata         this Spec 02
          |
          | load + validate
          v
immutable runtime metadata pools
          |
          | explicit demand only
          v
ordinary Phalcom reflection values              Spec 03
```

Those layers must not collapse into one representation.

`TypeStore` is optimized for compiler analysis: dense store-local IDs, hash-consed nodes, query-local views, and ephemeral inference variables. A durable artifact needs different properties: stable identity, deterministic encoding, security limits, no dependence on allocation order, and compatibility/version information. A runtime reflection value needs still different properties: ordinary object identity, GC integration, access control, lazy child materialization, and bounded dynamic composition.

This specification defines the boundary between those layers.

The central rule is:

> **A semantic fact may be exportable without being allocated as a runtime object, and a runtime object may denote a semantic fact without becoming the authority for that fact.**

The normal compiler, checker, LSP, and CLI path should use `TypeId`, `KindId`, canonical signature IDs, `TypeView`, and semantic query products directly. It must not serialize metadata merely to ask semantic questions, and it must not instantiate runtime reflection objects.

## 1.1 What this specification owns

This document owns:

1. the stable logical metadata schema;
2. schema/version/feature negotiation;
3. stable structural identities across semantic-store generations;
4. exportability and artifact-retention rules;
5. metadata profiles and executable semantic roots;
6. deterministic graph construction and fingerprints;
7. decoding/validation budgets and hostile-input behavior;
8. loader ownership and immutable runtime metadata pools;
9. runtime typing-context base/overlay/cache architecture;
10. lazy semantic descriptor materialization;
11. GC and weak-cache semantics;
12. runtime method-to-semantic-metadata indexing;
13. native metadata convergence into the same semantic language;
14. extension carriage for later effect/proof metadata;
15. performance and verification gates.

## 1.2 What this specification deliberately does not own

It does not redefine:

- kinding, generic application, variance laws, constraints, substitution, type lambdas, generic inference, generic inheritance, or `Self` — Spec 01.5 owns those;
- source spelling/precedence/recovery — Spec 04 owns those;
- the exact user-facing classes/selectors/result variants/capabilities — Spec 03 owns those;
- effect lattices, termination proofs, verification conditions, proof trust semantics, or proof-result meaning — Spec 05 owns those;
- project/module identity semantics or compiler query invalidation — Spec 01 and `phalcom-modules` own those;
- permanent package artifact container/binary format — a later package/reproducibility decision owns that.

This separation matters. A metadata serializer must never become a second type checker, and a reflection object must never become a second type store.

---

# 2. Current repository state

This section is observational. It records the repository inspected for this revision and must not be read as claiming that the target architecture already exists.

## 2.1 Current semantic export is recursive and store-independent, but not a durable graph

`phalcom-semantic/src/export.rs` currently defines `CompiledKindRef`, `CompiledTypeRef`, stable exported parameter owners, tuple/record/callable export structs, and `SemanticExportError` (`export.rs:1-150`). `export_type_form` recursively walks `TypeStore` and rejects inference variables and internal `ClassObject` forms.

That is a useful transitional seam. It already proves an important principle: exported semantic data does not need to expose raw `TypeId` values.

It is not suitable as the durable format because recursive trees:

- duplicate shared subforms;
- allocate repeatedly for common children;
- make maximum depth a Rust-stack concern unless converted iteratively;
- have no schema/model version;
- have no stable cross-module declaration/signature tables;
- have no profile/root-selection model;
- have no type-lambda scoped binder representation;
- cannot cleanly attach source occurrence/provenance separately from canonical identity;
- encourage downstream consumers to treat the recursive Rust enum as the wire ABI.

The revised target keeps `CompiledTypeRef` only as a compatibility/testing adapter until the indexed schema replaces it.

## 2.2 The current type store does not yet contain the full 01.5 graph

`phalcom-semantic/src/types/store.rs:32-74` currently contains `Never`, `Unit`, `ClassObject`, `Nominal`, `Applied`, `Union`, `Tuple`, `Record`, `Callable`, declaration/callable `Parameter`, and `Infer`. The store tracks a `KindId` for every canonical form and currently interns kinds/types in dense vectors plus hash maps.

`phalcom-semantic/src/types/parameter.rs:6-24` currently gives type parameters owner/index/name/kind and gives `GenericSignature` only owner plus parameter IDs.

Spec 01.5 adds semantic requirements that metadata must wait for and then encode:

- parameter variance;
- signature-owned `Subtype`/`Equivalent` constraints;
- type lambdas with alpha-normalized bound variables;
- owner-relative `Self`;
- generic superclass templates;
- canonical callable and field signature tables;
- transparent aliases with declaration provenance;
- store-independent structural fingerprints.

This is why schema v1 must not be frozen against today's `TypeData` enum alone.

## 2.3 Current type substitution is eager

`phalcom-semantic/src/types/substitution.rs:33-99` recursively rebuilds applications, unions, tuples, records, and callables into the `TypeStore`. `substitution_for_applied` creates a declaration-parameter map from applied arguments (`substitution.rs:110-143`).

Spec 01.5 replaces this as the hot-path specialization model with lazy environments/views. Metadata therefore exports already-publishable canonical declarations and terms; it does not serialize query-local `TypeView` environments or force every possible specialization to exist as an artifact node.

A declaration like:

```phalcom
class Box<T> {
  value() -> T
}
```

exports one declaration parameter and one unspecialized callable signature. It does **not** export `Box<Int>.value`, `Box<String>.value`, and every other specialization.

## 2.4 Current compiled artifacts contain runtime materialization information, not semantic typing metadata

`phalcom-core/src/modules/artifact.rs:1-42` defines `RuntimeDeclarationBlueprint`, `ClassBlueprint`, and `ModuleMaterializationPlan`; these carry runtime declaration/layout/linking information.

`phalcom-core/src/modules/compile.rs` defines `CompiledModule`, `CompiledProgram`, and `AnalyzedProgram`. The analyzed program holds an `Arc<phalcom_semantic::SemanticSnapshot>`, while the compiled program currently contains linked/materialization products but no stable semantic metadata pool.

The target insertion point is therefore clear:

```text
AnalyzedProgram
    -> code generation/materialization plan
    -> semantic metadata export from the same published semantic generation
    -> CompiledProgram carries immutable metadata pool + per-module roots
    -> runtime loader validates/publishes it before reflection use
```

The metadata exporter must consume the already-analyzed semantic snapshot, not reparse or recheck source.

## 2.5 Current reflection cache is intentionally strongly rooted

`phalcom-core/src/modules/reflection_cache.rs:12-29` stores cached `ObjRef`s for module/project/package/URI reflection objects. `ReflectionCache::trace` explicitly pushes those handles as GC roots (`reflection_cache.rs:31+`).

That policy is appropriate for the small, bounded set of loaded module/project reflection descriptors. It is wrong for type reflection because user code can synthesize an unbounded number of type applications/unions/tuples/records/lambdas through explicit reflection.

Typing descriptors therefore use a separate weak cache that is **not** traversed by `ReflectionCache::trace` or the GC root set.

## 2.6 `ObjRef` is a sound weak-handle key and `Heap::try_get` is the required lookup

`phalcom-core/src/heap/mod.rs` documents `ObjRef` as a `Copy` generational `slotmap` key. A stale handle resolves to `None` rather than aliasing a newly allocated object.

Critically, the heap exposes two different APIs:

```rust
pub fn get(&self, id: ObjRef) -> &Object        // panics if stale
pub fn try_get(&self, id: ObjRef) -> Option<&Object> // stale => None
```

A weak descriptor cache **must** call `try_get`. Collection of a descriptor is normal; a stale weak cache entry is therefore a cache miss, not an internal invariant violation.

The old Spec 02 algorithm's `heap.get(obj_ref)` step is superseded.

## 2.7 The heap and `Value` representation are hot runtime surfaces

`phalcom-core/src/value/mod.rs:1-12` explicitly documents `Value` as a 16-byte `Copy` representation. `Value::class` is on the ordinary dispatch path and maps heap object variants to runtime class IDs.

`phalcom-core/src/heap/object.rs:32+` stores heap values in one tagged `Object` enum; large payloads are boxed specifically to prevent the fattest variant from inflating every arena slot. `phalcom-core/src/heap/trace.rs` uses an exhaustive match with no wildcard so a new object variant must declare its outgoing GC edges.

The target runtime design therefore obeys three performance constraints:

1. do not add a `TypeId`/kind/type-lambda immediate tag to `Value`;
2. add at most one boxed typing/reflection heap representation for synthetic typing objects;
3. keep metadata and descriptor child graphs in Rust-owned immutable data, not as eagerly allocated Phalcom object graphs.

## 2.8 `MethodObject` correctly does not contain the static semantic signature

`phalcom-core/src/method/object.rs:158-211` stores runtime implementation kind, runtime calling `Signature`, holder/access information, retained contract closures, and attributes. It does not store the compiler's canonical semantic callable signature.

That is the desired baseline. Static parameter types, generic binders, `where` constraints, inferred/public return types, `Self`, effects, and proof summaries must not be copied into every `MethodObject`.

When typing reflection is retained, the VM owns an external O(1)-or-near-O(1) side mapping from live method handle to stable callable metadata. When typing reflection is not retained, that side mapping does not exist.

## 2.9 Native metadata is symbolic but incomplete and conflates opacity

`phalcom-native-meta/src/universe.rs:3` currently declares `NATIVE_SURFACE_SCHEMA_VERSION = 1`.

`phalcom-native-meta/src/types.rs:5-58` has `KindSpec::{Type, Arrow}`, symbolic type parameter declarations, `TypeExprSpec`, and callable parameter/return structures. It currently lacks canonical variance and signature constraints, and `TypeExprSpec::Unknown` conflates several possible states.

`phalcom-semantic/src/types/native.rs` normalizes native metadata into canonical `TypeId` forms, but currently treats `TypeExprSpec::Unknown` and `SelfType` as unsupported and collapses normalization failures to `TypeKnowledge::Unknown(UnknownReason::OpaqueNative)`.

The revised native contract requires explicit `Known` versus `Opaque(reason)` and convergence through the canonical 01.5 signature model before export.

## 2.10 Runtime method mutation already has a world-version invalidation seam

`phalcom-core/src/vm/mod.rs` owns `world_version`, documented as incrementing when methods are installed/replaced. This is already the correct invalidation seam for runtime queries whose answers depend on the currently open method world.

Structural type/kind equivalence does not depend on `world_version`. Runtime member lookup, runtime conformance, typed invocation validation, DNU-sensitive checks, and proof assumptions about mutable runtime surfaces do.

---

# 3. Core architecture

## 3.1 One-way semantic authority

The canonical authority direction is:

```text
compiler/native canonical semantics
          |
          | export
          v
validated stable metadata
          |
          | load
          v
runtime reflection view
```

Runtime reflection may:

- inspect exported semantic facts;
- construct new bounded synthetic type forms through checked APIs;
- compare/query those forms;
- request explicit runtime validation against values;
- use runtime world information for explicitly dynamic queries.

It may not:

- mutate the compiler snapshot;
- make a runtime descriptor become a compiler `TypeId` by authority alone;
- fabricate a declaration signature and have it retroactively affect static checking;
- infer that an arbitrary `List` instance is a `List<Int>` because a descriptor happens to exist;
- alter selector identity, class identity, object layout, or allocation strategy.

## 3.2 Reification is demand-driven

The default path is metadata-only:

```text
metadata TypeNodeId
    |
    | descriptor not requested
    v
zero Phalcom heap objects
```

Only an explicit value boundary creates a descriptor:

```text
metadata TypeNodeId
    |
    | reify()
    v
one root descriptor object
    |
    | .argumentAt(0) requested
    v
at most one additional child descriptor
```

Reifying a deeply nested form must not recursively allocate descriptors for every descendant.

## 3.3 Nominal forms reuse runtime class objects

For any loaded nominal class declaration `D`:

```text
reify(type-form(D)) === runtime-class-object(D)
```

Examples:

```phalcom
Typing.current.type(#Int).unwrap === Int
Typing.current.type(#List).unwrap === List
```

The exact public lookup API is Spec 03's responsibility; the identity law is owned here.

No wrapper such as `NominalTypeDescriptor(Int)` is allocated for an ordinary class form. This preserves Phalcom's runtime class-object model and makes common reflection cheap.

The compiler-internal `ClassObject` static type remains a different semantic fact: the value `Int` has a runtime metaclass and a static class-object type while simultaneously denoting the nominal type form `Int`. Metadata must preserve that distinction where occurrence/debug data is retained.

## 3.4 Synthetic forms are context-canonical, not process-global objects

Applied types, unions, tuples, records, callable types, type lambdas, type parameters, `Self` terms when reified, arrow kinds, and similar structural forms may need synthetic descriptors.

Within one live `TypingContext`:

- the same canonical runtime semantic handle reifies to the same **live** descriptor object;
- the cache is weak;
- after all descriptors are collected, later reification may allocate a new object identity;
- `===` is therefore a local live-object guarantee, not durable semantic identity.

Across contexts/VMs/processes:

- `===` has no semantic meaning;
- `equivalentTo(_)` compares canonical structural semantics under compatible semantic-model versions;
- semantic hashes derive from structural fingerprints/version, not object addresses or `ObjRef` values.

## 3.5 Erasure remains absolute for ordinary runtime values

A static fact:

```text
xs : List<Int>
```

does not add an `Int` token to `xs`.

It does not change:

- `xs.class`;
- instance storage;
- GC layout;
- allocator choice;
- method table;
- selector encoding;
- inline-cache key;
- metaclass relationships.

Runtime matching against an applied generic therefore cannot claim evidence that the runtime does not possess. A shallow nominal check can establish `xs.class <: List`; it cannot establish `List<Int>` without an explicit validation witness or deep validation operation.

## 3.6 `Value` stays unchanged

No implementation of this specification may add immediate `Value` representations for:

- `TypeId`;
- `KindId`;
- metadata node IDs;
- type-lambda IDs;
- generic application IDs.

Explicit reflection is comparatively rare. Saving an allocation at that boundary does not justify taxing every value and every dispatch operation.

## 3.7 Immutable base, bounded overlay

Each runtime typing context has two semantic regions:

```text
immutable loaded metadata pools
          +
bounded append-only synthetic overlay
```

The immutable base contains compiler/native-produced metadata. The overlay contains forms explicitly constructed at runtime through reflection APIs.

Ordinary descriptor getters read immutable arrays/maps directly. They do not take a broad `RwLock`.

Overlay mutation and descriptor-cache insertion are narrow internal mutations. The current VM is single-owner/cooperative; the implementation should use ordinary VM-owned mutation rather than embedding synchronization into every descriptor. If a future parallel runtime requires synchronization, it can wrap the overlay/registry implementation without changing descriptor semantics or public data shape.

---

# 4. Retention model and metadata profiles

The old binary distinction “metadata retained versus stripped” is insufficient because some semantic roots can be executable program data.

For example:

```phalcom
const ResultOf = <T> =>> Result<T, Error>
const IntList = List<Int>
```

If those values are evaluated at runtime, the compiler must retain enough semantic representation to materialize them even in a build that disables broad declaration reflection.

## 4.1 Profiles

The logical profile hierarchy is:

```rust
pub enum MetadataProfile {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
}
```

Profiles are ordered by retained semantic visibility, but **runtime-required roots are independent of public discoverability**.

### `RuntimeMinimal`

Contains only what executable semantics require:

- type-form constants that cross into runtime value space;
- runtime type-lambda constants;
- metadata required by explicitly enabled runtime type contracts/validators;
- stable references necessary to resolve those roots;
- native ABI/type metadata that the runtime configuration requires for sound boundaries;
- schema/version/fingerprint data.

It does not imply that arbitrary declarations can be discovered through `Typing.current`.

### `RuntimePublic`

Includes `RuntimeMinimal`, plus metadata required for public typing reflection:

- public declaration forms and kinds;
- generic signatures/parameters/constraints;
- public superclass templates;
- public callable/field semantic signatures;
- transparent public alias metadata;
- public source/presentation names necessary for reflection/display;
- public runtime method-to-callable mappings where methods are materialized.

This is the normal profile for artifacts that claim user-facing typing reflection.

### `ToolingDebug`

Includes `RuntimePublic`, plus selected non-public and occurrence information:

- private/internal declaration signatures subject to access policy;
- source ranges and written spellings;
- source type-use records;
- richer diagnostic/provenance mappings;
- optional local expression facts where a debug artifact explicitly retains them.

The LSP does not require this serialized profile during normal source analysis; it reads the compiler semantic snapshot directly. This profile is for post-build/debug tooling and reproducible artifact inspection.

### `Proof`

Includes the preceding profile plus advanced analysis extension sections defined by Spec 05. This document defines the carriage/version/fingerprint envelope, not the proof calculus or evidence semantics.

## 4.2 Explicit availability state

A runtime query never receives an empty table and guesses whether metadata was stripped.

The loader records explicit availability such as:

```rust
pub enum MetadataAvailability {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
    Unavailable(MetadataUnavailableReason),
}
```

Typical unavailable reasons include:

- disabled by build profile;
- module unloaded;
- incompatible semantic model;
- corrupt/rejected metadata;
- no metadata published for dynamically installed runtime member.

Public result projection is owned by Spec 03.

## 4.3 Reachability, not whole-store dumping

Metadata export is root-driven. It must never serialize the entire `TypeStore` merely because nodes exist there.

Roots are selected from:

- profile-visible declaration signatures;
- executable runtime type-form constants;
- required runtime contracts/validators;
- aliases/supertypes reachable from those signatures;
- requested debug occurrence records;
- optional advanced extension roots.

The exporter traverses only reachable publishable semantic nodes and hash-conses them into the stable graph.

This prevents compiler scratch/inference activity from bloating artifacts.

---

# 5. VM-independent metadata crate

## 5.1 Crate placement

Create workspace crate:

```text
phalcom-type-meta/
  Cargo.toml
  src/
    lib.rs
    header.rs
    feature.rs
    identity.rs
    kind.rs
    type_node.rs
    scoped_type.rs
    generic.rs
    declaration.rs
    callable.rs
    field.rs
    alias.rs
    occurrence.rs
    extension.rs
    fingerprint.rs
    validate.rs
    encode.rs
```

The crate contains only stable owned data, logical schema validation, deterministic logical encoding support, version constants, and bounded decoding helpers.

It must not depend on:

- `phalcom-ast`;
- `phalcom-modules` concrete runtime/compiler graph types;
- `phalcom-semantic`;
- `phalcom-core`;
- `phalcom-lsp`;
- VM heap/value classes;
- native procedural macros.

A dependency on small stable primitives from `phalcom-common` is acceptable only if it does not pull compiler/runtime ownership upward.

Recommended direction:

```text
phalcom-type-meta
    <- phalcom-native-meta
    <- phalcom-semantic
    <- phalcom-core

phalcom-lsp normally consumes phalcom-semantic directly;
optional artifact inspection may depend on phalcom-type-meta.
```

## 5.2 No high-level IDs in the schema crate

The crate must not expose `ModuleId`, `DeclarationId`, `CallableId`, `FieldId`, `TypeId`, `KindId`, `InferVarId`, semantic generation IDs, or `ObjRef` in persisted records.

Adapters convert between stable schema references and subsystem identities at the boundary.

---

# 6. Header, versions, features, and identity

## 6.1 Version axes are separate

At minimum:

```rust
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 1;
pub const SEMANTIC_MODEL_VERSION: u32 = 1;

pub struct SemanticMetadataHeader {
    pub schema_version: u32,
    pub semantic_model_version: u32,
    pub producer: ProducerIdentity,
    pub producer_version: Box<str>,
    pub native_surface_schema_version: u32,
    pub profile: MetadataProfile,
    pub features: MetadataFeatures,
    pub identity_scheme: ArtifactIdentityScheme,
    pub source_fingerprint: Fingerprint128,
    pub interface_fingerprint: Fingerprint128,
}
```

`schema_version` answers “can this byte/logical record be decoded?”

`semantic_model_version` answers “do producer and consumer mean the same thing by these nodes and relations?”

A consumer must not interpret metadata with a different semantic model merely because the struct layout is readable.

Advanced effect/proof extension sections carry their own extension schema versions in addition to the core header.

## 6.2 Feature negotiation

Use explicit feature bits/IDs for optional core extensions, for example:

```rust
pub struct MetadataFeatures {
    pub type_lambdas: bool,
    pub record_rows: bool,
    pub runtime_type_constants: bool,
    pub source_occurrences: bool,
    pub advanced_sections: Box<[FeatureSectionId]>,
}
```

A schema-v1 decoder rejects an unknown **required** feature and may ignore an unknown optional opaque extension only when the header explicitly marks it skippable.

Do not reserve semantic enum variants for unratified language constructs merely to “leave room.” Versioning is the room.

## 6.3 Stable artifact/project/module identity

Durable identity must not accidentally depend on transient `ProjectId` allocation or absolute local filesystem paths.

Use an explicit identity scheme:

```rust
pub enum StableProjectRef {
    Builtin {
        namespace: Box<str>,
        version: Box<str>,
    },
    Package {
        package: Box<str>,
        version: Box<str>,
        artifact_fingerprint: Fingerprint128,
    },
    SourceArtifact {
        logical_uri: Box<str>,
        source_fingerprint: Fingerprint128,
    },
    Session {
        session_fingerprint: Fingerprint128,
    },
}

pub struct StableModuleRef {
    pub project: StableProjectRef,
    pub path: Box<[Box<str>]>,
}
```

`Session` is allowed for REPL/ephemeral in-memory metadata that is not claimed to be a reproducible package artifact.

Before a permanent disk/package cache is enabled, package/reproducibility policy must ratify exactly which project identity schemes are accepted as durable.

## 6.4 Stable declaration/member identities

```rust
pub struct StableDeclarationRef {
    pub module: StableModuleRef,
    pub path: Box<[Box<str>]>,
}

pub enum StableDispatchSide {
    Instance,
    Class,
}

pub struct StableCallableRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub selector: Box<str>,
}

pub struct StableFieldRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub name: Box<str>,
}
```

The selector string is the canonical selector identity, not a type-bearing overload key. Static parameter types never enter `StableCallableRef` identity.

---

# 7. Indexed kind graph

## 7.1 Schema

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KindNodeId(pub u32);

pub enum KindNode {
    Type,
    Arrow {
        parameters: Box<[KindNodeId]>,
        result: KindNodeId,
    },
}

pub struct KindNodeEntry {
    pub node: KindNode,
    pub structural_fingerprint: Fingerprint128,
}
```

Schema v1 core accepts the kind language ratified by 01.5: `Type` and arrows.

`RecordRow` and kind parameters are added only when their owning semantic decisions land. A v1 decoder must not accept arbitrary numeric tags as “future kinds.”

## 7.2 Canonical rules

- `Type` has exactly one canonical entry or a reserved canonical index.
- Arrow parameter order is semantic.
- Arrow flattening follows the canonical 01.5 kind representation.
- Structural fingerprint excludes node index/allocation order.
- A kind graph is acyclic and topologically ordered in core schema v1.

---

# 8. Canonical type-term graph

Metadata must encode publishable semantic terms, not merely today's `TypeData` variants.

## 8.1 Node IDs and entries

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeNodeId(pub u32);

pub struct TypeNodeEntry {
    pub kind: KindNodeId,
    pub form: TypeNode,
    pub structural_fingerprint: Fingerprint128,
}
```

## 8.2 Core node vocabulary

```rust
pub enum TypeNode {
    Never,
    Unit,

    Nominal {
        declaration: StableDeclarationRef,
    },

    Applied {
        origin: TypeNodeId,
        arguments: Box<[TypeNodeId]>,
    },

    Union(Box<[TypeNodeId]>),
    Tuple(Box<[TupleElementRef]>),
    Record(RecordTypeRef),
    Callable(CallableTypeRef),

    Parameter(StableTypeParameterRef),

    SelfType(SelfTypeRef),

    TypeLambda(TypeLambdaRef),
}
```

Notably absent:

- `InferVarId`;
- `TypeView`/environment nodes;
- `Dynamic`;
- `Unknown`;
- invalid/missing annotation;
- cancellation;
- budget exhaustion;
- compiler-internal `ClassObject` type;
- unratified `Any` or intersection tags.

These are represented through separate status/occurrence/result records when needed.

## 8.3 Applied nodes and partial application

`Applied` may have residual non-`Type` kind. Therefore metadata supports both:

```text
Map<String>          :: Type -> Type
Map<String, Int>     :: Type
```

Validation computes/checks the entry kind from the origin and argument kinds according to 01.5.

The graph stores the canonical flattened application spine. It must not encode semantically equivalent chains differently:

```text
apply(apply(Map, String), Int)
```

and:

```text
apply(Map, [String, Int])
```

export to the same canonical node structure/fingerprint.

## 8.4 `Self` is recursively representable

`Self` may occur inside another form, for example:

```text
Option<Self>
Callable(Self) -> Self
```

Metadata therefore represents owner-relative `Self` as a type node rather than limiting it to a top-level signature wrapper:

```rust
pub enum SelfRoleRef {
    InstanceType,
    ReceiverValue,
}

pub struct SelfTypeRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub role: SelfRoleRef,
}
```

This is a serialization representation choice, not a requirement that `phalcom-semantic` use `TypeData::SelfType` internally. The exporter may lower a publishable `TypeTerm::SelfType` into this metadata node.

## 8.5 `Dynamic` remains a status, not a node

If a source/native signature is explicitly dynamic, metadata records that fact in the signature slot/occurrence status. It does not intern a fake `TypeNode::Dynamic` merely to satisfy a uniform enum.

This preserves the epistemic distinction between:

- a known canonical type form;
- an explicit dynamic boundary;
- missing type information;
- unresolved/blocked information.

---

# 9. Type-lambda metadata

Type lambdas are the largest mandatory addition over the previous Spec 02.

## 9.1 Alpha equivalence must survive export

These source forms are semantically equivalent:

```text
<T> =>> List<T>
<U> =>> List<U>
```

Their durable structural fingerprints must therefore be identical, apart from separately retained presentation/provenance records.

A metadata representation that encodes source binder names into the canonical lambda body is invalid.

## 9.2 Scoped node graph

Use a separate locally-nameless/scoped graph for lambda bodies:

```rust
#[repr(transparent)]
pub struct ScopedTypeNodeId(pub u32);

pub struct ScopedTypeNodeEntry {
    pub kind: KindNodeId,
    pub form: ScopedTypeNode,
    pub structural_fingerprint: Fingerprint128,
}

pub enum ScopedTypeNode {
    /// Lambda-bound variable. `depth = 0` means the innermost lambda scope.
    Bound {
        depth: u32,
        index: u32,
    },

    /// Canonical free term from the ordinary type graph.
    Free(TypeNodeId),

    Applied {
        origin: ScopedTypeNodeId,
        arguments: Box<[ScopedTypeNodeId]>,
    },

    Union(Box<[ScopedTypeNodeId]>),
    Tuple(Box<[ScopedTupleElementRef]>),
    Record(ScopedRecordTypeRef),
    Callable(ScopedCallableTypeRef),

    /// Nested anonymous type lambda.
    Lambda {
        parameter_kinds: Box<[KindNodeId]>,
        body: ScopedTypeNodeId,
    },
}

pub struct TypeLambdaRef {
    pub parameter_kinds: Box<[KindNodeId]>,
    pub body: ScopedTypeNodeId,
}
```

`TypeNodeEntry.kind` for a `TypeLambda` is the full arrow kind and is validated against `parameter_kinds` plus body result kind.

This graph allows nested lambdas and free references to enclosing stable declaration/callable parameters without circular “lambda ID owns parameter ID owns lambda ID” identity.

## 9.3 Presentation metadata is separate

Runtime/debug display may retain source names:

```rust
pub struct TypeLambdaPresentationRecord {
    pub lambda: TypeNodeId,
    pub parameter_names: Box<[Box<str>]>,
    pub parameter_sources: Box<[Option<SourceSpanRef>]>,
    pub source: Option<SourceSpanRef>,
}
```

Names/ranges are not part of semantic fingerprints or alpha equivalence.

`RuntimeMinimal` retains presentation only when required to display an executable type-lambda value according to chosen runtime presentation policy. `RuntimePublic` retains presentation for public lambda-bearing declarations. `ToolingDebug` may retain full source ranges/spellings.

## 9.4 No executable closure semantics

A `TypeLambda` descriptor is not backed by a `ClosureObject` and does not execute user code for beta reduction.

Runtime application of a reflected type lambda uses the trusted semantic type-form evaluator over the scoped graph and bounded overlay.

---

# 10. Generic signatures, parameters, and constraints

## 10.1 Stable parameter identity

```rust
pub enum StableTypeParameterOwnerRef {
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
}

pub struct StableTypeParameterRef {
    pub owner: StableTypeParameterOwnerRef,
    pub index: u32,
}
```

Generic aliases use declaration identity and therefore reuse the declaration owner category.

Names do not define identity.

## 10.2 Parameter record

```rust
pub enum VarianceRef {
    Covariant,
    Contravariant,
    Invariant,
}

pub struct TypeParameterRecord {
    pub id: StableTypeParameterRef,
    pub name: Box<str>,
    pub kind: KindNodeId,
    pub variance: VarianceRef,
    pub source: Option<SourceSpanRef>,
}
```

Validation rule:

- non-invariant variance is legal only where 01.5 permits declaration-site variance;
- callable generic parameters must be invariant;
- type-lambda binders are **not** represented as `StableTypeParameterRef` records.

## 10.3 Signature-owned constraints

```rust
pub struct GenericSignatureRecord {
    pub owner: StableTypeParameterOwnerRef,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub constraints: Box<[GenericConstraintRef]>,
}

pub enum GenericConstraintRef {
    Subtype {
        lower: TypeNodeId,
        upper: TypeNodeId,
    },
    Equivalent {
        left: TypeNodeId,
        right: TypeNodeId,
    },
}
```

There is no canonical `upper_bound` field on `TypeParameterRecord`.

There is no `default` field in schema v1.

A parameter-focused reflection API may derive “upper bounds” or “constraints mentioning this parameter” from the owning generic signature, but that is a view, not canonical storage.

Constraint source order may be preserved in the signature record for presentation/diagnostics. Semantic fingerprints normalize according to 01.5's canonical relation rules rather than treating irrelevant source order as semantic identity.

## 10.4 No finite-set constraint tag

Schema v1 contains no `InSet`, `OneOf`, or equivalent metadata constraint corresponding to the deferred source form `T in (Int, Float)`.

When/if that language feature is ratified, it receives a semantic design and schema/version amendment then.

---

# 11. Declaration, superclass, alias, callable, and field records

## 11.1 Declaration record

```rust
pub struct DeclarationTypeRecord {
    pub declaration: StableDeclarationRef,
    pub form: TypeNodeId,
    pub kind: KindNodeId,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub superclass_template: Option<TypeNodeId>,
    pub instance_callables: Box<[StableCallableRef]>,
    pub class_callables: Box<[StableCallableRef]>,
    pub instance_fields: Box<[StableFieldRef]>,
    pub class_fields: Box<[StableFieldRef]>,
    pub flags: DeclarationTypeFlags,
    pub source: Option<SourceSpanRef>,
}
```

`superclass_template` is the semantic generic template, not merely the runtime superclass declaration name.

Example:

```phalcom
class Names<T> is Sequence<Option<T>> { ... }
```

exports one template node representing `Sequence<Option<T>>` with `T` bound to the declaration parameter identity.

The runtime object model still has one ordinary superclass class-object link. The semantic template exists to specialize static inheritance/member views; it does not create runtime specialized superclasses.

## 11.2 Transparent alias record

```rust
pub struct TypeAliasRecord {
    pub declaration: StableDeclarationRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub target: TypeNodeId,
    pub source: Option<SourceSpanRef>,
}
```

Transparent alias declaration identity is retained for navigation, reflection provenance, invalidation, and display. Semantic equivalence expands to `target` according to 01.5.

No schema-v1 field chooses opaque/newtype alias semantics.

## 11.3 Canonical callable signatures

```rust
pub struct CallableSemanticRecord {
    pub callable: StableCallableRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub parameters: Box<[CallableParameterRecord]>,
    pub return_type: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}

pub struct CallableParameterRecord {
    pub index: u32,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestModeRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}
```

Selector shape remains in `StableCallableRef`; parameter semantic types do not alter selector identity.

`CallableSemanticRecord` is the durable projection of the one canonical `CallableId -> CallableSemanticSignature` table required by 01.5. Metadata export does not rediscover signatures from runtime methods or reconstruct them from selector strings.

## 11.4 Field signatures

```rust
pub struct FieldSemanticRecord {
    pub field: StableFieldRef,
    pub mutability: FieldMutabilityRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}
```

Generated getters/setters/constructors/variants must export the exact semantic types synthesized by compiler transforms when known. A generated accessor must not become “untyped” in metadata merely because its source AST was synthesized.

## 11.5 Published type slots

Optional typing means a public declaration can exist without one canonical known type at every slot. Preserve that status explicitly:

```rust
pub enum PublishedTypeSlot {
    Known {
        form: TypeNodeId,
        authority: PublishedTypeAuthority,
    },
    Dynamic {
        reason: DynamicReasonRef,
    },
    Unknown {
        reason: UnknownReasonRef,
    },
    Unavailable {
        reason: MetadataUnavailableReason,
    },
}
```

Typical authorities:

- declared source annotation;
- trusted native declaration;
- exact generated declaration;
- publishable compiler inference.

`Unknown(UnannotatedDeclaration)` is not the same as `Dynamic(ExplicitEscape)`.

Invalid/cancelled/budget-exhausted compiler products are not silently exported as a successful `PublishedTypeSlot`; publication either fails for required roots or omits an optional/debug product with explicit artifact diagnostics according to Spec 01 publication policy.

---

# 12. Canonical form versus source occurrence

A canonical type form and a written occurrence are different artifacts.

For example:

```phalcom
type UserId = Int
const id: UserId = 1
```

The static canonical form may be equivalent to `Int`, while tooling/debug metadata may still need to say that the programmer wrote `UserId` at a particular source range.

## 12.1 Occurrence schema

Optional `ToolingDebug` records:

```rust
pub struct TypeUseRecord {
    pub owner: TypeUseOwnerRef,
    pub role: TypeUseRoleRef,
    pub status: TypeUseStatusRef,
    pub denotation: Option<DenotationRef>,
    pub written: Option<Box<str>>,
    pub source: Option<SourceSpanRef>,
}

pub enum TypeUseStatusRef {
    Known(TypeNodeId),
    InternalClassObject(StableDeclarationRef),
    Dynamic(DynamicReasonRef),
    Missing,
    Unknown(UnknownReasonRef),
    Unresolved(UnresolvedRef),
    Invalid(DiagnosticRef),
}
```

Cancellation and budget exhaustion are query execution results, not completed source-use semantic facts. A snapshot cancelled before publication does not emit an occurrence pretending analysis completed.

## 12.2 Source spelling never enters canonical hash

Exclude from canonical type fingerprints:

- alias spelling;
- generic binder names;
- lambda binder names;
- source ranges;
- whitespace/formatting;
- diagnostic prose;
- project-local numeric IDs.

---

# 13. Advanced-analysis extension carriage

The previous Spec 02 partially froze effect/termination/proof record shapes before the advanced semantics were finalized. This revision removes that ownership error.

## 13.1 Core schema does not define advanced semantics

Core schema v1 reserves a versioned extension envelope:

```rust
pub struct MetadataExtensionSection {
    pub feature: FeatureSectionId,
    pub schema_version: u32,
    pub required: bool,
    pub semantic_fingerprint: Fingerprint128,
    pub payload: Box<[u8]>,
}
```

Exact encoding may instead be typed subrecords in the final implementation; the normative point is ownership/version separation.

## 13.2 Spec 05 owns typed advanced payloads

When revised Spec 05 ratifies:

- effect summaries;
- exit summaries;
- totality evidence;
- contract IR;
- verification conditions;
- proof results/evidence/trust;

it also defines the typed schema records or feature-section adaptor needed to carry them.

Spec 02 then supplies:

- artifact inclusion;
- version negotiation;
- size/depth budgets;
- fingerprints;
- loader lifecycle;
- stale dependency handling;
- reflection handoff.

It does not decide what “proven,” “pure,” or “total” means.

## 13.3 Unknown advanced sections never become proof

An unsupported/unknown optional advanced section may be ignored only if the artifact does not require it for execution. It can never be interpreted as a successful proof/effect assertion.

Unknown **required** sections reject the artifact.

---

# 14. Artifact organization

## 14.1 Program/package-level shared pool

The current `CompiledProgram` is the best initial owner for a deduplicated metadata pool because universe/shared type forms occur across modules.

Recommended shape:

```rust
pub struct CompiledProgram {
    // existing fields ...
    pub semantic_metadata: Option<Arc<SemanticMetadataBundle>>,
}

pub struct SemanticMetadataBundle {
    pub header: SemanticMetadataHeader,
    pub kinds: Box<[KindNodeEntry]>,
    pub types: Box<[TypeNodeEntry]>,
    pub scoped_types: Box<[ScopedTypeNodeEntry]>,
    pub parameters: Box<[TypeParameterRecord]>,
    pub generic_signatures: Box<[GenericSignatureRecord]>,
    pub declarations: Box<[DeclarationTypeRecord]>,
    pub aliases: Box<[TypeAliasRecord]>,
    pub callables: Box<[CallableSemanticRecord]>,
    pub fields: Box<[FieldSemanticRecord]>,
    pub module_roots: Box<[ModuleMetadataRoot]>,
    pub runtime_roots: Box<[RuntimeTypeFormRoot]>,
    pub occurrences: Box<[TypeUseRecord]>,
    pub extensions: Box<[MetadataExtensionSection]>,
}
```

The exact use of `Box`, `Arc`, or memory-mapped slices is an implementation concern so long as loaded common reads are immutable and compact.

## 14.2 Per-module roots

```rust
pub struct ModuleMetadataRoot {
    pub module: StableModuleRef,
    pub declarations: Box<[DeclarationRecordId]>,
    pub aliases: Box<[AliasRecordId]>,
    pub callables: Box<[CallableRecordId]>,
    pub fields: Box<[FieldRecordId]>,
    pub interface_fingerprint: Fingerprint128,
}
```

Do not duplicate universe/common type nodes in every `CompiledModule`.

## 14.3 Runtime type-form constants

Spec 04 permits explicit type forms to cross into value space. The compiled artifact therefore needs symbolic runtime roots:

```rust
pub struct RuntimeTypeFormRoot {
    pub module: StableModuleRef,
    pub local_key: RuntimeTypeFormKey,
    pub form: TypeNodeId,
}
```

Bytecode/codegen may reference a compact program-local root key/instruction operand. It must not embed compiler `TypeId`.

At execution, loading such a constant asks the current runtime typing context to reify that metadata root. No descriptor is allocated at compile/load time merely because the root exists.

This requirement is independent of whether broad typing reflection is enabled.

## 14.4 Multiple loaded bundles

The runtime registry must support multiple immutable metadata bundles so future dynamic package/module loading does not require rebuilding one global graph.

Use a compact runtime pool handle:

```rust
#[repr(transparent)]
pub struct MetadataPoolId(u32);
```

A runtime semantic handle can identify `(pool, domain, node)`.

---

# 15. Deterministic export and fingerprints

## 15.1 Export consumes published semantics only

The exporter is a projection of the published semantic snapshot. It does not:

- parse source;
- run generic inference;
- solve a `where` clause for the first time;
- eagerly specialize generic members;
- infer variance;
- invent missing native signatures;
- query the runtime VM.

If a required semantic product is not publishable, export reports the corresponding structured failure from Spec 01/01.5.

## 15.2 Export algorithm

Conceptual algorithm:

1. obtain the requested published semantic generation and retention profile;
2. select root declarations/runtime type constants/debug occurrences/extensions;
3. reject wrong-store/stale/non-publishable roots;
4. traverse canonical generic signatures, constraints, supertype templates, callable/field signatures, aliases, and required type forms;
5. convert `Self` terms into stable `SelfTypeRef` nodes;
6. convert type lambdas into alpha-normalized scoped graphs;
7. reject infer variables and query-local `TypeView` environments;
8. convert module/declaration/callable/field/parameter identities to stable structural references;
9. hash-cons canonical kind/type/scoped nodes by structural semantic key;
10. deterministically order identity tables and roots;
11. validate canonical union/record/application/lambda/parameter/constraint invariants;
12. calculate structural/interface/source fingerprints;
13. build a schema object;
14. run the same validator used by the loader before artifact publication;
15. encode through the chosen canonical logical encoding.

## 15.3 Determinism law

Given semantically equivalent source/native inputs under the same semantic model and profile:

```text
export(fresh-store-A) == export(fresh-store-B)
```

must hold independent of:

- `TypeId` allocation order;
- hash-map iteration order;
- semantic query scheduling order;
- thread scheduling if analysis becomes parallel;
- transient `ObjRef`/runtime state.

## 15.4 Fingerprint domains

Keep separate fingerprints for separate invalidation questions:

- **type structural fingerprint** — canonical form only;
- **generic signature fingerprint** — parameter kinds/variance + constraints;
- **callable signature fingerprint** — selector/side/generics/parameter labels/rest/types/return;
- **declaration interface fingerprint** — public semantic surface + superclass template;
- **module interface fingerprint** — exported declaration interfaces;
- **source fingerprint** — source artifact identity/content;
- **advanced extension fingerprint** — owned by extension semantics.

Do not make a source-range edit invalidate structural type equivalence.

---

# 16. Bounded validation and decoding

All artifact metadata is untrusted input from the runtime's perspective, even when produced by Phalcom tooling.

## 16.1 Required budgets

Before large allocations, enforce configured maxima for:

- total metadata bytes;
- string bytes/count;
- kind nodes/edges;
- type nodes/edges;
- scoped lambda nodes/depth;
- declaration/callable/field records;
- generic parameters/constraints per signature;
- source occurrence records;
- runtime roots;
- extension-section bytes/count;
- total graph traversal work.

Defaults are determined from measured Phalcom corpora with safety margin. They are not copied from another compiler.

## 16.2 Iterative validation

Core validation uses explicit worklists rather than recursive Rust call stacks for potentially attacker-shaped graphs.

Schema v1 may require topological child-before-parent ordering for canonical type/kind/scoped graphs. If so:

- forward/out-of-range indexes are rejected;
- structural cycles are rejected;
- recursive aliases are not enabled by simply allowing graph cycles;
- a future recursive-type feature receives an explicit binder encoding/version feature.

## 16.3 Semantic validation

Validate at least:

- every referenced node/index exists;
- every `TypeNodeEntry.kind` agrees with its form;
- applications satisfy kind arity/argument kinds and canonical flattening;
- unions contain proper types, canonical order, no duplicates, no redundant `Never` according to 01.5/current normalization;
- tuple/record/callable components are proper where required;
- generic parameter owner/index pairs are unique;
- parameter kinds match owner signature positions;
- variance legality matches owner category;
- constraints reference valid publishable nodes and kind-compatible operands;
- declaration form/kind/signature agree;
- superclass template is proper and owner-scoped correctly;
- callable/field stable IDs agree with owner/side records;
- type-lambda bound references are in scope and body kinds are correct;
- no raw compiler/runtime IDs occur in serialized representation;
- fingerprint fields match recomputation.

## 16.4 Failure is atomic

Do not publish partially validated metadata.

Loader flow:

```text
decode scratch
  -> validate all core sections
  -> validate required extensions
  -> resolve required nominal runtime bindings
  -> publish immutable LoadedSemanticMetadata
```

Any required failure rejects the bundle as a unit.

---

# 17. Runtime loaded metadata and registry

## 17.1 Immutable loaded pool

Recommended VM-side validated form:

```rust
pub struct LoadedSemanticMetadata {
    pub header: SemanticMetadataHeader,
    pub kinds: Arc<[ValidatedKindNode]>,
    pub types: Arc<[ValidatedTypeNode]>,
    pub scoped_types: Arc<[ValidatedScopedTypeNode]>,
    pub parameters: Arc<[ValidatedTypeParameter]>,
    pub generic_signatures: Arc<[ValidatedGenericSignature]>,
    pub declarations: Arc<[ValidatedDeclarationRecord]>,
    pub aliases: Arc<[ValidatedAliasRecord]>,
    pub callables: Arc<[ValidatedCallableRecord]>,
    pub fields: Arc<[ValidatedFieldRecord]>,
    pub runtime_roots: Arc<[ValidatedRuntimeTypeRoot]>,
    pub extensions: Arc<[ValidatedExtensionSection]>,
}
```

The loaded representation may transform stable string identities into compact lookup indexes after validation. Those compact indexes are runtime-local and never serialized back as durable identity.

## 17.2 VM-owned registry

Target shape:

```rust
pub struct RuntimeTypingRegistry {
    pools: Vec<Arc<LoadedSemanticMetadata>>,
    nominal_bindings: RuntimeNominalBindingTable,
    method_semantics: MethodSemanticIndex,
}
```

This registry is owned by `VM`.

It is not a global process singleton.

It is not compiler semantic authority.

It does not need a broad `RwLock` in the current single-owner VM.

## 17.3 Runtime semantic handles

```rust
pub enum RuntimeTypeRef {
    Base {
        pool: MetadataPoolId,
        node: TypeNodeId,
    },
    Overlay(RuntimeOverlayTypeId),
}

pub enum RuntimeKindRef {
    Base {
        pool: MetadataPoolId,
        node: KindNodeId,
    },
    Overlay(RuntimeOverlayKindId),
}
```

All runtime descriptor payloads use compact validated handles rather than copied recursive trees.

---

# 18. Typing context and bounded overlay

## 18.1 Semantic meaning

A runtime `TypingContext` is an immutable semantic **view** over:

- one or more loaded metadata pools;
- a fixed metadata availability/capability policy;
- a world stamp for dynamic/runtime-sensitive questions;
- a bounded synthetic overlay for explicitly constructed runtime forms.

Its public semantic base never mutates. Internal caches/overlay append operations are implementation state, not changes to compiler truth.

## 18.2 Target data shape

Do **not** embed `Arc<RwLock<RuntimeTypeArena>>` into every descriptor.

Recommended context-owned data:

```rust
pub struct TypingContextData {
    pub base_pools: Box<[MetadataPoolId]>,
    pub overlay: RuntimeTypingOverlay,
    pub descriptor_cache: HashMap<RuntimeSemanticHandle, ObjRef>, // weak, never traced
    pub capabilities: ReflectionCapabilities,
    pub world: WorldStamp,
    pub limits: RuntimeTypingLimits,
}

pub struct RuntimeTypingOverlay {
    pub kinds: Vec<RuntimeKindNode>,
    pub types: Vec<RuntimeTypeNode>,
    pub kind_interner: HashMap<RuntimeKindNode, RuntimeOverlayKindId>,
    pub type_interner: HashMap<RuntimeTypeNode, RuntimeOverlayTypeId>,
    pub bytes_used: usize,
}
```

In the current VM, operations obtain `&mut VM` and mutate this bounded context data directly. Common descriptor reads against base pools need no synchronization.

If future parallel execution requires locks, synchronization belongs around registry/context mutation internals. It must not become part of public descriptor identity or require a lock for immutable base reads.

## 18.3 Overlay references do not copy base nodes

Overlay nodes may reference base handles directly:

```rust
pub enum RuntimeTypeHandle {
    Base { pool: MetadataPoolId, node: TypeNodeId },
    Overlay(RuntimeOverlayTypeId),
}
```

Constructing `List<Int>` at runtime does not clone the loaded `List` and `Int` metadata trees.

## 18.4 Hard limits

A context has hard caps for:

- overlay type nodes;
- overlay kind nodes;
- overlay total bytes;
- application width;
- union/record width;
- lambda substitution work;
- relation/deep-validation work, where applicable;
- descriptor allocation per operation where needed.

Exceeding a cap returns `BudgetExceeded` through the public result algebra. It is not `Blocked`, `Unknown`, or an OOM-driven implicit behavior.

---

# 19. Heap representation and GC

## 19.1 One boxed typing object arm

To preserve heap slot size, add one boxed object representation rather than one Rust `Object` variant per semantic descriptor class.

Recommended conceptual shape:

```rust
Object::Typing(Box<TypingObject>)

pub struct TypingObject {
    pub class: ClassId,
    pub payload: TypingPayload,
}

pub enum TypingPayload {
    Context(TypingContextData),
    Descriptor {
        context: ObjRef,
        handle: RuntimeSemanticHandle,
    },
}
```

Spec 03 decides which ordinary Phalcom classes (`AppliedType`, `UnionType`, `TypeLambda`, `ArrowKind`, `TypeParameter`, etc.) correspond to each descriptor handle. `TypingObject.class` is the already-resolved ordinary class object.

Alternative internal layouts are allowed if they preserve:

- one bounded boxed arena payload class of representation rather than inflating every `Object` slot;
- compact descriptor handles;
- context lifetime rooting;
- no `Value` representation change;
- no eager child object graph.

## 19.2 Descriptor roots its context

A synthetic descriptor stores the `ObjRef` of its typing context. The GC tracer pushes that context handle.

This yields a useful lifetime law:

```text
descriptor alive => context alive => loaded metadata/overlay alive
```

Therefore each descriptor does not need its own `Arc<RwLock<...>>`.

The context owns ordinary Rust metadata structures containing no Phalcom `Value` handles except where explicitly documented. The GC does not traverse Rust metadata nodes.

## 19.3 Weak descriptor cache is not a root

`TypingContextData.descriptor_cache` is intentionally **not** traversed by GC.

Algorithm:

1. look up `RuntimeSemanticHandle` in the weak cache;
2. if absent, allocate;
3. if present, call `heap.try_get(obj_ref)`;
4. if `None`, delete the stale cache entry and allocate;
5. if live, verify the object is `Object::Typing`, is a descriptor, points to this same context, and carries the expected semantic handle;
6. reuse it only after validation;
7. insert newly allocated descriptor handle into the weak cache without tracing it.

`heap.get(obj_ref)` is forbidden in step 3 because collection is an expected lifecycle event.

## 19.4 GC tracer changes

`phalcom-core/src/heap/trace.rs` deliberately uses an exhaustive match. The new typing arm must be explicit.

Conceptual edges:

```text
Typing Context:
    class
    [no descriptor-cache ObjRefs traced]

Typing Descriptor:
    class
    context
```

If later typing payloads hold any Phalcom `Value`/`ObjRef` beyond these, that field must be added to tracing and memory-management documentation in the same change.

## 19.5 Nominal reification bypasses the typing object arm

For a nominal form, return the existing class object's `Value` directly.

Do not allocate `Object::Typing` for nominal forms merely for uniformity.

---

# 20. Runtime method-to-semantic signature index

Typed reflection needs to answer questions such as “what static type was declared for this method's parameter?” without putting that information into `MethodObject`.

## 20.1 Side-table design

Use a VM-owned side table equivalent to:

```rust
pub struct MethodSemanticIndex {
    by_method: slotmap::SecondaryMap<ObjRef, RuntimeCallableRef>,
}

pub struct RuntimeCallableRef {
    pub pool: MetadataPoolId,
    pub record: CallableRecordId,
}
```

A plain/hash map keyed by generational `ObjRef` is also acceptable if stale entries are pruned correctly. `SecondaryMap` is recommended because the heap already uses slotmap generational keys and because it does not make the method object itself larger.

## 20.2 Installation lifecycle

When compiler/runtime materialization creates a `MethodObject` from a callable that has retained semantic metadata:

1. allocate/install the method normally;
2. record `(method ObjRef -> RuntimeCallableRef)` in the side table;
3. method dispatch continues to use the existing runtime `Signature` only.

If typing reflection metadata is stripped, skip step 2.

If a method is dynamically replaced with no semantic metadata, the new method has no authoritative static signature mapping. Reflection reports explicit unavailability/dynamic boundary rather than borrowing the previous method's signature.

## 20.3 No type-directed runtime dispatch

The side table is observational metadata. Runtime selector lookup does not consult it.

Typed-dispatch libraries may explicitly inspect it through Spec 03 APIs, but that is user-level dynamic dispatch implemented *on top of* ordinary Phalcom method identity.

---

# 21. Reification algorithms

## 21.1 Reify a type

Given `(context, RuntimeTypeHandle)`:

1. validate context capability/metadata availability for the requested operation;
2. validate the handle belongs to one of the context's base pools or overlay;
3. if the form is nominal, resolve the stable declaration through `RuntimeNominalBindingTable` and return the existing class object;
4. otherwise probe the context's weak descriptor cache using `heap.try_get`;
5. determine the descriptor runtime class from the semantic node category;
6. charge descriptor-allocation budget;
7. allocate one boxed `Object::Typing` descriptor containing only class/context/handle;
8. insert the weak handle;
9. return it.

Child descriptors are not created in this algorithm.

## 21.2 Reify a child lazily

For operations such as:

```text
applied.argumentAt(0)
callable.parameterTypeAt(2)
lambda.body
kind.argumentAt(0)
```

read the child node handle from immutable metadata or overlay and call the root reification algorithm only for that child.

A `.arguments` convenience collection may lazily wrap indexed access; it must not force all children merely because the parent exists.

## 21.3 Reify a kind

- `Type` uses one VM-canonical atomic kind singleton supplied by the typing universe/bootstrap.
- Arrow kinds use weakly cached synthetic descriptors.
- kind descriptor creation does not create type descriptor children until requested.

Exact public class names/selectors remain Spec 03's responsibility.

## 21.4 Reify a type lambda

A lambda descriptor retains the canonical type-lambda node handle. It may expose parameter count/kinds/names and body through lazy operations.

It does not allocate declaration `TypeParameter` objects for its bound variables as semantic identity. Reflected parameter wrappers may expose `(lambda descriptor/node, index)` as logical identity, with source names as presentation.

---

# 22. Runtime type-form composition

Runtime composition is explicit reflection work. It is not ordinary source type checking and not user-overridable dispatch masquerading as semantic normalization.

## 22.1 Applying a reflected constructor

Conceptual `TypingContext.apply(origin, args)`:

1. require the construction capability defined by Spec 03;
2. resolve origin/argument handles;
3. read kinds without reifying child objects;
4. run canonical kind application rules;
5. create a small specialization environment for declaration/lambda parameters;
6. specialize known signature constraints and evaluate those now decidable;
7. for a nominal constructor/application, hash-cons the canonical flattened overlay `Applied` node;
8. for a type lambda, beta-reduce over the scoped graph with capture avoidance and directly hash-cons the normalized result rather than materializing an intermediate `Applied(TypeLambda, ...)` if unnecessary;
9. enforce overlay/work budgets;
10. return/reify the resulting handle on demand.

## 22.2 Partial application

Partial application remains valid and returns a constructor-kinded runtime form when semantically valid:

```text
Map<String> :: Type -> Type
```

`remainingParameters` is computed from the canonical constructor/application semantics. It is distinct from `freeParameters`.

## 22.3 Union/tuple/record/callable composition

Checked runtime constructors use the same normalization laws as 01.5:

- unions flatten/deduplicate/remove redundant `Never`/canonicalize order;
- tuple/record/callable children must be proper `Type` where required;
- record fields use canonical label order for semantic identity while presentation order may be retained separately where meaningful;
- no raw descriptor class constructor permits bypassing validation.

## 22.4 Runtime-created forms are not compiler truth

Overlay forms can be compared, displayed, applied, and used for explicit validation. They do not enter a past compiler semantic generation.

A future incremental compiler/REPL may separately accept a runtime-produced form as explicit input only through a new trusted boundary; this specification does not define one.

---

# 23. World-sensitive and world-insensitive queries

## 23.1 World stamp

```rust
pub struct WorldStamp {
    pub vm_world_version: u64,
    pub loaded_interface_fingerprint: Fingerprint128,
    pub native_surface_fingerprint: Fingerprint128,
}
```

The exact aggregate fingerprint calculation is implementation-defined but deterministic.

## 23.2 Structural queries ignore world mutation

These remain valid when methods are installed/replaced:

- kind of a canonical form;
- structural equivalence;
- application origin/arguments;
- lambda alpha-equivalent structure;
- generic declaration parameter kinds/variance/constraints from immutable artifact metadata;
- canonical hash/display derived only from type structure.

## 23.3 Runtime member/conformance queries observe world mutation

These depend on current runtime method surfaces:

- “does this live class currently respond to selector X?”;
- runtime protocol/conformance checks that inspect mutable method tables;
- reflected invocation validation;
- DNU-sensitive reasoning;
- proof/contract assumptions that explicitly include mutable runtime members.

When `VM.world_version` changes, only world-sensitive caches are invalidated.

Do not throw away immutable metadata pools or recreate structural descriptors merely because a method changed.

---

# 24. Unload, generations, and stale data

## 24.1 Metadata pool lifetime

A loaded program/package registry strongly retains its validated metadata pools while they are installed.

A live typing context may retain the pool set/generation it was created against even if a newer REPL/compiler generation exists. This matches Spec 01.5's rule that old contexts can remain pinned to old published semantic generations.

## 24.2 Module unload

If module unloading removes a runtime nominal declaration while a structural descriptor survives:

- structural metadata remains inspectable if its context still retains the metadata pool;
- operations requiring a live runtime class/member return explicit `Unavailable(UnloadedDeclaration)`/corresponding Spec 03 result;
- the runtime never silently retargets the stable declaration to a different class with the same name.

## 24.3 Stale weak object handles

A stale descriptor cache handle is removed lazily on lookup and may also be swept opportunistically after GC.

No stale `ObjRef` is dereferenced with a panicking accessor.

---

# 25. Native metadata convergence

## 25.1 Separate syntax, same semantics

`phalcom-native-meta` and `phalcom-type-syntax` may retain their own compact source/macro-facing syntax model. They must lower into the same canonical semantic signature language as source declarations before stable metadata export.

There is no separate “native generic semantics.”

## 25.2 Replace ambiguous `Unknown`

Target conceptual shape:

```rust
pub enum NativeTypeSurface<T> {
    Known(T),
    Opaque {
        reason: NativeOpaqueReason,
    },
}
```

A missing required type entry is a native metadata/build error.

`Opaque` is explicit and becomes a dynamic/open-world boundary with provenance. It is not canonical `Unknown` and not a successful type node.

## 25.3 Native generic signatures

Native declarations must be able to describe:

- generic parameter kind;
- declaration-site variance where legal;
- signature-owned subtype/equivalence constraints;
- `Self`;
- canonical applications/unions/tuples/records/callables;
- type lambdas when needed by native public APIs;
- callable generic parameters independently of declaration parameters.

Parameter references normalize by owner/index, not global name identity.

## 25.4 Native/source parity gate

Where a bundled `.ph` declaration and native metadata describe the same public callable/declaration, canonical normalized signatures must compare equal.

A parity test failure is a build/CI failure, not a runtime warning.

## 25.5 Version/fingerprint startup check

Runtime/compiler/native components validate:

- native surface schema version;
- semantic model version;
- native surface fingerprint.

Incompatibility fails before metadata is used as authoritative runtime reflection data.

---

# 26. Security, authority, and access control

## 26.1 Metadata is not an access-control bypass

Public reflection policy is Spec 03's responsibility, but the loader/registry must preserve enough owner/visibility identity to enforce it.

Private/internal signatures retained in `ToolingDebug` are not automatically exposed to arbitrary runtime code.

## 26.2 Runtime-created descriptors carry no compiler authority token

User code may construct a semantically valid `List<Int>` descriptor in a context. That does not prove that a particular runtime value was statically checked as `List<Int>`.

## 26.3 Hostile metadata

Decoder errors:

- bound attacker-controlled string lengths;
- report bounded node paths/indexes;
- avoid recursively formatting untrusted graphs;
- never partially register classes/method-semantic mappings from an invalid bundle;
- never execute payload bytes during validation.

## 26.4 Advanced evidence trust

Future proof artifacts/extensions must carry explicit trust/version/fingerprint data defined by Spec 05. Loader support must never convert “extension parsed” into “proof trusted.”

---

# 27. Diagnostics

Required stable diagnostic categories include:

| Code | Typical severity | Meaning/action |
|---|---:|---|
| `metadata.schema.unsupported` | error | schema version cannot be decoded |
| `metadata.semantic_model.mismatch` | error | producer/consumer semantic laws differ |
| `metadata.feature.unsupported` | error/info | required feature unsupported; optional section may be skipped only when marked skippable |
| `metadata.malformed` | error/internal | index/ordering/fingerprint/shape violation |
| `metadata.budget_exceeded` | error | reject oversized/hostile metadata before partial publication |
| `metadata.kind_mismatch` | error/internal | node kind contradicts canonical form |
| `metadata.generic.owner_mismatch` | error/internal | parameter/signature owner or index invalid |
| `metadata.generic.constraint_invalid` | error/internal | constraint operands/kinds invalid |
| `metadata.lambda.scope_invalid` | error/internal | bound-variable depth/index escapes lambda scope |
| `metadata.lambda.kind_invalid` | error/internal | lambda body/parameter kind mismatch |
| `metadata.nominal.unresolved` | error | required declaration cannot resolve to loaded runtime class |
| `metadata.runtime_root.unresolved` | error | executable type-form constant cannot be materialized |
| `metadata.profile.unavailable` | information/error by operation | requested reflection data was intentionally stripped |
| `native.surface.missing` | build error | required native semantic surface absent |
| `native.surface.opaque` | warning/strict error | explicit native dynamic boundary |
| `native.surface.version_mismatch` | error | native/compiler/runtime versions disagree |
| `reflection.context.stale_world` | result/warning | world-sensitive result requires refresh/requery |
| `reflection.budget_exceeded` | runtime typed error | explicit runtime semantic operation exceeded limit |
| `reflection.metadata.unloaded` | result | structural data may exist but live nominal/member target is gone |

Cancellation and budget exhaustion remain distinct throughout the compiler/query/export pipeline. Do not map either to `Unknown`/`Blocked` metadata.

---

# 28. Observability and performance requirements

This subsystem exists partly to make rich typing/reflection affordable. Performance counters are therefore normative acceptance data, not optional debugging extras.

Measure at least:

### Export/build

- reachable semantic roots;
- unique kind nodes;
- unique type nodes;
- scoped lambda nodes;
- deduplication ratio;
- metadata bytes by section;
- export time;
- validation time;
- fingerprints computed/reused.

### Runtime load

- metadata bytes loaded;
- validate/decode time;
- immutable pool bytes;
- nominal binding count;
- method-semantic side-table entries.

### Runtime reflection

- descriptors allocated;
- descriptors reused from weak cache;
- stale weak cache misses;
- child descriptors materialized;
- overlay node count/bytes/high-water;
- overlay interner hit rate;
- runtime semantic-operation budget failures.

## 28.1 Allocation laws

After metadata load, merely having typing metadata available allocates **zero** synthetic type descriptor objects.

Reifying one synthetic root allocates O(1) Phalcom objects.

Reading a scalar property such as kind tag, parameter count, variance, selector, or node fingerprint should allocate zero Phalcom objects.

Indexed child lookup allocates at most the demanded child descriptor when that child is synthetic and not already live.

## 28.2 Locking laws

In the current single-owner VM:

- immutable base metadata reads take no synchronization primitive;
- descriptor property reads do not acquire a global type-registry lock;
- overlay/cache mutation is scoped to the active context operation;
- compiler/LSP static queries never acquire runtime typing locks because they do not use this runtime subsystem.

## 28.3 Runtime invariance benchmark

Programs that never use runtime type reflection and whose build profile does not require runtime validation must show no material regression in:

- `Value` size;
- object slot size beyond the separately boxed new enum discriminant effect;
- ordinary message dispatch;
- object allocation;
- GC tracing of ordinary objects;
- selector lookup.

Any measurable regression requires profiling evidence and explicit review.

---

# 29. Dependency-ordered implementation plan

The old B1–B6 plan is superseded by this sequence.

## B0 — Semantic readiness gate

**Depends on:** relevant Spec 01 identity/publication infrastructure plus Spec 01.5 G1–G8 semantic products.

Before freezing core metadata schema, verify the semantic layer can publish:

- generic parameters with kind/variance;
- signature-owned constraints;
- type lambdas with alpha-normalized body representation;
- generic superclass templates;
- owner-relative `Self` terms;
- canonical callable signatures;
- canonical field signatures;
- stable declaration/callable/field identities;
- transparent alias targets where aliases are implemented;
- no escaping inference vars/views.

If a semantic product is still provisional, implement the schema crate around an internal feature branch but do not declare schema v1 frozen.

## B1 — Create `phalcom-type-meta` core schema and validator

**Create:**

```text
phalcom-type-meta/Cargo.toml
phalcom-type-meta/src/lib.rs
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/feature.rs
phalcom-type-meta/src/identity.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/callable.rs
phalcom-type-meta/src/field.rs
phalcom-type-meta/src/alias.rs
phalcom-type-meta/src/occurrence.rs
phalcom-type-meta/src/extension.rs
phalcom-type-meta/src/fingerprint.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/encode.rs
```

**Modify:** root workspace `Cargo.toml` and dependent crate manifests.

**Implementation order:**

1. IDs/header/features/identity records;
2. kind graph;
3. global type graph including `Self`;
4. scoped lambda graph;
5. parameter/generic signature/constraint records;
6. declaration/superclass/alias/callable/field records;
7. profile/root/occurrence/extension records;
8. structural fingerprint implementation;
9. iterative validator and budget object;
10. deterministic logical encoder used by fixtures/tests.

**Acceptance:** crate has no AST/semantic/core/LSP/VM dependency and no raw high-level ID types.

## B2 — Replace recursive durable export with metadata exporter

**Create:**

```text
phalcom-semantic/src/metadata/mod.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/metadata/reachability.rs
phalcom-semantic/src/metadata/stable_identity.rs
phalcom-semantic/src/metadata/fingerprint.rs
phalcom-semantic/src/metadata/lambda.rs
phalcom-semantic/tests/metadata_export.rs
```

**Modify:**

```text
phalcom-semantic/src/export.rs
phalcom-semantic/src/lib.rs
```

`export_type_form` remains temporarily as compatibility/test adapter backed by or differential-tested against the new graph exporter.

**Tests:**

- fresh-store deterministic equality;
- type-lambda alpha-equivalence;
- generic shadowing owner/index identity;
- signature constraints round trip;
- generic superclass template export;
- nested `Self` export;
- no infer/view/raw IDs;
- runtime-root reachability does not dump unrelated store nodes;
- `RuntimeMinimal` versus `RuntimePublic` root selection.

## B3 — Artifact carriage without runtime descriptors

**Modify:**

```text
phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/registry.rs
phalcom-core/src/modules/materialize.rs   # or the actual current materialization module after inspection
```

Add program/package-level shared semantic metadata carriage and compact per-module/runtime roots.

At this stage:

- export metadata;
- attach it to `CompiledProgram`;
- decode/validate it in tests;
- do **not** create typing heap objects yet.

This isolates artifact correctness from reflection/GC complexity.

## B4 — Runtime immutable pool loader and nominal binding table

**Create:**

```text
phalcom-core/src/typing/mod.rs
phalcom-core/src/typing/loader.rs
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/handle.rs
phalcom-core/src/typing/limits.rs
```

**Modify:** `phalcom-core/src/vm/mod.rs` to own `RuntimeTypingRegistry`.

Load/validate immutable pools, assign runtime `MetadataPoolId`, resolve stable nominal references to existing runtime class objects, and reject required unresolved roots atomically.

Still allocate no synthetic descriptors.

## B5 — Typing context, boxed descriptor, and weak cache

**Create:**

```text
phalcom-core/src/typing/context.rs
phalcom-core/src/typing/overlay.rs
phalcom-core/src/typing/reify.rs
phalcom-core/src/heap/typing.rs
```

**Modify:**

```text
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/universe/...             # exact class bootstrap files after Spec 03 class list is finalized
```

Requirements:

- one boxed typing heap arm;
- descriptor roots context;
- context cache does not root descriptors;
- cache uses `Heap::try_get`;
- nominal reification returns existing class object;
- base descriptor read path is lock-free in current VM;
- overlay bounded;
- no `Value` tag change.

## B6 — Runtime type constants and checked overlay composition

Wire Spec 04 explicit runtime type-form expressions to compact metadata runtime roots.

Implement overlay operations for:

- application/partial application;
- type-lambda beta reduction;
- union normalization;
- tuple/record/callable construction as authorized by Spec 03;
- lazy child descriptor access.

Do not implement user-facing selector names before revised Spec 03 is ratified; internal tests may invoke Rust APIs directly.

## B7 — Method semantic index and native convergence

**Modify:**

```text
phalcom-core/src/method/object.rs            # no new semantic fields; only integration call sites if necessary
phalcom-core/src/modules/materialization...  # actual method install path
phalcom-core/src/vm/dispatch.rs              # world-version integration only if needed
phalcom-native-meta/src/types.rs
phalcom-native-meta/src/universe.rs
phalcom-native-meta/src/lib.rs
phalcom-native-macros/src/lib.rs
phalcom-semantic/src/types/native.rs
```

Create VM-owned method-handle to callable-record mapping, populate it only for retained authoritative signatures, and replace ambiguous native `Unknown` with explicit opacity.

Add source/native canonical parity tests.

## B8 — Profiles, debug occurrences, extension envelope

Implement `RuntimeMinimal` first, then `RuntimePublic`.

`ToolingDebug` follows once source-use query products are stable.

`Proof` extension payloads wait for revised Spec 05; implement only the generic extension envelope beforehand.

## B9 — Fuzzing, corpus sizing, GC/performance gates

Add:

- hostile metadata decode fuzzing;
- scoped-lambda depth/index fuzzing;
- canonical graph permutation tests;
- descriptor GC/reuse/stale-handle tests;
- runtime overlay budget tests;
- metadata corpus size/load benchmarks;
- zero-reflection runtime regression benchmarks.

Freeze default limits only after measurements.

---

# 30. Migration and deletion criteria

## 30.1 Migration sequence

1. keep current `CompiledTypeRef` export for compatibility tests;
2. implement common logical schema/validator;
3. export new graph alongside recursive export;
4. differential-test shared current forms;
5. add 01.5-only forms (constraints/lambdas/`Self`/superclass/signatures);
6. attach graph to compiled artifacts;
7. load/validate graph without reflection objects;
8. add `RuntimeMinimal` runtime roots;
9. add runtime context/nominal reification;
10. add synthetic descriptors/weak cache;
11. add method semantic side table;
12. expose revised Spec 03 API;
13. add public/debug profiles;
14. remove transitional recursive export once no compiler/test/artifact consumer requires it.

## 30.2 Delete/forbid after migration

Delete or prevent:

- durable use of recursive `CompiledTypeRef` as wire schema;
- `TypeParameterRecord.upper_bound` as canonical storage;
- `TypeParameterRecord.default` before defaults are designed;
- per-parameter canonical constraint arrays duplicating signature constraints;
- broad per-descriptor `Arc<RwLock<RuntimeTypeArena>>` as permanent representation;
- weak-cache calls to `Heap::get`;
- strong rooting of synthetic type descriptor cache entries;
- semantic signature fields added directly to `MethodObject`;
- automatic eager descriptor trees;
- metadata paths that serialize `InferVarId`, query-local views, cancellation, or budget exhaustion as type nodes;
- native `Unknown` meaning “anything we could not normalize.”

---

# 31. Verification matrix

## 31.1 Schema and canonicalization

Required tests:

- byte/logical deterministic export from independent fresh `TypeStore`s;
- structural fingerprints independent of node numbering;
- round-trip all core 01.5 publishable kind/type forms;
- parameter owner/index survives same-name shadowing;
- generic constraints stay on owning signature;
- variance survives round-trip and illegal method variance is rejected;
- no generic default field/tag exists;
- generic superclass template specializes equivalently after re-import;
- transparent alias declaration provenance survives while semantic target remains equivalent;
- canonical application flattening survives round-trip;
- `Self` nested inside application/callable survives round-trip.

## 31.2 Type-lambda laws

- `<T> =>> List<T>` and `<U> =>> List<U>` export to identical structural fingerprint;
- nested lambda depth/index validates;
- capture-avoiding beta reduction on loaded/overlay representation matches semantic result;
- partial lambda application produces equivalent residual constructor;
- lambda source names/ranges do not affect semantic fingerprint;
- malformed bound depth/index is rejected within budget;
- no lambda binder is exported as a declaration/callable `StableTypeParameterRef`.

## 31.3 Publication hygiene

- inference variables cannot export;
- query-local `TypeView`/environment cannot export;
- internal `ClassObject` static value type cannot masquerade as nominal form;
- explicit `Dynamic` remains status, not type node;
- unannotated and dynamic slots remain distinguishable;
- cancelled/budgeted analysis publishes no fake completed metadata;
- runtime root reachability exports only required graph.

## 31.4 Runtime object-model invariants

- `reify(Int) === Int`;
- `reify(List) === List`;
- `reify(List<Int>).class` is the descriptor class chosen by Spec 03, not `List` and not a new specialized class;
- ordinary `List.new` dispatch is unchanged by existence of `List<Int>` descriptor;
- no class/metaclass/superclass wiring changes;
- `Value` remains 16 bytes;
- ordinary instances carry no generic tokens;
- selector identity contains no type data.

## 31.5 Lazy allocation and weak cache

- loading metadata allocates zero synthetic type descriptors;
- reifying one synthetic root allocates one root descriptor only;
- querying scalar metadata allocates nothing;
- querying one child allocates at most that child descriptor;
- repeated live reification returns same `ObjRef`;
- dropping all live descriptors permits GC collection;
- cache does not keep descriptor alive;
- stale cache `ObjRef` returns `None` via `try_get`, is removed, and is safely replaced;
- stale handle can never alias a newly recycled slot due generation validation;
- context survives while any descriptor roots it;
- context/overlay memory releases once context and descriptors are unreachable.

## 31.6 Method metadata

- runtime `MethodObject` size/layout does not gain canonical semantic signature fields;
- retained compiled method maps to exact callable metadata record;
- dynamically replaced method without metadata has no stale mapping;
- stripped profile creates no method semantic side-table entry;
- runtime dispatch result is identical with and without side-table presence.

## 31.7 Metadata profiles

- `RuntimeMinimal` can materialize executable type-form constants while arbitrary public declaration discovery remains unavailable;
- `RuntimePublic` exposes public declaration/callable/field generic metadata;
- `ToolingDebug` adds source occurrence data without changing canonical fingerprints;
- unsupported requested profile reports explicit unavailability;
- stripping public reflection does not break explicit runtime type-form constants.

## 31.8 Native convergence

- native/source equivalent signature normalizes to identical canonical metadata;
- missing required native type is build failure;
- explicit opaque native surface stays dynamic/opaque with reason;
- type lambda/variance/constraint native forms round-trip once syntax producers support them;
- native schema/model version mismatch fails before reflection.

## 31.9 Hostile decoding

- out-of-range indexes;
- forward indexes when topological ordering required;
- cycles;
- huge string lengths;
- huge node counts;
- huge lambda depth;
- invalid bound variable depth/index;
- duplicate owner/index parameter identities;
- wrong node kinds;
- invalid application arity/kinds;
- duplicate/unsorted canonical union or record nodes;
- mismatched fingerprints;
- unknown required features/extensions;
- truncated input.

All terminate within configured budget without stack overflow or partial publication.

## 31.10 Performance/corpus

Record and gate:

- metadata size for universe/std/representative project;
- export time cold/incremental;
- load/validate time;
- base pool memory;
- descriptor allocations for representative reflection tasks;
- weak-cache hit/stale rate;
- overlay high-water;
- GC reclamation;
- ordinary no-reflection runtime benchmark delta.

---

# 32. Cross-spec ownership and amendments

## 32.1 Spec 01

Spec 01 owns:

- compiler-owned `SemanticDb`;
- store/snapshot/generation identities;
- publishability;
- cancellation/budgets;
- deterministic query publication;
- relation outcome infrastructure.

Spec 02 consumes published semantic products. It does not create a second DB.

If Spec 01 changes store epoch representation, only exporter adapters should change; persisted schema remains raw-ID independent.

## 32.2 Spec 01.5

Spec 01.5 is semantic authority for:

- kinds/type application;
- generic parameters;
- variance;
- constraints;
- type lambdas;
- substitution/specialization;
- generic inheritance;
- `Self`;
- canonical callable/field signatures;
- generic inference.

Spec 02 serializes/reifies those products. If an implementation choice in this document appears to change their meaning, 01.5 wins.

## 32.3 Revised Spec 04

Spec 04 owns syntax and lowering. Relevant integration:

- explicit runtime type-form expressions create `RuntimeTypeFormRoot` artifact references;
- type-lambda binder names/ranges enter presentation/occurrence metadata, not canonical lambda identity;
- parser recovery/invalid syntax never becomes successful metadata;
- `Dynamic` remains an explicit status;
- `Unknown` is not a source type.

## 32.4 Spec 03 must be revised against this architecture

The revised Spec 03 should assume:

- nominal forms are existing class objects;
- synthetic descriptors carry compact context/node handles;
- descriptor children are lazy/indexed;
- `TypeLambda` is reifiable;
- `freeParameters` and `remainingParameters` are distinct;
- declaration/callable/field metadata is available by stable record IDs;
- result states use sealed variant classes and keep `Cancelled`/`BudgetExceeded` distinct;
- runtime generic matching is erased unless an explicit validation witness/deep check exists;
- public operations cannot rely on a mutable broad arena lock.

## 32.5 Revised Spec 05

Spec 05 should define advanced effect/proof payload semantics. Spec 02 supplies only the generic extension carriage until then.

## 32.6 Spec 07

The implementation plan must sequence:

```text
01 semantic infrastructure
    -> 01.5 publishable generic core
        -> 02 B1/B2 schema+export
        -> 04 parser/lowering where dependencies permit
        -> 02 B3/B4 artifact+loader
        -> 03 runtime reflection surface
```

Metadata implementation can overlap late 01.5 work only after the B0 semantic readiness contracts it consumes are stable.

---

# 33. Risks and gates

| Risk | Required gate |
|---|---|
| Schema freezes today's incomplete `TypeData` | B0: no schema-v1 freeze before 01.5 lambda/constraint/`Self`/signature shapes exist |
| Type lambda binder names leak into identity | alpha-equivalence fingerprint/property tests |
| Per-parameter bounds/defaults reappear | schema review: constraints only on signature; no default field/tag |
| Metadata exporter starts solving generics | exporter consumes publishable semantic products only; no solver dependency in exporter API |
| Recursive tree becomes durable ABI | indexed graph required; recursive export transitional only |
| Reflection stripping breaks runtime type constants | `RuntimeMinimal` required-root tests |
| Broad runtime lock contaminates every descriptor read | immutable base read benchmark + data-shape review |
| Weak cache panics after GC | `try_get` stale-handle regression test |
| Descriptor cache becomes GC root | tracer/root-set review and reclamation test |
| New typing variant inflates heap slots | boxed object representation and size assertion |
| Semantic signatures bloat all methods | external side-table size/layout test |
| Generic metadata affects runtime dispatch | dispatch differential tests with metadata enabled/disabled |
| Runtime generic descriptor is mistaken for per-instance evidence | deep-validation API/result tests in Spec 03 |
| Absolute source paths leak into reproducible artifacts | stable identity scheme/reproducibility tests before disk/package persistence |
| Proof/effect semantics frozen prematurely | core extension envelope only until revised Spec 05 |
| Hostile metadata causes allocation/stack DoS | iterative validator + preallocation budgets + fuzzing |
| Dynamic method replacement leaves stale static signature mapping | generational method index + world/version/install tests |

---

# 34. Ratified decisions checklist

An implementation conforms to this specification only if all of the following are true:

1. Durable metadata is a versioned indexed graph, not a recursive `CompiledTypeRef` tree.
2. Metadata depends on canonical semantics from both Spec 01 and Spec 01.5.
3. Raw store/query/solver/runtime IDs never cross the artifact boundary.
4. Type lambdas are encoded alpha-normalized with scoped bound variables.
5. Generic parameter canonical records contain owner/index/name/kind/variance/provenance, not canonical per-parameter bounds/defaults.
6. Generic constraints belong to the owning generic signature.
7. Schema v1 has exactly the ratified subtype/equivalence generic constraints; finite-set constraints remain absent.
8. Generic superclass templates are exported semantically.
9. Canonical callable and field signatures are exported by stable identity.
10. Owner-relative `Self` can appear recursively inside exported terms.
11. Transparent alias declaration provenance is separate from canonical target equivalence.
12. Export does not run generic inference or eager specialization.
13. Runtime-required type-form roots survive `RuntimeMinimal` even when public reflection is stripped.
14. Loading metadata allocates no synthetic Phalcom type descriptor objects.
15. Reification is lazy and nominal forms return existing class objects.
16. Synthetic descriptors are context-canonical only while live and are weakly cached.
17. Weak cache uses `Heap::try_get`, never panicking `Heap::get`, for potentially stale handles.
18. Descriptor caches are not GC roots.
19. A descriptor keeps its typing context alive; the context owns metadata/overlay lifetime.
20. Ordinary immutable descriptor reads do not require a broad registry/arena lock.
21. Runtime synthetic overlay is bounded and does not copy immutable base graphs.
22. `Value` remains unchanged; no type/kind immediate tag is added.
23. One boxed typing representation prevents heap slot-size inflation.
24. Ordinary runtime instances never receive generic tokens.
25. Static type metadata never enters selector identity or ordinary runtime method lookup.
26. `MethodObject` remains free of full canonical semantic signatures; a VM-owned side table supplies reflection mapping when retained.
27. Dynamic method replacement without metadata does not inherit stale semantic signature authority.
28. `Dynamic`, `Unknown`, missing, invalid, blocked, cancelled, and budget-exhausted states are never collapsed into a canonical type node.
29. Native metadata lowers into the same canonical semantic signature language and uses explicit opacity rather than ambiguous `Unknown`.
30. Advanced effect/proof payload semantics are versioned extensions owned by revised Spec 05.
31. Decoding/validation is bounded, iterative, deterministic, and atomic.
32. Structural fingerprints are independent of internal node numbering and source spelling.
33. World mutation invalidates only world-sensitive runtime queries, not structural metadata identity.
34. Public reflection API spelling and sealed result projection remain Spec 03's responsibility.
35. Runtime metadata/reification cost is paid only when metadata is retained/loaded or reflection/runtime type forms are explicitly used; ordinary dispatch/layout semantics remain invariant.

---

# 35. Take directly / adapt / reject

## 35.1 Take directly

Retain from the previous design and the current implementation trajectory:

- dense internal semantic IDs with a stable export boundary;
- indexed durable graph rather than recursive persisted trees;
- explicit schema/model versions and fingerprints;
- immutable loaded metadata;
- class objects as nominal type-form runtime values;
- generational `ObjRef` as safe weak-cache handle;
- explicit metadata profiles;
- deterministic export tests;
- hostile-input budgets;
- open-world `world_version` invalidation seam.

## 35.2 Adapt

Adapt those mechanisms to the completed generic semantic language:

- type lambdas use scoped alpha-normalized metadata;
- generic constraints are signature-owned;
- variance and generic supertype templates are first-class records;
- `Self` is owner-relative and recursively serializable;
- callable/field metadata is identity-indexed rather than reconstructed from runtime methods;
- runtime reflection uses compact context/node handles and a bounded overlay rather than a broad arena lock;
- executable type-form constants are independent from broad reflection retention;
- effects/proofs use extension carriage rather than being prematurely frozen in core schema.

## 35.3 Reject

Reject as incompatible with Phalcom's architecture/performance philosophy:

- serializing `TypeStore` memory or dense IDs;
- persistent pointer/object identity as semantic identity;
- per-instance generic metadata;
- specialized runtime classes/metaclasses for generic applications;
- type-bearing selector identity;
- always-reified type object graphs;
- strong global caching of unbounded synthetic descriptors;
- `Arc<RwLock<RuntimeTypeArena>>` on every descriptor as the permanent model;
- weak-cache dereference through panicking heap access;
- adding type IDs to the hot 16-byte `Value` representation;
- copying canonical static signatures into every runtime method object;
- using `Dynamic`/`Unknown` as metadata escape hatches for invalid/cancelled/budgeted semantics;
- treating metadata decoding as proof trust;
- freezing an unratified permanent binary package format during this work.

---

## Evidence note

This revision was produced from the attached previous Spec 02, the ratified Spec 01.5 and revised Spec 04 design, and repository archaeology against `aureat/phalcom-lang@a43f26e0ddd6b1d6e37ddf7a0b9588769bb41f3e`. It does not claim that the target metadata/reification implementation already exists. No repository test suite was executed as part of writing this documentation artifact.
