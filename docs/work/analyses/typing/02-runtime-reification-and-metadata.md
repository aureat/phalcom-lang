# 02 — Runtime Reification, Semantic Metadata, and Artifact Contract

**Status:** Ratified, implementation-ready

**Authority:** Normative metadata and runtime-reification design

**Primary owners:** new VM-independent metadata schema, `phalcom-semantic` exporter, `phalcom-core` loader/runtime registry, `phalcom-native-meta` producer

**Dependencies:** [01 — Compiler-Owned Typing Implementation Architecture](01-implementation-architecture.md)

**Consumed by:** [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), compiler artifacts, native surfaces, future proof tooling

## 1. Scope and non-goals

This specification defines how canonical semantic types, kinds, declaration signatures, source occurrences, effects, and proof results cross store/snapshot/process boundaries and become ordinary runtime reflection values.

It does not make runtime reflection the source of static truth. It does not add generic type tokens to values, specialize runtime classes, alter selector identity, create applied metaclasses, or change class/metaclass wiring.

Non-goals:

- serializing `TypeStore` internals or raw IDs;
- reconstructing the exact static generic type of an arbitrary runtime value;
- automatically validating every value mutation against erased generic metadata;
- treating `Unknown`, `Dynamic`, invalid annotation, inference variable, or proof unknown as a type node;
- retaining every expression/source/proof fact in release artifacts;
- allowing runtime code to forge authoritative compiler/native metadata;
- treating runtime contract weaving as static proof.

## 2. Current-state evidence and gap analysis

| Finding | Evidence | Classification | Gap/decision |
|---|---|---|---|
| Stable exports are store-independent recursive enums | [`CompiledKindRef` and `CompiledTypeRef`](../../../../phalcom-semantic/src/export.rs#L9) | **Observed current implementation** | Correct transitional boundary, but recursive trees duplicate subforms and lack schema/version/budget/source-use data. |
| Inference variables are rejected during export | [`SemanticExportError`](../../../../phalcom-semantic/src/export.rs#L74) | **Observed current implementation** | Correct; extend to all non-exportable epistemic/query states. |
| Current kinds are `Type` and arrows | [`KindData`](../../../../phalcom-semantic/src/types/kind.rs#L5) | **Observed current implementation** | Schema must represent these now and reserve compatible additions for `RecordRow` and kind parameters. |
| Current type store includes internal `ClassObject`, nominals, applications, unions, tuples, records, callables, parameters, and inference vars | [`TypeData`](../../../../phalcom-semantic/src/types/store.rs#L37) | **Observed current implementation** | Public type-form metadata excludes internal class-object type and inference vars; tooling occurrence metadata can report their status separately. |
| Compiled modules carry runtime interface/read/materialization data, not semantic typing metadata | [`CompiledModule`](../../../../phalcom-core/src/modules/compile.rs#L32), [`ModuleMaterializationPlan`](../../../../phalcom-core/src/modules/artifact.rs#L29) | **Observed current implementation** | Add a versioned semantic metadata section after static analysis and before runtime load. |
| Existing reflection descriptors use dedicated heap variants | [`Object`](../../../../phalcom-core/src/heap/object.rs#L32), [`reflection.rs`](../../../../phalcom-core/src/heap/reflection.rs#L7) | **Observed current implementation** | One boxed semantic-descriptor variant follows established representation without changing the object model. |
| Existing `ReflectionCache` traces every cached descriptor strongly | [`ReflectionCache`](../../../../phalcom-core/src/modules/reflection_cache.rs#L12), [`trace`](../../../../phalcom-core/src/modules/reflection_cache.rs#L31) | **Observed current implementation** | Safe for bounded loaded module descriptors; wrong for unbounded synthetic type applications. Use an untraced weak handle cache scoped to a bounded typing context. |
| `ObjRef` is a generational slot-map key; stale handles resolve safely | [`heap/mod.rs`](../../../../phalcom-core/src/heap/mod.rs#L81) | **Observed current implementation** | It can serve as a weak cache handle if the cache does not trace it and validates liveness before reuse. |
| Native metadata has schema version 1 and symbolic kinds/types | [`NATIVE_SURFACE_SCHEMA_VERSION`](../../../../phalcom-native-meta/src/universe.rs#L3), [`types.rs`](../../../../phalcom-native-meta/src/types.rs#L5) | **Observed current implementation** | Preserve symbolic production, but distinguish opaque/missing from semantic unknown and converge on common versioned metadata. |
| Runtime method-table changes bump `world_version` | [`VM`](../../../../phalcom-core/src/vm/mod.rs#L255), [`dispatch.rs`](../../../../phalcom-core/src/vm/dispatch.rs#L368) | **Observed current implementation** | Reflection/contract/proof assumptions about the open world must carry and validate world version or surface fingerprints. |
| Task 13 reports no runtime representation changes and all invariant suites passing | Supplied Task 13 report | **Observed test coverage** | Runtime model is protected baseline, not a migration target. |

### 2.1 Architectural assessment

**Ratified/normative design.** The implemented `CompiledTypeRef` export is good for tests and short-lived compiler handoff. It must not become the persisted format. A recursive tree:

- repeats shared subtypes;
- makes depth an implicit call-stack risk;
- lacks explicit schema/semantic-model versions;
- cannot cleanly separate a canonical form from a source occurrence;
- encourages accidental encoding of `Unknown` or inference state as a type;
- cannot attach declaration/member/effect/proof fingerprints coherently.

Use a versioned indexed canonical graph with bounded decoding. Current forms remain acyclic after nominal recursion is represented through declaration references, so version 1 rejects structural node cycles. Later recursive aliases require an explicit binder encoding and feature gate, not permissive cyclic indexes.

## 3. Semantic and runtime contract

### 3.1 Reification principle

```text
semantic form --export--> stable metadata --load--> runtime reflected value
```

The arrow is one-way authority:

- compiler/native metadata may create reflected values;
- reflected values may be queried and composed through checked runtime APIs;
- runtime values do not mutate the compiler snapshot or retroactively establish compile-time facts;
- a reflected proof/type object carries evidence; object identity alone confers no semantic authority.

### 3.2 Nominal identity

For a nominal declaration `D`:

```text
reify(type-form(D)) === runtime-class-object(D)
```

Examples:

```phalcom
Typing.current.type(#Int).unwrap === Int
Typing.current.type(#List).unwrap === List
```

Exact lookup spelling is finalized in Spec 03. This rule preserves current class object identity and avoids `ClassType` wrappers.

`TypeData::ClassObject { declaration }` remains an internal static value type. It is not materialized as a public type-form descriptor. Tooling occurrence metadata may report it as an internal class-value fact while the same expression separately denotes the nominal form.

### 3.3 Synthetic identity and equivalence

Applications, unions, tuples, records, callables, type parameters, special types, arrow kinds, and future row/kind forms may require synthetic descriptors.

- within one live `TypingContext` arena, reifying the same canonical node returns the same live descriptor object;
- the cache is weak: identity is not promised after all descriptors are collected and later recreated;
- across contexts/VMs/processes, `===` is not meaningful; `equivalentTo(_)` compares canonical structure under compatible semantic model versions;
- descriptor `hash` follows canonical structure and semantic version, so equivalence implies equal hash;
- `displayName` and source spelling never participate in identity or equivalence.

### 3.4 Erasure and runtime checks

Ordinary instances retain their current representation. `List<Int>` does not create a subclass, metaclass, per-instance token, changed allocator, or changed inline-cache key.

Runtime checks occur only at explicit boundaries:

- declared runtime contracts;
- `Dynamic` ingress/egress selected by strictness policy;
- FFI and opaque native surfaces;
- reflective construction/invocation APIs;
- deserialization or external data validation.

Nominal checks use existing runtime class/inheritance. Deep structural/generic checking is an explicit operation with cycle and size budgets. Higher-order callable checking requires wrappers or witnessed contracts and is not claimed by shallow `isInstance` checks.

### 3.5 Metadata availability

Metadata has profiles, never an ambiguous stripped/maybe state:

```rust
pub enum MetadataProfile {
    RuntimePublic,
    ToolingDebug,
    Proof,
}
```

- `RuntimePublic` is mandatory for artifacts that claim typing reflection. It contains public declaration kinds, generic signatures, public member signatures, normalized forms, effects/termination requirements needed at boundaries, and module fingerprints.
- `ToolingDebug` adds private/internal declarations as permitted, source occurrences/spellings/ranges, local expression facts, dispatch provenance, and richer diagnostic maps.
- `Proof` adds normalized obligations, results, assumptions, and artifacts; it includes `RuntimePublic`.

An artifact may explicitly disable typing reflection as a build feature, but the loader then exposes `MetadataUnavailable(DisabledByBuild)` rather than empty/unknown types. Native/runtime ABI metadata required for sound calls remains mandatory regardless of reflection profile.

## 4. VM-independent metadata schema

### 4.1 Crate placement and dependency rule

Create workspace crate `phalcom-type-meta` containing only stable owned data, validation, canonical encoding/decoding, and version constants. Dependencies may include `phalcom-common`; it must not depend on AST, modules, semantic, core, LSP, native macros, or VM.

Dependency direction:

```text
phalcom-type-meta
    <- phalcom-native-meta
    <- phalcom-modules (only if interface carriage is needed)
    <- phalcom-semantic
    <- phalcom-core
    <- phalcom-lsp tooling decoder
```

Stable module/declaration references use schema-owned structural strings/components. Conversion to `ModuleId`/`DeclarationId` lives in `phalcom-semantic` or `phalcom-core`, preventing a crate cycle.

### 4.2 Header, identities, and feature flags

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
    pub module: StableModuleRef,
    pub source_fingerprint: Fingerprint128,
    pub interface_fingerprint: Fingerprint128,
    pub world_assumptions: Box<[WorldAssumptionRef]>,
}

pub struct StableModuleRef {
    pub project_source: Box<str>, // canonical URI/logical source identity
    pub path: Box<[Box<str>]>,
}

pub struct StableDeclarationRef {
    pub module: StableModuleRef,
    pub path: Box<[Box<str>]>, // v1 normally one name; supports nested declarations
}

pub struct StableCallableRef {
    pub owner: StableDeclarationRef,
    pub selector: Box<str>,
    pub side: DispatchSideRef,
}
```

`project_source` is not a generation numeric ID. Reproducible package artifacts may replace local filesystem spelling with package artifact identity plus module path; the encoding records which identity scheme is used.

### 4.3 Indexed kind and type graph

```rust
#[repr(transparent)]
pub struct KindNodeId(pub u32);

#[repr(transparent)]
pub struct TypeNodeId(pub u32);

pub enum KindNode {
    Type,
    Arrow {
        parameters: Box<[KindNodeId]>,
        result: KindNodeId,
    },
    // Feature-gated later, not accepted by schema v1 decoders:
    RecordRow,
    Parameter(StableKindParameterRef),
}

pub enum TypeNode {
    Never,
    Unit,
    Nominal(StableDeclarationRef),
    Applied {
        origin: TypeNodeId,
        arguments: Box<[TypeNodeId]>,
    },
    Union(Box<[TypeNodeId]>),
    Tuple(Box<[TupleElementRef]>),
    Record(RecordTypeRef),
    Callable(CallableTypeRef),
    Parameter(StableTypeParameterRef),
    // Added only when their language decisions land:
    Any,
    Intersection(Box<[TypeNodeId]>),
}

pub struct TypeNodeEntry {
    pub kind: KindNodeId,
    pub form: TypeNode,
    pub structural_fingerprint: Fingerprint128,
}

pub struct RecordTypeRef {
    pub fields: Box<[RecordFieldRef]>,
    pub tail: RecordTailRef,
}

pub enum RecordTailRef {
    Closed,
    Parameter(StableTypeParameterRef), // feature-gated until RecordRow lands
}
```

Nodes are topologically ordered in schema v1. Children precede parents. Union members and record fields use canonical order; duplicates are invalid. `Applied` arity/kinds are validated against the declaration/kind graph. No node represents `Unknown`, `Dynamic`, missing/invalid annotations, an inference variable, cancellation, budget exhaustion, or proof unknown.

### 4.4 Parameters, aliases, and declarations

```rust
pub enum VarianceRef {
    Covariant,     // source +T
    Contravariant, // source -T
    Invariant,     // source T
}

pub struct StableTypeParameterRef {
    pub owner: TypeParameterOwnerRef,
    pub index: u16,
}

pub struct TypeParameterRecord {
    pub id: StableTypeParameterRef,
    pub name: Box<str>,
    pub variance: VarianceRef,
    pub kind: KindNodeId,
    pub upper_bound: Option<TypeNodeId>,
    pub constraints: Box<[ConstraintRef]>,
    pub default: Option<TypeNodeId>,
}

pub struct DeclarationTypeRecord {
    pub declaration: StableDeclarationRef,
    pub form: TypeNodeId,
    pub kind: KindNodeId,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub members: Box<[MemberTypeRecord]>,
    pub protocols: Box<[TypeNodeId]>,
    pub flags: DeclarationTypeFlags,
}

pub struct TypeAliasRecord {
    pub declaration: StableDeclarationRef,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub target: TypeNodeId,
    pub transparency: AliasTransparency,
}
```

Aliases retain declaration/source identity separately from normalized type nodes. Transparent alias equality uses the target; opaque/newtype behavior requires its own ratification. The schema must not accidentally decide it by encoding every alias as nominal or erasing every alias occurrence.

### 4.5 Canonical form versus source occurrence

Source spelling is occurrence data, not the canonical form:

```rust
pub struct TypeUseRecord {
    pub owner: TypeUseOwnerRef,
    pub role: TypeUseRole,
    pub normalized: TypeUseType,
    pub denotation: Option<DenotationRef>,
    pub written: Option<Box<str>>,
    pub source: Option<SourceSpanRef>,
    pub status: TypeUseStatus,
    pub diagnostics: Box<[DiagnosticRef]>,
}

pub enum TypeUseType {
    Reifiable(TypeNodeId),
    InternalClassObject(StableDeclarationRef),
    Missing,
    Invalid,
    Unknown(UnknownReasonRef),
    Dynamic(DynamicReasonRef),
    UnresolvedDependency(StableModuleRef),
}
```

This prevents common lies:

- an absent annotation is not `Dynamic`;
- an invalid annotation is not `Unknown`;
- the internal static type of the value `Int` is not the nominal `Int` form it denotes;
- a source alias spelling is not canonical identity;
- a cancelled/budgeted query is not a type use at all and is not serialized as completed metadata.

### 4.6 Effects, termination, contracts, and proofs

Callable records reserve independent components:

```rust
pub struct CallableContractRecord {
    pub callable: StableCallableRef,
    pub parameters: Box<[CallableParameterRef]>,
    pub return_type: TypeNodeId,
    pub effects: EffectSummaryRef,
    pub exits: ExitSummaryRef,
    pub termination_requirement: TerminationRequirementRef,
    pub termination_knowledge: TerminationKnowledgeRef,
    pub runtime_contracts: Box<[ContractRef]>,
    pub proof_results: Box<[ProofResultRef]>,
}
```

Proof artifact shape:

```rust
pub struct ProofArtifactRecord {
    pub obligation: Fingerprint256,
    pub evidence: ProofEvidenceRef,
    pub trust: ProofTrustRef,
    pub assumptions: Box<[TrustBoundaryRef]>,
    pub referenced_interfaces: Box<[Fingerprint128]>,
    pub backend: BackendIdentityRef,
    pub backend_version: Box<str>,
    pub semantic_model_version: u32,
    pub proof_kernel_version: u32,
}

pub enum ProofEvidenceRef {
    Certificate(Box<[u8]>),
    TrustedBackendAttestation(Box<[u8]>),
    Counterexample(CounterexampleModelRef),
}

pub enum ProofTrustRef {
    KernelChecked,
    TrustedBackend,
    AssumedAxiom,
}
```

`Proven` may reference a certificate or attestation with explicit trust. `Disproven` carries a counterexample when available. `Unknown(reason)` is a result record, never a `ProofArtifactRecord` that masquerades as evidence.

## 5. Runtime data model

### 5.1 Immutable base and bounded context arena

Add:

```text
phalcom-core/src/typing/mod.rs
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/context.rs
phalcom-core/src/typing/reify.rs
phalcom-core/src/typing/metadata_loader.rs
phalcom-core/src/heap/semantic_descriptor.rs
```

Rust-facing model:

```rust
pub struct LoadedTypeMetadata {
    pub header: SemanticMetadataHeader,
    pub kinds: Arc<[ValidatedKindNode]>,
    pub types: Arc<[ValidatedTypeNode]>,
    pub declarations: Arc<StableDeclarationRuntimeMap>,
    pub uses: Arc<[ValidatedTypeUse]>,
    pub contracts: Arc<[ValidatedCallableContract]>,
}

pub struct RuntimeTypeArena {
    pub base: Arc<LoadedTypeMetadata>,
    dynamic_kinds: Vec<RuntimeKindNode>,
    dynamic_types: Vec<RuntimeTypeNode>,
    kind_interner: HashMap<RuntimeKindNode, RuntimeKindId>,
    type_interner: HashMap<RuntimeTypeNode, RuntimeTypeId>,
    descriptor_cache: HashMap<RuntimeSemanticId, ObjRef>, // weak: never traced
    limits: RuntimeTypingLimits,
}

pub struct TypingContextObject {
    pub arena: Arc<RwLock<RuntimeTypeArena>>,
    pub capabilities: ReflectionCapabilities,
    pub world: WorldStamp,
}
```

Each `TypingContext` owns a bounded overlay. A live descriptor retains the arena through an `Arc`; the weak cache does not retain descriptors. When the context and all its descriptors die, overlay memory is released. A long-lived context cannot exceed `RuntimeTypingLimits`; construction returns `BudgetExceeded`, not unbounded growth.

`ObjRef` weak-cache algorithm:

1. Look up node ID in `descriptor_cache`.
2. Call `heap.get(obj_ref)` and validate the object is a semantic descriptor for the same arena/node.
3. Reuse if live; remove stale generational handle otherwise.
4. Allocate descriptor, insert handle without tracing it.

Do not reuse the strong-rooting policy in `ReflectionCache::trace` for these descriptors.

### 5.2 One boxed heap representation

Add one `Object` arm:

```rust
Object::SemanticDescriptor(Box<SemanticDescriptorObject>)

pub struct SemanticDescriptorObject {
    pub class: ClassId,
    pub arena: Arc<RwLock<RuntimeTypeArena>>,
    pub payload: SemanticDescriptorPayload,
}

pub enum SemanticDescriptorPayload {
    Type(RuntimeTypeId),
    Kind(RuntimeKindId),
    TypeUse(RuntimeTypeUseId),
    TypeParameter(RuntimeTypeParameterId),
    RelationEvidence(RuntimeRelationEvidenceId),
    Proof(RuntimeProofResultId),
}
```

Boxing prevents arena slot-size inflation. `Value::class` returns the stored `class`; new ordinary classes such as `AppliedType`, `UnionType`, `ArrowKind`, `TypeUse`, and `ProofResult` participate in the existing metaclass tower by ordinary class bootstrap/materialization. No new meta-level rule is introduced.

Nominal reification bypasses `Object::SemanticDescriptor` and returns the existing `ClassId` value.

### 5.3 Kind reification

The runtime value named `Type` is a canonical singleton instance of ordinary class `AtomicKind`, whose superclass is `Kind`. It is not a class object and does not make `Type :: Type` true.

```text
Type.class == AtomicKind
Type.kind == None                 // kinds classify forms; not themselves by Type
List.kind === ArrowKind(Type, Type)
Int.kind === Type
```

`ArrowKind` descriptors are canonical in the context. Future `RecordRow` is another atomic kind singleton. Kind parameters become immutable `KindParameter` descriptors only after prenex kind-polymorphism implementation.

### 5.4 World and surface assumptions

```rust
pub struct WorldStamp {
    pub vm_world_version: u64,
    pub loaded_interface_fingerprint: Fingerprint128,
    pub native_surface_fingerprint: Fingerprint128,
}
```

Pure structural type/kind queries do not depend on `world_version`. Member lookup, conformance against mutable runtime methods, reflective invocation checks, runtime-contract validation, and proof artifacts do.

If `VM.world_version` changes:

- structural descriptor equivalence remains valid;
- cached runtime member/conformance results become stale;
- proof results whose assumptions include open method surfaces become `Unknown(StaleWorld)` until revalidated;
- runtime reflection never rewrites a compiler snapshot to match monkey-patched behavior.

## 6. Algorithms

### 6.1 Export from semantic snapshot

1. Demand `ModuleMetadata(module, profile)` from the semantic DB.
2. Reject cancelled, blocked, invalid, or wrong-store roots.
3. Collect reachable public/profile-selected declarations, kinds, forms, member contracts, type uses, and proof results.
4. Convert generation-local declarations/modules to stable structural references.
5. Hash-cons structural kind/type nodes by stable form key.
6. Topologically order children before parents; sort roots by stable declaration/callable/source identity.
7. Validate proper kinds, application arity, parameter ownership, canonical union/record ordering, and absence of solver/query IDs.
8. Compute structural, interface, and module fingerprints.
9. Encode using canonical field/order rules. Same semantic input must produce byte-identical output independent of internal `TypeId` allocation.

### 6.2 Decode and validate

1. Read fixed-size header; reject unsupported major/schema/semantic model.
2. Enforce byte, string, node, edge, declaration, source-use, proof, and nesting budgets before allocation.
3. Validate all indexes are in range and topologically backward in schema v1.
4. Validate canonical sort/dedup rules and structural fingerprints.
5. Validate kinds and applications using an iterative worklist.
6. Validate owner/index parameter uniqueness and references.
7. Resolve stable nominal declarations to loaded runtime classes; unresolved public nominal roots are load errors, not `Unknown`.
8. Validate native schema/fingerprint compatibility.
9. Validate proof artifact fingerprints/trust; stale proof results load as reasoned `Unknown`, never `Proven`.
10. Publish `Arc<LoadedTypeMetadata>` only after complete validation.

### 6.3 Reify a form

1. Verify `TypingContext` capability and node belongs to its base/overlay.
2. Charge reification budget.
3. If nominal, resolve stable declaration and return existing class object.
4. Otherwise probe weak descriptor cache.
5. Resolve descriptor class from node kind (`AppliedType`, `UnionType`, and so on).
6. Allocate immutable boxed descriptor; insert weak handle.
7. Return it. Child descriptors remain lazy; getters reify children on demand.

No recursive eager object graph is built. Deep type forms cannot overflow the Rust stack or allocate every descendant merely because a root was inspected.

### 6.4 Runtime checked composition

For `TypingContext.apply(origin, arguments)`:

1. require `ConstructTypes` capability;
2. obtain canonical kind of origin and arguments;
3. check arity/kinds exactly and compute residual kind;
4. validate bounds/constraints when implemented;
5. hash-cons an overlay `Applied` node;
6. enforce node budget;
7. reify through weak cache.

Normalization, union construction, tuple/record/callable construction, and substitution follow the same pattern. User code cannot instantiate raw descriptor classes to bypass validation.

### 6.5 Metadata unload and GC

- compiled programs/modules strongly retain `LoadedTypeMetadata` while loaded;
- `TypingContext`/descriptor `Arc`s may extend metadata lifetime, exactly like a reflected module object may extend related data lifetime;
- weak `ObjRef` cache entries are pruned on failed lookup and optionally at GC completion;
- no descriptor cache participates in GC root tracing;
- a context overlay has a hard node/byte cap;
- unloading a module does not invalidate already materialized structural data, but member/invocation operations requiring live declarations return `Unknown(UnloadedDeclaration)`.

## 7. Native surfaces and dynamic boundaries

### 7.1 Authoritative native schema

Refactor `phalcom-native-meta` to produce the common stable schema or a lossless input to it. Replace ambiguous:

```rust
TypeExprSpec::Unknown
```

with:

```rust
pub enum NativeTypeSurface {
    Known(TypeExprSpec),
    Opaque { reason: NativeOpaqueReason },
}
```

A missing required return/parameter type is a native metadata error. `Opaque` is explicit, versioned, and becomes a dynamic/proof boundary with an actionable diagnostic. Native Rust declarations and bundled `.ph` declarations must normalize to equivalent signatures; divergence is a build/test failure.

### 7.2 FFI, reflection, `perform`, and DNU

- FFI metadata records ABI, ownership/lifetime policy where relevant, type contract, effects, raises, and trust boundary.
- `perform` with a runtime selector remains an explicit dynamic boundary even when runtime metadata can validate the selected method.
- DNU overrides are ordinary runtime methods and invalidate world-sensitive lookup facts. They do not create a static member surface implicitly.
- opaque native calls invalidate proof completeness across their effects unless a trusted contract/axiom is recorded.
- reflected metadata inspection is pure; reflective invocation uses ordinary access checks and runtime dispatch.

## 8. Diagnostics and developer experience

Required codes:

| Code | Severity | Action |
|---|---:|---|
| `metadata.schema.unsupported` | error | Rebuild with compatible compiler/runtime |
| `metadata.semantic_model.mismatch` | error | Do not interpret nodes under different laws |
| `metadata.malformed` | error/internal depending origin | Report node/index/path and reject artifact |
| `metadata.budget_exceeded` | error | Reject hostile/oversized artifact without partial load |
| `metadata.kind_mismatch` | error | Producer bug or corrupt artifact; reject |
| `metadata.nominal.unresolved` | error | Missing/incompatible loaded declaration |
| `metadata.profile.unavailable` | information/error by requested operation | Explain build profile limitation |
| `native.surface.missing` | build error | Native declaration lacks required typed surface |
| `native.surface.opaque` | warning/strict error | Explicit boundary with native symbol and reason |
| `native.surface.version_mismatch` | error | Compiler/runtime/native metadata versions disagree |
| `reflection.context.stale_world` | warning/result unknown | Recreate or refresh context |
| `reflection.budget_exceeded` | runtime typed error | Narrow requested operation or raise explicit limit |
| `proof.artifact.stale` | information/warning | Re-run proof; never report old artifact as proven |

Metadata decoding errors identify artifact/module and a bounded node path. They never include attacker-controlled unbounded strings or recursive debug output.

## 9. Dependency-ordered implementation plan

### Unit B1 — common metadata crate and validator

**Files:**

- add workspace member `phalcom-type-meta/`;
- create `src/{lib,header,identity,kind,type,decl,use_record,contract,proof,validate,encode}.rs`;
- update root `Cargo.toml` and dependent crate manifests.

Steps:

1. Implement owned schema types and version constants.
2. Implement canonical structural fingerprints and deterministic encoder.
3. Implement iterative budgeted validator/decoder.
4. Add hostile input, depth, index, cycle, duplicate, sort, kind, and arity tests.

Acceptance: crate contains no AST, VM, semantic ID, filesystem, or LSP dependency.

### Unit B2 — semantic exporter

**Files:**

- create `phalcom-semantic/src/metadata/{mod,export,stable_identity,fingerprint}.rs`;
- adapt `phalcom-semantic/src/export.rs`;
- add `phalcom-semantic/tests/metadata.rs`.

Steps:

1. Export stable project/module/declaration/callable/parameter identities.
2. Build canonical indexed nodes and profile-selected roots.
3. Export type-use status separately from forms.
4. Reject `InferVarId`, wrong-kind `Known`, cancellation, blocked products, and internal-only class-object form as a public TypeForm.
5. Keep `export_type_form` as a compatibility adapter backed by the new exporter for tests.

Deletion criteria: recursive exporter no longer owns normalization or durable format; remove once no consumer/test needs it.

### Unit B3 — compiled artifact carriage

**Files:**

- modify `phalcom-core/src/modules/{artifact,compile,registry,materialize}.rs` as applicable;
- add semantic metadata field to `CompiledModule`/`ModuleMaterializationPlan` or one program-level deduplicated section;
- add CLI build/check artifact tests.

Recommendation: one program/package metadata pool plus per-module root indexes, avoiding duplicated shared universe forms. `CompiledProgram` owns the pool; module plans carry module root IDs.

Compilation fails if reachable required metadata export is invalid. A disabled reflection profile is explicit in header/build settings.

### Unit B4 — loader, arena, and descriptor representation

**Files:**

- create `phalcom-core/src/typing/` modules in §5.1;
- create `phalcom-core/src/heap/semantic_descriptor.rs`;
- modify `phalcom-core/src/heap/{mod,object,trace,accessors}.rs`;
- modify `phalcom-core/src/value/mod.rs` classification;
- extend universe descriptors/materialization with ordinary reflection classes.

Steps:

1. Validate/load immutable metadata.
2. Implement context overlay and generational runtime IDs.
3. Add one boxed object variant and GC tracing for its `class` plus any directly held Phalcom values; Rust `Arc` data contains no `Value` handles.
4. Implement weak descriptor cache using untraced generational `ObjRef`.
5. Return existing `ClassId` for nominal forms.
6. Add context caps and stale-world state.

Deletion criteria: no type descriptor enters strong `ReflectionCache::trace`; no duplicate nominal wrapper.

### Unit B5 — native metadata convergence

**Files:**

- modify `phalcom-native-meta/src/{types,universe,lib}.rs`;
- modify `phalcom-native-macros/src/lib.rs`;
- modify semantic native normalization in `phalcom-semantic/src/types/native.rs` only after inspecting and preserving user changes;
- add generated native-vs-source parity tests.

Steps:

1. Add variance/kind/bound/effect/raise/flow fields without name-based parameter identity.
2. Replace ambiguous unknown with explicit known/opaque.
3. Emit common schema/fingerprint and validate version at compiler/runtime startup.
4. Remove hardcoded checker exceptions only after every native surface is authoritative.

### Unit B6 — profiles, contracts, proofs, and observability

Implement `RuntimePublic` first. `ToolingDebug` follows source-use query support. `Proof` waits for VC/prover spec but schema and fingerprint fields remain reserved.

Measure metadata bytes/module, unique/reused nodes, load/validation time, lazy descriptor allocations, cache hit/miss/stale rates, overlay high-water, and GC reclamation. Set limits from Phalcom corpus measurements, not another compiler's numbers.

## 10. Migration and “must not preclude”

Migration sequence:

1. build common schema alongside `CompiledTypeRef`;
2. differential-test both exports structurally;
3. attach schema to compiled artifacts without runtime objects;
4. load/validate but keep reflection disabled;
5. add nominal reification;
6. add synthetic descriptors and weak cache;
7. expose Spec 03 API;
8. delete transitional export only after compiler, LSP tooling, tests, and artifacts no longer require it.

This design must not preclude kind parameters/schemes, `RecordRow`, distinct effect/variant rows, variance, bounds/F-bounds, `Self`, aliases, constraints, recursive ADTs, protocols, intersections, exhaustive matching, per-expression debug facts, totality evidence, or checkable proof certificates.

## 11. Verification and acceptance

### Schema/unit/property

- deterministic byte-identical encoding from semantically identical fresh stores;
- round-trip every current kind/type form except intentionally non-exportable internal states;
- duplicate/canonical ordering and fingerprint properties;
- owner/index parameter identity survives same-name shadowing;
- application kind/arity validation rejects corrupt graphs;
- no raw semantic/query/solver ID type appears in public schema;
- decoder terminates within budgets for adversarial depth/width/cycles/indexes.

### Runtime/object model

- `reify(Int) === Int` and `reify(List) === List`;
- no class/metaclass/superclass or method-lookup invariant changes;
- applied descriptors do not create classes/metaclasses or alter `List.new` dispatch;
- repeated live reification in one context returns identical descriptor;
- dropping all descriptors permits collection; weak cache does not retain them;
- stale `ObjRef` is detected and safely replaced;
- context budget halts adversarial construction with typed error;
- descriptors across contexts are structurally equivalent but need not be identical;
- class-object internal static type is not exposed as a nominal TypeForm.

### Native/dynamic/security

- generated native surface equals bundled source declaration after normalization;
- missing/opaque/version-mismatched native metadata has exact diagnostics;
- reflective/private member queries and invocation obey current caller authority;
- method installation changes world stamp and invalidates only world-sensitive caches;
- `perform`, DNU, FFI, and opaque native operations remain explicit proof/type boundaries.

### Corpus/performance

- runtime public metadata size and load time recorded for universe, std, representative project;
- lazy root inspection allocates O(1) descriptors, not whole descendant graph;
- edit/rebuild produces deterministic metadata;
- repeated runtime composition remains under context cap and releases on context collection;
- full workspace/object/invariant/GC suites rerun when implementation is authorized.

Implementation reports separate passing, baseline/unrelated, deferred, and unverified evidence. This documentation task ran no tests.

## 12. Risks and ratification gates

| Risk | Gate |
|---|---|
| New schema crate creates dependency cycle | Enforce zero high-level dependencies in B1 |
| Filesystem spelling leaks into reproducible identity | Ratify artifact identity mapping and reproducibility tests before disk cache |
| Internal class-object form accidentally becomes public type | Export/type-use tests and API review |
| Weak cache returns reused slot | Validate generational `ObjRef` and payload arena/node before reuse |
| Context overlay retained forever | Hard node/byte limits and GC/lifetime tests |
| Metadata claims sound native signature while Rust differs | Generated parity/build gate |
| Proof survives semantic/backend change | Full fingerprint/trust validation; stale becomes `Unknown` |
| Deep generic checks become implicit runtime tax | Require explicit boundary/check API and benchmark |

**Proposed design needing ratification.** Exact canonical binary encoding and on-disk cache location remain open until package artifact/reproducible-build policy is ratified. Rust schema and deterministic logical encoding are fixed; implementers must not choose a permanent wire format incidentally.

## 13. Take directly / adapt / reject

### Take directly

**Pyrefly architectural transfer.** Stable cheap identities, canonical type-store discipline, immutable serialized semantic products, explicit schema/version/fingerprint data, deterministic regression tests, bounded decoding, and observability.

### Adapt

**Pyrefly architectural transfer.** Metadata represents Phalcom kinds, selectors/labels, class/instance side, nominal class objects, native surfaces, dynamic message boundaries, future HKTs/rows/effects, and proof trust. Runtime publication uses Phalcom's safe generational heap handles and GC.

### Reject

**Ratified/normative design.** Python type semantics and descriptor lookup, raw-pointer cache publication, persisting dense semantic IDs, strongly rooting unbounded synthetic forms, runtime class specialization, per-instance generic tokens, or treating an SMT “unsat” response as kernel-checked proof without evidence.
