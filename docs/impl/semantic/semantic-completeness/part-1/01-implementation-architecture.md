# 01 — Compiler-Owned Typing Implementation Architecture

**Status:** Ratified, implementation-ready
**Authority:** Normative architecture after completed two-axis tower
**Primary owners:** `phalcom-semantic` and `phalcom-modules`; consumer migrations in `phalcom-core` and `phalcom-lsp`
**Dependencies:** Completed Task 1–13 two-axis implementation; [program map](README.md)
**Produces:** bounded semantic relations, compiler-owned query database, stable identity boundary, partial-result diagnostics, immutable incrementally derived snapshots

## 1. Scope, owners, and non-goals

This specification turns the current whole-workspace analyzer into the single incremental semantic platform for compiler, CLI, REPL, and LSP. It retains the implemented type/kind algebra and fixes architecture around it.

Crate ownership is normative:

- `phalcom-modules`: stable project/module/source identities, parsed module products, interface construction, linking, reference/semantic/runtime graphs.
- `phalcom-semantic`: type/kind store, declaration and callable typing, query scheduling, relations, constraint/flow/effect/proof facts, invalidation, diagnostics, immutable snapshots.
- `phalcom-core`: entry selection, compile gate, semantic metadata export, bytecode/materialization, REPL sessions, runtime.
- `phalcom-lsp`: source overlays, publication scheduling, protocol adaptation, advisory `ValueShape`; no independent formal type rules.

Non-goals:

- no runtime `Value`, `ClassObject`, superclass, metaclass, selector, dispatch, or instance-layout change;
- no public syntax in this document;
- no editor-only formal checker;
- no global mutable type store or solver;
- no unbounded fixed point, recursive display, relation, query, or hierarchy walk;
- no persistence of raw `TypeId`, `KindId`, `TypeParameterId`, `InferVarId`, query ID, or process-local `ResolvedProjectId`;
- no proof that TypeScript/Python-style permissive behavior is sound merely because checking continues.

## 2. Authority and current-state evidence

| Finding | Evidence | Classification | Consequence |
|---|---|---|---|
| Type and kind IDs are explicitly store/snapshot-local | [`id.rs`](../../../../phalcom-semantic/src/types/id.rs#L3) | **Observed current implementation** | Raw IDs cannot key durable caches or public reflection. |
| `TypeStore` densely interns types/kinds and records a kind for every type | [`store.rs`](../../../../phalcom-semantic/src/types/store.rs#L64) | **Observed current implementation** | Keep dense tables and canonicalization. |
| Proper-type checks in tuple/record/callable construction are `debug_assert!` only | [`store.rs`](../../../../phalcom-semantic/src/types/store.rs#L312) | **Observed current implementation** | Release builds need checked construction or sealed trusted callers. |
| `TypeKnowledge::known` accepts any raw `TypeId` without a store/kind check | [`evidence.rs`](../../../../phalcom-semantic/src/types/evidence.rs#L41) | **Observed current implementation** | The implemented ontology invariant is conventional, not API-enforced. Fix in Unit A1. |
| Static type and denotation are independent fields | [`denotation.rs`](../../../../phalcom-semantic/src/types/denotation.rs#L6) | **Observed current implementation** | Preserve two-axis fact representation. |
| Generic parameter IDs are interned from owner plus index | [`parameter.rs`](../../../../phalcom-semantic/src/types/parameter.rs#L6), [`store.rs`](../../../../phalcom-semantic/src/types/store.rs#L107) | **Observed current implementation** | Correct identity model; add variance/bounds without name-based identity. |
| Workspace analysis creates a new `TypeStore` and performs phases in one function | [`workspace.rs`](../../../../phalcom-semantic/src/workspace.rs#L47) | **Observed current implementation** | Correct cold path; not the target incremental architecture. |
| Immutable snapshot owns store, sources, surfaces, dispatch, declarations, hierarchy, diagnostics, and semantic graph | [`snapshot.rs`](../../../../phalcom-semantic/src/snapshot.rs#L17) | **Observed current implementation** | Good publication boundary; fields need stronger encapsulation and status/fingerprint tables. |
| Semantic graph computes deterministic SCCs; runtime graph rejects cycles | [`graph.rs`](../../../../phalcom-modules/src/graph.rs#L194), [`graph.rs`](../../../../phalcom-modules/src/graph.rs#L251) | **Observed current implementation** | Reuse graph layers and SCC algorithm. Never bypass runtime-cycle error. |
| Whole-workspace LSP path skips multiple failure classes and sorts modules after runtime-cycle error | [`analysis_service.rs`](../../../../phalcom-lsp/src/analysis_service.rs#L944) | **Observed current implementation** | This is a correctness/DX defect, not successful analysis. Replace with structured partial outcomes. |
| Current semantic invalidation is only a fingerprint map | [`invalidation.rs`](../../../../phalcom-semantic/src/invalidation.rs#L6) | **Observed current implementation** | Move dependency recording/reverse invalidation into compiler-owned semantic DB. |
| Advisory LSP already has source delta classification and reverse edges | [`invalidation.rs`](../../../../phalcom-lsp/src/semantic/invalidation.rs#L9), [`module_graph.rs`](../../../../phalcom-lsp/src/semantic/module_graph.rs#L96) | **Observed current implementation** | Transfer mechanics, not advisory semantics, into compiler DB. |
| Compiler requires `AnalyzedProgram` before compilation | [`compile.rs`](../../../../phalcom-core/src/modules/compile.rs#L140), [`compile.rs`](../../../../phalcom-core/src/modules/compile.rs#L462) | **Observed current implementation** | Preserve one formal compile gate. |
| LSP wraps the shared static snapshot while retaining advisory tables | [`snapshot.rs`](../../../../phalcom-lsp/src/semantic/snapshot.rs#L49) | **Observed current implementation** | Correct migration direction; delete LSP-owned static workspace builder after DB adoption. |
| Fresh-store outputs compare structurally across generations | [`workspace.rs`](../../../../phalcom-semantic/tests/workspace.rs#L497) | **Observed test coverage** | Preserve structural determinism; internal numeric ID equality is not required across stores. |

### 2.1 Implementation assessment

**Ratified/normative design.** Keep these implemented choices:

- dense, canonical store-local IDs;
- explicit kind table and checked application;
- owner/index type-parameter identity;
- separate class-object type and denoted nominal form;
- separate formal static and advisory editor snapshots;
- declaration predeclaration before semantic SCC realization;
- deterministic `BTreeMap` graph/publication ordering;
- compiler refusal to compile a semantically erroneous analyzed program.

**Ratified/normative design.** Fix these seams before adding effects, protocols, proof queries, or runtime reflection:

1. enforce proper-type construction in release builds;
2. replace boolean/coarse relation APIs with explicit terminal outcomes;
3. give every diagnostic range an owning module/source identity;
4. publish project/link failures as facts and diagnostics rather than dropping modules;
5. move formal source catalog/link/analyze ownership out of LSP;
6. add stable workspace/project/module keys and database/store identity;
7. add dependency-recorded staged queries, cancellation, budgets, SCC policy, and reverse invalidation;
8. remove runtime-cycle sorted fallback.

## 3. Semantic contract

### 3.1 Semantic domains

The following domains are disjoint even if a UI displays similar text:

```text
RuntimeValue       runtime object/immediate value
RuntimeClass       result of value.class
TypeForm           canonical static form; proper type or constructor
Kind               classifier of TypeForm
TypeParameter      stable owner/index binder
InferenceVariable  solver-local unknown
TypeKnowledge      Known | Unknown | Dynamic
ConstantFact       exact literal/constant analysis fact
DispatchFact       receiver + selector + labels + side + lookup mode result
EffectFact         possible externally visible behavior/control exit
ProofProposition   normalized verification condition
ProofResult        Proven | Disproven | Unknown(reason)
```

Core judgments:

```text
Γ ⊢ e : T                         expression has proper value type T
Γ ⊢ e ⇝ F                         expression denotes type/kind form F
Σ ⊢ F :: K                        type form F has kind K
Σ ⊢ F<A...> ⇓ G :: K             checked canonical application
Σ; Γ ⊢ S <: T ⇒ RelationOutcome  subtyping query
Σ; Γ ⊢ S ≼ T ⇒ RelationOutcome   assignability query
Σ; Γ ⊢ S ~ T ⇒ RelationOutcome   consistency query
Σ; Γ ⊢ S conforms P              protocol conformance query
Σ; Γ ⊢ member(T, selector, side)  member/dispatch lookup query
```

`Type`, values that denote types, runtime class objects, and `Class`/`Metaclass` remain stratified. No object-model change may manufacture `Type :: Type`.

### 3.2 Knowledge and relation laws

**Ratified/normative design.** `Known(T)` is constructible only when `kind(T) = Type`. `Unknown`, `Dynamic`, a missing annotation, an invalid annotation, unresolved dependency, inference variable, and proof unknown are never encoded as the same state.

Relation laws:

- equivalence is reflexive, symmetric, transitive, and normalization-respecting;
- subtyping is reflexive and transitive; `Never <: T` for every proper `T`;
- assignability is directional and policy-aware; it may use subtyping, declared conversions, or explicit gradual boundaries, but each justification is recorded;
- consistency is symmetric and answers whether two types can safely interact at a declared dynamic boundary; it is not subtyping;
- protocol conformance is a structural/nominal obligation with evidence; it is not member lookup;
- member lookup returns a specialized callable/member view and provenance; it is not a boolean relation;
- no `Dynamic` operation returns an unqualified `Proven` fact. It returns a boundary outcome carrying the runtime obligation;
- no `Unknown`, cancellation, budget exhaustion, or internal failure is converted to success.

### 3.3 Numeric and constant facts

**Ratified/normative design.** Literal syntax fixes runtime class:

```text
1   : Int     plus ConstantFact::Int(1)
1.0 : Float   plus ConstantFact::FloatBits(...)
```

Expected type never changes this synthesis. A check from `1` to expected `Float` is rejected unless an explicit language conversion rule exists; recommendation is explicit `toFloat`-style conversion. Exact facts support narrowing, exhaustiveness, constant folding, and proofs without making singleton types the default inferred/public type.

Initial representation:

```rust
pub enum ConstantFact {
    Int(Box<str>),          // normalized exact integer spelling/value
    FloatBits(u64),         // exact IEEE payload, including signed zero/NaN policy
    Bool(bool),
    String(Arc<str>),       // budget-capped; otherwise NotRetained
    Symbol(Symbol),
    NotRetained,
}
```

The numeric spec must separately ratify NaN equality/refinement rules before float constant facts participate in type refinements. Until then they are proof/optimization evidence only.

### 3.4 Termination, cycles, and budgets

Every recursive semantic operation receives a `QueryBudget` and `CancellationToken`. Minimum dimensions:

```rust
pub struct QueryBudget {
    pub max_steps: u64,
    pub max_relation_pairs: u32,
    pub max_scc_iterations: u32,
    pub max_type_depth: u16,
    pub max_diagnostic_notes: u16,
}
```

Cycle policy:

- type display/serialization: maintain an active-node set; emit a stable recursive marker or error, never recurse indefinitely;
- equivalence/subtyping: memoize `(relation, lhs, rhs, environment)` with `Visiting/Resolved`; use relation-specific coinductive rules only where ratified;
- superclass traversal: repeated declaration means invalid inheritance cycle and `Refuted`, never an infinite walk;
- semantic SCC: predeclare all binders, then process a deterministic worklist until unchanged or budget exhaustion;
- callable inference recursion: create callable shells, solve SCC summaries monotonically, and return `Blocked(RecursiveFixpoint)` if no bounded terminal state exists;
- runtime dependency graph: cycles are hard link errors; no sorted fallback;
- cancellation: return `Cancelled`, discard unpublished products, leave prior snapshot intact.

Ordinary callable correctness is partial. Future totality uses separate `TerminationRequirement` and `TerminationKnowledge`; `Never` does not prove termination or divergence.

## 4. Target architecture and Rust data model

### 4.1 Stable, store, and solver identities

Add to `phalcom-semantic/src/identity.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkspaceId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticRevision(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeStoreId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SnapshotId {
    pub workspace: WorkspaceId,
    pub revision: SemanticRevision,
    pub store: TypeStoreId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SnapshotTypeRef {
    pub store: TypeStoreId,
    pub id: TypeId,
}
```

`TypeId` and `KindId` stay compact. Cross-snapshot/query-consumer APIs use paired handles or snapshot methods. `InferVarId`, future `KindVarId`, CFG block IDs, and query execution IDs remain solver/query-local.

Add stable physical identities in `phalcom-modules/src/identity.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableProjectKey {
    pub source: ProjectSourceIdentity, // canonical logical/physical root
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableModuleKey {
    pub project: StableProjectKey,
    pub path: ModulePath,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectRevisionFingerprint(pub [u8; 16]);
```

`ResolvedProjectId` remains a cheap graph-local index. `StableProjectKey` is identity; manifest/dependency fingerprints are revision inputs. Changing the manifest must invalidate products without pretending the project became a different logical project. Synthetic contexts use a stable session key plus cell path; they do not serialize a process counter as durable identity.

### 4.2 Type-store lifecycle and invariant enforcement

Add `TypeStoreId` to `TypeStore` and make its internals append-only during one DB epoch. A `SemanticDb` owns one store epoch; immutable snapshots share `Arc<TypeStore>` views. Store entries never mutate, so old snapshots remain coherent when new nodes append. All output sorting and fingerprints use structural forms, never allocation order.

When the store exceeds a measured high-water threshold, compaction builds a new store, reinterns live query roots structurally, changes `TypeStoreId`, invalidates raw-ID products, and atomically publishes a new snapshot. Compaction is explicit and observable; it never mutates an existing snapshot.

API changes in `types/store.rs` and `types/evidence.rs`:

```rust
pub fn proper_type(&self, id: TypeId) -> Result<ProperTypeId, KindId>;

#[repr(transparent)]
pub struct ProperTypeId(TypeId);

impl TypeKnowledge {
    pub fn known(ty: ProperTypeId, authority: EvidenceAuthority) -> Self;
}
```

Tuple, record, union, and callable constructors accept `ProperTypeId` children. Type constructors continue using `TypeId`, because they may have arrow kind. Transitional raw constructors become `pub(crate)` and are deleted after all call sites migrate. This makes invalid states unrepresentable in release builds without inflating every `TypeId`.

### 4.3 Semantic database

Create `phalcom-semantic/src/db/{mod.rs,key.rs,state.rs,dependency.rs,scheduler.rs,budget.rs}.rs`:

```rust
pub struct SemanticDb {
    workspace: WorkspaceId,
    store_epoch: RwLock<TypeStoreEpoch>,
    inputs: RwLock<InputTable>,
    queries: RwLock<QueryTable>,
    reverse: RwLock<ReverseDependencyIndex>,
    current: ArcSwap<SemanticSnapshot>,
    scheduler: QueryScheduler,
}

pub enum QueryKey {
    ParsedModule(StableModuleKey),
    UnlinkedInterface(StableModuleKey),
    LinkedInterface(StableModuleKey),
    DeclarationShell(DeclarationId),
    SemanticComponent(SemanticComponentKey),
    DeclarationSurface(DeclarationId),
    CallableBody(CallableId),
    ModuleDiagnostics(StableModuleKey),
    ModuleMetadata(StableModuleKey, MetadataProfile),
    ProofObligation(ProofObligationKey), // inactive until proof spec lands
}

pub enum QueryState {
    Vacant,
    Computing { revision: SemanticRevision, stack_index: u32 },
    Ready { revision: SemanticRevision, fingerprint: ProductFingerprint, value: Arc<QueryValue> },
    Blocked { revision: SemanticRevision, reason: BlockReason },
    Cancelled { revision: SemanticRevision },
    Failed { revision: SemanticRevision, failure: InternalFailureId },
}

pub enum QueryOutcome<T> {
    Ready(T),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    Failed(InternalFailureId),
}
```

`ArcSwap` above is architectural notation; use an existing safe atomic-Arc abstraction or `RwLock<Arc<_>>` if dependency policy rejects the crate. Unsafe/raw-pointer publication is forbidden.

Query dependencies are recorded dynamically while reading other products:

```rust
pub struct DependencyEdge {
    pub dependent: QueryKey,
    pub dependency: QueryKey,
    pub observed_fingerprint: ProductFingerprint,
}
```

The reverse index is compiler-owned. LSP may retain its advisory graph only for advisory products.

### 4.4 Snapshot boundary

Refactor `SemanticSnapshot` fields to private and add:

```rust
pub struct SemanticSnapshot {
    id: SnapshotId,
    store: Arc<TypeStore>,
    modules: Arc<BTreeMap<StableModuleKey, ModuleSemanticState>>,
    declarations: Arc<DeclarationTypeTable>,
    surfaces: Arc<SurfaceTable>,
    dispatch: Arc<SurfaceDispatchResolver>,
    hierarchy: Arc<MapTypeHierarchy>,
    facts: Arc<SemanticFactTables>,
    diagnostics: Arc<DiagnosticSet>,
    semantic_graph: Arc<SemanticGraph>,
    fingerprints: Arc<SnapshotFingerprints>,
    status: SnapshotStatus,
}

pub enum ModuleSemanticState {
    Complete(ModuleSemanticProducts),
    Blocked { phase: AnalysisPhase, reason: BlockReason },
    Invalid { phase: AnalysisPhase, diagnostics: Arc<[DiagnosticId]> },
}

pub enum SnapshotStatus {
    Complete,
    Partial { blocked_modules: u32 },
}
```

Snapshots never publish `Cancelled`, `BudgetExceeded`, or `InternalFailure` as completed facts. A partial snapshot may retain valid independent modules while explicitly marking blocked modules. Compiler strict mode refuses an entry whose reachable closure is invalid or blocked; LSP publishes the partial snapshot plus reasons.

### 4.5 Relation and lookup outcomes

Replace public `is_subtype(...) -> bool` and coarse `Assignability` with:

```rust
pub enum RelationOutcome<T = ()> {
    Proven { value: T, evidence: RelationEvidence },
    Refuted(RelationFailure),
    DynamicBoundary(DynamicBoundaryObligation),
    Blocked(BlockReason),
}

pub enum RelationKind {
    Equivalent,
    Subtype,
    Assignable,
    Consistent,
    Conforms,
}

pub enum BlockReason {
    UnknownType(UnknownReason),
    UnresolvedDependency(StableModuleKey),
    InvalidAnnotation(DiagnosticId),
    RecursiveFixpoint,
    OpaqueNative(NativeSurfaceKey),
    ReflectionBoundary,
    BudgetExceeded(BudgetReport),
}

pub type MemberLookupOutcome = RelationOutcome<SpecializedMemberView>;
```

Internal fast predicates may return `bool` only after operands are validated, cycle-free, and within a parent query that owns the budget. They are not public semantic judgments.

### 4.6 Diagnostics contract

Change [`DiagnosticLabel`](../../../../phalcom-semantic/src/diagnostic.rs#L55) and primary locations from bare ranges to owned spans:

```rust
pub struct SemanticSourceSpan {
    pub module: ModuleId,
    pub range: SourceRange,
}

pub struct DiagnosticLabel {
    pub span: SemanticSourceSpan,
    pub message: String,
}

pub struct SemanticDiagnostic {
    pub id: DiagnosticId,
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary: SemanticSourceSpan,
    pub labels: Vec<DiagnosticLabel>,
    pub notes: Vec<DiagnosticNote>,
    pub provenance: DiagnosticProvenance,
}
```

This fixes cross-module related information and makes CLI/LSP URI adaptation deterministic.

Required new stable categories:

| Code | Default severity | Meaning |
|---|---:|---|
| `project.load.failed` | error | Owning project could not be loaded; module is not reinterpreted as standalone |
| `module.interface.failed` | error | Interface product invalid |
| `module.import.unresolved` | error | Import target resolution failed |
| `module.link.failed` | error | Link component invalid |
| `module.runtime_cycle` | error | Eager runtime dependency cycle |
| `analysis.blocked` | information/warning by strictness | Valid conclusion unavailable; reason attached |
| `analysis.cancelled` | not persisted as source diagnostic | Request cancelled; prior snapshot remains published |
| `analysis.budget_exceeded` | warning or strict error | Query stopped at configured bound; no success fact created |
| `analysis.internal_failure` | error, tool source | Internal invariant failure with stable incident ID |
| `type.relation.cycle` | error or blocked | Illegal recursive relation/hierarchy cycle |
| `type.dynamic_boundary` | warning in strict mode | Runtime check/uncertainty required |

Project/link errors must flow through the same JSON/LSP schema as type diagnostics. Internal failures are never attributed to user source unless a primary source span is known.

## 5. Algorithms

### 5.1 Revision update and invalidation

1. Normalize source identity at the `phalcom-modules` boundary; resolve `StableProjectKey` and `StableModuleKey`.
2. Compare source content fingerprint. Ignore a byte-identical update except for client revision bookkeeping.
3. Recompute `ParsedModule`; record parser outcome without deleting the previous published snapshot.
4. Fingerprint imports and declaration/interface surface independently from bodies.
5. Seed invalidation:
   - body-only change: callable/body queries plus dynamic reverse dependents;
   - interface change: linked interface, semantic SCC, imported declaration/body closure;
   - module add/remove: resolution candidates, importers, SCC membership, runtime graph;
   - project manifest/lock change: project graph and every module whose stable resolution inputs changed;
   - native schema change: universe surface and all queries that consumed it.
6. Walk the reverse index with a deterministic queue and a configured maximum; if the maximum is reached, conservatively invalidate the project closure and report an observability event, not a source error.
7. Recompute demanded products; unchanged fingerprints stop propagation.
8. Freeze all complete/blocked/invalid module states and publish atomically only if revision and cancellation token are current.

### 5.2 Interface-first project checking

1. Discover project and module identities. Preserve failures as `ModuleSemanticState::Invalid`.
2. Parse all reachable module headers/sources demanded by the entry or editor workspace.
3. Build unlinked interfaces for every parseable module before any body checking.
4. Resolve imports into linked interfaces. Keep unresolved edges with source spans and candidate identities.
5. Build semantic graph and compute deterministic SCCs.
6. For each SCC in dependency order:
   1. allocate declaration shells for every binder;
   2. assign owner-qualified type/kind parameter binders;
   3. lower annotations and kinds against shells;
   4. reject inheritance cycles; allow only ratified recursive aliases/ADTs/protocol structures;
   5. publish typed interfaces for the SCC.
7. Check bodies/callable SCCs against typed interfaces.
8. Build diagnostics and exported metadata queries.
9. Validate runtime DAG separately. A cycle returns `module.runtime_cycle`; never produce synthetic initialization order.

### 5.3 Bounded relation evaluation

1. Validate both handles belong to the relation context's `TypeStoreId` and are proper when the relation requires proper types.
2. Normalize operands and create `RelationKey { kind, lhs, rhs, environment_fingerprint }`.
3. Return memoized terminal result if present.
4. If key is `Visiting`, apply the relation's ratified recursion rule:
   - inheritance: refute illegal cycle;
   - equirecursive alias/protocol relation: coinductive success only after that feature ratifies guardedness;
   - otherwise `Blocked(RecursiveFixpoint)`.
5. Charge one step and one pair. On exhaustion return `Blocked(BudgetExceeded)`.
6. Decompose structurally using an explicit worklist; never recursive Rust calls for attacker-controlled depth.
7. Record evidence or minimal refutation path.
8. Store terminal result; clear `Visiting` on cancellation/failure.

### 5.4 Dynamic dispatch boundaries

For a statically known receiver/selector, member lookup uses actual selector labels, side, inheritance, `super` lookup start, generic substitution, visibility, and constructor rules. For `perform`:

- constant selector plus statically known complete argument pack may reuse ordinary lookup but records reflection-boundary provenance;
- nonconstant selector or argument shape yields `DynamicBoundary`;
- DNU overrides never manufacture a statically known return type unless an explicit, checked DNU contract is later ratified;
- opaque native/FFI calls use typed metadata when available, otherwise `Blocked(OpaqueNative)` or explicit `DynamicBoundary` by policy.

### 5.5 Cancellation and publication

Cancellation is checked before expensive query entry, on every solver/worklist interval, before metadata serialization, and before publication. A cancelled revision may fill no reusable cache entry except source-independent canonical interning already committed. Latest-wins publication compares workspace, revision, source fingerprints, and document revision. Old immutable snapshots remain readable until consumers release them.

## 6. Diagnostics and developer experience

CLI, JSON, REPL, and LSP consume the same `DiagnosticSet`. Adapters may change rendering only.

Severity policy:

- hard error: syntax/interface/link/kind/type contradiction, illegal cycle, violated strict obligation;
- warning: explicit dynamic/reflection/native boundary under configured strictness, unproven optional totality, deprecated transitional syntax;
- information/hint: analysis blocked in permissive/editor mode with actionable cause;
- cancelled: operational status only, never a source diagnostic and never a successful fact;
- budget exhaustion: reasoned blocked result plus one deduplicated diagnostic/event;
- internal failure: stable tool diagnostic and incident ID; never downgrade to `Unknown` silently.

Minimum JSON fields:

```json
{
  "schemaVersion": 1,
  "code": "module.import.unresolved",
  "severity": "error",
  "message": "...",
  "primary": { "module": "...", "uri": "...", "range": {} },
  "related": [],
  "notes": [],
  "provenance": { "phase": "link", "revision": 42 },
  "status": "invalid"
}
```

LSP keeps exact generation/revision/text guards already implemented. Static and advisory hover may coexist, but formal type facts lead and advisory facts are labeled “runtime shape estimate.”

## 7. Dependency-ordered implementation plan

### Unit A1 — enforce semantic invariants and terminal results

**Files:**

- modify `phalcom-semantic/src/types/{id,store,evidence,relation}.rs`;
- modify checker call sites under `phalcom-semantic/src/checker/`;
- add `phalcom-semantic/src/types/outcome.rs`;
- extend `phalcom-semantic/tests/{kinds,checker,substitution,denotation}.rs`.

Steps:

1. Add `ProperTypeId`; migrate `TypeKnowledge::known` and proper-type constructors.
2. Make unchecked raw constructors `pub(crate)` and document trusted preconditions.
3. Add `RelationOutcome`, `BlockReason`, evidence/refutation paths, visited-pair worklist, and budgets.
4. Keep compatibility wrappers only inside tests; delete when no production call site uses boolean `is_subtype`.
5. Add `ConstantFact` to `TypedExpression`/`ValueSemanticFact`; retain numeric runtime-class synthesis.

Deletion criteria: no public raw `known(TypeId, ...)`; no public semantic relation that maps blocked/dynamic to `true`; no release-only malformed proper type.

### Unit A2 — source-owned diagnostics and stable identities

**Files:**

- modify `phalcom-semantic/src/{diagnostic,identity,snapshot}.rs`;
- modify `phalcom-modules/src/{identity,project}.rs`;
- modify `phalcom-core/bin/phalcom/cli.rs` and `phalcom-lsp/src/diagnostics.rs` adapters;
- add cross-module related-information tests.

Steps:

1. Add stable project/module keys and separate revision fingerprints.
2. Add `SnapshotId`, `TypeStoreId`, and checked snapshot handles.
3. Convert every diagnostic range to `SemanticSourceSpan`.
4. Add project/link/blocked/internal codes and a versioned JSON representation.
5. Preserve current human rendering until snapshot tests approve intentional changes.

Deletion criteria: no placeholder/current-document URI for a cross-module label; durable products contain no `ResolvedProjectId` without its stable key.

### Unit A3 — semantic DB and staged products

**Files:**

- create `phalcom-semantic/src/db/` modules listed in §4.3;
- refactor `phalcom-semantic/src/workspace.rs` into cold orchestration over queries;
- extend `phalcom-semantic/src/invalidation.rs`;
- reuse `phalcom-modules/src/{source,interface,graph,linker,declaration}.rs` APIs.

Steps:

1. Implement input/query tables, fingerprints, dependency recorder, reverse index, cancellation, budget, deterministic scheduler.
2. Wrap current cold phases as query implementations without semantic changes.
3. Publish typed interfaces before bodies.
4. Implement semantic SCC worklist and callable recursive summary shells.
5. Make `analyze_workspace` a compatibility cold-entry facade backed by a temporary `SemanticDb`.

Deletion criteria: current phase logic has one owner; no copied interface/link/check implementation in DB and legacy function.

### Unit A4 — partial workspace outcomes and runtime-cycle correctness

**Files:**

- modify `phalcom-semantic/src/{snapshot,workspace,diagnostic}.rs`;
- modify `phalcom-modules/src/{linker,graph,error}.rs` only where structured source ownership is missing;
- modify `phalcom-lsp/src/analysis_service.rs`.

Steps:

1. Represent `Complete/Blocked/Invalid` module states.
2. Preserve project, interface, resolution, load, and link failures.
3. Publish valid independent modules with explicit partial status.
4. Replace `initialization_order().unwrap_or_else(sorted)` with structured failure and no linked runtime program.
5. Add scan cancellation symmetry after static analysis.

Deletion criteria: no failure-swallowing `if let Ok`/`let Ok(..) else { continue }` in formal workspace construction without a recorded diagnostic/state.

### Unit A5 — compiler, CLI, REPL, and LSP consumers

**Files:**

- modify `phalcom-core/src/modules/compile.rs`;
- modify REPL/session modules discovered from `phalcom-core` entry points;
- modify `phalcom-lsp/src/{analysis_service,backend,diagnostics}.rs` and `phalcom-lsp/src/semantic/{engine,snapshot,mod}.rs`.

Steps:

1. Make `ProgramAnalyzer` request a DB snapshot and entry reachability status.
2. Give REPL sessions `StableProjectKey::SyntheticSession` and monotonically named cell modules; record cell dependency edges.
3. Replace LSP `run_static_workspace_analysis` with a source-overlay provider feeding the compiler DB.
4. Keep advisory engine publication; attach formal snapshot atomically through existing bridge.
5. Publish only affected open URI diagnostics unless configuration/full refresh requires all.

Deletion criteria: remove LSP-owned project loading, recursive import loading, linking, and formal `analyze_workspace` call; only compiler DB owns them.

### Unit A6 — observability, compaction, and performance

**Files:**

- add `phalcom-semantic/src/db/metrics.rs`;
- extend CLI/LSP performance harnesses;
- add deterministic store-compaction tests.

Counters: query hits/misses, invalidation seeds/closure, SCC iterations, relation pairs, cancellations, blocked outcomes by reason, store nodes/bytes, compactions, snapshot publish latency, per-phase cold/warm time.

No optimization may change outcomes or suppress diagnostics. Parallel query execution is deferred until deterministic single-scheduler behavior is measured; solver-local state is never shared.

## 8. Migration compatibility and “must not preclude”

Compatibility rules:

- `analyze_workspace` remains during consumer migration and returns the new snapshot through a cold temporary DB.
- current `SemanticSnapshot` getters are added before public fields become private.
- `CompiledTypeRef` remains the stable structural test adapter until Spec 02 metadata DAG lands.
- LSP advisory facts and current editor features remain live throughout migration.
- no user syntax or runtime representation changes in Units A1–A6.

Architecture must not preclude:

- prenex kind schemes with stable `KindParameterId` and ephemeral `KindVarId`;
- record-specific `RecordRow`, separate variant/effect rows, and row inference;
- declared `+`/`-` variance, capture-safe substitution, F-bounds, `Self`, aliases, constraints, HKTs;
- protocols, class-side requirements, constructors, overload policy, ADTs, exhaustiveness;
- effects, explicit totality, persistent proof artifacts, VC/prover queries;
- explicit reflection and `perform`/DNU/FFI boundaries;
- incremental LSP, project/package checks, and coherent REPL cells.

## 9. Verification and acceptance

### Unit and property tests

- every constructible `Known` fact contains a proper type;
- every interned form has exactly one kind; application kind/arity laws hold;
- normalization/equivalence laws: reflexive/symmetric/transitive and idempotent normalization;
- subtype reflexivity/transitivity and `Never` bottom property over generated bounded type graphs;
- assignability, consistency, conformance, and lookup never call one another by accidental boolean alias;
- generated inheritance cycles terminate with stable failure;
- generated deep/recursive forms reach success, refutation, or budget outcome within bound;
- constant fact retention never changes inferred nominal numeric class.

### Integration and corpus tests

- typed interfaces exist for mutually referring semantic modules before body checking;
- body edit recomputes changed callable and recorded dependents only;
- public signature edit invalidates exact reverse closure; unrelated module products retain fingerprints;
- provider addition repairs unresolved candidate edges;
- invalid project/import/interface/link state publishes diagnostics for the owning source and does not become standalone;
- runtime dependency cycle never yields a compiled initialization order;
- compiler, CLI JSON, LSP, and REPL render identical codes/messages/ranges for the same revision;
- cancellation during parse, SCC, relation, body, metadata, and pre-publication retains prior snapshot;
- source edit after worker start cannot publish stale facts.

### Fuzz and robustness tests

- fuzz type DAG decoding/normalization/relation with depth/step limits;
- fuzz semantic graphs and SCC order; no panic or nondeterministic public ordering;
- fuzz project/link error paths; every consumed failure becomes a state/diagnostic;
- fuzz cancellation injection at every scheduler yield point.

### Performance acceptance

Measure before setting thresholds. Required benchmark scenarios:

1. cold representative multi-module project;
2. no-op refresh;
3. body-only leaf edit;
4. public-interface root edit;
5. provider add/remove;
6. 100 rapid cancelled editor revisions;
7. store compaction.

Minimum qualitative gates:

- no-op refresh performs zero semantic query recomputation;
- leaf body edit does not rebuild/link whole workspace;
- final work is proportional to invalidated reverse closure plus fixed publication cost;
- no unbounded memory growth across an edit/revert loop after compaction;
- deterministic structural output matches fresh cold analysis.

### Evidence reporting

Implementation reports must separate:

- passing evidence run on current code;
- baseline/unrelated failures;
- deferred performance measurements;
- unverified claims;
- protected user changes left untouched.

Supplied Task 13 results establish the pre-migration baseline only. Each unit must rerun focused tests and final workspace acceptance when implementation is authorized.

## 10. Risks and ratification gates

| Risk | Required gate |
|---|---|
| Append-only interner grows under editor churn | Benchmark and compaction gate before long-lived LSP default |
| Stable path identity breaks on symlink/rename | Ratify canonical logical-source policy; do not hash transient numeric project IDs |
| Coinductive recursion accidentally accepts illegal cycles | Feature-specific guardedness rule before enabling recursive aliases/protocols |
| Dynamic becomes silent success | API review: only `DynamicBoundary`, never `Proven` |
| Partial snapshot lets compiler compile blocked entry | Reachability gate in `ProgramAnalyzer`; strict compile requires complete reachable closure |
| Parallel execution changes type ID allocation/order | Structural output determinism and single-writer phase barrier |
| Query dependency under-recording yields stale facts | Differential test: incremental result equals fresh cold result after every edit sequence |
| Advisory LSP facts regain authority | Architecture review: no advisory type converts into `TypeKnowledge::Known` with rejecting authority |

Open language gates intentionally not solved here: row surface syntax, kind-polymorphic syntax, general overload policy, protocol coherence, recursive alias/ADT guardedness, effect-row laws, totality annotation syntax, and proof trust configuration.

## 11. Take directly / adapt / reject

### Take directly

**Pyrefly architectural transfer.** Dense IDs, canonical tables, query states, immutable snapshots, dependency recording, reverse invalidation, SCC worklists, cancellation, bounded evaluation, deterministic regression fixtures, and observability.

### Adapt

**Pyrefly architectural transfer.** Query keys use Phalcom stable modules, declarations, callables, selectors, labels, side, `super`, native surface fingerprints, and semantic/runtime graph separation. Type queries must remain open for future kind schemes, rows, constraints, and proof artifacts.

### Reject

**Ratified/normative design.** Python `Any`/unknown/import/attribute/protocol/descriptor/overload rules; editor-owned formal semantics; unsafe raw-pointer publication; global solver state; unbounded fixed points; runtime selector or dispatch changes driven by types; performance targets copied without Phalcom measurements.
