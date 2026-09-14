# Phalcom Incremental Semantics — Step 6: DB-Projected Formal Read Model and Snapshot Authority

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this specification task-by-task. Use test-driven development for each slice and verification-before-completion before claiming Step 6 complete.

**Goal:** Make `SemanticDb` query products the sole authority for source declaration metadata, hierarchy edges, member surfaces and callable signatures; turn `DeclarationTypeTable`, `MapTypeHierarchy`, `SurfaceDispatchResolver`, `CallableSignatureTable` and snapshot surface maps into deterministic immutable read projections of those products with structural sharing across semantically stable revisions.

**Architecture:** Step 5.5 makes query validity, declaration-shell dependencies, surface reuse and body fingerprints sound. Step 6 finishes the authority transfer: a temporary producer table may still exist while source `DeclarationShell` products are computed, but after formal query products are current, every downstream checker and every published snapshot must consume materializations derived exclusively from current-validated DB products plus immutable bootstrapped core seeds. Projection construction is read-only and must not mutate semantic revision, publish products or create new semantic facts.

**Tech stack:** Rust nightly `nightly-2026-07-10`; `phalcom-semantic`; immutable `Arc` publication; staged `SemanticDb`; `TypeStore`; `DeclarationTypeTable`; `MapTypeHierarchy`; `SurfaceDispatchResolver`; `CallableSignatureTable`.

**Governing specification:** `docs/work/analyses/phalcom_compiler_lsp_incremental_semantics_architectural_completion_spec.md`, original Task 6, re-grounded after Steps 1–5.5.

---

# 0. Hard precondition: re-ground the actual Step-5.5 commit

The connected GitHub repository exposed `main` at Step-5 commit `06f6bcd2375a7e62c46eabee967da95fa99652cf` while this specification was written.

The user's newer Step-5.5 commit must therefore be verified in the implementation checkout before any Step-6 edit.

Run:

```bash
git status --short
git rev-parse HEAD
git log -n 5 --oneline
```

The tree must be clean.

Then run:

```bash
rg "DeclarationShell" phalcom-semantic/src
rg "declaration_surface_source_prerequisite|DeclarationSurface.*ParsedModule" \
  phalcom-semantic/src/db
rg "callable body requires ready" phalcom-semantic
rg "declaration_surface_query_input_fingerprint" phalcom-semantic
```

Expected:

1. `DeclarationShell` is present as a DB product/query/dependency.
2. There is no query-specific `DeclarationSurface`/`ParsedModule` exception in `is_reusable`.
3. There is no body “requires ready CallableSignature” prewarming failure.
4. Declaration-surface cache lookup no longer computes a candidate surface before validating reuse.

Run Step-5.5 focused tests before Step 6:

```bash
cargo fmt --check

cargo test -p phalcom-semantic --test semantic_db_incremental
cargo test -p phalcom-semantic --test semantic_fingerprints
cargo test -p phalcom-semantic --test checker_dependency_tracking
cargo test -p phalcom-semantic --test formal_query_ownership
cargo test -p phalcom-semantic --test callable_dependency_invalidation
cargo test -p phalcom-semantic --test product_stability_invalidation
cargo test -p phalcom-semantic --test type_store_revisions
```

If any Step-5.5 invariant is missing, do not adapt Step 6 around the defect. Repair Step 5.5 first.

---

# 1. Why Step 6 still exists after DB-owned queries

Steps 4 and 5 correctly moved computation/publication of:

```text
HierarchyEdge
DeclarationSurface
CallableSignature
CallableBody
```

into the semantic DB.

That does **not** yet make those query products the sole runtime authority.

The session still follows this pattern:

```text
query HierarchyEdge
    ->
copy result into mutable MapTypeHierarchy

query DeclarationSurface
    ->
copy result into mutable SurfaceDispatchResolver

query CallableSignature
    ->
copy result into mutable CallableSignatureTable
```

and it separately builds/enriches:

```text
DeclarationTypeTable
```

before snapshot publication.

This leaves two semantic representations:

```text
DB query product
        +
session-owned mutable materialization
```

Even if both are currently kept synchronized, they can drift, retain removed declarations, or be constructed from different validity generations.

Step 6 makes the relationship one-way:

```text
current Ready DB products
        |
        v
immutable formal read projection
        |
        +--> body checking
        +--> field/default checking
        +--> snapshot
        +--> presentation/LSP consumers
```

The projection is not another semantic authority. It is a deterministic read index over DB authority.

---

# 2. Non-negotiable Step-6 invariants

## 2.1 DB products are source semantic authority

For every query-owned source declaration:

```text
DeclarationShell
HierarchyEdge
DeclarationSurface
CallableSignature
```

the current-validated DB product is authoritative.

No independently reconstructed table may supply a different source fact.

---

## 2.2 Core bootstrap remains an immutable seed

These existing session fields may remain:

```rust
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
```

only as immutable bootstrap/native inputs.

They may not accumulate source declarations across revisions.

Projection algorithm:

```text
immutable core seed
    +
current-validated source DB products
    =
current immutable read projection
```

---

## 2.3 Only current-validated products may enter a projection

A `Ready` product computed in an older revision but not validated for the current revision must not appear in a newly published snapshot.

This protects declaration removal and stale products left cached for potential reuse.

The materializer must never iterate `SemanticDb::product()` blindly.

---

## 2.4 Projection is read-only

Materialization must not:

```text
begin a DB revision
publish a DB product
record a semantic dependency
invalidate a query
parse source
resolve imports
resolve a type annotation
perform dispatch
intern new semantic types
```

It may clone/copy already-published semantic values into read indexes.

---

## 2.5 Body analysis and snapshot use the same projection objects

Once member projections are materialized, callable checking must receive:

```text
projection.declarations
projection.hierarchy
projection.dispatch
```

and snapshot publication must store the same `Arc`s.

Forbidden:

```text
materialization A for body analysis
materialization B for snapshot
```

---

## 2.6 Stable projections are structurally shared

When the semantic inputs to one projection component are unchanged:

```rust
Arc::ptr_eq(old_component, new_component)
```

must hold.

A body-only edit should not allocate/rebuild:

```text
DeclarationTypeTable
MapTypeHierarchy
surface map
SurfaceDispatchResolver
CallableSignatureTable
```

if all formal products underlying them remain semantically stable.

---

## 2.7 Direct hierarchy and generic supertype templates remain separate

`MapTypeHierarchy` has two distinct relations:

```text
superclasses
templates
```

Installing a `GenericSupertypeTemplate` must never invent a direct superclass.

Direct parents come only from:

```text
base core hierarchy
HierarchyEdgeProduct
```

---

## 2.8 Projection identity must be deterministic

Projection-reuse decisions must not depend on `HashMap` iteration order or pointer addresses.

Use DB `BTreeMap<QueryKey, ...>` ordering and canonical identities.

---

# 3. Explicit non-goals

Step 6 MUST NOT:

- move `ProjectUniverse` ownership into the session;
- wire `OverlaySourceProvider` into the session lifecycle;
- fix the overlay's two-lock bug in this patch;
- add `resolve_source_path`;
- implement document/path session APIs;
- remove the externally supplied `LinkedProgram`;
- move `ModuleResolver`/`ModuleLinker` ownership;
- delete LSP workspace reconstruction;
- redesign module diagnostics;
- redesign `ModuleQueryProducts`;
- rewrite import completion/navigation;
- rename all last-known-good concepts;
- introduce persistent HAMT/arena data structures;
- change language semantics.

Those belong to Step 7 and later.

---

# 4. Files

## Create

```text
phalcom-semantic/src/materialize.rs
phalcom-semantic/tests/semantic_projection.rs
```

## Modify

```text
phalcom-semantic/src/lib.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/tests/product_stability_invalidation.rs
phalcom-semantic/tests/semantic_db_incremental.rs
```

## Possibly modify only if Step-5.5 names differ

```text
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/declarations.rs
```

Do not change product semantics merely to fit the materializer.

---

# 5. Add a current-product DB read API

Modify:

```text
phalcom-semantic/src/db/mod.rs
```

Current:

```rust
pub fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>>
```

returns a product when the query state is `Ready`, even if it has not been validated for the current revision.

That behavior is useful for generic cache management and should not be silently redefined.

Add an explicit current-view API.

Recommended:

```rust
pub struct CurrentProductRef<'a> {
    pub key: &'a QueryKey,
    pub product_fingerprint: ProductFingerprint,
    pub product: &'a Arc<SemanticProduct>,
}
```

and:

```rust
impl SemanticDb {
    pub fn current_product(
        &self,
        key: &QueryKey,
    ) -> Option<CurrentProductRef<'_>>;

    pub fn current_products(
        &self,
    ) -> impl Iterator<Item = CurrentProductRef<'_>> + '_;
}
```

`current_product` returns `Some` only when:

```text
state == Ready
validated_revision == self.revision()
typed product exists
```

`current_products` must iterate in deterministic `QueryKey` order.

Because `query_states` is already a `BTreeMap`, drive iteration from it.

Do not iterate `products` independently.

---

# 6. Current-product API tests

Modify:

```text
phalcom-semantic/tests/semantic_db_incremental.rs
```

Add:

```rust
#[test]
fn current_products_exclude_ready_but_unvalidated_products()
```

Sequence:

1. publish a Ready product in revision 1;
2. assert it appears in `current_products`;
3. begin revision 2;
4. assert the old Ready product does **not** appear;
5. validate reuse;
6. assert it appears again;
7. assert its computation `revision()` remains revision 1.

Add:

```rust
#[test]
fn current_products_are_deterministically_key_ordered()
```

Publish several distinct keys in non-sorted insertion order.

Expected returned keys equal sorted `QueryKey` order.

Add:

```rust
#[test]
fn current_product_requires_matching_typed_payload()
```

If DB invariant construction APIs permit a Ready state without the corresponding typed product, `current_product` must return `None`.

---

# 7. Fix generic hierarchy template insertion

Modify:

```text
phalcom-semantic/src/types/relation.rs
```

Current behavior:

```rust
pub fn insert_template(&mut self, template: GenericSupertypeTemplate) {
    self.superclasses.insert(
        template.declaration.clone(),
        DeclarationId::new(ModuleId::core(), "generic_super".into()),
    );
    self.templates.insert(template.declaration.clone(), template);
}
```

Replace with:

```rust
pub fn insert_template(&mut self, template: GenericSupertypeTemplate) {
    self.templates
        .insert(template.declaration.clone(), template);
}
```

No synthetic direct parent.

Add a small direct API if useful:

```rust
pub fn remove_template(
    &mut self,
    declaration: &DeclarationId,
) -> Option<GenericSupertypeTemplate>
```

but do not add it unless a real caller needs it.

---

# 8. Hierarchy regression

Add to:

```text
phalcom-semantic/tests/semantic_projection.rs
```

or existing relation tests:

```rust
#[test]
fn generic_supertype_template_does_not_invent_direct_superclass()
```

Construct:

```text
Child<T> template -> Parent<T>
```

Call:

```rust
hierarchy.insert_template(template)
```

Expected:

```rust
hierarchy.superclass(&child) == None
hierarchy.supertype_template(&child) == Some(...)
```

Then separately insert the real direct parent:

```rust
hierarchy.insert(child.clone(), parent.clone())
```

and assert:

```rust
hierarchy.superclass(&child) == Some(&parent)
```

Repository search after the change:

```bash
rg '"generic_super"|generic_super' phalcom-semantic
```

Expected:

```text
no production synthetic superclass
```

---

# 9. Formal projection data model

Create:

```text
phalcom-semantic/src/materialize.rs
```

The module should be crate-internal implementation infrastructure.

Add to `lib.rs`:

```rust
mod materialize;
```

Do not `pub use` internal materialization helpers unless an external consumer has a demonstrated need.

---

## 9.1 Projection stamp entry

Use a small deterministic stamp entry:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueryProductStamp {
    pub key: QueryKey,
    pub product_fingerprint: ProductFingerprint,
}
```

Always collect in `QueryKey` order.

Do not hash these entries into one `u64` merely for convenience. Equality over canonical entries avoids adding another collision-dependent correctness boundary.

---

## 9.2 Declaration projection stamp

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DeclarationProjectionStamp {
    pub shells: Box<[QueryProductStamp]>,
}
```

It describes every current source:

```text
DeclarationShell
```

product used to build `DeclarationTypeTable`.

Immutable core seed content is session-constant and does not need to be repeated in the stamp.

Projection reuse is only allowed against a previous snapshot from the same session/workspace epoch.

---

## 9.3 Hierarchy projection stamp

Use:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HierarchyProjectionStamp {
    pub direct_edges: Box<[QueryProductStamp]>,
    pub templates: Box<[(DeclarationId, Option<GenericSupertypeTemplate>)]>,
}
```

Why not all shell fingerprints?

Because changing an unrelated generic constraint should not rebuild the hierarchy if:

```text
direct superclass
generic supertype template
```

are unchanged.

The exact template values are already canonical semantic objects and derive equality.

---

## 9.4 Surface projection stamp

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SurfaceProjectionStamp {
    pub surfaces: Box<[QueryProductStamp]>,
}
```

---

## 9.5 Dispatch projection stamp

Dispatch contains:

```text
declaration surfaces
TypeId -> DeclarationId registrations
```

so its reuse identity is:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DispatchProjectionStamp {
    pub surfaces: SurfaceProjectionStamp,
    pub type_forms: Box<[(DeclarationId, TypeId)]>,
}
```

`type_forms` is derived from the current materialized declaration table for query-owned source declarations.

This avoids rebuilding dispatch for a shell constraint-only change when the nominal form itself remains unchanged.

Core type mappings are constant in the base dispatch.

---

## 9.6 Callable signature projection stamp

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CallableSignatureProjectionStamp {
    pub signatures: Box<[QueryProductStamp]>,
}
```

Only actual current `CallableSignature` products participate.

Partial source signatures that deliberately remain surface-backed do not fabricate entries.

---

## 9.7 Combined stamps

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FormalProjectionStamps {
    pub declarations: DeclarationProjectionStamp,
    pub hierarchy: HierarchyProjectionStamp,
    pub surfaces: SurfaceProjectionStamp,
    pub dispatch: DispatchProjectionStamp,
    pub callable_signatures: CallableSignatureProjectionStamp,
}
```

---

# 10. Projection result model

Use two stages because `DeclarationSurface` computation needs declaration/hierarchy inputs.

## 10.1 Foundation projection

```rust
#[derive(Clone, Debug)]
pub(crate) struct FoundationProjection {
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub declaration_stamp: DeclarationProjectionStamp,
    pub hierarchy_stamp: HierarchyProjectionStamp,
}
```

---

## 10.2 Member projection

```rust
#[derive(Clone, Debug)]
pub(crate) struct MemberProjection {
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    pub surface_stamp: SurfaceProjectionStamp,
    pub dispatch_stamp: DispatchProjectionStamp,
    pub callable_signature_stamp: CallableSignatureProjectionStamp,
}
```

---

## 10.3 Combined projection

For snapshot construction:

```rust
#[derive(Clone, Debug)]
pub(crate) struct FormalProjection {
    pub declarations: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    pub stamps: Arc<FormalProjectionStamps>,
}
```

Construct it from foundation + member projections without rebuilding data.

---

# 11. Projection errors

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MaterializationError {
    KeyProductKindMismatch {
        key: QueryKey,
    },
    DeclarationIdentityMismatch {
        key: DeclarationId,
        product: DeclarationId,
    },
    HierarchyIdentityMismatch {
        key: DeclarationId,
        product: DeclarationId,
    },
    CallableIdentityMismatch {
        key: CallableId,
        product: CallableId,
    },
    MissingSurfaceIdentity {
        key: DeclarationId,
    },
    ImmutableSeedCollision {
        declaration: DeclarationId,
    },
}
```

Exact variant names may be adjusted to existing naming conventions, but preserve the distinctions.

Materialization is an internal consistency boundary.

Identity mismatch is not a user semantic error.

Map it in the session to:

```rust
QueryOutcome::Failed(...)
```

with actionable text.

Do not silently skip malformed products.

---

# 12. Materialize declarations from `DeclarationShell`

Implement:

```rust
pub(crate) fn materialize_declarations(
    db: &SemanticDb,
    base: &DeclarationTypeTable,
    previous: Option<(&Arc<DeclarationTypeTable>, &DeclarationProjectionStamp)>,
) -> Result<(Arc<DeclarationTypeTable>, DeclarationProjectionStamp), MaterializationError>
```

Algorithm:

1. Iterate `db.current_products()`.
2. Select only:

```text
QueryKey::DeclarationShell(declaration)
```

3. Require typed product:

```text
SemanticProduct::DeclarationShell
```

4. Require:

```text
product.declaration == key declaration
```

5. Collect `QueryProductStamp`.
6. If the collected stamp equals previous stamp:

```text
return previous Arc
```

7. Otherwise:
   - clone immutable `base`;
   - insert each source `DeclarationTypeInfo`;
   - reject collision with immutable base declaration identity unless it is an explicitly allowed bootstrap identity;
   - return new `Arc`.

Do not inspect source AST.

Do not recompute generic signatures.

Do not intern types.

---

# 13. Materialize hierarchy from shell templates + hierarchy edges

Implement:

```rust
pub(crate) fn materialize_hierarchy(
    db: &SemanticDb,
    base: &MapTypeHierarchy,
    declarations: &DeclarationTypeTable,
    previous: Option<(&Arc<MapTypeHierarchy>, &HierarchyProjectionStamp)>,
) -> Result<(Arc<MapTypeHierarchy>, HierarchyProjectionStamp), MaterializationError>
```

Algorithm:

1. Iterate current `DeclarationShell` products.
2. Read each:

```rust
DeclarationTypeInfo.supertype_template
```

and collect:

```text
(declaration, Option<GenericSupertypeTemplate>)
```

in declaration order.

3. Iterate current:

```text
QueryKey::HierarchyEdge(declaration)
```

products.
4. Require edge payload class identity equals key declaration.
5. Collect direct-edge stamps.
6. Compare combined hierarchy stamp with previous.
7. If equal, return previous hierarchy `Arc`.
8. Otherwise:
   - clone immutable base hierarchy;
   - insert each non-`None` generic supertype template using corrected `insert_template`;
   - insert each `HierarchyEdgeProduct.super_decl` direct parent;
   - return new `Arc`.

The declaration table argument is for consistency validation if needed; do not use it to recompute hierarchy facts.

Optional consistency check:

```text
every query-owned HierarchyEdge declaration must have a current DeclarationShell
```

If this is already guaranteed by Step-5.5 query topology, assert/fail rather than silently fabricate a declaration.

---

# 14. Foundation materializer

Implement:

```rust
pub(crate) fn materialize_foundation(
    db: &SemanticDb,
    base_declarations: &DeclarationTypeTable,
    base_hierarchy: &MapTypeHierarchy,
    previous: Option<&SemanticSnapshot>,
) -> Result<FoundationProjection, MaterializationError>
```

It:

1. builds/reuses declarations;
2. builds/reuses hierarchy;
3. does nothing else.

It must not materialize member surfaces before they have been queried with the current foundation.

---

# 15. Materialize surface map

Implement:

```rust
pub(crate) fn materialize_surfaces(
    db: &SemanticDb,
    previous: Option<(&Arc<HashMap<DeclarationId, DeclarationSurface>>, &SurfaceProjectionStamp)>,
) -> Result<(Arc<HashMap<DeclarationId, DeclarationSurface>>, SurfaceProjectionStamp), MaterializationError>
```

Algorithm:

1. Iterate current `DeclarationSurface` products.
2. Require typed product and identity:

```text
surface.id == Some(key declaration)
```

for ordinary source declarations.
3. Collect product stamps.
4. Reuse previous map `Arc` when stamp unchanged.
5. Otherwise build a fresh map from current source surface products only.

Do not add core surfaces here.

Core surfaces live in `base_dispatch`, not the snapshot's source-surface map unless existing snapshot semantics explicitly require otherwise.

Preserve the current behavior of `snapshot.surfaces`.

---

# 16. Materialize dispatch resolver

Implement:

```rust
pub(crate) fn materialize_dispatch(
    base: &SurfaceDispatchResolver,
    declarations: &DeclarationTypeTable,
    surfaces: &HashMap<DeclarationId, DeclarationSurface>,
    surface_stamp: &SurfaceProjectionStamp,
    previous: Option<(&Arc<SurfaceDispatchResolver>, &DispatchProjectionStamp)>,
) -> Result<(Arc<SurfaceDispatchResolver>, DispatchProjectionStamp), MaterializationError>
```

Build deterministic:

```text
type_forms = sorted [(DeclarationId, TypeId)]
```

for source declarations represented by the current surface map.

The stamp is:

```text
surface stamp + type_forms
```

If unchanged, reuse previous dispatch `Arc`.

Otherwise:

1. clone immutable `base_dispatch`;
2. for source surfaces in canonical declaration order:
   - `register_surface`;
   - if `declarations.form(decl)` exists, `register_type(form, decl)`;
3. return new `Arc`.

Do not traverse hierarchy while materializing dispatch.

The resolver receives hierarchy separately at query time.

Therefore a pure superclass edit does not by itself require rebuilding dispatch unless a surface or declaration form changes.

---

# 17. Materialize callable signature table

Implement:

```rust
pub(crate) fn materialize_callable_signatures(
    db: &SemanticDb,
    base: &CallableSignatureTable,
    previous: Option<(&Arc<CallableSignatureTable>, &CallableSignatureProjectionStamp)>,
) -> Result<(Arc<CallableSignatureTable>, CallableSignatureProjectionStamp), MaterializationError>
```

Algorithm:

1. select current `CallableSignature` products;
2. require product callable identity equals key;
3. collect stamp;
4. reuse previous Arc when unchanged;
5. otherwise clone immutable base table and insert each current source signature.

No signature may be generated from a `DeclarationSurface` inside the materializer.

Projection only copies DB-owned canonical signature products.

---

# 18. Member materializer

Implement:

```rust
pub(crate) fn materialize_members(
    db: &SemanticDb,
    base_dispatch: &SurfaceDispatchResolver,
    base_callable_signatures: &CallableSignatureTable,
    foundation: &FoundationProjection,
    previous: Option<&SemanticSnapshot>,
) -> Result<MemberProjection, MaterializationError>
```

It:

1. materializes/reuses source surface map;
2. materializes/reuses dispatch;
3. materializes/reuses callable signature table.

It does not run declaration-surface or signature queries itself.

The session owns query scheduling.

The materializer owns read projection only.

---

# 19. Snapshot projection metadata

Modify:

```text
phalcom-semantic/src/snapshot.rs
```

Add crate-private:

```rust
pub(crate) formal_projection_stamps: Arc<FormalProjectionStamps>,
```

or an equivalent private field with crate accessor.

Do not expose stamps as public language semantics.

Add:

```rust
pub(crate) fn formal_projection_stamps(
    &self,
) -> &FormalProjectionStamps
```

if materializer access requires it.

Compatibility snapshot constructors may initialize:

```rust
FormalProjectionStamps::default()
```

A default stamp means:

```text
do not structurally reuse this component on the first production materialization
```

not “the projection is semantically empty.”

---

# 20. Coherent production snapshot constructor

Current constructors accept formal tables independently.

Add a production-only constructor or builder:

```rust
#[allow(clippy::too_many_arguments)]
pub(crate) fn new_from_formal_projection(
    workspace: WorkspaceId,
    revision: SemanticRevision,
    generation: u64,
    store: Arc<TypeStore>,
    sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    projection: FormalProjection,
    diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
    semantic_graph: Arc<SemanticGraph>,
    callable_analyses: Arc<HashMap<CallableId, Arc<CallableAnalysis>>>,
    module_products: Arc<ModuleQueryProducts>,
) -> Self
```

It must install:

```text
projection.surfaces
projection.dispatch
projection.callable_signatures
projection.declarations
projection.hierarchy
projection.stamps
```

as one coherent unit.

Keep existing public constructors only for compatibility/tests if removing them creates unrelated churn.

Production `SemanticWorkspaceSession` should use only the coherent constructor.

---

# 21. Refactor session into producer and consumer phases

Modify:

```text
phalcom-semantic/src/session.rs
```

The update path should have explicit phases.

---

## 21.1 Phase A — source and module prerequisites

Keep current Step-5.5 behavior that brings current:

```text
ParsedModule
UnlinkedInterface
LinkedInterface
```

products into the DB.

Do not move module lifecycle ownership in Step 6.

---

## 21.2 Phase B — temporary declaration-shell producer state

The current source predeclaration/generic enrichment machinery may temporarily build a table needed to compute/publish:

```text
DeclarationShell
```

products.

Rename local variables to make its role explicit, for example:

```rust
let mut candidate_declarations = self.base_declarations.clone();
```

instead of:

```rust
let mut declarations = ...
```

This table is not allowed to survive as downstream authority.

Use it only to:

- predeclare current source types;
- resolve declaration generic signatures;
- resolve supertype templates;
- publish/current-validate `DeclarationShell`.

After the required shell products are current, stop passing `candidate_declarations` into downstream body/member analysis.

---

## 21.3 Phase C — hierarchy query production

Bring every current:

```text
HierarchyEdge
```

query product current.

The resolver used to resolve the superclass name may use current transition inputs as required by Step 5.5.

Do not insert the returned edge into a session-local `MapTypeHierarchy`.

Delete code equivalent to:

```rust
hierarchy.insert(class_decl, super_decl)
```

from the query-production loop.

---

## 21.4 Phase D — foundation projection

Immediately after all current shell/hierarchy products exist:

```rust
let foundation = materialize_foundation(
    &self.db,
    &self.base_declarations,
    &self.base_hierarchy,
    self.last_snapshot.as_deref(),
)?;
```

From this point onward, source semantic consumers use:

```text
foundation.declarations
foundation.hierarchy
```

not `candidate_declarations`.

---

## 21.5 Phase E — declaration surface queries

Run `query_declaration_surface` using:

```rust
foundation.declarations.as_ref()
foundation.hierarchy.as_ref()
```

plus current resolver/linked prerequisites.

Do not insert returned surfaces into a mutable session dispatch table.

The returned product is already in DB authority.

---

## 21.6 Phase F — callable signature queries

Enumerate current declaration-surface products or current source class members as needed to request per-callable signatures.

Call:

```rust
query_callable_signature(...)
```

Do not insert successful results into a mutable `CallableSignatureTable`.

---

## 21.7 Phase G — member projection

After all member products are current:

```rust
let members = materialize_members(
    &self.db,
    &self.base_dispatch,
    &self.base_callable_signatures,
    &foundation,
    self.last_snapshot.as_deref(),
)?;
```

Build:

```rust
let formal_projection = FormalProjection {
    declarations: foundation.declarations.clone(),
    hierarchy: foundation.hierarchy.clone(),
    surfaces: members.surfaces.clone(),
    dispatch: members.dispatch.clone(),
    callable_signatures: members.callable_signatures.clone(),
    stamps: ...
};
```

No extra rebuild.

---

## 21.8 Phase H — callable body analysis

Every body query must consume exactly:

```rust
foundation.hierarchy.as_ref()
foundation.declarations.as_ref()
members.dispatch.as_ref()
```

Field/default/top-level compatibility checking must consume the same objects.

Delete downstream uses of temporary candidate tables.

This creates the law:

```text
query products
   -> projection
       -> checker
       -> snapshot
```

rather than:

```text
query products -> mutable copy A -> checker
query products -> mutable copy B -> snapshot
```

---

## 21.9 Phase I — snapshot publication

Construct `SemanticSnapshot` with:

```text
formal_projection
```

and store the exact Arcs used during body analysis.

Do not clone these tables again at publication.

---

# 22. Session fields after Step 6

The session may still retain:

```rust
workspace
db
store
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
sources
source_fingerprints
last_snapshot
last_known_good/last_published
```

Step 6 does not yet add project/module lifecycle fields.

Critically, it must **not** add persistent mutable:

```text
source_declarations
source_hierarchy
source_dispatch
source_callable_signatures
```

fields.

The current source-state read model exists in the last immutable snapshot/projection, not as a second mutable authority.

---

# 23. Source declaration removal law

A source declaration can disappear while its old DB product remains cached but unvalidated.

Example:

Revision 1:

```phalcom
class Gone {
  value() -> Int { 1 }
}
```

Revision 2:

```phalcom
class StillHere {}
```

The module remains.

The materializer must not copy old:

```text
DeclarationShell(Gone)
HierarchyEdge(Gone)
DeclarationSurface(Gone)
CallableSignature(Gone.value)
```

into revision 2 merely because old query states are still `Ready`.

This is why `current_products()` requires:

```text
validated_revision == current revision
```

---

# 24. Required Step-6 tests

Create:

```text
phalcom-semantic/tests/semantic_projection.rs
```

Use helpers consistent with existing semantic integration tests.

---

## 24.1 Snapshot declaration table equals DB shells

Test:

```rust
#[test]
fn snapshot_declarations_are_projected_from_current_declaration_shell_products()
```

For each query-owned source declaration:

1. fetch current `DeclarationShell` product from DB;
2. fetch snapshot declaration info;
3. assert equality.

Also assert no source declaration appears in the snapshot without a current shell product.

Ignore immutable core seed declarations when applying the second rule.

---

## 24.2 Snapshot hierarchy equals hierarchy-edge products

Test:

```rust
#[test]
fn snapshot_direct_hierarchy_is_projected_from_current_hierarchy_edges()
```

For every source class:

```text
db HierarchyEdge.super_decl
==
snapshot.hierarchy.superclass(class)
```

---

## 24.3 Snapshot generic template equals declaration shell

Test:

```rust
#[test]
fn snapshot_generic_supertype_templates_are_projected_from_declaration_shells()
```

Use a generic inheritance source program.

Assert:

```text
DeclarationShell.supertype_template
==
snapshot.hierarchy.supertype_template
```

and direct parent remains the actual nominal superclass.

Assert no declaration named:

```text
generic_super
```

appears as a direct parent.

---

## 24.4 Snapshot surface map equals DB surfaces

Test:

```rust
#[test]
fn snapshot_surfaces_are_projected_from_current_surface_products()
```

For every current source `DeclarationSurface` product, assert snapshot surface equality.

Also assert:

```rust
snapshot.dispatch.get_surface(decl)
==
snapshot.surfaces.get(decl)
```

---

## 24.5 Snapshot callable table equals DB signature products

Test:

```rust
#[test]
fn snapshot_callable_signatures_are_projected_from_current_signature_products()
```

For every current source `CallableSignature` product:

```text
snapshot.callable_signatures.get(callable)
==
product
```

For intentionally partial source signatures:

```text
no CallableSignature product
no fabricated snapshot canonical signature
DeclarationSurface still contains the partial contract
```

---

## 24.6 Removed declaration cannot leak from stale Ready cache

Test:

```rust
#[test]
fn removed_declaration_is_absent_from_all_current_snapshot_projections()
```

Revision 1 defines `Gone`.

Revision 2 removes only `Gone`, preserving the module.

Assert revision-2 snapshot has no:

```text
declarations.get(Gone)
hierarchy.superclass(Gone)
surfaces.get(Gone)
dispatch.get_surface(Gone)
callable_signatures.get(Gone.value)
```

This test is required even if old DB products remain cached internally.

---

## 24.7 Body-only edit reuses all formal projection Arcs

Strengthen:

```text
phalcom-semantic/tests/product_stability_invalidation.rs
```

Capture revision-1 snapshot Arcs:

```rust
let declarations = snapshot.declarations.clone();
let hierarchy = snapshot.hierarchy.clone();
let surfaces = snapshot.surfaces.clone();
let dispatch = snapshot.dispatch.clone();
let signatures = snapshot.callable_signatures.clone();
```

Perform body-only edit with unchanged contracts.

Revision 2 must satisfy:

```rust
Arc::ptr_eq(&declarations, &snapshot2.declarations)
Arc::ptr_eq(&hierarchy, &snapshot2.hierarchy)
Arc::ptr_eq(&surfaces, &snapshot2.surfaces)
Arc::ptr_eq(&dispatch, &snapshot2.dispatch)
Arc::ptr_eq(&signatures, &snapshot2.callable_signatures)
```

This is a hard Step-6 performance gate.

---

## 24.8 Signature edit reuses foundation

Test:

```rust
#[test]
fn member_signature_edit_reuses_foundation_and_rebuilds_member_projection()
```

Change:

```phalcom
value() -> Int
```

to:

```phalcom
value() -> String
```

Expected where declaration shell metadata is unchanged:

```text
declarations Arc reused
hierarchy Arc reused
surfaces Arc changed
dispatch Arc changed
callable_signatures Arc changed
```

If the actual Step-5.5 shell includes a source fact that intentionally changes here, adjust only the declaration expectation after verifying why. Do not weaken the member-projection assertions.

---

## 24.9 Superclass edit does not force dispatch rebuild unless its own inputs changed

Test:

```rust
#[test]
fn superclass_only_change_reuses_surface_dispatch_when_surfaces_and_forms_are_stable()
```

Expected:

```text
hierarchy Arc changes
surface map Arc reused if surface product fingerprints stay stable
dispatch Arc reused if surface stamp + type-form mapping stay stable
callable signature table Arc reused if signatures stay stable
```

`DeclarationTypeTable` may change because a supertype template is part of `DeclarationShell`.

Do not assert declaration Arc reuse unless the shell product remains identical.

---

## 24.10 Generic constraint-only shell edit does not over-rebuild hierarchy

Create a generic declaration edit that changes shell generic constraints but leaves:

```text
direct HierarchyEdge
supertype_template
```

unchanged.

Expected:

```text
declarations Arc changes
hierarchy Arc reused
```

This validates the exact hierarchy stamp rather than “all shells changed => rebuild hierarchy.”

---

## 24.11 Projection is read-only

Test:

```rust
#[test]
fn materialization_does_not_mutate_db_revision_or_query_states()
```

Capture:

```text
db.revision
relevant QueryState clones
metrics hit/miss/invalidations
```

Materialize.

Expected:

```text
revision unchanged
query states unchanged
no product publication
no invalidation
```

If metrics currently count read-only accessor calls, do not add new materialization-specific hit/miss accounting.

Projection is not a query computation.

---

## 24.12 Mismatched key/product fails closed

Add a focused unit/integration test if construction API permits:

```text
QueryKey::HierarchyEdge(A)
payload.class_decl == B
```

or analogous shell/signature mismatch.

Expected:

```text
MaterializationError
no snapshot publication
```

Never silently index the payload under one identity or the key under another.

---

# 25. Structural-sharing implementation rules

## 25.1 Reuse whole component Arcs

Step 6 does not need a persistent collection library.

Reuse at component granularity:

```text
DeclarationTypeTable Arc
MapTypeHierarchy Arc
surface HashMap Arc
SurfaceDispatchResolver Arc
CallableSignatureTable Arc
```

This already removes substantial repeated work.

---

## 25.2 Do not use pointer equality to decide reuse

Use deterministic stamps.

Pointer equality is only the test/observable consequence of successful reuse.

---

## 25.3 Do not use one aggregate hash as correctness identity

The semantic DB already uses product fingerprints for dependency semantics.

For projection structural sharing, keep the ordered `(QueryKey, ProductFingerprint)` entries and exact additional values.

This makes collision behavior no worse than the underlying DB dependency identity and avoids introducing a second opaque aggregate hash.

---

# 26. Projection consistency checks

Before snapshot publication, assert or validate:

## Declarations

```text
every current source DeclarationShell identity matches key
```

## Hierarchy

```text
every current HierarchyEdge.class_decl matches key
every source hierarchy edge refers to declarations known in current projection or immutable core seed
```

Unresolved superclass queries should already have their own diagnostic/outcome semantics. Materializer does not resolve them.

## Surfaces

```text
surface.id matches DeclarationSurface key
surface owner exists in declaration projection
```

## Signatures

```text
signature.callable matches CallableSignature key
signature.owner exists in declaration projection
```

## Dispatch

```text
every source surface is registered exactly once
every source nominal form mapping comes from current declaration projection
```

Do not silently repair malformed product relationships.

---

# 27. Snapshot API compatibility

Existing consumers access public fields directly:

```text
snapshot.declarations
snapshot.hierarchy
snapshot.surfaces
snapshot.dispatch
snapshot.callable_signatures
```

Keep these fields in Step 6 to avoid unrelated LSP/API churn.

The change is their **construction authority**, not public read syntax.

A later API cleanup may replace public fields with accessors.

---

# 28. Metrics

Do not count projection reuse as DB query hits.

Add optional `SemanticUpdateStats` fields only if useful and stable, for example:

```rust
pub formal_projections_reused: usize,
pub formal_projections_rebuilt: usize,
```

But this is optional.

The mandatory observable is `Arc` structural sharing in tests.

Do not expand scope solely to add telemetry.

---

# 29. Failure/publication semantics

Projection materialization occurs before final snapshot publication.

If materialization detects an internal consistency failure:

```text
do not publish a mixed/partial formal snapshot
```

Propagate:

```rust
QueryOutcome::Failed(...)
```

through the existing session update path.

The existing publication/LKG behavior then retains the previously published snapshot.

Ordinary semantic diagnostics are not materialization failures.

---

# 30. TDD task sequence

Implement in reviewable slices.

---

## Task 6.1 — Current DB product view

**Files**

```text
phalcom-semantic/src/db/mod.rs
phalcom-semantic/tests/semantic_db_incremental.rs
```

### Red tests

- current products exclude unvalidated old Ready entries;
- revalidated unchanged product reappears;
- deterministic key order.

### Implementation

Add `CurrentProductRef`, `current_product`, `current_products`.

### Gate

```bash
cargo test -p phalcom-semantic --test semantic_db_incremental
```

### Commit

```text
feat(semantic-db): expose current validated product view
```

---

## Task 6.2 — Hierarchy template correctness

**Files**

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/tests/semantic_projection.rs
```

### Red test

`insert_template` must not create a fake superclass.

### Implementation

Remove `core.generic_super` insertion.

### Gate

```bash
cargo test -p phalcom-semantic --test semantic_projection \
  generic_supertype_template_does_not_invent_direct_superclass
cargo test -p phalcom-semantic --test substitution
```

### Commit

```text
fix(semantic): separate hierarchy templates from direct parents
```

---

## Task 6.3 — Declaration + hierarchy foundation materializer

**Files**

```text
phalcom-semantic/src/materialize.rs
phalcom-semantic/src/lib.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/tests/semantic_projection.rs
```

### Red tests

- declarations equal current shells;
- hierarchy equals current edge products;
- generic templates equal shell templates;
- removed declaration omitted;
- mismatch fails closed.

### Implementation

Add foundation stamps and materialization.

### Gate

```bash
cargo test -p phalcom-semantic --test semantic_projection
cargo test -p phalcom-semantic --test type_store_revisions
```

### Commit

```text
feat(semantic): materialize declaration and hierarchy projections
```

---

## Task 6.4 — Surface/dispatch/signature materializer

**Files**

```text
phalcom-semantic/src/materialize.rs
phalcom-semantic/tests/semantic_projection.rs
```

### Red tests

- source surface map equals DB products;
- dispatch surface equals map;
- canonical signature table equals DB products;
- partial signatures are not fabricated.

### Implementation

Add member stamps/materialization.

### Gate

```bash
cargo test -p phalcom-semantic --test semantic_projection
cargo test -p phalcom-semantic --test formal_query_ownership
```

### Commit

```text
feat(semantic): materialize member semantic projections
```

---

## Task 6.5 — Session authority transfer

**Files**

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
```

### Red/integration tests

Use existing tests plus new projection equality tests.

### Implementation

Refactor update phases:

```text
candidate shell producer
-> DB shells/edges
-> foundation projection
-> surface/signature queries
-> member projection
-> body checking
-> snapshot from same projection
```

Delete mutable hierarchy/dispatch/signature insertions from query loops.

### Gate

```bash
cargo test -p phalcom-semantic --test formal_query_ownership
cargo test -p phalcom-semantic --test callable_dependency_invalidation
cargo test -p phalcom-semantic --test checker_dependency_tracking
cargo test -p phalcom-semantic --test semantic_projection
```

### Commit

```text
refactor(semantic): make formal tables DB projections
```

---

## Task 6.6 — Structural sharing

**Files**

```text
phalcom-semantic/src/materialize.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/tests/product_stability_invalidation.rs
phalcom-semantic/tests/semantic_projection.rs
```

### Red tests

- body-only edit ptr-equality for all formal projection Arcs;
- signature edit reuses foundation;
- hierarchy-only semantic change reuses dispatch when inputs stable;
- generic constraint-only edit does not over-rebuild hierarchy.

### Implementation

Compare current stamps with previous snapshot stamps and reuse exact Arcs.

### Gate

```bash
cargo test -p phalcom-semantic --test product_stability_invalidation
cargo test -p phalcom-semantic --test semantic_projection
```

### Commit

```text
perf(semantic): structurally share stable formal projections
```

---

## Task 6.7 — Full authority/deletion audit

Run:

```bash
rg "let mut declarations = self.base_declarations.clone" \
  phalcom-semantic/src/session.rs

rg "let mut hierarchy = self.base_hierarchy.clone" \
  phalcom-semantic/src/session.rs

rg "let mut dispatch = self.base_dispatch.clone" \
  phalcom-semantic/src/session.rs

rg "let mut callable_signatures = self.base_callable_signatures.clone" \
  phalcom-semantic/src/session.rs

rg "hierarchy\.insert|dispatch\.register_surface|callable_signatures\.insert" \
  phalcom-semantic/src/session.rs
```

Expected:

- no downstream source materialization performed ad hoc in session;
- any remaining candidate declaration construction is explicitly producer-only;
- all final table building occurs in `materialize.rs`.

Also run:

```bash
rg "generic_super" phalcom-semantic
```

Expected no production fake parent.

Commit any final test-only audit improvements separately.

---

# 31. Full verification gate

Use pinned toolchain:

```text
nightly-2026-07-10
```

Run:

```bash
cargo fmt --check
```

Focused:

```bash
cargo test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
cargo test -p phalcom-semantic --test semantic_projection -- --nocapture
cargo test -p phalcom-semantic --test formal_query_ownership -- --nocapture
cargo test -p phalcom-semantic --test product_stability_invalidation -- --nocapture
cargo test -p phalcom-semantic --test callable_dependency_invalidation -- --nocapture
cargo test -p phalcom-semantic --test checker_dependency_tracking -- --nocapture
cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture
cargo test -p phalcom-semantic --test substitution -- --nocapture
```

Semantic crate:

```bash
cargo test -p phalcom-semantic
```

Workspace:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not proceed to Step 7 if a projection/current-product test fails.

---

# 32. Performance acceptance matrix

## Body-only edit

Expected:

```text
ParsedModule                     recompute
edited CallableBody             recompute

DeclarationShell products       reuse/current validate
HierarchyEdge products          reuse
DeclarationSurface products     reuse
CallableSignature products      reuse

DeclarationTypeTable Arc        ptr_eq old
MapTypeHierarchy Arc            ptr_eq old
surface map Arc                 ptr_eq old
SurfaceDispatchResolver Arc     ptr_eq old
CallableSignatureTable Arc      ptr_eq old
```

---

## Callable signature edit

Expected:

```text
owning DeclarationSurface       change
owning CallableSignature        change
exact callers                   recompute

DeclarationTypeTable Arc        reuse if shell unchanged
MapTypeHierarchy Arc            reuse if hierarchy inputs unchanged
surface map Arc                 rebuild
dispatch Arc                    rebuild
signature table Arc             rebuild
```

---

## Superclass edit

Expected:

```text
HierarchyEdge                   change
exact hierarchy consumers       recompute
MapTypeHierarchy Arc            rebuild

surface/dispatch/signature
projections                     reuse whenever their own stamps remain stable
```

---

## Generic shell constraint/kind edit

Expected:

```text
DeclarationShell                change
DeclarationTypeTable Arc        rebuild

Hierarchy Arc                   rebuild only when direct edge/template changes
Dispatch Arc                    rebuild only when surface/type-form stamp changes
```

This is an important anti-overinvalidation gate.

---

## Declaration deletion inside surviving module

Expected:

```text
old unvalidated DB products     may remain cached internally
new snapshot                    contains no deleted source declaration
```

---

# 33. Code-quality constraints

## No semantic logic in materializer

Forbidden examples:

```text
resolve_type_annotation
resolve_import
resolve_dispatch
is_subtype
infer generic arguments
build interface
parse source
```

Materializer indexes existing facts.

---

## No second DB

Do not create an auxiliary cache/database for projection stamps.

Stamps live only in the immutable snapshot needed for next-revision structural sharing.

---

## No source-range projection key

Projection reuse follows semantic DB product fingerprints and exact canonical semantic values.

Do not add source range/URI to projection stamps.

---

## No `Debug` hashing

Projection stamps are structured values, not `format!("{:?}")`.

---

## No unsafe stale fallback

If a required current product is absent, do not pull it from:

```text
db.last_known_good_product
```

to populate the current snapshot.

Last-published fallback happens at snapshot publication boundary when the entire current revision cannot publish.

It is not a per-product merge policy.

---

# 34. Interaction with Step 5.5 `DeclarationShell`

Step 6 assumes Step 5.5 has made `DeclarationShell` dependency-visible.

The source declaration flow becomes:

```text
candidate source declaration metadata
        |
        v
query_declaration_shell
        |
        v
current DB DeclarationShell product
        |
        v
materialize_declarations
        |
        v
immutable DeclarationTypeTable read projection
```

Do not skip the middle and materialize from the candidate table directly.

That would restore the dual-authority problem Step 6 exists to remove.

---

# 35. Interaction with generic inheritance

`DeclarationTypeInfo` carries:

```text
generic_signature
supertype_template
```

`HierarchyEdgeProduct` carries:

```text
direct superclass declaration identity
```

Therefore projection must combine both:

```text
DeclarationShell.supertype_template
+
HierarchyEdge.super_decl
```

This is intentional.

Do not try to encode a generic applied parent into the direct declaration edge.

Do not invent a synthetic declaration as a bridge.

---

# 36. Interaction with dispatch

`SurfaceDispatchResolver` stores:

```text
DeclarationId -> DeclarationSurface
TypeId -> DeclarationId
```

It does not own hierarchy.

`resolve_dispatch_with_trace` receives:

```rust
&dyn TypeHierarchy
```

at query time.

Therefore:

```text
hierarchy changes
```

do not require dispatch reconstruction unless:

```text
surface mapping
or type-form mapping
```

also changes.

The Step-6 dispatch stamp should preserve that separation.

---

# 37. Interaction with callable bodies

Body query semantic dependencies still point to query products, not projection Arcs.

Projection Arcs are execution/read acceleration.

A caller does **not** depend on:

```text
snapshot dispatch Arc identity
snapshot hierarchy Arc identity
```

It depends on:

```text
CallableSignature
DeclarationSurface
HierarchyEdge
DeclarationShell
LinkedInterface
```

as captured by Step 3/5.5.

Do not add projection objects to the DB dependency graph.

---

# 38. Snapshot observational semantics

External consumers should see no semantic behavior change from Step 6.

Existing snapshot reads remain:

```rust
snapshot.declarations
snapshot.hierarchy
snapshot.surfaces
snapshot.dispatch
snapshot.callable_signatures
```

But the implementation guarantee changes from:

```text
session happened to rebuild equivalent structures
```

to:

```text
snapshot structures are deterministic indexes of current DB truth
```

This is the purpose of Step 6.

---

# 39. What Step 7 may assume after Step 6

Only after Step 6 passes may compiler-owned module lifecycle work assume:

1. source formal semantic facts have one authority: `SemanticDb`;
2. declaration/hierarchy/member lookup tables are immutable projections;
3. removed stale products cannot leak into snapshots;
4. body analysis and snapshot presentation use the same projection objects;
5. body-only edits do not rebuild stable formal indexes;
6. generic supertype templates no longer fabricate direct hierarchy edges.

Step 7 can then safely focus on:

```text
persistent ProjectUniverse
safe OverlaySourceProvider
canonical physical-path -> ModuleId resolution
source/document change APIs
compiler-owned resolver/linker lifecycle
canonical module products
```

without simultaneously refactoring the formal read model.

---

# 40. Known Step-7 issues deliberately deferred

The current module substrate already contains useful pieces.

Do not reimplement them during Step 6:

```text
ImportResolutionTrace
resolve_import_with_trace
ModuleQueryProducts
ModuleQueryFacade
import_root_entries
module_children
external_import_children
workspace input model types
```

Step 7 should re-ground and reuse them.

The current `OverlaySourceProvider` should be repaired there:

### Lock ordering

Current mutation acquires:

```text
module map -> source map
```

while read path can acquire:

```text
source map -> module map
```

Use one state lock or a single consistent locking protocol.

### Reverse-map replacement

When a module overlay changes from source ID A to B, remove the old:

```text
A -> module
```

mapping before publishing:

```text
B -> module
```

These are not Step-6 changes because projection construction never touches the mutable provider.

---

# 41. Final Step-6 acceptance checklist

## DB view

- [ ] `current_product` exists.
- [ ] `current_products` exists.
- [ ] old unvalidated Ready products are excluded.
- [ ] iteration order is deterministic.

## Hierarchy

- [ ] `insert_template` does not create `core.generic_super`.
- [ ] direct edges come from `HierarchyEdgeProduct`.
- [ ] generic templates come from `DeclarationShell`.
- [ ] generic inheritance tests pass.

## Declarations

- [ ] current source declaration table is materialized from `DeclarationShell`.
- [ ] core declarations remain immutable seed.
- [ ] removed source declaration cannot leak from stale DB cache.

## Surfaces and dispatch

- [ ] source surface map is materialized from current `DeclarationSurface` products.
- [ ] dispatch is base seed + current source surfaces + current source type forms.
- [ ] dispatch is structurally reused when its own stamp is unchanged.

## Signatures

- [ ] canonical source signature table is materialized from current `CallableSignature` products.
- [ ] partial source signatures are not fabricated.

## Session

- [ ] candidate declaration table is producer-only.
- [ ] downstream body analysis uses foundation/member projections.
- [ ] session no longer manually inserts source hierarchy/surface/signature results into authoritative mutable maps.
- [ ] snapshot receives exact same projection Arcs used by body analysis.

## Snapshot

- [ ] projection stamps retained privately for next revision.
- [ ] production snapshot constructor consumes one coherent formal projection.
- [ ] body-only edit preserves Arc identity of all stable formal projections.

## Tests

- [ ] semantic DB current-product tests pass.
- [ ] semantic projection tests pass.
- [ ] product-stability tests pass.
- [ ] formal ownership tests pass.
- [ ] generic substitution/hierarchy tests pass.
- [ ] semantic crate passes.
- [ ] workspace check/clippy/tests pass.

---

# 42. Recommended implementation commits

Keep the work reviewable:

```text
1. feat(semantic-db): expose current validated product view
2. fix(semantic): separate hierarchy templates from direct parents
3. feat(semantic): materialize declaration and hierarchy projections
4. feat(semantic): materialize member semantic projections
5. refactor(semantic): make formal tables DB projections
6. perf(semantic): structurally share stable formal projections
7. test(semantic): complete step-6 authority regressions
```

A single patch is acceptable if the verification report preserves these logical boundaries.

Do not mix Step-7 module lifecycle changes into Step 6.
