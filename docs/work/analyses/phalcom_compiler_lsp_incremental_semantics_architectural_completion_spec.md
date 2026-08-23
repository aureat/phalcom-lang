# Phalcom Compiler/LSP/Incremental Semantics — Architectural Completion Implementation Specification

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to execute this specification task-by-task. Use `superpowers:test-driven-development` for every implementation slice, `superpowers:systematic-debugging` for failures, and `superpowers:verification-before-completion` before claiming any task complete.

**Goal:** Complete the remaining compiler/LSP/module integration so Phalcom has one type universe per semantic workspace epoch, one compiler-owned incremental formal query graph, one canonical module-resolution authority, source-accurate and revision-correct diagnostics, canonical module completion/navigation, constructor-`Self` semantics, and deterministic cold-vs-incremental parity.

**Repository authority:** implementation source of truth is `aureat/phalcom-lang`, `main` at commit `6518231e9cc6f8849a67f862874bf9bef6f746dd` (`docs: update implementation plan with completed LSP and semantic tasks`). If `main` moves before execution, re-run the archaeology/verification gate in §2 before touching code.

**Governing architecture:** `phalcom-modules` remains the sole implementation authority for project identity, logical module identity, manifests, dependency roots, source location rules, import resolution, package exposure, interfaces, exports and linking. `phalcom-semantic::SemanticWorkspaceSession` owns the lifetime/revision of those canonical module products and the formal semantic products that consume them. `phalcom-lsp` schedules source updates and serves immutable snapshot queries; it MUST NOT resolve modules or rebuild formal semantic meaning independently. The advisory LSP engine may remain as recovery/editor evidence, but it is never allowed to replace formal `Unknown`, `Dynamic`, `Invalid`, `Blocked`, `Cancelled`, `BudgetExceeded`, or `InternalFailure` states with a more optimistic language-truth result.

**Tech stack:** Rust workspace, `phalcom-ast`, `phalcom-modules`, `phalcom-semantic`, `phalcom-core`, `phalcom-lsp`, `phalcom-native-*`, `tower-lsp`, Tokio, immutable `Arc` publication, VS Code extension host, `examples/ide-golden`.

---

# 1. Non-negotiable acceptance invariants

Implementation is complete only when all of the following are true.

1. **Exactly one type universe exists per semantic workspace epoch.**
   - `SemanticWorkspaceSession` is the sole owner of the mutable canonical `TypeStore`.
   - No API can fabricate a second `TypeStore` with the same `TypeStoreId`.
   - Every `TypeId` in a published semantic product is meaningful in the `TypeStore` frozen into the same snapshot.

2. **Formal incrementality is dependency-correct, not merely source-text-cached.**
   - Reuse occurs only when the query input fingerprint is unchanged **and** every observed semantic dependency still has the same product fingerprint.
   - An unchanged caller recomputes when a consumed callable signature, declaration surface, hierarchy edge, linked interface, type alias/declaration semantic, or relevant native signature changes.
   - An unrelated callable remains structurally reusable.

3. **Query input identity and product identity are distinct.**
   - The DB MUST NOT overload one fingerprint field for both “inputs are unchanged” and “this product’s semantic result is unchanged.”
   - Dependency edges observe **product** fingerprints.
   - Cache lookup compares **input** fingerprints and dependency product fingerprints.

4. **The semantic query graph is a DAG, not a hard-coded linear pipeline.**
   - The common path is `ParsedModule -> UnlinkedInterface -> LinkedInterface -> DeclarationSurface -> CallableSignature -> CallableBody`, but body queries may also consume hierarchy, imports, fields, aliases, generic declarations, native surfaces and protocol facts.
   - Every semantic product actually read during a query must be represented as a dependency edge.

5. **`phalcom-modules` is the only module semantics implementation.**
   - `SemanticWorkspaceSession` owns the lifetime of a `ProjectUniverse` and invokes `phalcom-modules` APIs.
   - `phalcom-semantic` MUST NOT reproduce import/exposure/link algorithms.
   - `phalcom-lsp` MUST NOT decide module meaning from URI/path strings.

6. **The LSP does not build a fresh formal workspace on each edit.**
   - Production LSP edit refresh contains no fresh `ProjectUniverse::new()`, `ModuleResolver::new()`, or `ModuleLinker::new()`.
   - `run_static_workspace_analysis(...)` is deleted from the production path.
   - The LSP sends source/workspace events into one persistent `SemanticWorkspaceSession` and publishes the returned immutable snapshot.

7. **Module completion is a pure immutable query.**
   - Completion performs zero filesystem reads, zero directory scans, zero project loads, zero resolver/linker construction, zero semantic DB revision mutations, and zero query recomputation.
   - Every completion candidate must correspond to a canonical module/export product that resolves successfully through canonical module semantics.

8. **Module failure is explicit.**
   - Unresolved import, exposure rejection, missing export, invalid relative root, and link failure become stable canonical module diagnostics.
   - No production analysis branch silently `continue`s and turns module failure into downstream `Unknown` without an owning diagnostic.

9. **Incremental and cold analysis are observationally equivalent.**
   - For identical source content, incremental and fresh-cold analysis must agree on diagnostics, declaration/callable signatures, formal binding types, expression types, callable result state and module-resolution results.
   - Raw `TypeId`s are not compared across independent stores; canonical type presentations/exports are.

10. **Semantic-error revisions publish. Infrastructure-failure revisions do not replace good state.**
    - A type-error edit produces a new current snapshot containing the diagnostic.
    - Cancellation, budget exhaustion, stale generation, or internal analysis failure preserves the last successfully published formal snapshot.

11. **Constructors are receiver-relative.**
    - A public `@constructor` callable returns semantic `Self`, not the nominal declaring class.
    - `Derived.new(...)` inherited from `Base` returns `Derived`.
    - An ordinary inherited method explicitly returning `Base` remains `Base`.
    - No nominal-return-equality heuristic may impersonate `Self`.

12. **Formal editor identity is semantic identity.**
    - Bindings are projected by `BindingId`/formal site identity, not by “first binding,” name string, or loose range heuristics.
    - Module occurrences carry canonical `phalcom_modules::ModuleId`.
    - Imported declarations navigate to declaration origin.

13. **Observed historical bugs are release-gate regressions.**
    - `Point.new(...) -> Point`.
    - `Parcel.new(...) -> Parcel`.
    - `Planner.plan(...) -> Shipment`.
    - `CellNum` cannot silently satisfy an `Int` annotation.
    - Explicit annotations do not receive duplicate inferred type inlays.
    - Branch/return diagnostics update correctly without restarting the LSP.
    - `from geo.|`, `from units.|`, relative imports and selective export completion work in `examples/ide-golden`.
    - Unresolved module diagnostics appear and clear incrementally.

---

# 2. Re-grounding gate before implementation

Before editing code, run:

```bash
git status --short
git rev-parse HEAD
git log -n 10 --oneline
```

Expected starting commit for this spec:

```text
6518231e9cc6f8849a67f862874bf9bef6f746dd
```

If HEAD differs, inspect at minimum:

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/state.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/dependency.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/presentation.rs
phalcom-modules/src/identity.rs
phalcom-modules/src/project.rs
phalcom-modules/src/source.rs
phalcom-modules/src/query.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/linker.rs
phalcom-lsp/src/analysis_service.rs
phalcom-lsp/src/import_completion.rs
phalcom-lsp/src/semantic/module_graph.rs
phalcom-lsp/src/semantic/occurrence.rs
phalcom-lsp/src/semantic/snapshot.rs
phalcom-lsp/tests/module_navigation.rs
phalcom-core/src/primitive/system.rs
phalcom-core/tests/invariants.rs
examples/ide-golden/
```

Re-run:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo test -p phalcom-modules
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp
cargo test -p phalcom-core --test invariants
```

Do not “adapt” architecture merely because implementation names moved. Preserve the invariants in §1.

---

# 3. Verified current-state defects at the baseline commit

The implementation agent must understand the exact defects being removed.

## 3.1 Duplicate type stores with one identity

Current `SemanticDb` owns:

```rust
store: Arc<TypeStore>
```

while `SemanticWorkspaceSession::with_workspace()` currently does:

```rust
let db = SemanticDb::with_workspace(workspace);
let mut store = TypeStore::with_id(db.store().id());
```

`TypeStore::with_id()` creates a new store and then overwrites its identity. This creates two independent interning arenas that claim the same `TypeStoreId`.

This is forbidden after Task 1.

## 3.2 `QueryState::Ready` conflates input and product fingerprints

Current shape:

```rust
Ready {
    revision: SemanticRevision,
    fingerprint: ProductFingerprint,
    value: QueryValue,
}
```

Current callable query uses the body/input fingerprint as this `fingerprint`, while dependency edges conceptually need the fingerprint of the resulting semantic product. This prevents correct generic cache validation.

Task 2 replaces this model.

## 3.3 Callable DB publication has zero dependency edges

Current `query_callable_body()` ends successful analysis with:

```rust
db.publish_product_ready(
    key,
    rev,
    input_fingerprint,
    SemanticProduct::CallableBody(arc_analysis.clone()),
    Vec::new(),
)
```

Thus reverse invalidation cannot know that an unchanged caller depends on a changed callee/interface/hierarchy fact.

Task 3 removes this.

## 3.4 The persistent session still rebuilds most formal workspace state globally

`SemanticWorkspaceSession::update_with_budget_and_cancel()` currently clones/rebuilds:
- declarations;
- declaration shell table;
- hierarchy;
- semantic graph enrichment;
- generic signatures;
- supertype templates;
- source dispatch;
- callable signature table;
- diagnostics pass.

Only body query reuse is meaningfully selective.

Tasks 4–7 turn these into staged products.

## 3.5 The LSP still reconstructs formal module infrastructure

`phalcom-lsp/src/analysis_service.rs` still calls `refresh_static_workspace_analysis(...)`; its static path reconstructs project roots, `ProjectUniverse`, interfaces, `ModuleResolver`, resolved imports, `ModuleLinker`, then calls `identity.session.update(...)`.

Task 8 deletes that lifecycle.

## 3.6 Import completion is currently backed by empty products

`import_completions()` currently creates:

```rust
let dummy_universe = phalcom_modules::ProjectUniverse::new();
let unlinked = BTreeMap::new();
let linked = BTreeMap::new();
let resolved_imports = BTreeMap::new();
let sources = BTreeMap::new();
```

and builds `ModuleQueryFacade` from those empty maps. Its unit tests only validate syntax-context detection.

Task 10 replaces this with real snapshot products and real integration tests.

## 3.7 Constructor publication returns nominal class, not `Self`

`register_class_surface()` currently special-cases constructors with:

```rust
let class_ty = ctx.nominal_type_of(&decl_id);
(DispatchSide::Class, TypeKnowledge::known(class_ty, EvidenceAuthority::Declared))
```

and `SurfaceDispatchResolver::resolve_dispatch_on_owner()` rewrites an inherited signature returning the defining nominal form to the starting subclass form.

That heuristic is unsound for ordinary methods explicitly returning their declaring class.

Task 12 replaces it with semantic `Self`.

## 3.8 Formal binding lookup is source-name/range heuristic

The earlier “first binding” bug was improved, but current LSP presentation still searches `BindingState` by name and a permissive source-range predicate. It does not query a canonical source site -> `BindingId` mapping.

Task 13 fixes this.

## 3.9 Module occurrence/navigation identity remains partially legacy

The LSP still retains a URI/string-oriented local `ModuleId` layer and current module navigation tests can stabilize navigation to a local import binding rather than the exported declaration origin.

Task 14 canonicalizes navigation.

---

# 4. Target ownership and data-flow model

After implementation, the high-level ownership graph MUST be:

```text
LSP document/workspace events
        |
        v
SemanticWorkspaceSession
  |-- one ProjectUniverse lifetime
  |-- one overlay-capable SourceProvider lifetime
  |-- one SemanticDb lifetime
  |-- one mutable TypeStore lifetime
  |-- one semantic revision sequence
  |
  +--> phalcom-modules algorithms
  |      project discovery
  |      import roots
  |      source location
  |      InterfaceBuilder
  |      ModuleResolver
  |      ModuleLinker
  |
  +--> SemanticDb query DAG
         ParsedModule
         UnlinkedInterface
         LinkedInterface
         DeclarationShell / DeclarationSurface
         HierarchyEdge
         CallableSignature
         CallableBody
         ModuleDiagnostics
         presentation/source provenance
        |
        v
immutable SemanticSnapshot
        |
        +--> compiler consumers
        +--> LSP formal presentation
        +--> ModuleQueryFacade view
```

Forbidden alternative:

```text
LSP
  -> fresh ProjectUniverse
  -> fresh resolver/linker
  -> fresh linked workspace
  -> session
```

Forbidden alternative:

```text
LSP URI graph decides what import means
compiler resolver separately decides what import means
```

Forbidden alternative:

```text
SemanticDb.store #1
SemanticWorkspaceSession.store #2
same TypeStoreId
```

---

# 5. Exact core data model

## 5.1 Fingerprint types

Modify `phalcom-semantic/src/db/key.rs`.

Retain:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductFingerprint(pub u64);
```

Add:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputFingerprint(pub u64);

impl InputFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
```

Re-export both from `phalcom-semantic/src/db/mod.rs`:

```rust
pub use key::{InputFingerprint, ProductFingerprint, QueryKey};
```

`ProductFingerprint` means only:

> semantic identity of a successfully produced query result.

`InputFingerprint` means only:

> direct non-query inputs to one query invocation.

Never use one as the other.

## 5.2 Query keys

Modify `phalcom-semantic/src/db/key.rs`.

Current keys are preserved. Add exactly:

```rust
ResolvedImports(ModuleId),
CallableSignature(CallableId),
HierarchyEdge(DeclarationId),
```

Do NOT add an alternate callable-signature cache elsewhere.

`HierarchyEdge(DeclarationId)` represents the declaration's direct nominal superclass relationship and all information necessary to answer one direct hierarchy lookup. Transitive hierarchy queries consume multiple `HierarchyEdge` products and therefore record multiple edges.

## 5.3 Query state

Modify `phalcom-semantic/src/db/state.rs`.

Replace:

```rust
Ready {
    revision: SemanticRevision,
    fingerprint: ProductFingerprint,
    value: QueryValue,
}
```

with:

```rust
Ready {
    revision: SemanticRevision,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    value: QueryValue,
}
```

Add exact accessors:

```rust
pub fn input_fingerprint(&self) -> Option<InputFingerprint>
pub fn product_fingerprint(&self) -> Option<ProductFingerprint>
```

Delete/rename the old ambiguous `fingerprint()` accessor. Do not retain an accessor whose name fails to distinguish input from product.

## 5.4 Query products

Modify `phalcom-semantic/src/db/product.rs`.

Required enum:

```rust
pub enum SemanticProduct {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<UnlinkedModuleInterface>),
    LinkedInterface(Arc<LinkedModuleInterface>),
    ResolvedImports(Arc<ResolvedImportsProduct>),
    SemanticComponent(Arc<LinkedProgram>),
    DeclarationSurface(Arc<DeclarationSurface>),
    HierarchyEdge(Arc<HierarchyEdgeProduct>),
    CallableSignature(Arc<CallableSemanticSignature>),
    CallableBody(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
}
```

Define module-resolution product types in a new file:

```text
phalcom-semantic/src/module_product.rs
```

```rust
#[derive(Clone, Debug)]
pub struct ResolvedImportsProduct {
    pub module: ModuleId,
    pub targets: BTreeMap<String, ModuleId>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}
```

`targets` contains only successful canonical resolutions keyed by exact `ImportPath::to_string()`. `diagnostics` contains source-owned failures for unresolved/exposure-invalid paths; one bad import does not erase successful resolutions for other imports in the same module.

`SemanticProduct::SemanticComponent` uses the existing `QueryKey::SemanticComponent(entry_module)` and stores the one `LinkedProgram` produced for that active reachable component. The linker is invoked once per component recomputation, never once per `LinkedInterface`. Individual `LinkedInterface(M)` products are projections published from the linked component so downstream queries can depend on a stable per-module product fingerprint.

Define `HierarchyEdgeProduct` in a new file:

```text
phalcom-semantic/src/hierarchy_product.rs
```

with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyEdgeProduct {
    pub declaration: DeclarationId,
    pub superclass: Option<DeclarationId>,
}
```

Re-export `HierarchyEdgeProduct`, `ResolvedImportsProduct`, and the new fingerprint/query vocabulary from their owning modules through `phalcom-semantic/src/lib.rs` only when they are intended for cross-crate consumers.

For every new `SemanticProduct` variant add the matching `as_*` accessor and a unique `to_query_value()` discriminator; do not leave variants that can be published but not retrieved through typed accessors.

Do not store another mutable `MapTypeHierarchy` in the DB. The published snapshot may contain an immutable materialized `MapTypeHierarchy` derived from all current `HierarchyEdgeProduct`s for compatibility/performance.

## 5.5 Semantic dependency vocabulary

Modify `phalcom-semantic/src/checker/analysis.rs`.

Do not make the checker itself depend directly on DB cache mechanics. Add a semantic-level dependency enum:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticDependency {
    CallableSignature(CallableId),
    DeclarationSurface(DeclarationId),
    HierarchyEdge(DeclarationId),
    LinkedInterface(ModuleId),
}
```

Preserve existing callable call-graph compatibility:

```rust
pub dependencies: Arc<[CallableId]>,
```

and add:

```rust
pub semantic_dependencies: Arc<[SemanticDependency]>,
```

`dependencies` is the resolved-call graph.
`semantic_dependencies` is query-invalidating semantic consumption.

The query layer maps `SemanticDependency` to `QueryKey`.

---

# 6. Task 1 — Make `SemanticWorkspaceSession` the only TypeStore owner

**Files**
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify: `phalcom-semantic/src/db/mod.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify/Test: `phalcom-semantic/tests/type_store_revisions.rs`

## 6.1 Delete duplicate-store construction

Delete:

```rust
TypeStore::with_id(...)
```

from `store.rs`.

Search:

```bash
rg "with_id\(" phalcom-semantic
```

Every use must disappear.

## 6.2 Remove TypeStore ownership from SemanticDb

Remove:

```rust
store: Arc<TypeStore>
```

from `SemanticDb`.

Remove:

```rust
pub fn store(&self) -> &Arc<TypeStore>
```

from `SemanticDb`.

`SemanticDb` owns:
- workspace identity;
- semantic revision;
- query states;
- typed products;
- last-known-good products;
- dependency index;
- scheduler;
- metrics.

It does not own type interning.

`SemanticWorkspaceSession` owns:

```rust
store: TypeStore
```

and creates it exactly once with:

```rust
let store = TypeStore::new();
```

per `SemanticWorkspaceSession::with_workspace(...)`.

Remove `Clone` from both `SemanticWorkspaceSession` and `SemanticDb`. These are mutable lifecycle authorities and must not be duplicated by ordinary cloning. Immutable `Arc<SemanticSnapshot>` and typed query products remain freely cloneable.

## 6.3 Snapshot freezing discipline

Snapshots currently hold:

```rust
pub store: Arc<TypeStore>
```

Keep this.

At successful publication, freeze with:

```rust
Arc::new(self.store.clone())
```

provided `TypeStore` remains append-only for already-interned IDs. The mutable session may intern new values later, but it MUST NEVER reorder/remove/reassign existing `TypeId`, `KindId`, `TypeParameterId`, row IDs or lambda IDs.

Add internal debug assertions if any future compaction API exists.

## 6.4 Required tests

Add:

```rust
#[test]
fn one_session_has_one_type_store_identity_across_revisions()
```

Assert:
- one session `TypeStoreId` remains stable through at least 100 revisions;
- every published snapshot's `store.id()` equals `session.store().id()`.

Add:

```rust
#[test]
fn retained_old_snapshot_preserves_type_denotation_after_later_revisions()
```

Algorithm:
1. Publish revision 1 containing `Point`.
2. Retain `Arc<SemanticSnapshot>` revision 1.
3. Capture `Point`'s `TypeId` and cloned `TypeData`.
4. Perform revisions 2 and 3 that intern unrelated types.
5. Assert revision-1 snapshot still maps the old `TypeId` to the exact cloned `TypeData`.
6. Assert all snapshots share the same `TypeStoreId`.

## 6.5 Snapshot identity must use the real workspace and semantic revision

Current `SemanticSnapshot::new*` constructs `SnapshotId` with `WorkspaceId::from_raw(1)`. Delete that hard-coded identity.

Change snapshot constructors to accept:

```rust
workspace: WorkspaceId,
revision: SemanticRevision,
generation: u64,
```

and construct:

```rust
let id = SnapshotId::new(workspace, revision, store.id());
```

`SemanticWorkspaceSession` always passes `self.workspace` and `self.db.revision()` when freezing a snapshot. The LSP generation remains a separate publication generation and MUST NOT substitute for `SemanticRevision`.

Add a test with two independent `SemanticWorkspaceSession::with_workspace(...)` values asserting their snapshot workspace IDs remain distinct even if both publish generation `1`.

Add a compile-time/deletion gate:

```bash
rg "TypeStore::with_id|db\.store\(\)|WorkspaceId::from_raw\(1\)" phalcom-semantic/src
```

Expected: zero `TypeStore::with_id`; zero semantic-session logic relying on `db.store()`; zero hard-coded workspace identity in snapshot construction.

Commit after Task 1.

---

# 7. Task 2 — Separate input/product fingerprints and make cache validation generic

**Files**
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/state.rs`
- Modify: `phalcom-semantic/src/db/mod.rs`
- Modify: `phalcom-semantic/src/db/dependency.rs`
- Test: `phalcom-semantic/tests/semantic_db_incremental.rs` (create if absent)

## 7.1 SemanticDb publication API

Replace the ambiguous publication API with:

```rust
pub fn publish_product_ready(
    &mut self,
    key: QueryKey,
    revision: SemanticRevision,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    product: SemanticProduct,
    dependencies: impl IntoIterator<Item = DependencyEdge>,
) -> Result<(), PublishError>
```

Likewise change `publish_ready(...)`.

## 7.2 Generic reuse validation

Add to `SemanticDb`:

```rust
pub fn is_reusable(
    &self,
    key: &QueryKey,
    input_fingerprint: InputFingerprint,
) -> bool
```

Algorithm, exactly:

1. Fetch `QueryState`.
2. Require `Ready`.
3. Require stored `input_fingerprint == requested input_fingerprint`.
4. Fetch every `DependencyEdge` from `DependencyIndex::dependencies_of(key)`.
5. For each dependency:
   - its current state must be `Ready`;
   - current `product_fingerprint` must equal `edge.observed_fingerprint`.
6. Only then return `true`.

Do not infer reuse from revision equality.
Do not infer reuse only from input fingerprint.
Do not silently ignore missing dependency products.

Add:

```rust
pub fn ready_product_fingerprint(
    &self,
    key: &QueryKey,
) -> Option<ProductFingerprint>
```

## 7.3 DependencyRecorder helper

Add:

```rust
impl SemanticDb {
    pub fn record_dependency(
        &self,
        recorder: &mut DependencyRecorder,
        dependency: QueryKey,
    ) -> Result<(), String>
}
```

It must:
- obtain the dependency's current ready product fingerprint;
- return an error if it is not `Ready`;
- call `recorder.record(...)`.

Do not publish a dependent product with missing required dependencies.

## 7.4 Tests

Test:
- same input + same dependency products => reusable;
- same input + changed dependency product fingerprint => not reusable;
- changed input + same dependencies => not reusable;
- dependency cancelled/blocked/missing => not reusable;
- product fingerprint can remain stable across a newer revision and still be reused.

Commit after Task 2.

---

# 8. Task 3 — Record all formal body semantic dependencies

**Files**
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify/Test: `phalcom-semantic/tests/type_store_revisions.rs`
- Create/Test: `phalcom-semantic/tests/callable_dependency_invalidation.rs`

## 8.1 CheckingContext storage

Replace:

```rust
pub dependencies: Vec<CallableId>,
```

with:

```rust
pub dependencies: BTreeSet<CallableId>,
pub semantic_dependencies: BTreeSet<SemanticDependency>,
```

Convert to sorted boxed slices/`Arc` during `finalize()`.

## 8.2 Stop cloning the workspace dispatch table per callable

At baseline, `analyze_callable_body()` receives `&SurfaceDispatchResolver` and immediately clones it into `CheckingContext::new_with_dispatch(...)`. This is unacceptable once the workspace surface becomes large.

Change `CheckingContext` dispatch storage to:

```rust
pub enum DispatchAccess<'a> {
    Owned(SurfaceDispatchResolver),
    Borrowed(&'a SurfaceDispatchResolver),
}

impl<'a> DispatchAccess<'a> {
    pub fn get(&self) -> &SurfaceDispatchResolver;
    pub fn get_mut(&mut self) -> Option<&mut SurfaceDispatchResolver>;
}
```

`CheckingContext` field:

```rust
pub dispatch: DispatchAccess<'a>,
```

Constructors:

```rust
pub fn new(...) -> Self
```
uses `Owned` for declaration/surface-building compatibility.

```rust
pub fn new_with_dispatch_ref(
    store: &'a mut TypeStore,
    hierarchy: &'a dyn TypeHierarchy,
    resolver: &'a dyn TypeResolver,
    declarations: &'a DeclarationTypeTable,
    dispatch: &'a SurfaceDispatchResolver,
    current_module: ModuleId,
) -> Self
```
uses `Borrowed`.

`analyze_callable_body()` MUST call `new_with_dispatch_ref`; no workspace dispatch clone is permitted per callable.

`with_resolver()` must borrow `self.dispatch.get()` rather than clone dispatch.

`register_surface()` is valid only for `DispatchAccess::Owned`; return an explicit internal error/assertion if accidentally called on a borrowed body context.

All dispatch reads use `self.dispatch.get()`.

## 8.3 Dispatch trace model

Current `resolve_dispatch_on_owner()` loses information about which hierarchy edges were traversed. Add in `dispatch.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDispatch {
    pub callable: CallableId,
    pub signature: CallableSignature,
    pub visited_owners: Box<[DeclarationId]>,
}
```

Add:

```rust
pub enum ResolvedDispatchResult {
    Found(ResolvedDispatch),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing {
        visited_owners: Box<[DeclarationId]>,
    },
    Dynamic,
}
```

Add method:

```rust
pub fn resolve_dispatch_with_trace(
    &self,
    hierarchy: &dyn TypeHierarchy,
    start_decl: &DeclarationId,
    side: DispatchSide,
    selector: &Selector,
) -> ResolvedDispatchResult
```

`resolve_dispatch_on_owner()` becomes a compatibility wrapper that projects this trace result to the existing `DispatchResult`.

The trace algorithm:
1. Start at `start_decl`.
2. Push each declaration inspected into `visited_owners`.
3. On found callable, return its canonical `CallableId`, cloned signature and complete visited path.
4. On missing, return the visited path.

## 8.4 CheckingContext::resolve_dispatch recording

Change `CheckingContext::resolve_dispatch(...)` to use `resolve_dispatch_with_trace`.

For every owner in `visited_owners`, insert:

```rust
SemanticDependency::HierarchyEdge(owner.clone())
```

On `Found`, insert:
```rust
self.dependencies.insert(resolved.callable.clone());
self.semantic_dependencies.insert(
    SemanticDependency::CallableSignature(resolved.callable.clone())
);
```

Do not record a callee `CallableBody` dependency for ordinary dispatch. A caller's type checking depends on the callable signature, not the callee implementation body.

## 8.5 Declaration/field reads

Any body-checking path that obtains a field/member declaration type outside callable dispatch must record:

```rust
SemanticDependency::DeclarationSurface(owner_decl)
```

Perform a repository audit:

```bash
rg "get_field|get_surface|declarations\.|resolve_type_name|superclass\(" \
  phalcom-semantic/src/checker
```

For body-time semantic reads:
- route field/member declaration access through `CheckingContext` helper methods;
- helpers record the semantic dependency before returning the value.

Do NOT add dependency recording to pure local-flow reads.

## 8.6 Linked import/type-name reads

If a callable body resolves a non-local declaration through its module's linked import binding, record:

```rust
SemanticDependency::LinkedInterface(current_module.clone())
```

The body does not need a direct edge to every imported module if its own `LinkedInterface` product already depends on those imported interfaces. Depend on the nearest canonical product actually consumed.

## 8.7 Query mapping

In `db/query.rs`, map:

```text
SemanticDependency::CallableSignature(c)
    -> QueryKey::CallableSignature(c)

SemanticDependency::DeclarationSurface(d)
    -> QueryKey::DeclarationSurface(d)

SemanticDependency::HierarchyEdge(d)
    -> QueryKey::HierarchyEdge(d)

SemanticDependency::LinkedInterface(m)
    -> QueryKey::LinkedInterface(m)
```

Create `DependencyRecorder::new(QueryKey::CallableBody(callable))`, call `db.record_dependency(...)` for every semantic dependency, and publish the resulting edges.

## 8.8 Mandatory regression matrix

Create tests:

### A. unchanged caller + changed callee return signature

```phalcom
class Api {
  @class value() -> Int { 1 }
}

class Consumer {
  @class read() {
    Api.value()
  }
}
```

Revision 2 changes only `Api.value() -> String`.
Expected:
- `Consumer.read` source body unchanged;
- its old `CallableAnalysis` `Arc` is not reused;
- new formal result observes `String`.

### B. callee body change, signature unchanged

Change `Api.value` body `1` -> `2` while `-> Int` stays.
Expected:
- `Api.value` body product recomputes;
- `Api.value` signature product fingerprint unchanged;
- `Consumer.read` body `Arc` reused.

### C. field annotation change

```phalcom
class Data {
  _value: Int = 1
  read() { _value }
}
```

Change `_value: Int` -> `_value: String` with compatible initializer.
Expected `read` recomputes even though body is unchanged.

### D. superclass change

Change:

```phalcom
class Child is A {}
```

to:

```phalcom
class Child is B {}
```

An unchanged body invoking an inherited member on `Child` must recompute.

### E. imported linked surface change

Change an exported imported callable/type signature in dependency module.
Expected importing unchanged body recomputes.

### F. unrelated edit

Edit a completely unrelated callable.
Expected unaffected `CallableAnalysis` `Arc::ptr_eq(old, new)`.

Commit after Task 3.

---

# 9. Task 4 — Add first-class staged query functions

**Files**
- Create: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: `phalcom-semantic/src/db/mod.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/db/key.rs`
- Modify: `phalcom-semantic/src/db/product.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic_db_incremental.rs`

## 9.1 Fingerprint rules

Do not use `Debug` formatting as the long-term semantic fingerprint source.

Implement stable hash functions in `db/fingerprint.rs`.

### `parsed_module_input_fingerprint`

Hash:
- `ModuleId`;
- `ModuleKind`;
- exact source text bytes.

Any source edit changes this.

### `unlinked_interface_product_fingerprint`

Hash only semantic interface fields:
- module identity;
- kind;
- import surfaces (path, alias/selective/re-export items and relevant metadata);
- exports;
- package exposed children;
- module metadata affecting linking/visibility.

Do NOT hash callable body AST, statement source positions unrelated to interface semantics, or whitespace.

This is what allows body-only edits to keep interface product fingerprints stable.

### `linked_interface_product_fingerprint`

Hash:
- module identity/kind;
- canonical resolved linked exports;
- metadata;
- canonical linked import/read bindings needed for semantic name resolution.

Use canonical IDs, never URIs.

### `declaration_surface_product_fingerprint`

Hash:
- `DeclarationId`;
- declaration kind/kind signature;
- generic signature;
- direct superclass template;
- field signatures: side/name/type;
- callable signatures: side/selector/generic binders/parameter labels/rest/type/return type;
- semantic attributes that affect type checking.

Do NOT hash callable bodies.

### `callable_signature_product_fingerprint`

Hash canonical `CallableSemanticSignature` fields:
- owner;
- side;
- selector;
- generics;
- parameter external label/local position/rest/type term;
- return type term;
- effects/raises/flow/lifecycle if these participate in checking.

### `callable_body_input_fingerprint`

Hash:
- `CallableId`;
- exact body AST semantic structure;
- body source range only if range participates in identity/diagnostics;
- own `CallableSignature` is **not** folded into this direct input fingerprint; it is a semantic dependency edge.

### `callable_body_product_fingerprint`

Hash semantic output, not counts only:
- callable ID;
- analysis status;
- every expression ID + range + formal knowledge/status/call-resolution identity;
- every binding ID + declared/current formal knowledge + source site;
- dependency identities;
- diagnostics code/severity/range;
- control/exit facts that downstream queries consume.

Do not hash pointer addresses.

## 9.2 Required query functions

Implement in `db/query.rs` or split into focused `db/queries/*.rs` files if `query.rs` becomes unwieldy.

Required public/internal functions:

```rust
query_parsed_module(...)
query_unlinked_interface(...)
query_resolved_imports(...)
query_semantic_component(...)
query_linked_interface(...)
query_hierarchy_edge(...)
query_declaration_surface(...)
query_callable_signature(...)
query_callable_body(...)
query_module_diagnostics(...)
```

Every query follows one template:

1. Construct `QueryKey`.
2. Compute `InputFingerprint` from direct inputs only.
3. `if db.is_reusable(&key, input_fp)` return current typed product.
4. Mark miss/recompute.
5. Construct `DependencyRecorder`.
6. Read canonical prerequisite query products.
7. Record every prerequisite with `db.record_dependency`.
8. Compute semantic product.
9. Compute `ProductFingerprint`.
10. Publish typed product with input fp, product fp and recorded edges.
11. Return `QueryOutcome::Ready`.

Blocked/cancelled/budget/failed states must not overwrite `last_known_good_product`.

### 9.2.1 Resolved-import query algorithm

`query_resolved_imports(module, ...)`:
1. obtains ready `UnlinkedInterface(module)`;
2. creates one canonical `ModuleResolver` over the session-owned `ProjectUniverse` and overlay source provider for the revision/component computation;
3. for each import/re-export surface, calls `resolve_import_with_trace`;
4. records every `trace.package_interfaces` module as an `UnlinkedInterface` dependency;
5. inserts successful `(path_string, target ModuleId)` mappings;
6. converts each resolver failure into a source-owned module diagnostic attached to that exact import range;
7. returns a **Ready partial product** even when some imports fail, so diagnostics/completion can explain the failure; unresolved imports do not become silent absence.

The product fingerprint includes both successful target mappings and stable diagnostic `(code, range, semantic target/path)` content.

### 9.2.2 Link once per semantic component, then project linked interfaces

Do NOT implement `query_linked_interface(M)` by constructing a new `ModuleLinker` for every module.

`query_semantic_component(entry)` is the only query in this stage that invokes `ModuleLinker::link(...)`.

Algorithm:
1. determine the reachable interface/import-resolution closure from the active entry/open-document component;
2. obtain ready `UnlinkedInterface` and `ResolvedImports` products for that closure;
3. if a required import is unresolved, return `Blocked(SuppressedDependency)` **after** module diagnostics have been published; do not invent a linked program missing the dependency;
4. build one `BTreeMap<ModuleId, UnlinkedModuleInterface>` from the ready products;
5. build one combined successful resolved-import map;
6. instantiate `ModuleLinker` once;
7. call `link(entry, &resolved)` once;
8. publish `SemanticProduct::SemanticComponent(Arc<LinkedProgram>)`;
9. for every `LinkedModule` in the resulting program, publish/update `QueryKey::LinkedInterface(module)` using that module's `LinkedModule.interface.clone()` and its own stable product fingerprint.

`query_linked_interface(M)` is therefore a lookup/projection query: it returns the ready per-module product, triggering/scheduling the owning semantic component only when the product is absent/stale. It never invokes the linker independently.

## 9.3 Exact primary dependency topology

At minimum:

```text
UnlinkedInterface(M)
  depends on ParsedModule(M)

ResolvedImports(M)
  depends on UnlinkedInterface(M)
  depends on canonical project/import-root revision input
  records every package interface consumed while validating external exposure

SemanticComponent(E)
  depends on every reachable UnlinkedInterface(M)
  depends on every reachable ResolvedImports(M)
  invokes ModuleLinker exactly once for the component rooted at E

LinkedInterface(M)
  is published as a projection of the ready SemanticComponent containing M
  depends on that SemanticComponent product
  retains its own product fingerprint so unchanged module interfaces do not invalidate downstream consumers

DeclarationSurface(D)
  direct input fingerprint = declaration/member signature syntax for D only
  depends on LinkedInterface(owner_module) when annotations resolve imported names
  records referenced declaration surfaces dynamically where annotation/generic bounds consume them

HierarchyEdge(D)
  direct input fingerprint = D's superclass syntax plus declaration identity
  depends on LinkedInterface(owner_module) when superclass name is imported
  depends on the referenced superclass declaration product after successful resolution
  MUST NOT depend on CallableSignature(D), preventing a declaration/signature cycle

CallableSignature(C)
  direct input fingerprint = source/native callable signature syntax/metadata for C
  depends on DeclarationSurface(owner)
  depends on LinkedInterface(owner_module) for referenced annotation names
  depends on referenced declaration surfaces as required by annotation resolution

CallableBody(C)
  depends on CallableSignature(C)
  depends dynamically on semantic dependencies recorded by CheckingContext

ModuleDiagnostics(M)
  depends on module/interface/link products and semantic products whose diagnostics it aggregates
```

Do not introduce cycles such as:
`DeclarationSurface -> CallableSignature -> DeclarationSurface`.
The declaration-surface product may contain source-level member surface facts, while the separate callable-signature product is the canonical per-callable projection used by body queries.

## 9.4 Required staged-product test

After one normal session commit, assert `Ready` typed products exist for:
- `ParsedModule`;
- `UnlinkedInterface`;
- `ResolvedImports`;
- `SemanticComponent`;
- `LinkedInterface`;
- one `HierarchyEdge`;
- one `DeclarationSurface`;
- one `CallableSignature`;
- one `CallableBody`;
- `ModuleDiagnostics`.

This test must inspect `SemanticDb::product()` and query state, not merely final snapshot contents.

Commit after Task 4.

---

# 10. Task 5 — Fine-grained invalidation laws

**Files**
- Modify: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic_db_incremental.rs`
- Test: `phalcom-semantic/tests/type_store_revisions.rs`

## 10.1 Stop seeding all module products from whole-source fingerprint

Current session invalidates `ParsedModule`, `UnlinkedInterface`, `LinkedInterface`, diagnostics together whenever the module source hash changes.

Replace this policy.

A changed source first invalidates:

```rust
QueryKey::ParsedModule(module)
```

only.

Reverse dependency closure then invalidates downstream products **only where product dependencies demand it**.

When reparsing yields an `UnlinkedInterface` product with an unchanged product fingerprint, its dependents must remain reusable. Do not eagerly seed them merely because source text changed.

## 10.2 Product-stability propagation rule

A recomputed query whose **product fingerprint is unchanged** must not cause downstream invalidation solely because its revision is newer.

This requires either:
- lazy `is_reusable()` dependency-fingerprint validation; or
- invalidation that is only propagated when product fingerprint changes.

Use the generic `is_reusable()` mechanism from Task 2 as the correctness backstop.

## 10.3 Structural edit tests

### Body-only edit

Change only an implementation expression.

Expect:
- `ParsedModule(M)` recomputed;
- affected `CallableBody` recomputed;
- `UnlinkedInterface(M)` may recompute from new parse but product fingerprint is unchanged;
- `LinkedInterface(M)`, declaration surfaces, hierarchy and unrelated callables remain reusable;
- no project reload/resolution.

### Signature edit

Change `foo() -> Int` to `foo() -> String`.

Expect:
- parsed/unlinked relevant product changed;
- declaration surface and callable signature changed;
- exact caller reverse closure invalidated;
- unrelated declaration/callable products reused.

### Import/export edit

Change one import or `expose`.

Expect:
- affected unlinked/linked interface changes;
- reverse importer closure changes;
- downstream declaration/callable products depending on changed linked interface recompute;
- unrelated project/module products reused.

### Superclass edit

Expect one `HierarchyEdge` product change and exact semantic dependents invalidated.

Commit after Task 5.

---

# 11. Task 6 — Make hierarchy and declaration materializations snapshot projections, not independent authorities

**Files**
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/types/relation.rs` only if helper required
- Test: `phalcom-semantic/tests/semantic_db_incremental.rs`

The final snapshot may retain:

```rust
pub declarations: Arc<DeclarationTypeTable>,
pub hierarchy: Arc<MapTypeHierarchy>,
pub dispatch: Arc<SurfaceDispatchResolver>,
pub callable_signatures: Arc<CallableSignatureTable>,
```

for efficient immutable reads.

But these must be materialized from current ready DB products at publication time.

They MUST NOT be separately rebuilt as authoritative mutable state every revision.

Publication algorithm:
1. Enumerate current declaration-surface products.
2. Materialize declaration table.
3. Enumerate `HierarchyEdgeProduct`s and insert direct edges into a new immutable `MapTypeHierarchy`.
4. Enumerate declaration surfaces and materialize `SurfaceDispatchResolver`.
5. Enumerate callable-signature products and materialize `CallableSignatureTable`.
6. Freeze all into the snapshot.

`base_declarations`, `base_hierarchy`, `base_dispatch`, `base_callable_signatures` may remain immutable bootstrapped core seeds only if they are treated as immutable session inputs; source-state authority comes from query products.

Add a test that a body-only edit keeps the `Arc` identities of unaffected materialized components where the implementation supports structural sharing. At minimum verify fingerprints/recomputation counts.

Commit after Task 6.

---

# 12. Task 7 — Compiler-owned project/module lifecycle

**Files**
- Modify: `phalcom-modules/src/source.rs`
- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/query.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Create: `phalcom-semantic/src/workspace_inputs.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Test: `phalcom-semantic/tests/workspace_session_modules.rs` (new)
- Create/Test: `phalcom-modules/tests/source_overlay.rs`
- Create/Test: `phalcom-modules/tests/query_facade.rs`

## 12.1 Authority rule

Do not move module algorithms into `phalcom-semantic`.

`SemanticWorkspaceSession` owns:
- the lifetime of one `ProjectUniverse`;
- source overlays;
- resolver generation;
- semantic revisions;
- DB products.

It calls:
- `ProjectUniverse`;
- `InterfaceBuilder`;
- `ModuleResolver`;
- `ModuleLinker`.

Those implementations remain in `phalcom-modules`.

After this task, the long-lived state of `SemanticWorkspaceSession` is modeled as:

```rust
pub struct SemanticWorkspaceSession {
    workspace: WorkspaceId,
    db: SemanticDb,
    store: TypeStore,

    universe: Arc<ProjectUniverse>,
    source_provider: OverlaySourceProvider<FilesystemSourceProvider>,
    workspace_roots: BTreeSet<PathBuf>,
    document_revisions: BTreeMap<ModuleId, u64>,

    base_declarations: DeclarationTypeTable,
    base_hierarchy: MapTypeHierarchy,
    base_dispatch: SurfaceDispatchResolver,
    base_callable_signatures: CallableSignatureTable,

    last_snapshot: Option<Arc<SemanticSnapshot>>,
    last_published_snapshot: Option<Arc<SemanticSnapshot>>,
}
```

`last_published_snapshot` is the publication/LKG concept. Do not use the name `last_known_good` to mean “last semantically error-free program”; a snapshot containing ordinary semantic diagnostics is still a successfully published snapshot.

`ProjectUniverse` must become cheaply snapshot-shareable. Make `ProjectUniverse` and its stateless `SyntheticProjectIdAllocator` `Clone`, keep the session field as `Arc<ProjectUniverse>`, and mutate project graph state only through `Arc::make_mut(&mut self.universe)` when workspace roots/manifests actually change. Ordinary source edits reuse the same `Arc`. Published snapshots retain the old `Arc` if a later manifest change creates a new universe version.

### 12.1.1 Trace module-resolution semantic reads

`ResolvedImports(M)` must know which package exposure interfaces were consumed during canonical resolution. Add to `phalcom-modules/src/resolver.rs`:

```rust
#[derive(Clone, Debug)]
pub struct ImportResolutionTrace {
    pub target: SourceUnit,
    pub package_interfaces: BTreeSet<ModuleId>,
}

pub fn resolve_import_with_trace(
    &mut self,
    importer: &ModuleId,
    syntax: &ImportPath,
) -> Result<ImportResolutionTrace, ModuleResolutionError>
```

Keep current:

```rust
pub fn resolve_import(...) -> Result<SourceUnit, ModuleResolutionError>
```

as a compatibility projection over `resolve_import_with_trace(...).map(|trace| trace.target)`.

During external hierarchical exposure validation, every package module whose interface is loaded/consulted is inserted into `package_interfaces`. Relative/self imports usually have an empty exposure trace because current resolver semantics do not call `validate_external_path` for them.

`query_resolved_imports(M)` maps each traced package module to `QueryKey::UnlinkedInterface(package_module)` dependency edges. Therefore editing a dependency package's `expose` list invalidates import resolution and completion without depending on filesystem timestamps or LSP scanning heuristics.

## 12.2 Overlay-capable canonical SourceProvider

Add to `phalcom-modules/src/source.rs`:

```rust
#[derive(Clone, Debug)]
pub struct SourceOverlay {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub text: Arc<str>,
}
```

Add:

```rust
pub struct OverlaySourceProvider<P> {
    base: P,
    overlays_by_module: RwLock<BTreeMap<ModuleId, SourceOverlay>>,
    overlays_by_source: RwLock<BTreeMap<SourceId, ModuleId>>,
}
```

Implement `SourceProvider`:
- `locate(project, path)` constructs canonical candidate `ModuleId { project: project.id.into(), path: path.clone() }`;
- if overlay exists for that module, return its `SourceUnit`;
- otherwise delegate to `base.locate(...)`;
- `read(source_id)` checks overlay map first, otherwise delegates to base.

Required methods:

```rust
pub fn new(base: P) -> Self
pub fn set_overlay(&self, overlay: SourceOverlay)
pub fn remove_overlay(&self, module: &ModuleId)
pub fn clear_overlays(&self)
```

Do not put module-resolution logic in the overlay provider.

Use `std::sync::RwLock` for overlay maps. Query paths into immutable snapshots never access this mutable provider.

### 12.2.1 Canonical reverse source-path resolution

The LSP MUST NOT derive `ModuleId` by parsing a URI. Add to `phalcom-modules/src/source.rs`:

```rust
pub fn resolve_source_path(
    project: &ResolvedProject,
    path: &Path,
) -> Result<SourceUnit, ModuleResolutionError>
```

This is the physical-path-to-logical-module inverse of `FilesystemSourceProvider::locate`. Algorithm, exactly:

1. Canonicalize `path`.
2. Require it to be inside `project.source_root`.
3. Strip `project.source_root` and inspect the relative path.
4. If basename is `package.ph`, `ModuleKind::Package`; logical components are the relative parent directories.
5. Otherwise require `.ph`; `ModuleKind::Module`; logical components are relative parent directories plus the file stem.
6. Convert every physical component with `ModuleComponent::from_kebab`; reject non-canonical snake/mixed-case physical names exactly as forward source location does.
7. Reject a nested `project.toml` boundary between the owning project root and target source.
8. Build `ModuleId { project: project.id.into(), path: ModulePath::from_components(...) }`.
9. Build `SourceLocation` from the canonical file path and canonical `SourceId`.
10. Return `SourceUnit`.

Add round-trip tests:

```text
source.locate(project, logical_path) -> SourceUnit U
resolve_source_path(project, U.source.display_path) -> same U.id + U.kind
```

This function belongs in `phalcom-modules` because physical/logical module spelling is module-system authority.

## 12.3 Session source/workspace input API

Create `workspace_inputs.rs`:

```rust
#[derive(Clone, Debug)]
pub struct WorkspaceRootInput {
    pub root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SourceOverlayUpdate {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub revision: u64,
    pub text: Arc<str>,
}

#[derive(Clone, Debug)]
pub enum SourceChange {
    Update(SourceOverlayUpdate),
    Remove(ModuleId),
}

#[derive(Debug)]
pub enum WorkspaceSessionError {
    Project(phalcom_modules::ProjectError),
    Resolution(phalcom_modules::ModuleResolutionError),
    Load(phalcom_modules::ModuleLoadError),
    Source(phalcom_modules::SourceError),
    UnknownDocument(PathBuf),
}
```

Provide `From` implementations for the wrapped canonical module/source errors; do not stringify them prematurely.

Add session APIs exactly:

```rust
pub fn set_workspace_roots(
    &mut self,
    roots: impl IntoIterator<Item = WorkspaceRootInput>,
) -> Result<(), WorkspaceSessionError>;

pub fn resolve_document_path(
    &mut self,
    display_path: &Path,
) -> Result<phalcom_modules::ResolvedDocumentIdentity, WorkspaceSessionError>;

pub fn apply_document_change(
    &mut self,
    display_path: PathBuf,
    revision: u64,
    text: Arc<str>,
) -> Result<phalcom_modules::ResolvedDocumentIdentity, WorkspaceSessionError>;

pub fn apply_source_change(
    &mut self,
    change: SourceChange,
) -> Result<(), WorkspaceSessionError>;

pub fn commit_revision(
    &mut self,
    generation: u64,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> Result<SemanticWorkspaceUpdate, QueryOutcome<()>>;
```

Retain `update(SemanticWorkspaceInput)` only as a compatibility/cold wrapper, implemented by feeding the same session primitives. It must not remain a second semantic algorithm.

`resolve_document_path(...)` algorithm:
1. use canonical `phalcom_modules::discover_owning_project` for the physical path;
2. load/lookup that project through the session-owned `ProjectUniverse`;
3. call `phalcom_modules::source::resolve_source_path`;
4. return `ResolvedDocumentIdentity { source, module, generation }`.

`apply_document_change(...)` calls `resolve_document_path`, creates/updates the overlay for the returned module/source, records the document revision, and queues `ParsedModule(module)` invalidation for the next commit. The LSP should normally call this API, not construct a `ModuleId` itself.

## 12.4 Project universe persistence

`set_workspace_roots`:
1. canonicalizes/deduplicates roots;
2. discovers/loads manifest-backed projects once;
3. retains `ProjectUniverse` in the session;
4. starts/updates resolver generation only when roots/manifests/dependency graph change.

Ordinary source-body edits MUST NOT create a new `ProjectUniverse`.

## 12.5 Canonical module products in snapshot

Define in `phalcom-semantic/src/snapshot.rs`:

```rust
#[derive(Clone, Debug)]
pub struct ModuleQueryProducts {
    pub universe: Arc<ProjectUniverse>,
    pub unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
    pub linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
    pub resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
    pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
}
```

Add to `SemanticSnapshot`:

```rust
pub module_products: Arc<ModuleQueryProducts>,
```

Add:

```rust
pub fn module_queries(&self) -> ModuleQueryFacade<'_> {
    ModuleQueryFacade::new(
        &self.module_products.universe,
        &self.module_products.unlinked,
        &self.module_products.linked,
        &self.module_products.resolved_imports,
        &self.module_products.sources,
    )
}
```

IMPORTANT: this method is implemented in `phalcom-semantic`, not `phalcom-modules`. `phalcom-modules` must not depend on semantic snapshots.

### 12.5.1 Preserve self-vs-external import-root semantics

Current `ResolvedProject::import_roots()` stores `(ImportRootTarget, bool)` where the boolean is `is_self`; current `ModuleQueryFacade::import_roots()` discards that boolean. Completion cannot match `ModuleResolver` exposure rules if this information is lost.

Add in `phalcom-modules/src/query.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportRootQueryTarget {
    pub target: ImportRootTarget,
    pub is_self: bool,
}

pub fn import_root_entries(
    &self,
    importer: &ModuleId,
) -> BTreeMap<ModuleComponent, ImportRootQueryTarget>
```

Builtin roots have `is_self = false`. The current project's namespace has `is_self = true`. Dependency aliases have `is_self = false`.

Keep `import_roots()` only as a compatibility projection if existing callers need it; new completion/navigation code MUST use `import_root_entries()`.

### 12.5.2 Separate child enumeration from exposure filtering

The current `import_children_in_project()` applies parent `exposed_children` filtering unconditionally. That is not a correct generic primitive:
- relative imports are project-internal and `ModuleResolver` does not apply external exposure validation;
- the project's own absolute namespace root has `is_self = true` and does not apply external exposure validation;
- `expose .child` must be able to suggest a child that is not already exposed, otherwise the user cannot add new exposure;
- cross-project dependency imports **do** require hierarchical exposure validation.

Replace/augment the facade with these exact pure queries:

```rust
pub fn module_children(
    &self,
    project: ProjectIdentity,
    prefix: &ModulePath,
) -> Vec<ModuleId>;

pub fn external_import_children(
    &self,
    target_project: ProjectIdentity,
    prefix: &ModulePath,
) -> Vec<ModuleId>;
```

`module_children` returns all direct canonical children known in linked/unlinked products, with no exposure filtering.

`external_import_children` MUST implement the same hierarchical exposure law as `ModuleResolver::validate_external_path`:
1. validate every component of `prefix` from the root package through its parent package `exposed_children` sets;
2. if any prefix component is not exposed, return empty;
3. enumerate direct children of `prefix`;
4. include a candidate child only if the package represented by `prefix` exposes it.

Do not use filesystem scanning in either query.

## 12.6 Resolved import product

For each successfully resolved import path, publish/retain the canonical mapping:

```text
(importer ModuleId, exact ImportPath::to_string())
    -> target ModuleId
```

Use this map for completion, navigation and reverse importer queries.

Do not reconstruct import meaning from URI spelling.

Commit after Task 7.

---

# 13. Task 8 — Delete LSP formal workspace reconstruction

**Files**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/semantic/snapshot.rs`
- Modify: any tests using `StaticWorkspaceIdentity`
- Delete production helpers only when no longer referenced

## 13.1 `StaticWorkspaceIdentity`

Replace any identity structure that owns/rebuilds independent formal project state with a structure whose meaningful long-lived formal owner is:

```rust
session: phalcom_semantic::SemanticWorkspaceSession
```

The LSP may retain:
- URI -> document metadata;
- source revision;
- open/closed status;
- advisory engine state;
- status/log counters.

It must not retain a second formal module lifecycle.

## 13.2 Delete `run_static_workspace_analysis`

Delete the function and all production call sites.

Delete production constructions from `phalcom-lsp/src`:

```text
ProjectUniverse::new
ModuleResolver::new
ModuleLinker::new
```

Exceptions are allowed only in isolated tests explicitly testing `phalcom-modules`; not in production edit/refresh code.

## 13.3 LSP update flow

For every open/change/remove event:

1. Resolve/document-bind canonical module identity using the compiler session/module products.
2. Build `SourceChange`.
3. `session.apply_source_change(change)`.
4. Coalesce edits as current worker already does.
5. Call `session.commit_revision(generation, budget, cancel)`.
6. If `Ready`, attach returned compiler snapshot to the immutable LSP snapshot.
7. If cancelled/stale/budget/internal failure, do not clear the previous formal snapshot.
8. Publish status/log terminal event.

The advisory engine may run in parallel, but formal type display never substitutes advisory facts for a non-ready formal state.

## 13.4 Last-known-good semantics

Do NOT implement:

```text
source has type error
    -> keep prior clean snapshot
```

A semantic-error snapshot is a successful analysis and must publish.

Only:
- cancelled;
- budget-exhausted;
- stale generation;
- infrastructure failure

preserve the prior publication.

## 13.5 Deletion gate

Before completion:

```bash
rg "run_static_workspace_analysis" phalcom-lsp/src
rg "ProjectUniverse::new" phalcom-lsp/src
rg "ModuleResolver::new" phalcom-lsp/src
rg "ModuleLinker::new" phalcom-lsp/src
```

Expected for production source: zero relevant matches.

Commit after Task 8.

---

# 14. Task 9 — Canonical module diagnostics

**Files**
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Modify: `phalcom-semantic/src/db/query.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-lsp/src/diagnostics.rs` or current formal diagnostic adapter
- Test: `phalcom-semantic/tests/module_diagnostics.rs` (new)
- Test: `phalcom-lsp/tests/module_diagnostics.rs` (new)

## 14.1 Stable diagnostic codes

Add/retain stable codes whose rendered string forms are exactly:

```text
module.import.unresolved
module.exposure.rejected
module.export.missing
module.relative.invalid_root
module.link.failed
```

Map canonical `phalcom-modules` errors to these codes. Do not emit generic `Unknown` in place of the root module failure.

## 14.2 Source ownership

Every module diagnostic has:
- importer/source module;
- precise import/export/expose source range when available;
- canonical target/path in message;
- related information only if canonical source location exists.

No convenience constructor may default a user module failure to `ModuleId::core()`.

## 14.3 No silent continue

Audit:

```bash
rg "continue;|else \{ continue|let Ok\(.*\) = .* else" \
  phalcom-lsp/src phalcom-semantic/src
```

Every module resolver/linker failure in production semantic analysis must:
- produce a typed failure/diagnostic product; or
- return an explicit infrastructure failure.

It may not silently disappear.

## 14.4 LSP behavior

Unresolved import diagnostics come from compiler snapshot products.
Fixing the import in a later revision removes the diagnostic without restarting LSP.

Commit after Task 9.

---

# 15. Task 10 — Real import completion over immutable canonical module products

**Files**
- Modify: `phalcom-lsp/src/import_completion.rs`
- Modify: `phalcom-lsp/src/completion.rs`
- Modify: `phalcom-modules/src/identity.rs`
- Modify: `phalcom-modules/src/query.rs` only for pure canonical path/query helpers
- Test: `phalcom-lsp/tests/module_completion.rs` (new)
- Test: `phalcom-modules` path helper tests

## 15.1 Delete dummy products

Delete from `import_completion.rs` all creation of:
- dummy `ProjectUniverse`;
- empty unlinked map;
- empty linked map;
- empty resolved import map;
- empty source map.

Use:

```rust
let Some(static_snapshot) = snapshot.static_snapshot.as_ref() else {
    return Vec::new();
};

let facade = static_snapshot.module_queries();
```

## 15.2 Replace import-context data model with structural roots

The current `ImportContext::SelectiveExport { root: String, ... }` cannot represent a relative selective import such as:

```phalcom
from .domain.parcel import |
from ..domain import |
```

Replace the context model in `phalcom-lsp/src/import_completion.rs` with exactly:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportPathRootContext {
    Absolute(String),
    Relative { dots: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModulePathCompletionKind {
    Import,
    Expose,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportContext {
    ImportRoot {
        partial: String,
    },
    ModulePath {
        kind: ModulePathCompletionKind,
        root: ImportPathRootContext,
        segments: Vec<String>,
        partial: String,
    },
    SelectiveExport {
        root: ImportPathRootContext,
        segments: Vec<String>,
        partial: String,
    },
}
```

Parsing rules:
- `import ge|` -> `ImportRoot { partial: "ge" }`;
- `import geo.po|` -> `ModulePath { Import, Absolute("geo"), segments=[], partial="po" }`;
- `import .dom|` -> `ModulePath { Import, Relative { dots: 1 }, segments=[], partial="dom" }`;
- `import ..domain.mo|` -> `ModulePath { Import, Relative { dots: 2 }, segments=["domain"], partial="mo" }`;
- `expose .dom|` -> `ModulePath { Expose, Relative { dots: 1 }, ... }`;
- `from geo.point import Po|` -> `SelectiveExport { Absolute("geo"), segments=["point"], partial="Po" }`;
- `from .domain.parcel import Pa|` -> `SelectiveExport { Relative { dots: 1 }, segments=["domain", "parcel"], partial="Pa" }`.

Do not discard leading dots during `split('.')`.

## 15.3 Relative-path semantics live in canonical module queries

Copy the **existing `ModuleResolver::resolve_import` relative semantics exactly** into a pure read-only `ModuleQueryFacade` helper. Do not invent a second interpretation.

Add to `phalcom-modules/src/query.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelativeQueryError {
    ImporterInterfaceMissing(ModuleId),
    BeyondRoot { dots: usize, depth: usize },
}

impl<'a> ModuleQueryFacade<'a> {
    pub fn resolve_relative_prefix(
        &self,
        importer: &ModuleId,
        dots: usize,
        suffix: &[ModuleComponent],
    ) -> Result<ModulePath, RelativeQueryError>;
}
```

Algorithm MUST match `ModuleResolver::resolve_import` at baseline commit:

1. Require `dots >= 1`.
2. Read importer kind from canonical unlinked interface for `importer`.
3. If importer is a package, base package path = `importer.path.clone()`.
4. If importer is a module, base package path = `importer.path.parent().unwrap_or_else(ModulePath::root)`.
5. `ascend_count = dots - 1`.
6. If `ascend_count > base_package.components().len()`, return `BeyondRoot`.
7. Keep `base_len = depth - ascend_count` components.
8. Append canonicalized `suffix` components.
9. Return resulting `ModulePath`.

Do not add a different `ancestor(dots)` interpretation that would make one leading dot ascend one package; in Phalcom's existing resolver, **one dot means current package and `dots - 1` is the ascent count**.

## 15.4 Completion algorithms

### Import root

For:

```phalcom
import ge|
```

call:

```rust
facade.import_root_entries(importer)
```

Filter by `starts_with(partial)`.

For `from ge|` before a complete path/import clause, use the same absolute root candidates if the parser-recovery context classifies it as a root completion.

### Absolute child

For:

```phalcom
from geo.|
```

1. canonicalize `geo` with `ModuleComponent::from_identifier`;
2. look up `ImportRootTarget` in `facade.import_root_entries(importer)`;
3. map it to `ProjectIdentity`;
4. canonicalize complete `segments` into `ModuleComponent`s;
5. build prefix `ModulePath`;
6. if the root entry has `is_self = true`, call `facade.module_children(project, &prefix)`; otherwise call `facade.external_import_children(project, &prefix)`;
7. filter the final component by `partial`.

### Relative child

For:

```phalcom
from ..domain.|
```

1. canonicalize complete suffix segments (`domain` here);
2. call `facade.resolve_relative_prefix(importer, 2, &suffix)`;
3. call `facade.module_children(importer.project, &prefix)`;
4. filter by `partial`.

For `expose`, use the same relative prefix arithmetic but enumerate with `facade.module_children(...)`, not exposure-filtered children: the purpose of `expose` is to add a child to the exposure set, so already-unexposed children must still be offered. Do not route through receiver/member completion.

### Absolute selective export

For:

```phalcom
from geo.point import |
```

1. resolve root `geo` through `facade.import_root_entries(importer)`;
2. create canonical target project;
3. create target `ModulePath` from all complete path segments (`point`);
4. construct canonical target `ModuleId`;
5. require target to exist in canonical linked products;
6. call `facade.public_exports(&target)`;
7. emit only linked public exports.

### Relative selective export

For:

```phalcom
from .domain.parcel import |
```

1. call `facade.resolve_relative_prefix(importer, 1, &[domain, parcel])`;
2. construct target `ModuleId { project: importer.project, path }`;
3. call `facade.public_exports(&target)`.

Do not infer completion item kind by capitalization. Use `LinkedExportTarget` metadata.

## 15.5 Exposure rules

Completion MUST respect package `expose`.

Golden expected examples:

```text
from geo.|
    point
    route
    NOT internal/private

from units.|
    distance
    weight
    NOT internal/private

from .domain.|
    parcel
    shipment
    status
```

## 15.6 Completion resolvability invariant

In integration tests, for every emitted module candidate:
- reconstruct the canonical intended path through module query APIs;
- assert it is present/resolvable in canonical products.

No LSP-only candidate.

## 15.7 Read-only/no-I/O invariant

Instrument test-only counters around:
- filesystem provider reads/locates;
- project loads;
- resolver constructions if observable;
- linker constructions;
- SemanticDb revision/recompute counters.

Issue completion request after snapshot is ready.

Assert delta:

```text
filesystem reads       0
filesystem locates     0
project loads          0
resolver/linker builds 0
db revision changes    0
query recomputations   0
```

Commit after Task 10.

---

# 16. Task 11 — Project-aware progressive readiness

**Files**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify: workspace/status tests
- Test: `phalcom-lsp/tests/project_readiness.rs` (new)

Manifest/project discovery must precede broad deep semantic work.

Startup order for a manifest-backed document:

```text
1. identify workspace/project root
2. load project manifest and dependency roots
3. resolve canonical ModuleId for open document
4. resolve/open document dependency closure
5. publish module/interface readiness
6. analyze formal declarations/signatures needed by open document
7. publish usable formal snapshot
8. continue broad background workspace scan
```

Do not wait for a full unrelated workspace scan before module completion becomes available.

Required test:
- initialize LSP on `examples/ide-golden`;
- open `src/main.ph`;
- before full background scan completion, poll/request completion on `from geo.|`;
- require `point`/`route` to become available once the open-document dependency closure is ready.

Status must terminate in `Ready` or an explicit degraded/error state; never remain stuck in `Publishing`.

Commit after Task 11.

---

# 17. Task 12 — Replace constructor nominal return with semantic `Self`

**Files**
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/types/substitution.rs`
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-lsp/src/semantic/snapshot.rs`
- Test: `phalcom-semantic/tests/workspace.rs`
- Test: constructor-specific semantic tests

## 17.1 Constructor publication

In `register_class_surface()`:
- constructor effective side is `DispatchSide::Class`;
- constructor return is a `TypeData::SelfType(SelfTypeTerm)` with owner = declaring class and instance-type role.

Do not use nominal `class_ty`.

Keep source constructor body analysis instance-side. The public factory semantic identity and initializer-body semantic identity must not be conflated.

## 17.2 Distinct semantic roles — mirror the existing compiler lowering exactly

The baseline compiler already defines the canonical lowering in `phalcom-core/src/compiler/attributes.rs::lower_constructors`: each source constructor is lowered to:

1. a class-side factory with the original source name/parameters, which sends `self._$new`, calls the generated initializer, then returns `instance`;
2. an instance-side initializer whose generated name is exactly `format!("init {}", constructor_name)`.

Formal semantic identity MUST mirror this existing compiler contract. For source:

```phalcom
@constructor
new(_ x: Int, y: Int) { ... }
```

model:

```text
public factory:
    CallableId(
        owner = C,
        side = Class,
        selector = new(_, y:)
    )
    return = Self

internal initializer body:
    CallableId(
        owner = C,
        side = Instance,
        selector = init new(_, y:)
    )
    implementation = original source constructor body
```

The generated `init ...` name is compiler-owned/unspellable source protocol and is not offered as ordinary user completion.

Call graph/dependency edges from `C.new(...)` callers target the public class-side factory identity. Body diagnostics/explanations for the constructor source body are owned by the internal initializer identity but retain source provenance pointing to the original constructor declaration/body range.

Do **not** seed the initializer body's expected return from the public factory's `Self` result. The actual compiler factory ignores the initializer call result and separately returns the allocated instance. Public constructor result semantics and initializer-body result semantics are distinct.

This task does not independently change parser legality for explicit constructor return annotations; regardless of temporary source syntax, such an annotation may never change the public factory result away from `Self`.

## 17.3 Delete nominal return heuristic

Delete from `SurfaceDispatchResolver::resolve_dispatch_on_owner()` the logic:

```text
if inherited method return == defining nominal form
    replace with starting nominal form
```

No ordinary explicit return type is rewritten merely because it equals the defining class.

## 17.4 `Self` specialization

Add in `types/substitution.rs`:

```rust
pub fn specialize_self_type(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    receiver: TypeId,
    ty: TypeId,
) -> TypeId
```

Rules:
- `SelfType(ReceiverValue)` -> receiver value type;
- `SelfType(InstanceType)` and receiver is `ClassObject { declaration }` -> declaration instance form;
- instance nominal/applied receiver -> receiver;
- recursively specialize nested `Applied`, `Union`, `Tuple`, `Record`, `Callable`;
- leave unrelated nominal/parameter/lambda/unit/never forms unchanged.

Apply after generic substitution in `CheckingContext::resolve_dispatch`.

## 17.5 Required tests

```phalcom
class Base {
  @constructor
  new() {}
}
class Derived is Base {}
```

Assert:
- `Base.new()` -> `Base`;
- `Derived.new()` -> `Derived`.

Counterexample:

```phalcom
class Base {
  @class
  ordinary() -> Base { ... }
}
class Derived is Base {}
```

Assert `Derived.ordinary()` -> `Base`.

## 17.6 Presentation

Current LSP formal callable presentation treats `SelfType` as `Unknown`. Remove that behavior.

Generic declaration hover may render `Self`.
Receiver-specialized call-site presentation should render the resolved concrete receiver result.

Constructor signature help for `Point.new(...)` must not display `Unknown` return merely because canonical declaration uses `Self`.

Commit after Task 12.

---

# 18. Task 13 — Canonical formal binding/source-site identity

**Files**
- Modify: `phalcom-semantic/src/presentation.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs` if source site metadata insufficient
- Modify: `phalcom-lsp/src/semantic/snapshot.rs`
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Test: semantic presentation tests
- Test: LSP inlay tests

## 18.1 Extend FormalSiteId

Current `FormalSiteId::Expression(ExpressionId)` is not globally safe because `ExpressionId` is documented as stable only within a callable-body analysis product and current body contexts can allocate the same local IDs independently. Replace the enum, not merely extend it:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FormalSiteId {
    Expression {
        callable: CallableId,
        expression: ExpressionId,
    },
    Binding {
        callable: CallableId,
        binding: BindingId,
    },
    Callable(CallableId),
}
```

Do not key expression/binding identity by local numeric ID alone, name, or source text. Callable identity is part of the global formal-site identity.

## 18.2 Formal binding site

`FormalTypeSite.range` for a binding MUST be the individual binding identifier token range, never the whole declaration statement. Extend pattern/binding lowering so every `Pattern::Name` contributes its own exact token range to the resulting `BindingState`.

If `BindingState.range` currently stores too broad a range, add an exact `declaration_range` field to binding state at bind allocation time.

## 18.3 Presentation index

`SemanticPresentationIndex::insert_callable(...)` must insert:
- callable site;
- expression sites using `FormalSiteId::Expression { callable: analysis.callable.clone(), expression: expression.id }`;
- every binding site using `FormalSiteId::Binding { callable: analysis.callable.clone(), binding: state.binding }`.

Model source indexes explicitly:

```rust
pub struct SemanticPresentationIndex {
    sites: BTreeMap<FormalSiteId, FormalTypeSite>,
    binding_sites: BTreeMap<(ModuleId, SourceRange), FormalSiteId>,
    expression_sites: BTreeMap<ModuleId, Vec<FormalTypeSite>>,
}
```

Binding lookup requires exact identifier range. Expression lookup by offset selects the smallest containing expression range (most specific AST expression); ties are ordered deterministically by `FormalSiteId`. Do not return the first hash-map iteration match.

This is a projection only; it owns no independent invalidation state.

## 18.4 LSP lookup

Replace formal binding presentation logic that iterates binding states by name/range.

Algorithm:
1. URI -> canonical module.
2. source offset -> exact formal binding site from presentation index.
3. `FormalSiteId::Binding { callable, binding }`.
4. `CallableAnalysis.bindings[&binding]`.
5. present `state.current`.

No fallback to:
- first binding;
- name-only;
- `state.range.start <= offset`.

## 18.5 Tests

- two sibling bindings of different types;
- nested same-name shadowing;
- parameter shadowed by local;
- destructuring/tuple pattern producing multiple binding IDs;
- explicit annotation suppresses inlay;
- removing explicit annotation restores formal inferred inlay without restart.

Commit after Task 13.

---

# 19. Task 14 — Canonical module occurrences and navigation

**Files**
- Modify: `phalcom-lsp/src/semantic/occurrence.rs`
- Modify: occurrence builder/index code
- Modify: `phalcom-lsp/src/backend.rs` definition/reference handling
- Modify: `phalcom-lsp/src/semantic/module_graph.rs`
- Modify/Test: `phalcom-lsp/tests/module_navigation.rs`

## 19.1 Identity

`SemanticTarget::Module` must carry canonical:

```rust
phalcom_modules::ModuleId
```

not a string/path-derived LSP module ID.

Imported declaration targets must carry canonical declaration/export target identity.

## 19.2 Import path component occurrences

For:

```phalcom
from geo.point import Point
```

publish semantically resolved occurrences:

```text
geo
  -> canonical import-root/project target

point
  -> canonical ModuleId for geo.point

Point
  -> canonical exported declaration target
```

Do not create `ModuleId::new(seg.name)` string identities.

## 19.3 Definition behavior

Go to Definition:
- on `point` path component -> module source/package source for `geo.point`;
- on imported `Point` -> actual declaration source in `geo.point`;
- on later usage `Point.new(...)` -> actual declaration source, not the local import binding;
- on builtin declarations -> canonical physical or virtual `phalcom://` source provenance.

Find References may include import binding sites, but definition origin remains canonical declaration origin.

Correct any existing integration test that asserts an imported declaration resolves to the importer file.

## 19.4 LSP module graph

Remove module meaning from `ModuleGraph::update(...)`.

The LSP graph becomes an index projection populated from canonical resolved import edges from the compiler snapshot.

No URI/path guess resolver remains in production.

Commit after Task 14.

---

# 20. Task 15 — System primitive/source/runtime alignment and example cleanup

This task is not the architectural core, but it removes currently committed inconsistencies.

**Files**
- Modify: `phalcom-core/src/primitive/system.rs`
- Modify: `phalcom-core/core/universe/src/concurrency/fiber.ph`
- Modify: `phalcom-core/tests/invariants.rs`
- Modify: `examples/sheetcalc/src/main.ph`
- Modify: `packages/immer/project.toml`
- Modify: `packages/immer/src/package.ph`

## 20.1 System.print

Ratified contract:

```text
System.print(Object) -> Unit
```

Native descriptor:
```rust
returns = Unit
types = "(Object) -> Unit"
```

Runtime:
```rust
Ok(vm.unit_value())
```

Invariant:
```rust
assert!(print_res.is_unit());
```

No `None` compatibility alternative.

Audit all System source/native/runtime triples while touching this file. Resolve every discovered mismatch deliberately.

## 20.2 SheetCalc

Remove duplicate local declarations introduced by prior example edits.

Normal runnable example code must not intentionally contain:

```phalcom
const n: Int = Value.CellNum.of(...)
```

unless it is in a dedicated negative test fixture asserting a mismatch.

## 20.3 Immer

`packages/immer/project.toml` must be a valid real project manifest, modeled after current repository package manifests.
`src/package.ph` must not retain undefined exploratory references.

Commit after Task 15.

---

# 21. Task 16 — Exact no-restart incremental LSP regressions

**Files**
- Create: `phalcom-lsp/tests/incremental_formal_equivalence.rs`
- Create `phalcom-lsp/tests/support/mod.rs` containing the JSON-RPC framing, response-wait and revision-wait helpers currently duplicated in integration tests; use that shared support from this new test and migrate only the directly touched tests.

This task directly reproduces historical user-visible failures.

## 21.1 TokenSample stale diagnostic sequence

Run one LSP server process. Never restart it.

Initial:

```phalcom
class TokenSample {
  _value: Int = 1

  read(_ fallback: Int) -> Int {
    const current = _value

    if current > 0 {
      current
    } else {
      fallback
    }
  }
}
```

Revision sequence:

1. Baseline: expect no return mismatch; `current` formally `Int`.
2. Add `: Int` to `current`: expect no return mismatch and no duplicate inlay.
3. Remove annotation: expect no return mismatch and formal `Int` inferred hint again.
4. Replace else tail `fallback` with a `String`: expect formal return mismatch.
5. Restore `fallback`: mismatch disappears.
6. Change field `_value: Int` to another incompatible/compatible type as needed: unchanged body invalidates appropriately.

After every `didChange`, wait for matching document revision/generation publication and assert diagnostics/inlays belong to that revision.

## 21.2 Cold comparison

For each revision's exact text:
- separately run a fresh cold semantic session;
- compare canonical diagnostic code/range and formal type presentations.

Invariant:

```text
incremental(revision N) == cold(revision N)
```

## 21.3 CellNum/Int mismatch

Fixture:

```phalcom
const wrong: Int = Value.CellNum.of(5)
```

Expected:
- compiler/formal initializer mismatch;
- LSP publishes it;
- advisory evidence cannot suppress it.

Change annotation to `Value.CellNum`.
Expected diagnostic clears without restart.

Commit after Task 16.

---

# 22. Task 17 — `examples/ide-golden` final acceptance runner

**Files**
- Add/extend LSP integration runner targeting `examples/ide-golden`
- Update golden fixture only where its intended semantics are wrong
- Add/extend the existing `tools/vsphalcom` automated tests so the packaged client preserves the LSP completion/diagnostic/navigation results and virtual-source provider behavior.

## 22.1 Formal type expectations

In `examples/ide-golden/src/main.ph`:

```text
origin       Point
destination  Point
parcel       Parcel
shipment     Shipment
```

Validate at four layers:
1. compiler `CallableAnalysis`;
2. compiler presentation index;
3. LSP inlay/hover;
4. packaged VS Code client/extension test result through the existing `tools/vsphalcom` test harness.

The LSP must not manufacture these types if compiler formal analysis is `Unknown`.

## 22.2 Constructor expectations

- `Point.new(0, y: 0)` -> `Point`;
- `Parcel.new(...)` -> `Parcel` even if legacy initializer source had `-> ()`; remove that stale annotation from golden source;
- inherited constructor `Self` test exists separately.

## 22.3 Class method

`Planner.plan(...)` -> `Shipment`.

## 22.4 Completion expectations

```text
from geo.|
  point
  route
  not internal/private

from units.|
  distance
  weight
  not internal/private

from .domain.|
  parcel
  shipment
  status

from geo.point import |
  public linked exports only
```

Test partial prefixes.

## 22.5 Navigation

- module path -> canonical module source;
- imported class -> declaration origin;
- builtin/core declaration -> physical/virtual canonical source.

## 22.6 Diagnostics

- unresolved module produces `module.import.unresolved`;
- fixing it clears diagnostic incrementally;
- `CellNum` assigned to `Int` produces formal mismatch;
- explicit annotations suppress inferred inlay.

## 22.7 Startup

Module completion for open-document dependency roots must become available before unrelated full workspace scanning is required.

Commit after Task 17.

---

# 23. Task 18 — Cold-vs-incremental equivalence harness

**Files**
- Create: `phalcom-semantic/tests/incremental_cold_equivalence.rs`
- Extend `phalcom-lsp/tests/incremental_formal_equivalence.rs`

For each edit class:
- body-only;
- callable signature;
- field annotation;
- superclass;
- import;
- export/expose;
- new module;
- removed module;
- type error introduction/fix.

Run:
1. persistent-session incremental revision;
2. fresh-cold compatibility analysis of exact resulting source/project state.

Compare semantic meaning, not store-local raw IDs.

Required comparison helpers:
- diagnostics sorted by `(module, code, range, severity, message)`;
- declaration surfaces exported to stable textual/canonical form;
- callable signatures exported to stable textual/canonical form;
- binding sites `(callable stable identity, binding source range, presented type/state)`;
- expression sites `(callable stable identity, expression range, presented type/state)`;
- resolved import mapping `(importer stable identity, path, target stable identity)`.

Any mismatch is a correctness failure.

---

# 24. Task 19 — Performance/structural acceptance

Performance optimizations are accepted only after correctness.

Required structural assertions:
- one `TypeStore` allocation per session epoch;
- body-only edit does not rebuild `ProjectUniverse`;
- body-only edit does not construct `ModuleResolver` or `ModuleLinker`;
- completion request performs zero semantic mutation/I/O;
- unrelated callable `Arc` reuse is demonstrated;
- exact reverse invalidation closure sizes are asserted in tests;
- no full core body analysis is required before ordinary workspace module completion.

Expose/update `SemanticUpdateStats` with at least:

```rust
pub struct SemanticUpdateStats {
    pub parsed_modules_recomputed: usize,
    pub unlinked_interfaces_recomputed: usize,
    pub linked_interfaces_recomputed: usize,
    pub declaration_surfaces_recomputed: usize,
    pub hierarchy_edges_recomputed: usize,
    pub callable_signatures_recomputed: usize,
    pub callables_recomputed: usize,
    pub callables_reused: usize,
}
```

Do not keep only `modules_recomputed`, which is too coarse to enforce the architecture.

---

# 25. Task 20 — Last-known-good publication laws

**Files**
- Extend semantic DB/session revision tests
- Extend LSP worker tests

Test independently:

### Semantic error

Revision 1 valid.
Revision 2 contains type mismatch.

Expected:
- revision 2 snapshot publishes;
- it becomes current;
- diagnostic visible.

### Fix

Revision 3 fixes mismatch.
Expected:
- new snapshot publishes;
- diagnostic disappears.

### Cancel

Start revision 4, cancel before completion.
Expected:
- no revision-4 formal snapshot replaces revision 3;
- last-known-good/current published formal snapshot remains revision 3.

### Budget

Budget-exhausted refresh similarly preserves prior publication.

### Stale generation

A superseded edit batch cannot publish over the newer generation.

Formal query states for cancelled/budgeted products remain explicit; advisory evidence does not overwrite them.

---

# 26. Task 21 — Plan/status correction and deletion gates

Only after all preceding acceptance gates pass, update:

```text
implementation_plan.md
docs/work/analyses/typing/2026-08-23-phalcom-compiler-lsp-ide-integration-incremental-semantics.md
```

Do not mark a task complete merely because an API exists.

Completion requires behavior and deletion gates.

Required repository searches:

```bash
rg "TypeStore::with_id" .
rg "run_static_workspace_analysis" phalcom-lsp/src
rg "ProjectUniverse::new" phalcom-lsp/src
rg "ModuleResolver::new" phalcom-lsp/src
rg "ModuleLinker::new" phalcom-lsp/src
rg "dummy_universe|let unlinked = BTreeMap::new\(\)|let linked = BTreeMap::new\(\)" phalcom-lsp/src/import_completion.rs
rg "state\.range\.start <= offset" phalcom-lsp/src
```

Expected: no production occurrences corresponding to deleted architecture.

---

# 27. Mandatory test command matrix

Run focused tests after every task. Before final completion run all:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings

cargo test -p phalcom-modules

RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test callable_dependency_invalidation -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test incremental_cold_equivalence -- --nocapture

RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test module_completion -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test module_navigation -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test module_diagnostics -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test project_readiness -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test incremental_formal_equivalence -- --nocapture

cargo test -p phalcom-core
cargo test -p phalcom-core --test invariants

cargo test --workspace

cd tools/vsphalcom
npm test
npm run compile
npm run package

cd ../..
git diff --check
git status --short
```

If some named test target is created under a different Cargo integration-test filename during implementation, use that exact created target; do not silently omit the coverage.

---

# 28. Required commit boundaries

Use small reviewable commits. Recommended boundaries:

```text
fix(semantic): unify workspace TypeStore ownership

refactor(semantic): separate query input and product fingerprints

feat(semantic): record dynamic formal query dependencies

feat(semantic): stage declaration signature and hierarchy products

feat(semantic): enforce fine-grained incremental invalidation

feat(semantic): own canonical module lifecycle in workspace session

refactor(lsp): remove formal workspace rebuild path

feat(semantic): publish canonical module diagnostics

feat(lsp): query canonical module completion products

feat(lsp): prioritize project dependency readiness

fix(semantic): model constructor returns with Self

fix(lsp): index formal bindings by semantic identity

feat(lsp): canonicalize module navigation identities

fix(core): align System primitive contracts

test(lsp): add no-restart formal equivalence regressions

test(ide): enforce ide-golden semantic acceptance

docs: record completed compiler lsp integration gates
```

Do not combine all architecture work into one unreviewable commit.

---

# 29. Explicit anti-patterns / forbidden implementations

The implementing agent MUST NOT:

1. create another incremental type database;
2. create another module resolver in LSP;
3. keep two `TypeStore`s with one `TypeStoreId`;
4. use revision number alone as cache validity;
5. use source-body hash alone as callable semantic validity;
6. publish callable body products with empty dependency edges;
7. encode all invalidation as “module changed -> rebuild module everything”;
8. use `Debug` formatting as the permanent canonical semantic product fingerprint strategy;
9. treat `SemanticProduct` enum variants as sufficient without actual DB query ownership;
10. copy `ProjectUniverse`/resolver/linker construction into a new LSP helper under a different name;
11. build import completion candidates by scanning directories on request;
12. parse URI strings to decide canonical module identity;
13. ignore `parent_dots` in relative completion;
14. silently ignore module resolver/linker errors;
15. turn compiler formal `Unknown` into a known advisory type;
16. use nominal declaring-class return rewriting as a substitute for `Self`;
17. look up bindings by “first match,” name-only, or loose range ordering;
18. navigate imported declarations only to the local import clause when canonical declaration origin is available;
19. preserve a stale clean snapshot when a new revision successfully analyzed and contains semantic errors;
20. mark Tasks 16–19 complete while production LSP still constructs fresh module lifecycle objects.

---

# 30. Final acceptance checklist

Do not report completion until every item is checked.

## Type universe

- [ ] One mutable `TypeStore` owner per `SemanticWorkspaceSession`.
- [ ] `TypeStore::with_id()` deleted.
- [ ] Old snapshots preserve old `TypeId` denotations.
- [ ] Store ID stable through ordinary revisions.

## Query engine

- [ ] Input/product fingerprints separated.
- [ ] Generic dependency-fingerprint reuse validation implemented.
- [ ] `CallableSignature` and `HierarchyEdge` query keys/products live.
- [ ] Body queries record all semantic dependencies they consume.
- [ ] Caller invalidates on callee signature change.
- [ ] Caller reuses on callee body-only change.
- [ ] Field/hierarchy/import changes invalidate unchanged consumers.
- [ ] Unrelated callables structurally reuse.

## Staged semantics

- [ ] Parsed, unlinked, linked, hierarchy, declaration, callable signature/body and diagnostics are actual DB products.
- [ ] Snapshot declaration/hierarchy/dispatch/signature maps are materializations of DB products.
- [ ] Body-only edit does not rebuild project/module/declaration meaning unnecessarily.

## Module lifecycle

- [ ] Session owns persistent `ProjectUniverse`.
- [ ] `phalcom-modules` remains the sole module algorithm authority.
- [ ] Overlay source provider works for unsaved/open document text.
- [ ] LSP does not construct project universe/resolver/linker in production formal path.
- [ ] Canonical source provenance is in snapshot module products.

## Completion

- [ ] Dummy facade removed.
- [ ] Absolute dependency root completion works.
- [ ] Relative completion uses canonical path arithmetic.
- [ ] Selective export completion uses linked public exports.
- [ ] Package exposure filters candidates.
- [ ] Every emitted candidate is canonically resolvable.
- [ ] Request path performs zero I/O/recomputation/mutation.

## Diagnostics

- [ ] Stable module diagnostic codes implemented.
- [ ] No silent module-resolution failure.
- [ ] Unresolved module diagnostic appears.
- [ ] Fixing module clears diagnostic incrementally.
- [ ] Semantic-error snapshot publishes.
- [ ] cancellation/budget/stale refresh preserves last published snapshot.

## Constructor semantics

- [ ] Constructors publish `Self`.
- [ ] Inherited constructor specializes to receiver.
- [ ] Explicit ordinary `-> Base` remains `Base`.
- [ ] Nominal return heuristic deleted.
- [ ] LSP presentation does not render constructor `Self` as `Unknown`.

## Presentation/navigation

- [ ] Binding presentation uses `BindingId`/formal site.
- [ ] Shadowing/destructuring tests pass.
- [ ] Explicit type annotation suppresses inferred hint.
- [ ] Module occurrences carry canonical IDs.
- [ ] Imported symbol definition goes to declaration origin.
- [ ] Core/builtin source navigation remains functional.

## Historical bug regressions

- [ ] `origin: Point`.
- [ ] `destination: Point`.
- [ ] `parcel: Parcel`.
- [ ] `shipment: Shipment`.
- [ ] `CellNum` is rejected where `Int` is required.
- [ ] If/else return mismatch appears and clears without restart.
- [ ] Adding/removing local type annotation never requires restart.
- [ ] Incremental revision equals cold analysis of same text.

## IDE golden

- [ ] `from geo.|` correct.
- [ ] `from units.|` correct.
- [ ] `from .domain.|` correct.
- [ ] selective export completion correct.
- [ ] private/unexposed modules excluded.
- [ ] module navigation correct.
- [ ] declaration navigation correct.
- [ ] startup readiness does not require full unrelated scan.

## Structural gates

- [ ] No `run_static_workspace_analysis`.
- [ ] No production LSP fresh `ProjectUniverse`.
- [ ] No production LSP fresh `ModuleResolver`.
- [ ] No production LSP fresh `ModuleLinker`.
- [ ] No empty/dummy module completion facade.
- [ ] No permissive name/range formal-binding fallback.
- [ ] Full workspace tests, clippy, formatting and diff checks pass.

---

# 31. Completion report required from implementing agent

At the end of each task, report:

1. exact root cause addressed;
2. exact files changed;
3. exact public/internal APIs added/removed;
4. query/product ownership before and after;
5. dependency edges introduced;
6. failing regression test added first;
7. focused test command and result;
8. broad test command and result;
9. `examples/ide-golden` observable behavior affected;
10. deletion-gate search results;
11. remaining blockers;
12. plan checkboxes/status changed.

At final completion, provide a table mapping every historical user-visible bug to the exact regression test that now prevents it.

---

# 32. Why this specification is stricter than the previous implementation plan

The previous implementation introduced many correct nouns—`SemanticWorkspaceSession`, typed `SemanticProduct` variants, `ModuleQueryFacade`, import-completion syntax contexts—but several old ownership paths remained underneath them.

This specification therefore treats **behavioral authority** rather than API existence as the completion criterion.

A persistent session is not sufficient if it rebuilds project/module/declaration meaning globally.

A `SemanticProduct::LinkedInterface` enum variant is not sufficient if no real DB query publishes/consumes it.

A callable cache is not sufficient if an unchanged caller survives a changed callee signature.

An import-completion provider is not sufficient if it constructs a facade over empty maps.

A constructor that appears to return the subclass is not sufficient if it gets that answer through an unsound nominal-return heuristic.

A green semantic unit test is not sufficient if the actual long-lived LSP still shows stale diagnostics until restart.

The final system must make these properties mechanically true through ownership, dependency edges, immutable snapshots and end-to-end regression tests.
