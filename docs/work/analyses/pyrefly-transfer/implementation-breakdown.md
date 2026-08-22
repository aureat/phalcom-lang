# Pyrefly transfer: Phalcom implementation breakdown

## Purpose

This document is an implementation-ready decomposition for transferring Pyrefly's performance and semantic-analysis patterns into Phalcom. It describes ownership, data structures, query boundaries, solver behavior, invalidation, tests, and measurement.

It is not a request to copy Pyrefly source code. Pyrefly's repository is MIT-licensed, but any direct code reuse still requires normal license and attribution review. The intended transfer is architecture and implementation logic adapted to Phalcom's message-send semantics, runtime classes, families, reflection, native contracts, modules, and open-world behavior.

## Status key

- **CURRENT** — exists in the checkout; may be experimental or part of dirty work and is not automatically accepted as shipped.
- **TARGET** — intended architecture after this work.
- **BRIDGE** — compatibility layer that lets current code and target code coexist.
- **DEFERRED** — wait for semantic specification or measurement.
- **ACCEPTANCE** — required evidence before treating phase as complete.

## Current Phalcom seams

The transfer should build on these existing areas:

| Current seam | What it provides | Transfer use |
|---|---|---|
| `phalcom-semantic/src/identity.rs` | `ModuleId`, `DeclarationId`, `CallableId`, `FieldId`, and binding identity | Stable semantic ownership. Extend with typed query, expression, and constraint IDs. |
| `phalcom-semantic/src/types/id.rs` | `TypeId`, `KindId`, `TypeParameterId`, `InferVarId` | Compact type and inference identities. Keep inference variables distinct from persistent types. |
| `phalcom-semantic/src/types/store.rs` | Hash-consed `TypeStore`, `TypeData`, normalized flat unions | Canonical type representation and future allocation seam. |
| `phalcom-semantic/src/types/relation.rs` | Type hierarchy, assignability tri-state, subtype relation | Semantic-order boundary and conservative relation results. |
| `phalcom-semantic/src/types/evidence.rs` | `Known`, `Unknown`, `Dynamic`, evidence authority, provenance | Preserve uncertainty and explain widening/blocked analysis. |
| `phalcom-semantic/src/types/annotation.rs` | `TypeResolver` and simple annotation resolution | Bridge syntax annotations into canonical descriptors. |
| `phalcom-semantic/src/checker/` | Shared semantic checker and expression/statement checking | Replace one-pass local checking with query-backed facts incrementally. |
| `phalcom-semantic/src/snapshot.rs` | Generation-tagged immutable semantic products | Publish one coherent generation to CLI and LSP. |
| `phalcom-lsp/src/semantic/engine.rs` | Worker-owned candidate state, affected closure, immutable snapshot, reuse metrics | Migrate reusable engine mechanics into shared semantic services. |
| `phalcom-lsp/src/semantic/invalidation.rs` | Body/import/declaration/core change classes and deterministic queues | Drive module, declaration, and callable invalidation. |
| `phalcom-lsp/src/semantic/infer.rs` | Callable worklist, dependency propagation, bounded feedback cycle | Bridge current advisory summaries into formal callable facts. |
| `phalcom-semantic/src/dispatch.rs` | Dispatch result categories and surface resolver | Semantic lookup service behind type relations; never key runtime dispatch by static type. |
| `phalcom-semantic/src/surface.rs` | Declaration fields, callables, signatures | Export/surface stage and member contract facts. |

The current experimental type store and constraint solver are valuable seeds. They must be described and tested as experimental until acceptance gates pass.

## Target ownership model

Use one mutable semantic worker per project/session and immutable snapshots for readers:

```text
editor / CLI request
        │
        ▼
immutable SemanticSnapshot(generation, revision)
        ▲
        │ publish only after complete, cancellation-safe build
worker-owned SemanticEngine
        │
        ├── source/module products
        ├── dependency graph and invalidation frontier
        ├── canonical TypeStore session
        ├── query/answer cells
        └── solver worklists and metrics
```

Rules:

- Readers never observe a partially rebuilt module.
- A result is reusable only when its source revision, semantic revision, dependency fingerprint, and solver configuration match.
- Cancellation discards or quarantines candidate state; it must not publish partial answers.
- LSP, CLI, diagnostics, hover, completion, definition, and inlay hints query the same semantic products.
- `Arc::ptr_eq` can measure structural product reuse; it cannot decide semantic type equality.

## Target data model

### Stable indexes

Adopt typed index wrappers for hot tables. Prefer stable semantic identity over source byte offsets for long-lived Phalcom products.

```rust
struct ModuleIndex(u32);
struct DeclarationIndex(u32);
struct BindingIndex(u32);
struct CallableIndex(u32);
struct QueryIndex(u32);
struct ConstraintIndex(u32);

struct IndexMap<K, V> {
    values: Vec<Option<V>>,
    reverse: HashMap<K, u32>,
}
```

The exact implementation can use `NonZeroU32` or a newtype around `u32`. The invariant is typed ownership and dense lookup, not the spelling of the wrapper.

Required properties:

- IDs from one store cannot be passed to another store without an explicit conversion.
- Deleted or stale cross-module indexes are detectable.
- Hot paths do not allocate strings for every binding lookup.
- Source ranges remain diagnostic metadata, not durable semantic identity.
- IDs are either snapshot-scoped and generation-tagged, or explicitly stable across snapshots.

### Query keys and answers

Start with safe, explicit cells. Optimize publication only after measuring contention and allocation.

```rust
struct QueryKey {
    kind: QueryKind,
    owner: SemanticOwner,
    input: QueryInput,
    semantic_revision: u64,
}

enum QueryState<T> {
    Uncomputed,
    Computing { owner: QueryOwner, cycle: CycleId },
    Complete(Arc<T>),
    Blocked(BlockReason),
    Cancelled,
}

struct QueryAnswer<T> {
    state: QueryState<T>,
    dependencies: SmallVec<[DependencyKey; 4]>,
    provenance: EvidenceSet,
    metrics: QueryMetrics,
}
```

The final form can separate mutable state and published `Arc<T>` value. Do not encode unresolved state as an absent map entry: `Uncomputed`, `Computing`, `Blocked`, `Cancelled`, and `Complete` have different scheduler and diagnostic meaning.

### Constraints and solver outcomes

```rust
enum ConstraintRelation {
    Equal,
    Subtype,
    HasMember(MemberSelector),
    Callable(CallShape),
    Conforms(ProtocolId),
}

struct Constraint {
    id: ConstraintId,
    relation: ConstraintRelation,
    lhs: TypeTerm,
    rhs: TypeTerm,
    origin: SourceOrigin,
    depends_on: SmallVec<[QueryKey; 2]>,
    evidence: EvidenceSet,
}

enum SolveStatus {
    Solved,
    Underconstrained,
    Ambiguous,
    Inconsistent,
    BlockedByDynamicBoundary,
    RecursiveFixpoint,
    BudgetExceeded,
    Cancelled,
}
```

`TypeTerm` may contain canonical `TypeId`, `InferVarId`, tuples, applied types, callables, and temporary projections. Do not store every partially solved term permanently in the type store. Temporary solver state belongs to the query/session; only canonical completed descriptors enter the persistent store.

### Dependency keys

```rust
enum DependencyKey {
    Module(ModuleId),
    ExportName { module: ModuleId, name: Symbol },
    ExportType { module: ModuleId, name: Symbol },
    ExportMetadata { module: ModuleId, name: Symbol },
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    NativeContract(NativeContractId),
    CoreSurface(CoreSurfaceId),
    WildcardExports(ModuleId),
}
```

Record the narrowest dependency known. A wildcard or reflective dependency can intentionally widen to a coarse key. This is safer than pretending to know a fine-grained dependency that cannot be proven.

## Phase 0 — Establish baseline and semantic boundaries

**Goal:** freeze the current mental model before changing architecture.

### Work

- [ ] Record current behavior of `phalcom check`, compiler, semantic checker, and LSP semantic engine.
- [ ] Mark current `TypeStore`, `TypeConstraint`, `LocalConstraintSolver`, `ValueShape`, and `SemanticSnapshot` as current/experimental where appropriate.
- [ ] Write a relation matrix for equivalence, subtype, assignability, consistency, conformance, and dispatch.
- [ ] Record which facts are `Declared`, `Proven`, `ExactSyntax`, `TrustedNative`, and `Advisory`.
- [ ] Specify selector and callable identity as type-independent.
- [ ] Identify dynamic boundaries: reflection, `perform`, family dispatch, native/opaque calls, method mutation, open classes, and missing modules.
- [ ] Define which typing modes are sound enough to reject code and which are advisory only.

### Acceptance

- [ ] A reader can tell formal type facts from LSP advisory shapes.
- [ ] No design document claims that the current VM-coupled pipeline is already a formal checker.
- [ ] At least one test proves adding an annotation does not change selector identity or dispatch table identity.
- [ ] Unknown, Dynamic, Never, and budget-limited outcomes have distinct diagnostic policy.

## Phase 1 — Semantic identity and indexed storage

**Goal:** remove repeated string/large-object identity from hot paths.

### Work

- [ ] Add typed indexes for module, declaration, binding, callable, field, query, constraint, and expression ownership.
- [ ] Keep `DeclarationId`, `ModuleId`, `CallableId`, and `FieldId` as public semantic identities; use dense indexes behind stores where useful.
- [ ] Add an `IndexMap`-style store with O(1) index lookup and reverse lookup where needed.
- [ ] Add explicit store/snapshot identity to IDs that cannot safely cross generations.
- [ ] Keep `TextRange`, URI, and source offsets in source metadata; never use them as the only long-lived binding identity.
- [ ] Add deterministic iteration for diagnostics and tests. Hash map iteration must not control observable ordering.

### Phalcom mapping

- `phalcom-semantic/src/identity.rs` remains the semantic identity boundary.
- `phalcom-semantic/src/types/id.rs` remains the type identity boundary.
- `phalcom-semantic/src/types/store.rs` owns type IDs and validates store ownership.
- `phalcom-lsp/src/semantic/invalidation.rs` uses typed dependency keys instead of raw filenames where possible.

### Tests

- [ ] Same declaration receives same stable identity across body-only edits.
- [ ] Declaration/surface edit changes the relevant revision or fingerprint.
- [ ] Cross-store `TypeId` misuse is rejected in debug/test builds.
- [ ] Dense lookup and reverse lookup property tests.
- [ ] Deterministic ordering test across repeated runs.

### Performance gate

Measure string allocations, hash lookups, table lookup latency, memory per binding, and index reuse before and after. Do not claim improvement from IDs without a baseline.

## Phase 2 — Canonical TypeStore and semantic equality

**Goal:** make type identity cheap, deterministic, and semantically correct.

### Current seed

`TypeStore` already hash-conses `TypeData` and normalizes flat unions by flattening, sorting, deduplicating, and removing `Never`. Preserve this behavior and make its rules explicit.

### Work

- [ ] Define type-store ownership: project core types, module/session types, and temporary solver terms.
- [ ] Add constructors for all approved type forms through one normalization boundary.
- [ ] Ensure union normalization is order-independent, idempotent, and deterministic.
- [ ] Decide whether intersection, refinement, callable effects, type parameters, aliases, and projections are canonical store nodes or solver terms.
- [ ] Add explicit variance metadata for applied types; do not infer covariance from recursive implementation convenience.
- [ ] Add semantic `TypeEq` with a context for recursive types, alpha-equivalent binders, and paired-node memoization.
- [ ] Keep ordinary `Eq`/`Hash` appropriate for store keys; use semantic equality when binder identity or recursive structure requires it.
- [ ] Add normalization budgets for union width, literal-like expansions, recursive type arguments, and nested structural terms.
- [ ] Return a widening/complexity outcome with provenance when a cap fires.

### Suggested API split

```rust
impl TypeStore {
    fn intern(&mut self, data: TypeData) -> TypeId;
    fn union(&mut self, members: impl IntoIterator<Item = TypeId>) -> TypeResult;
    fn normalize(&mut self, term: TypeTerm, budget: &mut NormalizeBudget) -> TypeResult;
}

fn type_equivalent(store: &TypeStore, left: TypeId, right: TypeId) -> Equivalence;
fn type_subtype(ctx: &RelationContext, left: TypeId, right: TypeId) -> RelationResult;
```

`TypeResult` must be able to report `Known`, `Unknown`, or `Dynamic` evidence rather than forcing all normalization failures into a type node.

### Tests

- [ ] `union(A, A) == A`.
- [ ] `union(A, union(B, C))` equals `union(C, A, B)`.
- [ ] `union(Never, A) == A`.
- [ ] Union normalization is idempotent.
- [ ] Semantic equality handles recursive and alpha-equivalent structures without infinite recursion.
- [ ] Structural store equality and semantic equality are tested separately.
- [ ] Type-store IDs never leak across incompatible store/snapshot ownership.
- [ ] Complexity caps produce explicit evidence and stable diagnostics.

### Performance gate

Track canonicalization hit rate, type node count, union member width, recursive comparison count, pair-memo hits, allocations, and bytes retained by each snapshot.

## Phase 3 — Staged module semantic pipeline

**Goal:** make formal typing operate on source/module products, never top-level VM execution.

### Target stages

```text
Load source/configuration
  -> Parse source snapshot
  -> Build exports and declaration surfaces
  -> Build bindings/scopes/imports/flow skeleton
  -> Construct callable/member/type queries
  -> Solve demanded facts and publish diagnostics
```

### Work

- [ ] Define a source snapshot with URI, canonical internal module key, revision, parsed representation, and source ranges.
- [ ] Preserve original URI for diagnostics while using canonical internal keys for cache/dependency identity.
- [ ] Build exports and declaration surfaces without executing user code.
- [ ] Build binding identities for definitions, uses, anonymous values, exports, and selectors/member references.
- [ ] Build import/module dependency products before solving bodies.
- [ ] Make class-side and instance-side declaration storage explicit.
- [ ] Record native/core contracts as versioned semantic inputs.
- [ ] Publish each completed stage as immutable `Arc` data owned by a generation.

### Phalcom mapping

- `SemanticSnapshot` becomes the public published product.
- `SemanticEngine` remains worker-owned and becomes the builder/orchestrator.
- `DeclarationSurface`, `SurfaceDispatchResolver`, and module graph become export/surface products.
- The checker consumes a semantic snapshot and query service; it must not reach into VM bytecode execution.

### Acceptance

- [ ] Identical source produces equivalent semantic products through CLI and LSP.
- [ ] A body edit preserves unchanged export/surface products.
- [ ] A surface edit invalidates dependent module products.
- [ ] No top-level execution is required to resolve a static declaration surface.
- [ ] Snapshot publication is all-or-nothing under cancellation.

## Phase 4 — Query and answer engine

**Goal:** calculate semantic facts on demand with explicit cache and cycle states.

### Initial query kinds

- module exports;
- declaration surface;
- binding type;
- callable parameter/return summary;
- expression type;
- member/field fact;
- dispatch fact;
- constraint solution;
- diagnostic explanation/provenance.

### Work

- [ ] Define `QueryKey` and `DependencyKey` with semantic revision and source ownership.
- [ ] Add `Uncomputed`, `Computing`, `Complete`, `Blocked`, and `Cancelled` states.
- [ ] Track query dependencies while calculating; commit them with the answer.
- [ ] Detect same-thread recursion through a query stack.
- [ ] Give recursive queries a solver placeholder or `RecursiveFixpoint` outcome, not `Dynamic` by default.
- [ ] Ensure cross-thread waits cannot deadlock mutually recursive queries. First implementation may schedule or duplicate safe work rather than block indefinitely.
- [ ] Add cancellation checks at query boundaries, worklist steps, SCC iterations, and before publication.
- [ ] Tag answers with source revision, semantic generation, dependency fingerprint, and solver configuration.
- [ ] Keep diagnostic trace side effects separate until an answer commits, so abandoned calculations do not leak stale diagnostics.

### Cache hierarchy

Implement and measure distinct layers:

1. Source and parse cache.
2. Module export/surface products.
3. Binding and flow skeletons.
4. Query answer cells.
5. Type-store canonicalization.
6. Solver memoization and subset/relation cache.
7. Published generation snapshot.

Do not merge all layers into one map. Each layer has different invalidation, ownership, and memory lifetime.

### Acceptance

- [ ] Repeated query on unchanged generation is a hit with no semantic recomputation.
- [ ] Stale answer cannot be reused after a relevant source/core/native revision.
- [ ] Same-thread recursive query terminates with explicit recursive state.
- [ ] Cancellation does not publish partial answer or diagnostics.
- [ ] Query hit/miss/recompute metrics are visible in debug/benchmark mode.

## Phase 5 — Constraint and fixed-point solver

**Goal:** replace sequential substitution with bounded, explainable, dependency-aware solving.

### Work

- [ ] Extend constraints with source origin, evidence, dependency keys, and relation kind.
- [ ] Separate `TypeId` descriptors from `InferVarId` solver variables.
- [ ] Add equality, subtype, callable, member, conformance, and projection constraints only as semantics are specified.
- [ ] Implement occurs check or an explicit recursive-type policy before binding variables to structures containing themselves.
- [ ] Replace one-pass vector solving with a worklist.
- [ ] Track variable bounds, substitutions, unresolved obligations, and contradiction reasons.
- [ ] Add SCC detection for mutually recursive bindings/callables.
- [ ] Use recursive variables/placeholders to prevent infinite expansion.
- [ ] Retain previous SCC answers when dependencies are unchanged; use them as warm starts, never as unvalidated truth.
- [ ] Add iteration, depth, union-width, constraint-count, and allocation budgets.
- [ ] Expose exact outcome when a budget or dynamic boundary prevents proof.

### Semantic-order boundary

Create a small interface analogous to Pyrefly's `TypeOrder`:

```rust
trait SemanticOrder {
    fn nominal_parents(&self, ty: TypeId) -> RelationResult<ParentSet>;
    fn member(&self, receiver: TypeId, selector: Selector) -> DispatchResult;
    fn callable(&self, ty: TypeId) -> CallableFacts;
    fn variance(&self, origin: KindId) -> Variance;
    fn conforms(&self, actual: TypeId, protocol: ProtocolId) -> RelationResult<()>;
}
```

The exact methods depend on Phalcom's type specification. The boundary matters: subtype/constraint code should request semantic facts rather than know module graphs, declarations, native registries, or LSP state.

### Relation caching

Cache relation results by a key including relation kind, actual type, expected type, relevant semantic revision, and policy mode. A subtype result from one hierarchy revision is not valid after a superclass or protocol contract change.

Cache positive and negative results separately from uncertainty. A `Refuted` result may be reusable; `Uncertain` often depends on a dynamic or incomplete boundary and must retain its reason.

### Acceptance

- [ ] Recursive functions terminate and preserve explicit recursive provenance.
- [ ] Mutually recursive modules terminate with stable results or explicit underconstrained status.
- [ ] Equalities are order-independent where semantics require it.
- [ ] Contradictions identify source origins and relation paths.
- [ ] Budget exhaustion never returns a sound `Known` fact without authority.
- [ ] Clean full solve and incremental solve produce equivalent semantic answers.

## Phase 6 — Flow, calls, members, and dispatch

**Goal:** connect formal types to Phalcom expressions while preserving runtime semantics.

### Work

- [ ] Keep `ValueShape` or similar runtime-shape knowledge separate from formal `TypeId` facts.
- [ ] Translate literals, collections, records, tuples, branches, loops, closures, sends, and returns into constraints and flow facts.
- [ ] Model branch refinement as a new flow fact, not mutation of declared type identity.
- [ ] Model call argument/result constraints through callable signatures and overload/family policy.
- [ ] Use `DispatchResolver` for member/send existence and ambiguity; ask `SemanticOrder` for type conformance separately.
- [ ] Keep selector identity, method table keys, inline cache keys, and dispatch side independent of annotation metadata.
- [ ] Make class-side versus instance-side lookup explicit.
- [ ] Treat reflection, `perform`, family dispatch, method mutation, opaque native calls, and open classes as explicit dynamic/unknown boundaries unless trusted contracts exist.
- [ ] Preserve declaration and callable provenance in diagnostics.

### Acceptance

- [ ] Type annotation changes checker facts but not runtime dispatch identity.
- [ ] Member lookup can be `Found`, `Missing`, `Ambiguous`, or `Dynamic` independently of subtype result.
- [ ] Branch refinement narrows the current flow fact without rewriting declared surface.
- [ ] Callable variance tests cover parameter contravariance and result covariance.
- [ ] Dynamic boundaries produce conservative results and useful explanations.

## Phase 7 — Incrementality and parallelism

**Goal:** recompute the smallest safe frontier while preserving coherent publication.

### Invalidation policy

Use the current change classification as a starting point:

| Change | Rebuild | Preserve |
|---|---|---|
| Body-only callable edit | callable body facts, dependent callable summaries, affected diagnostics | parse, exports, declaration surface, unaffected callable facts |
| Import/export edit | module exports, binding/import products, reverse module dependents | unrelated modules and contracts |
| Declaration/signature edit | declaration surface, member/call facts, dependent modules/callables | unrelated body facts when their dependencies remain valid |
| File add/remove | module graph, import/export frontier, dependent products | independent connected components |
| Core/native contract edit | explicitly dependent contract frontier; potentially project-wide | products outside declared contract dependencies |

### Work

- [ ] Replace broad file-level dependency edges with exported name/type/metadata, class/member, alias, wildcard, declaration, and callable keys.
- [ ] Use coarse wildcard/reflection keys when fine-grained tracking is impossible.
- [ ] Compute reverse dependency closure deterministically.
- [ ] Schedule independent module chunks in parallel only after dependency stages are available.
- [ ] Keep each module's shared mutable semantic state on one worker at a time.
- [ ] Separate open-file priority and diagnostic streaming from core semantic truth.
- [ ] Track generation and source revisions in every published product.
- [ ] Compare incremental output with clean full output in tests.

### Parallelism rules

Parallelize:

- independent parse/load work;
- independent module export/surface work;
- independent callable queries with no conflicting publication;
- diagnostics rendering after immutable facts are available.

Do not parallelize blindly:

- every expression in one recursive SCC;
- mutable type-store interning without ownership design;
- query publication without first-writer and generation checks;
- LSP client calls from worker code;
- runtime compiler/VM state that is not `Send`/`Sync` by contract.

### Acceptance

- [ ] Body-only edit does not rebuild an unchanged dependent surface.
- [ ] Surface edit reaches every reverse dependent that consumes the changed key.
- [ ] Parallel and serial modes produce equivalent answers and deterministic diagnostics.
- [ ] Cancellation and stale generations cannot overwrite newer snapshots.
- [ ] Open-file diagnostics can arrive before unrelated workspace propagation completes without presenting an incoherent local answer.

## Phase 8 — CLI, compiler, and LSP integration

**Goal:** expose one checker without coupling formal analysis to one frontend.

### CLI

- [ ] Add semantic checking mode separate from syntax-only parsing.
- [ ] Load project/module configuration and canonical module identities.
- [ ] Report diagnostics with source ranges, evidence authority, dependency path, and widening/unknown reasons.
- [ ] Provide a machine-readable summary for benchmark and CI comparison.

### Compiler/runtime

- [ ] Keep runtime compilation and bytecode generation independent of static type metadata unless an explicitly sealed optimization consumes proven facts.
- [ ] Do not use annotations as method identity, dispatch identity, allocation identity, or unchecked optimization permission.
- [ ] Add typed metadata only through the semantic model and explicit compiler contracts.

### LSP

- [ ] Replace duplicated inference with semantic snapshot/query requests.
- [ ] Keep worker ownership and nonblocking request handling.
- [ ] Rebuild server and validate configured server path after semantic changes.
- [ ] Restart language server and inspect output panel during manual checks.
- [ ] Test hover, completion, diagnostics, definition, references, and inlay hints against the same generation.
- [ ] Mark advisory answers visibly when formal proof is unavailable.

### Acceptance

- [ ] CLI and LSP agree on formal diagnostics for the same snapshot.
- [ ] LSP never calls into VM execution to answer a static query.
- [ ] Manual editor validation covers a clean file, body-only edit, surface edit, recursive code, and dynamic boundary.

## Phase 9 — Measurement and hardening

**Goal:** prove efficiency and semantic safety with repeatable workloads.

### Required metrics

Collect per run and per incremental edit:

- total wall time and CPU time;
- p50/p95/p99 open-file diagnostic latency;
- parse/export/surface/binding/solve time;
- modules considered, invalidated, recomputed, reused;
- query hits, misses, cycles, blocked answers, cancellations;
- constraint count, solver steps, SCC count, SCC iterations, demotions;
- subtype/equivalence cache hits and misses;
- type-store nodes, canonicalization hits, union widths, normalization caps;
- allocations, retained bytes, snapshot size, and peak memory;
- diagnostic count and deterministic ordering hash.

### Benchmark corpus

Create Phalcom-specific fixtures for:

- small fully annotated modules;
- unannotated inference-heavy code;
- wide unions and overload/family dispatch;
- mutually recursive modules/functions;
- large import/export graph;
- reflection and dynamic sends;
- native/opaque boundaries;
- many unchanged modules plus one body edit;
- one declaration edit with a large reverse dependency frontier;
- repeated hover/completion queries on one generation.

### Differential checks

- [ ] Full clean solve equals incremental solve for all trusted facts.
- [ ] Serial and parallel solve agree.
- [ ] Reordered union inputs agree.
- [ ] Repeated query does not change answer or diagnostics.
- [ ] Cancelled solve does not affect later clean solve.
- [ ] Removing an input invalidates all products that depend on it and no unrelated product in the tested corpus.
- [ ] Dynamic/unknown boundaries remain conservative under every budget mode.

### Suggested commands

Run focused validation only after the documentation/design work is complete and in a separate implementation change:

```text
cargo fmt --all --check
cargo test -p phalcom-semantic
cargo test -p phalcom-lsp --lib --no-fail-fast
cargo test --test integration
git diff --check
```

These commands are acceptance inputs, not proof that the target architecture exists. Add benchmark commands once a stable harness and corpus are committed.

## Type-theoretic design rules

### Type identity

Use `TypeId` for canonical descriptors and `InferVarId` for temporary unknown variables. Generic parameters require owner/binder identity so same-name parameters from different declarations do not compare equal accidentally.

### Relations

Define separate APIs and result types:

```text
equivalent(A, B)
subtype(A, B)
assignable(actual, expected)
consistent(A, B)
conforms(actual, protocol)
dispatch(receiver, selector)
```

Do not implement each as a call to one permissive relation. `Dynamic` may be consistent with many types while still being insufficient authority for a rejecting proof. `Unknown` may require a deferred query. `Never` should propagate as bottom or unreachable, not as missing data.

### Recursive types and fixed points

Represent recursive obligations with a stable variable or binder. Iteration should compare semantic answers, not allocation identity. Stop on semantic equivalence, a stable widening, an explicit budget, or cancellation. Record which stop condition occurred.

### Canonical unions and intersections

Normalize by flattening nested forms, removing identity elements, deduplicating semantically equivalent members where safe, and applying stable ordering. Bound width. Preserve provenance for members collapsed by a cap. Do not discard useful distinctions merely to make hash keys small.

### Callable and protocol variance

Keep callable parameter and result relations explicit. Protocol/member conformance needs member presence, callable variance, field mutability, side/selector rules, and dynamic boundary policy. A nominal parent map is insufficient for final conformance.

## Proposed commit-sized implementation order

The user did not request commits in this task. When implementation begins, keep these as separate cohesive units so the current dirty checkout remains reviewable:

1. semantic identity/index store and tests;
2. canonical type-store/equality/normalization and tests;
3. staged source/module products and snapshot publication;
4. query cells, dependency recording, and cycle tests;
5. constraint worklist/SCC solver and relation cache;
6. flow/call/member bridges and diagnostics;
7. precise invalidation and parallel scheduling;
8. CLI/LSP integration and manual editor validation;
9. benchmarks, differential tests, and documentation/status updates.

Do not stage unrelated current Rust changes or deleted typing documents with these units. Keep this transfer package separate from implementation commits.

## Acceptance matrix

| Capability | Current checkout | Target evidence |
|---|---|---|
| Canonical type IDs | `TypeId` and hash-consed `TypeStore` exist in experimental semantic work | Ownership, deterministic normalization, semantic equality, cap behavior, and tests accepted |
| Annotation resolution | Simple resolver handles a subset; composite forms are deferred | Full specified annotation forms resolve into canonical descriptors with provenance |
| Constraints | Equality/subtype/member seed; local sequential substitution | Worklist/SCC solver with bounds, occurs/recursive policy, budgets, and explicit outcomes |
| Semantic checking | Shared checker exists; broader pipeline remains incomplete | Demand-driven checker consumes staged snapshot products and no VM execution |
| Flow inference | LSP callable summaries and `ValueShape` are advisory/current | Formal flow facts bridge into constraints while preserving advisory layer distinction |
| Incrementality | LSP engine has candidate state, generation, affected closure, and reuse | Shared semantic invalidation keyed by exports/contracts/callables; clean/incremental equivalence |
| Caching | Multiple current caches/products exist | Layered cache metrics, revision-safe reuse, query cycle state, coherent publication |
| Parallelism | Some worker-oriented infrastructure exists | Deterministic parallel module work with safe publication and serial equivalence |
| LSP integration | Semantic engine and snapshots exist | LSP queries one formal semantic service; manual rebuild/restart/output validation passes |
| Performance | No Phalcom Pyrefly-equivalent baseline yet | Corpus, metrics, regression thresholds, and published before/after measurements |

## Final implementation principle

Build Phalcom's checker as a reusable semantic knowledge system. Make identities cheap, products staged, answers explicit, constraints bounded, dynamic boundaries honest, and snapshots coherent. Optimize only after each boundary exposes metrics and invalidation reasons. This gives Phalcom Pyrefly's practical efficiency without importing Python's type philosophy or compromising Phalcom's runtime semantics.
