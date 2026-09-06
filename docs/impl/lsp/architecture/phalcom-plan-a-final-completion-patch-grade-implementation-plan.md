# Phalcom LSP Module Architecture — Plan A Final Completion Patch-Grade Implementation Plan

**Prepared:** 2026-09-06
**Repository:** `aureat/phalcom-lang`
**Current pushed `main`:** `6410ebcb3a0d1df5b86dc34d00ec820c88e96658`
**Plan-A entry baseline:** `d60e4589352ac5f4167ba295e7e2a5f6c870ef4b`
**Purpose:** Drive Plan A from the current pushed C1–C5 state to a defensible, reproducible `COMPLETE` release verdict.
**Execution rule:** This is a terminal closure plan. Do not stop after a focused green test or another partial checkpoint. Stop only at an explicitly documented blocker that cannot safely be resolved inside Plan A; otherwise continue through the final release matrix.

---

# 1. Final Objective

Plan A is complete only when the ordinary compiler/LSP update path has this shape:

```text
source mutation(s)
    ↓
transaction-local module deltas
    ↓
exact changed module/interface/import/component products
    ↓
exact source-semantic contribution deltas
    ↓
typed SemanticDb roots
    ↓
exact reverse dependency closure
    ↓
changed/revalidated semantic products only
    ↓
retained module-owned aggregate contributions
    ↓
immutable snapshot assembled from retained products
    ↓
cold-equivalent observable semantics
```

And, critically:

```text
ordinary edit work
    ∝ changed products
      + actual dependency closure
      + bounded local/index maintenance
```

It must **not** remain:

```text
exact semantic recomputation
    +
O(workspace) map cloning / aggregate scanning / advisory rebuilding
```

The final implementation must satisfy both semantic correctness and asymptotic incrementality.

---

# 2. Repository-Grounded Current State

## 2.1 What is already pushed and should be preserved

The current pushed head already contains the C1–C5 correctness slice and several previously open acceptance fixtures. Do not reimplement these from scratch.

### Typed dependency currentness

`phalcom-semantic/src/db/mod.rs::SemanticDb::record_dependency` now enforces:

```rust
dependency.validated_revision == db.revision()
```

before a dependency edge can be recorded. It has both a debug assertion and a release-safe failure path.

`phalcom-semantic/src/db/query.rs` now centralizes formal semantic dependency replay/currentness, including typed products such as:

- `DeclarationShell`
- `DeclarationSurface`
- `CallableSignature`
- `FieldSignature`
- `HierarchyEdge`
- `LinkedInterface`
- `LinkedName`
- `PublicExport`
- `ResolvedImport`
- enum/associated products reached by formal queries

This is the correct invariant and must remain intact.

### Implicit hierarchy completeness

`phalcom-semantic/src/session.rs` now seeds hierarchy work from declaration additions/removals and publishes the implicit `Class → Object` semantic graph contribution for source classes without explicit superclasses.

The focused regression:

```text
a6_implicit_object_hierarchy_edge_is_published_for_new_declarations
```

checks hierarchy, declaration surface, graph edge, current query validation, and cold parity.

### PA-5 is now a valid fixture

The current `unused_public_export_input` makes the consumer genuinely use `Used`:

```phalcom
import unused_provider.Used

class Consumer {
  @class read() -> Int { Used.value() }
}
```

The test also rejects spurious dependencies on `Unused`.

Do **not** treat PA-5 fixture redesign as remaining implementation work. It only requires final certification.

### PA-8 is now a valid three-layer stability fixture

`phalcom-semantic/tests/semantic/incremental/product_stability.rs` contains an A→B→C fixture where:

- A changes;
- B actually recomputes;
- B republishes the same product fingerprint;
- C does not recompute;
- C computation revision remains old;
- C validation revision advances to current.

Do **not** redesign PA-8 again unless a later architectural patch breaks this invariant.

### PA-10 now uses a true fresh cold session

The 15-step parity test reinitializes the cold `SemanticWorkspaceSession` for each mutation state, rather than comparing two incremental histories.

The projection includes the important evidence-bearing presentation state:

- module products;
- declarations;
- dispatch surfaces;
- hierarchy;
- callable signatures;
- field signatures;
- aliases;
- diagnostics;
- semantic graph;
- callable analyses;
- source-index products;
- evidence/origin/return-validation information embedded in those products.

Preserve this as the integration oracle.

### Core comparator has been corrected

`scripts/verify_plan_a_core_baseline.sh` now:

- includes the `phalcom-core` library target;
- includes integration test targets;
- distinguishes PASS / FAIL / TIMEOUT-HANG;
- enumerates exact timeout test identities when a target hangs;
- compares failure identities;
- compares timeout identities;
- compares `(target, incident kind, test)` identities;
- compares target-level status regressions.

Do not rewrite this comparator unless execution exposes a concrete bug. The remaining task is to run it and record the exact result.

---

## 2.2 What remains genuinely incomplete

### A0 remains incomplete

`phalcom-modules/src/session.rs` has good source/module/source-alias `StagedMap` overlays, but the ordinary transaction is not yet fully private or O(delta).

Current issues include:

1. `reclassify_tracked_sources` still commits reclassified ownership maps to `self` before the final rebuild/late-failure barrier.
2. filesystem/provider cache invalidations happen before final commit.
3. `derive_rebuild` starts by cloning complete retained maps:

```rust
let mut interfaces = self.interfaces.clone();
let mut linked_modules = self.linked_modules.clone();
let mut import_products = self.import_products.clone();
let mut resolved_imports = self.resolved_imports.clone();
let mut reverse_importers = (*self.reverse_importers).clone();
let mut sites_by_importer = (*self.sites_by_importer).clone();
let mut reverse_site_importers = (*self.reverse_site_importers).clone();
```

4. removal uses broad operations such as:
   - `resolved_imports.retain(...)`;
   - scanning all `reverse_importers.values_mut()`.
5. interface discovery scans all tracked source states even when only one module changed.
6. the body-only fast path is reached only after several O(workspace) clones/scans.
7. body-only statistics currently count all import sites as considered/validated even though no import resolution should have been touched.
8. component partitioning is rebuilt from the complete interface/import universe for non-body edits.
9. `WorkspaceModuleUpdate` still republishes/clones workspace-sized product maps at the module→semantic boundary.

A0 therefore needs both a **transactional correctness closure** and a **derived-product delta/COW closure**.

### A6 remains incomplete

The semantic query graph is now much better, but orchestration still performs broad work.

Current ordinary-update broad paths include:

- rebuilding a fresh `semantic_structure_shards` map by walking all input sources;
- rebuilding source and field-lifecycle fingerprint maps by walking all sources;
- deriving `SemanticContributionDelta` from complete current/previous shard maps;
- repeatedly scanning `generic_header_dependencies` to discover reverse consumers;
- cloning/scanning full alias source/dependency maps;
- scanning the entire previous declaration table to retain declarations;
- building `initial_blueprints` from the complete retained declaration table;
- rebuilding retained semantic graph declaration edges by walking every current module;
- `SemanticGraph::declaration_edges_from_module` itself scanning graph nodes;
- republishing every linked interface on every update;
- constructing formal work at declaration-module granularity and scanning members to find exact identities;
- rebuilding retained callable analyses by filtering the complete previous analysis map;
- scanning all semantic shards and filtering by `semantic_work_modules` during body checking;
- rebuilding source-index publication state with broad map clones/scans;
- building advisory analysis across the full workspace;
- discarding/recomputing every advisory callable;
- rebuilding advisory module products broadly;
- rebuilding/flattening advisory workspace maps;
- rebuilding module-query snapshot maps from all sources/linked modules;
- computing publication effects via global old/new aggregate comparisons;
- reporting several counters as `changed_modules.len()` rather than measuring actual product work.

Task 28 has substantial implementation. Tasks 29–33 are not yet fully closed.

### A7 is not certified

Even where focused tests are green, the exact current pushed head does not have an attached CI result and the final post-C1–C5 full test matrix has not been established in the implementation ledger.

---

# 3. Non-Negotiable Completion Invariants

Use these invariants when reviewing every patch.

## PLAN-A-FINAL-1 — No pre-commit state mutation

Before the module transaction commit barrier, a failed update must leave unchanged:

- source/module identity maps;
- project/package ownership;
- provider overlays;
- provider current cache generation/state visible to later queries;
- interfaces;
- import products;
- reverse indexes;
- component products;
- linked products;
- diagnostics;
- generation.

A transient read-through view may differ. Committed state may not.

## PLAN-A-FINAL-2 — Ordinary update must not clone the retained workspace

An ordinary source edit must not copy complete maps merely to obtain private mutability.

Cold workspace creation, root reclassification, and explicitly documented cold materialization may be O(workspace).

## PLAN-A-FINAL-3 — Exact typed roots remain exact

Never fix a propagation bug by reopening an entire module when an exact product identity is available.

Examples:

```text
CallableBody(callable)
CallableSignature(callable)
FieldSignature(field)
DeclarationShell(declaration)
DeclarationSurface(declaration)
HierarchyEdge(declaration)
LinkedName(module, name)
PublicExport(module, name)
ResolvedImport(site)
```

## PLAN-A-FINAL-4 — Dependency recording requires current dependencies

Keep the current `SemanticDb::record_dependency` current-revision requirement.

Every query implementation must make its dependency current before recording it.

## PLAN-A-FINAL-5 — Product stability stops propagation

Recomputation is not itself a reason to invalidate downstream products.

If:

```text
old ProductFingerprint == new ProductFingerprint
```

then downstream exact consumers validate/reuse.

PA-8 is the canonical proof.

## PLAN-A-FINAL-6 — Retained observable contributions remain complete

Retaining a semantic product also retains every snapshot contribution derived from it:

- diagnostics;
- hierarchy;
- declaration metadata;
- dispatch;
- signatures;
- aliases;
- source attachments;
- advisory products where applicable.

## PLAN-A-FINAL-7 — Cold and incremental meaning are identical

Incrementality may change the path to a product, not the product.

Evidence level, evidence origin, validation state, diagnostics, and editor-visible source semantics are part of observable meaning where PA-10 compares them.

## PLAN-A-FINAL-8 — Historical cache identity is not current workspace identity

Old query/cache entries may remain cached, but must not become ordinary invalidation roots or current snapshot members after their source declaration/module disappears.

## PLAN-A-FINAL-9 — No ordinary total semantic traversal

For the production delta-driven path, ordinary edits must not traverse total workspace semantic state merely to filter it afterward.

Allowed broad paths:

- first cold build;
- explicit `set_workspace_roots` / ownership reclassification when the workspace root topology truly changes;
- bounded canonical Universe/bootstrap tables independent of workspace size;
- explicitly Plan-B-owned source/reference indexing internals only where Plan A is not adding broad semantic orchestration around them.

## PLAN-A-FINAL-10 — Release evidence is taken from one exact revision

The final ledger must identify one exact commit whose:

- modules suite;
- semantic suite;
- incremental suite;
- full LSP suite;
- PA matrix;
- core baseline comparison

all produced the recorded result.

---

# 4. Implementation Sequence

Execute in this order:

```text
F0  Re-certify current pushed C1–C5 state
F1  Finish A0 transaction purity
F2  Finish A0 derived-product delta/COW architecture
F3  Establish delta-owned semantic retained state
F4  Finish A6 declaration/header/alias reverse indexes
F5  Finish A6 hierarchy/graph and formal aggregate exactness
F6  Finish A6 exact callable/body/diagnostic aggregate retention
F7  Finish A6 snapshot/source publication composition
F8  Incrementalize advisory publication
F9  Add strict no-workspace-scan acceptance instrumentation
F10 Re-run PA-1…PA-10 and repair only real regressions
F11 Full LSP release gate
F12 Exact phalcom-core baseline gate
F13 Final ledger, scoped diff, commit, push, post-push verification
```

Do not reorder F10 ahead of F3–F9 and call focused green evidence “completion.” The point of F3–F9 is to remove the remaining architecture violations even if PA-10 is already semantically green.

---

# 5. F0 — Re-Certify the Current Pushed C1–C5 State

**Risk:** LOW
**Type:** verification-only unless a regression is reproduced

## Goal

Establish the exact pre-closure baseline at:

```text
6410ebcb3a0d1df5b86dc34d00ec820c88e96658
```

before changing A0/A6 architecture.

## Run

```bash
git rev-parse HEAD
git status --short
git log -n 12 --oneline

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental -- --test-threads=1
RUST_MIN_STACK=8388608 RUSTFLAGS='' cargo test -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-modules

RUSTFLAGS='' cargo test -p phalcom-lsp \
  --test imported_binding_resolution \
  imported_binding_definition_crosses_module_boundary_at_declaration_and_use \
  -- --exact

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  incremental::a7_performance::a7_unused_public_export_does_not_recompute_unrelated_consumer_body \
  -- --exact

RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  incremental::product_stability \
  -- --test-threads=1
```

Run the exact PA-6/PA-9/PA-10 tests present in `a7_performance.rs` as separate focused commands as well.

## Record

Create a new “Final Closure Baseline” section in the working implementation ledger, but **do not yet change checkpoint completion statuses**.

Record:

- head;
- exact test counts;
- any ignored tests;
- exact failures;
- whether focused PA-5/6/8/9/10 are green.

## Exit gate

No unexplained C1–C5 regression remains before architecture refactoring.

---

# 6. F1 — Finish A0 Transaction Purity

**Primary file:**
- `phalcom-modules/src/session.rs`

**Tests:**
- `phalcom-modules/tests/workspace_session.rs`
- `phalcom-modules/tests/checkpoint_a7.rs`

**Risk:** HIGH — ownership/module identity correctness

## 6.1 Make ownership reclassification pure before commit

### Current defect

The pre-commit path calls `reclassify_tracked_sources(...)`, and the helper writes:

```rust
self.modules_by_source = ...
self.sources_by_module = ...
self.project_roots = ...
self.standalone_projects = ...
self.synthetic_ids = ...
self.universe = ...
```

before `derive_rebuild` and before the late-failure seam.

### Patch

Split into:

```rust
struct ReclassificationDelta {
    modules_by_source: BTreeMap<SourceId, EntryDelta<ModuleId>>,
    sources_by_module: BTreeMap<ModuleId, EntryDelta<WorkspaceSourceState>>,
    project_roots: BTreeMap<ProjectSourceIdentity, EntryDelta<ResolvedProjectId>>,
    standalone_projects: BTreeMap<SourceId, EntryDelta<SyntheticProjectId>>,
    universe: Option<ProjectUniverse>,
    synthetic_ids: SyntheticProjectIdAllocator,
    changed_modules: BTreeSet<ModuleId>,
    removed_modules: BTreeSet<ModuleId>,
    identity_changes: BTreeSet<ModuleId>,
}
```

Rename/refactor:

```text
reclassify_tracked_sources
```

into a pure derivation helper such as:

```rust
derive_reclassification(...)
    -> Result<ReclassificationDelta, WorkspaceModuleSessionError>
```

It must read committed state and transaction overlays but not mutate `self`.

Replay the returned delta into the active transaction views.

Apply it to `self` only inside the single final commit block.

### Hostile regression

Add:

```text
failed_reclassification_transaction_does_not_mutate_committed_identity_state
```

Fixture:

1. establish package/project-owned sources;
2. mutate `package.ph` or another ownership marker so reclassification is required;
3. enable `late_failure_injected`;
4. execute mutation;
5. assert failure;
6. assert byte-for-byte/equality identity of:
   - `generation`;
   - source→module mappings;
   - module→source mappings;
   - project roots;
   - standalone project IDs;
   - module identities;
   - linked state;
   - topology;
   - diagnostics;
7. disable failure;
8. perform the same mutation successfully;
9. assert the expected identity transition.

This test must fail on the current pre-patch implementation.

---

## 6.2 Stage provider cache invalidation instead of mutating it

### Current issue

Content/topology cache invalidations occur before the final commit.

### Patch

Extend `StagedOverlayProvider` or introduce:

```rust
struct StagedSourceProvider<'a, P> {
    base: &'a OverlaySourceProvider<P>,
    ...
    content_invalidations: &'a BTreeSet<SourceId>,
    purged_identities: &'a BTreeSet<SourceId>,
    topology_invalidated: bool,
}
```

Reads through this view must bypass stale base-cache entries for staged-invalidated identities without changing the committed provider.

At commit:

```text
apply provider overlay operations
apply content invalidations
apply identity purges
apply topology invalidation
```

in a deterministic order.

### Regression

Extend the late-failure test to prove a failed `RefreshDisk`/ownership mutation does not change subsequent committed-provider behavior or metrics in a way that changes semantic output.

---

## 6.3 Establish the one-commit-barrier rule in code structure

Move all committed mutations into one visibly bounded section/function:

```rust
fn commit_transaction(
    &mut self,
    transaction: WorkspaceModuleTransaction,
    derived: DerivedWorkspaceDelta,
) -> WorkspaceModuleUpdate
```

No ordinary `apply_batch` path before this call may assign to committed workspace product fields.

Add a comment documenting `PLAN-A-FINAL-1`.

## F1 exit gate

- existing A0 hostile tests green;
- new reclassification late-failure test green;
- no committed module/provider state is mutated before `commit_transaction`.

Suggested commit:

```text
fix(modules): make ownership reclassification fully transactional
```

---

# 7. F2 — Finish A0 Derived-Product Delta/COW Architecture

**Primary files:**
- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/linker.rs` only if component facade support is required
- `phalcom-modules/src/graph.rs` only if retained graph shards need APIs
- `phalcom-modules/tests/checkpoint_a7.rs`
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/src/session.rs` for module→semantic delta plumbing

**Risk:** HIGH
**Goal:** An ordinary edit no longer clones complete module product maps before doing exact work.

---

## 7.1 Introduce a structurally shared retained-map primitive

The code now has transaction-local `StagedMap`, but committed immutable products/snapshots still need cheap sharing.

Prefer an established persistent ordered-map implementation rather than hand-writing a persistent tree. Before patching, inspect workspace dependencies for an existing structurally shared map/set. If none exists, add one small workspace dependency and wrap it behind an internal Phalcom type so public APIs do not depend directly on that crate.

Conceptual API:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RetainedMap<K, V> {
    // structurally shared immutable map
}

impl<K: Ord + Clone, V: Clone> RetainedMap<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V);
    fn remove(&mut self, key: &K) -> Option<V>;
    fn iter(&self) -> impl Iterator<Item = (&K, &V)>;
    fn keys(&self) -> impl Iterator<Item = &K>;
    fn len(&self) -> usize;
}
```

Required property:

```text
clone retained map = O(1)
update one key     = O(log N) structural path copy
```

Do not use `Arc<BTreeMap> + Arc::make_mut` as the final solution; the first mutation clones the full tree.

If adding a persistent-map dependency is rejected after repository review, implement an equivalent internal structurally shared map. Do not fall back to whole-map clone.

---

## 7.2 Move all retained module products onto structurally shared maps

Convert at minimum:

```text
interfaces
linked_modules
linked_dependency_fingerprints
import_products
resolved_imports
reverse_importers
sites_by_importer
reverse_site_importers
retained_components
module_components
diagnostics
blocked_modules
```

The transaction should operate as deltas over these retained maps.

Remove the full clones at the beginning of `derive_rebuild`.

---

## 7.3 Add exact reverse-index mutation helpers

Do not remove a module by scanning every reverse-index value.

Introduce exact helpers:

```rust
fn replace_import_site_product(...)
fn remove_import_site(...)
fn remove_importer(...)
fn replace_reverse_import_edge(...)
```

Use the existing:

```text
sites_by_importer
reverse_site_importers
```

to find exactly which import products and reverse relationships must be retracted.

If the legacy `(ModuleId, String) -> ModuleId` `resolved_imports` map needs efficient importer retraction, add:

```rust
resolved_names_by_importer: RetainedMap<ModuleId, RetainedSet<Box<str>>>
```

or eliminate the legacy duplication in favor of a projection from `ImportSiteId` products where possible.

---

## 7.4 Move the body-only fast path ahead of workspace-sized work

For an ordinary body edit:

1. inspect only explicitly changed modules;
2. rebuild/check only their unlinked interfaces;
3. compare interface fingerprints;
4. if no interface/product topology input changed:
   - do not enumerate all sources;
   - do not enumerate all import sites;
   - do not inspect components;
   - do not clone/rebuild linked maps;
   - publish retained linked/topology products immediately.

Correct body-only stats should resemble:

```text
interfaces_built = number of changed source interfaces checked
imports_resolved = 0
import_sites_considered = 0
linked_components_considered = 0
linked_components_recomputed = 0
```

Do not claim every retained import site was “validated” when no import-site validation was performed.

Update existing tests that encoded the older broad bookkeeping behavior.

---

## 7.5 Stop scanning all source states for changed interfaces/additions

With `changed_modules`, `removed_modules`, and transaction deltas available:

```rust
for module in &changed_modules {
    if let Some(state) = transaction.source(module) {
        ...
    }
}
```

Determine additions from transaction/base presence, not by walking every source.

Cold build remains allowed to iterate all states.

---

## 7.6 Recompute connected components only in the affected region

Current `compute_connected_components(&interfaces, &import_products)` partitions the whole workspace.

Replace ordinary-update use with affected-region partitioning.

### Seed affected region from

- changed interfaces;
- added modules;
- removed modules;
- identity changes;
- import-site products whose target changed;
- exposure/re-export topology changes.

### Expand through old component membership

For every seed module:

```text
old module_components[module]
    → all old members of that component
```

For a changed/new import edge joining previously separate components, add both endpoint components.

Then recompute connected components only over the induced affected region.

This handles:

- component split;
- component merge;
- removal;
- new edge;
- retarget.

Retain all unaffected `ComponentLinkedProduct` Arcs and module→component mappings.

Add:

```text
component_members_considered
```

to `WorkspaceModuleStats`.

---

## 7.7 Replace full module→semantic publication with an exact delta payload

Do not force `WorkspaceModuleUpdate` to clone complete sources/interfaces/import/diagnostic maps merely to hand the current generation to `SemanticWorkspaceSession`.

Introduce a production delta payload, e.g.:

```rust
pub struct WorkspaceModuleProductDelta {
    pub changed_sources:
        BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub removed_modules:
        BTreeSet<ModuleId>,

    pub changed_interfaces:
        BTreeMap<ModuleId, Option<Arc<UnlinkedModuleInterface>>>,

    pub changed_import_products:
        BTreeMap<ImportSiteId, Option<Arc<ImportResolutionProduct>>>,

    pub changed_diagnostics:
        BTreeMap<ModuleId, Arc<[ModuleDiagnostic]>>,

    pub changed_linked_interfaces:
        BTreeSet<ModuleId>,

    pub identity_changes:
        BTreeSet<ModuleId>,
}
```

`WorkspaceModuleUpdate` should contain:

- retained linked/topology/reverse-index products by structurally shared handle;
- exact delta;
- stats.

`SemanticWorkspaceSession::update_module_workspace` should consume this delta directly.

Keep `SemanticWorkspaceInput` as the explicit whole-workspace/cold/test construction surface if needed. The production LSP path must use the delta API.

---

## 7.8 A0 deterministic evidence

Extend the 10,000-source fixture.

For one body edit among 10,000 disconnected sources assert:

```text
source entries staged             = 1
module entries staged             = 1
source alias entries staged       = 0

source states considered          = 1
interfaces checked                = 1

import sites considered           = 0
imports resolved                  = 0

component members considered      = 0
linked components considered      = 0
linked components recomputed      = 0

full retained map materializations = 0
```

Add a batch-K fixture:

```text
10,000 sources
edit K=7 unrelated body-only modules
```

and prove staged/considered work is O(K), not O(10,000).

Add deletion and retarget variants to prove exact reverse-index cleanup.

## F2 exit gate

A0 can be marked `COMPLETE` only after:

- transaction purity from F1;
- ordinary derived product state no longer starts from full map copies;
- body-only work-count test proves O(delta);
- batch-K work-count test proves O(K);
- existing module topology/import/link tests remain green.

Suggested commits:

```text
perf(modules): retain module products with structural sharing
perf(modules): recompute only affected component regions
feat(modules): publish exact workspace product deltas
```

---

# 8. F3 — Establish Delta-Owned Semantic Retained State

**Primary files:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/tests/semantic/incremental/a7_performance.rs`

**Risk:** HIGH

## Goal

The production `apply_module_mutations` path must update semantic source/shard/fingerprint state only for changed/removed modules.

---

## 8.1 Split cold full-input update from production delta update

Keep:

```rust
SemanticWorkspaceSession::update(SemanticWorkspaceInput)
```

as a whole-world compatibility/cold/test API.

Introduce or formalize a distinct production path:

```rust
fn update_from_module_delta(
    &mut self,
    retained_module_products: &WorkspaceModuleProducts,
    delta: WorkspaceModuleProductDelta,
    ...
)
```

The LSP/module-session path must call this.

A full-input API necessarily has to inspect the supplied full map to discover differences; do not use that limitation to justify broad scans in the production delta path.

---

## 8.2 Update structure shards only for delta modules

Replace:

```text
new BTreeMap
for every input source:
    retain or reconstruct shard
```

with:

```text
retained shard state
for removed module:
    remove shard
for changed module:
    compare structural fingerprint
    replace shard or with_source(...)
```

No iteration over all current modules on an ordinary edit.

Track:

```text
semantic_structure_shards_considered
```

separately from recomputed/reused counts.

---

## 8.3 Compute `SemanticContributionDelta` only from changed shards

Replace whole-map:

```rust
contribution_delta(current_all_shards, previous_all_shards)
```

with:

```rust
SemanticContributionDelta::for_changed_module(
    previous_shard,
    current_shard
)
```

then union the O(K) module deltas.

For removal, the previous shard alone supplies all removed identities.

The delta must explicitly contain:

```rust
struct SemanticContributionDelta {
    structural_modules: BTreeSet<ModuleId>,

    declarations_added: BTreeSet<DeclarationId>,
    declarations_removed: BTreeSet<DeclarationId>,
    declaration_headers_changed: BTreeSet<DeclarationId>,

    hierarchy_edges_changed: BTreeSet<DeclarationId>,

    aliases_added: BTreeSet<DeclarationId>,
    aliases_removed: BTreeSet<DeclarationId>,
    aliases_changed: BTreeSet<DeclarationId>,

    callable_signatures_added: BTreeSet<CallableId>,
    callable_signatures_removed: BTreeSet<CallableId>,
    callable_signatures_changed: BTreeSet<CallableId>,

    field_signatures_added: BTreeSet<FieldId>,
    field_signatures_removed: BTreeSet<FieldId>,
    field_signatures_changed: BTreeSet<FieldId>,

    callable_bodies_added: BTreeSet<CallableId>,
    callable_bodies_removed: BTreeSet<CallableId>,
    callable_bodies_changed: BTreeSet<CallableId>,
}
```

Do not collapse added/removed/changed if later publication needs to retract exact identities.

---

## 8.4 Delta-maintain source/field lifecycle fingerprints

Do not reconstruct full `new_fingerprints` or `next_field_lifecycle_fingerprints`.

Update retained maps at changed/removed modules only.

## F3 tests

On a 5,000-module semantic fixture, body edit one module:

```text
semantic_structure_shards_considered = 1
semantic_structure_shards_recomputed = 0 or 1 as appropriate
field_lifecycle_modules_considered <= 1
contribution_delta_modules_considered = 1
```

Cold/full-input behavior remains unchanged.

Suggested commit:

```text
perf(semantic): derive source contributions from exact module deltas
```

---

# 9. F4 — Finish Declaration Headers and Alias Dependency Indexes

**Primary files:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/declarations.rs`
- `phalcom-semantic/src/type_alias.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- incremental tests

**Risk:** HIGH

---

## 9.1 Add exact reverse generic-header dependency index

Current code repeatedly scans:

```text
generic_header_dependencies: consumer → dependencies
```

to find consumers of changed declarations.

Retain both:

```rust
generic_header_dependencies:
    DeclarationId -> BTreeSet<DeclarationId>

generic_header_reverse_dependencies:
    DeclarationId -> BTreeSet<DeclarationId>
```

Provide one replacement helper:

```rust
fn replace_generic_header_dependencies(
    consumer: DeclarationId,
    new_dependencies: BTreeSet<DeclarationId>,
)
```

It must retract old reverse edges and install new ones atomically.

For deletion:

```rust
fn remove_generic_header_dependencies(consumer: &DeclarationId)
```

Worklist:

```text
changed declaration/header identities
    ↓
reverse-header closure
    ↓
generic_header_work
```

Do not scan all headers.

Add metric:

```text
generic_header_reverse_candidates_considered
```

---

## 9.2 Make declaration storage module-owned and structurally shared

`DeclarationTypeTable` currently has one flat `HashMap`.

Refactor internal storage so a snapshot can retain unchanged module declaration contributions without filtering the old global table.

Recommended logical structure:

```rust
pub struct DeclarationTypeTable {
    base: Arc<...Universe declarations...>,
    modules: RetainedMap<
        ModuleId,
        Arc<BTreeMap<DeclarationId, DeclarationTypeInfo>>
    >,
}
```

Because `DeclarationId` contains `module`, direct lookup is efficient:

```text
lookup declaration
    → base if Universe/bootstrap
    → modules[declaration.module][declaration]
```

Required APIs:

```rust
replace_module(...)
replace(declaration, info)
remove(declaration)
remove_module(module)
declarations_for_module(module)
get(...)
iter(...)
```

`iter()` may remain broad for explicit cold/debug output, but ordinary update orchestration must not call it over the whole table.

---

## 9.3 Retain declaration shell realization state

The current path builds `initial_blueprints` from all declarations before `DeclarationShellTable::realize_semantic_graph`.

Replace this with retained shell contributions.

Either:

1. refactor `DeclarationShellTable` to support exact insert/remove and retain it as a session product; or
2. replace this legacy realization use with the exact DB-owned hierarchy/shell products if those fully subsume it.

Preferred minimal patch: retained exact shell table.

Do not reconstruct all declaration blueprints on ordinary edits.

---

## 9.4 Shard alias source products by module

Replace flat cloned:

```text
alias_sources: DeclarationId -> (ModuleId, TypeAliasDef)
```

with module-owned immutable contributions from `ModuleSemanticStructureShard`:

```rust
alias_sources_by_module:
    RetainedMap<
        ModuleId,
        Arc<BTreeMap<DeclarationId, TypeAliasDef>>
    >
```

For direct lookup by declaration, use its owner module.

No `.retain(|declaration| declaration.module != changed_module)` over the whole map.

---

## 9.5 Add exact alias reverse dependencies

Retain:

```rust
alias_dependencies:
    alias -> direct alias dependencies

alias_reverse_dependencies:
    dependency -> direct alias consumers
```

Update both with exact replace/remove helpers.

Compute:

```text
changed alias seeds
    ↓
reverse alias closure
    ↓
aliases that must re-lower
```

Do not discover reverse consumers by repeatedly scanning all forward alias edges.

---

## 9.6 Keep alias SCC bounded, but measure the actual SCC region

The Plan explicitly allows a compact alias SCC operation.

`find_alias_cycles_from_seeds` may remain conceptually similar, but it must operate only over the affected alias region.

Set:

```text
alias_dependency_nodes_considered
```

to the number of alias graph nodes actually visited for the closure/SCC, not merely the number of aliases eventually recomputed.

Add a fixture with many disconnected alias SCCs:

```text
1,000 disconnected 3-node alias regions
edit one region
```

Assert SCC work is bounded to that region plus exact reverse consumers.

---

## 9.7 Generic-header/alias tests

Required:

1. body-only edit:
   - zero header reverse candidates;
   - zero alias SCC nodes;
2. generic bound target changes:
   - exact dependent header recomputes;
   - unrelated header retains computation revision;
3. alias `Number = Int` → `Number = String`:
   - exact cross-module alias consumer re-lowers;
4. alias retarget:
   - old reverse edge removed;
   - new reverse edge installed;
5. alias deletion:
   - exact consumers become unresolved;
6. unrelated alias SCC:
   - no relowering;
7. cold/incremental parity after every mutation.

Suggested commits:

```text
perf(semantic): index generic-header reverse dependencies
perf(semantic): shard alias sources and reverse dependencies
perf(semantic): retain declaration header contributions
```

---

# 10. F5 — Finish Hierarchy, Semantic Graph, and Formal Aggregate Exactness

**Primary files:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/types/relation.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-modules/src/graph.rs`
- `phalcom-semantic/src/signature.rs`
- `phalcom-semantic/src/dispatch.rs`
- incremental tests

**Risk:** HIGH

---

## 10.1 Remove the whole-workspace semantic-graph retention pass

Current logic reconstructs declaration edges by:

```text
for every current module
    previous.semantic_graph.declaration_edges_from_module(module)
```

and `declaration_edges_from_module` scans graph nodes.

Replace `SemanticGraph` declaration ownership with indexed/sharded contributions.

Conceptual structure:

```rust
pub struct SemanticGraph {
    module_edges: ...,
    declaration_edges:
        RetainedMap<
            ModuleId,
            Arc<BTreeMap<SemanticNodeId, Arc<[SemanticEdge]>>>
        >,
}
```

or an equivalent structure.

Required operations:

```rust
replace_module_projection(...)
replace_declaration_edges(declaration, edges)
remove_declaration(declaration)
remove_module_declarations(module)
edges_from(node)
nodes()
components()
module_projection()
```

A changed hierarchy declaration replaces exactly one declaration edge contribution.

A module/link graph change replaces only its affected module-level contribution.

Do not rebuild all retained source declaration edges.

---

## 10.2 Ensure implicit Object edge is one canonical product

The newly added implicit Object graph behavior must be unified with `HierarchyEdge(declaration)`.

Do not maintain one rule in the graph and a different rule in `MapTypeHierarchy`.

For every source class/enum where implicit Object applies:

```text
HierarchyEdge(declaration).super_decl == Object
semantic graph contribution           == declaration → Object
```

Both should be projected from the same query/product result or same normalized helper.

Keep the current C1 regression and add deletion parity.

---

## 10.3 Make hierarchy aggregate structurally shared

`MapTypeHierarchy` already has a module index and exact remove APIs, but snapshot construction still clones the whole table from the previous snapshot.

Move internal maps to structurally shared retained maps or module-owned shards so:

```text
clone snapshot hierarchy = O(1)
replace one hierarchy edge = O(log N)
```

Keep `remove_module` for true module deletion.

---

## 10.4 Add exact syntax indexes to `ModuleSemanticStructureShard`

Current formal work identifies exact IDs but then scans declarations/members inside each selected module to find their syntax.

During `ModuleSemanticStructureShard::from_source`, build retained lookup descriptors:

```rust
declaration_syntax:
    BTreeMap<DeclarationId, DeclarationSyntaxLocator>

callable_syntax:
    BTreeMap<CallableId, CallableSyntaxLocator>

field_syntax:
    BTreeMap<FieldId, FieldSyntaxLocator>
```

Use stable statement/member indices or compact cloned source descriptors; do not hold self-referential AST references.

Example:

```rust
struct CallableSyntaxLocator {
    statement_index: u32,
    member_index: u32,
    declaration: DeclarationId,
}
```

Provide:

```rust
fn declaration_syntax(&self, id: &DeclarationId) -> Option<...>
fn callable_syntax(&self, id: &CallableId) -> Option<...>
fn field_syntax(&self, id: &FieldId) -> Option<...>
```

---

## 10.5 Process exact formal worklists directly

Replace:

```text
formal_declaration_modules
    → scan all class members
    → if exact worklist contains identity, query it
```

with:

```text
for field in field_signature_work:
    locate exact field syntax
    query_field_signature(...)

for callable in callable_signature_work:
    locate exact callable syntax
    query_callable_signature(...)

for declaration in declaration_surface_work:
    locate exact declaration syntax
    query_declaration_surface(...)
```

Retain module-granular removal only for module deletion/reidentification.

---

## 10.6 Structurally share formal aggregate tables

The current exact remove APIs are good. Keep them.

Refactor backing storage for:

- `CallableSignatureTable`
- `FieldSignatureTable`
- `SurfaceDispatchResolver`

so cloning a previous snapshot does not copy complete workspace tables.

Prefer module/declaration-owned structurally shared contributions.

---

## 10.7 Preserve canonical body-derived return evidence

Keep the C3 behavior already introduced:

- if a formal signature is rebuilt with unchanged formal inputs, preserve valid body-derived `return_validation` / `inferred_return`;
- restore dispatch from canonical `signature.published_return_knowledge()`;
- if retained body evidence cannot be proven current, revalidate the exact owning body.

Move this rule into one named helper rather than duplicating it around class/body loops, e.g.:

```rust
fn canonicalize_callable_publication(
    previous: Option<&CallableSemanticSignature>,
    rebuilt: CallableSemanticSignature,
    body: Option<&CallableAnalysis>,
) -> CallableSemanticSignature
```

Add focused cold/incremental evidence-state regression.

## F5 exit gate

Task 30 and Task 32 can be marked complete when:

- hierarchy direct edges are exact and retained;
- graph composition is exact without all-module scans;
- exact formal IDs are queried directly;
- formal aggregates do not full-clone/rebuild;
- PA-9 remains green at scale.

Suggested commits:

```text
perf(semantic): retain exact hierarchy and graph contributions
perf(semantic): address formal products by exact source identity
perf(semantic): structurally share formal aggregate tables
```

---

# 11. F6 — Exact Callable Body and Diagnostic Aggregate Retention

**Primary files:**
- `phalcom-semantic/src/session.rs`
- new/existing callable-analysis aggregate support
- tests

**Risk:** HIGH

## 11.1 Introduce `callable_body_work`

Do not reduce exact query closure to only `semantic_work_modules`.

While processing reverse closure collect:

```rust
callable_body_work: BTreeSet<CallableId>
```

from:

- changed body roots;
- reverse dependencies reaching `CallableBody`;
- exact owning body revalidation required by canonical return evidence;
- field-lifecycle/constructor dependencies where genuinely required.

Also track top-level module body work separately if `<main>` is represented as a synthetic callable.

---

## 11.2 Stop querying every body in a semantic-work module

Current body loop walks all semantic shards and then all members of modules in `semantic_work_modules`.

Change to:

```rust
for callable in &callable_body_work {
    let shard = semantic_structure_shards
        .get(callable.module())
        .expect(...);

    let syntax = shard.callable_syntax(callable)
        .expect(...);

    query exact body
}
```

Constructors may require ordered processing. Build:

```text
constructor_body_work
ordinary_body_work
```

from the exact set, preserving constructor-first behavior only for affected constructors.

---

## 11.3 Retain callable analyses by module-owned shards

Current code filters the complete previous `callable_analyses` map.

Introduce a wrapper such as:

```rust
pub struct CallableAnalysisTable {
    modules: RetainedMap<
        ModuleId,
        Arc<BTreeMap<CallableId, Arc<CallableAnalysis>>>
    >,
}
```

Required compatibility methods:

```rust
get(callable)
values()
iter()
module(module)
replace_module(...)
insert(...)
remove(...)
remove_module(...)
```

Snapshot code/tests can continue using a table-like surface.

Ordinary edit:

```text
retain all unchanged module shards by structural sharing
replace only affected callable/module contributions
```

No full-map filter/collect.

---

## 11.4 Keep diagnostics module-owned

The diagnostic-retention fix is correct; preserve it.

Refactor backing storage if necessary so immutable snapshots share unchanged module diagnostic `Arc<[SemanticDiagnostic]>` contributions.

Effects must be computed only for `diagnostic_work_modules`, as already intended.

---

## 11.5 Exact body-work evidence

Add 5,000-module fixture:

- one provider body-only edit;
- 100 exact importers exist but provider signature stays stable.

Assert:

```text
callable_body_work_considered = 1
callable_bodies_recomputed = 1
callable_analysis_modules_considered = 1
external callable bodies recomputed = 0
```

For a signature change used by 100 consumers:

```text
callable_body_work_considered ≈ 101
```

not 5,001.

Suggested commit:

```text
perf(semantic): retain callable analyses and query exact body worklists
```

---

# 12. F7 — Finish Snapshot and Source Publication Composition

**Primary files:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/source_index/mod.rs`
- module query product types
- tests

**Risk:** HIGH

---

## 12.1 Do not rebuild module query maps from every source/linked module

Current snapshot freeze reconstructs:

- unlinked interfaces;
- linked interfaces;
- resolved import projection;
- source locations

from complete inputs.

Instead consume the retained module product snapshot/delta from F2.

`ModuleQueryProducts` should retain unchanged products by structural sharing.

Ordinary semantic publication should touch only exact module delta entries.

---

## 12.2 Narrow source-index DB publication without taking Plan B work

Plan B owns the deeper source/reference indexing performance design. Do not redesign reference semantics here.

Plan A must still stop broad semantic orchestration around the source index.

`build_source_semantic_index` already has:

```text
previous
rebuild_modules
```

Use that ownership to ensure:

- source scope/index rebuild only for `source_index_rebuild_modules`;
- `query_source_structure` only for rebuilt/removed modules;
- `query_source_formal_attachment` only for changed/recomputed/rebased callable attachments;
- unchanged source-index module shards are retained.

Do not loop over every source merely to query already retained `SourceStructure` products.

Cache canonical Universe presentation source shards once in session/bootstrap state rather than loading/reinserting them on every ordinary edit.

---

## 12.3 Replace global effect comparisons with exact delta effects

Current `module_graph_changed` and `declaration_index_changed` use broad old/new comparisons.

Add exact publication delta fields:

```rust
struct SemanticPublicationDelta {
    diagnostics_changed: BTreeSet<ModuleId>,

    declarations_added: BTreeSet<DeclarationId>,
    declarations_removed: BTreeSet<DeclarationId>,

    callables_added: BTreeSet<CallableId>,
    callables_removed: BTreeSet<CallableId>,

    fields_added: BTreeSet<FieldId>,
    fields_removed: BTreeSet<FieldId>,

    hierarchy_changed: BTreeSet<DeclarationId>,
    graph_changed: bool,

    source_index_changed: BTreeSet<ModuleId>,
    formal_changed: BTreeSet<ModuleId>,
    advisory_changed: BTreeSet<ModuleId>,
}
```

Derive these from actual transaction/query product changes, not from total old/new table set construction.

`declaration_index_changed` becomes:

```text
added/removed declaration/callable/field identity set non-empty
```

`module_graph_changed` becomes exact graph/import/topology product delta.

---

## 12.4 Fix misleading work metrics

Do not assign:

```rust
stats.source_indexes_recomputed = changed_modules.len();
stats.advisory_sources_recomputed = changed_modules.len();
stats.advisory_callables_recomputed = stats.callables_recomputed;
```

unless those products actually recomputed.

Increment counters at the product operation.

Add counters needed by F9.

---

## 12.5 Snapshot immutability test

Retain snapshots from revisions 1, 2, 3 while continuing to mutate the session.

Assert revision-1/2 aggregate queries remain unchanged after revision 3.

This proves structural sharing does not accidentally expose mutable session state.

Suggested commit:

```text
perf(semantic): compose snapshots from retained exact contributions
```

---

# 13. F8 — Incrementalize Advisory Publication

**Primary files:**
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/advisory/workspace.rs`
- advisory solver/analyzer files as needed
- `phalcom-semantic/src/db/query.rs`
- advisory incremental tests

**Risk:** HIGH

This is currently one of the clearest remaining A6 violations.

---

## 13.1 Make advisory query dependencies authoritative

Every `AdvisoryCallable` must record exact dependencies on the formal/source/advisory products it consumes.

Where advisory result of caller depends on advisory result of callee, record:

```text
AdvisoryCallable(caller)
    → AdvisoryCallable(callee)
```

Where it consumes formal semantics, retain exact existing typed dependencies.

Where a module product consumes callable summaries/source structure:

```text
AdvisoryModule(module)
    → SourceStructure(module)
    → relevant AdvisoryCallable(...)
```

No parallel advisory invalidation graph should exist outside `SemanticDb`.

---

## 13.2 Build an exact advisory worklist

Seeds:

- changed/recomputed `SourceStructure`;
- changed/recomputed formal callable products;
- changed callable bodies;
- changed relevant declaration/dispatch/hierarchy products.

Then:

```text
SemanticDb reverse closure
    ↓
advisory_callable_work
advisory_module_work
```

If advisory call relationships form cycles, process the affected advisory SCC/fixed-point region only.

Do not run the whole workspace solver because one callable changed.

---

## 13.3 Refactor `build_advisory_workspace`

Change from whole-workspace construction to:

```rust
build_advisory_delta(
    previous: &AdvisoryWorkspace,
    module_work: &BTreeSet<ModuleId>,
    callable_work: &BTreeSet<CallableId>,
    ...
) -> AdvisoryWorkspaceDelta
```

Retain unchanged:

```text
Arc<AdvisoryModuleProduct>
Arc<AdvisoryCallableSummary>
```

Recompute only affected shards.

Remove the current behavior that:

1. collects all previous advisory summaries;
2. bootstraps all;
3. `discard_for_recompute`s all advisory callables;
4. re-queries all advisory callables;
5. loops every advisory module.

---

## 13.4 Remove full flattened advisory workspace rebuild

`AdvisoryWorkspace::from_parts` currently flattens all module shard facts into workspace-wide maps every time.

Make per-module shards authoritative.

Because `SourceSiteId` carries owner identity, route lookup to the owning module/callable shard where possible.

Compatibility facade methods should preserve:

```rust
expression(site)
binding(site)
field(field)
parameter(slot)
callable(callable)
target(site)
```

without rebuilding full maps each revision.

For any necessary workspace fingerprint, maintain it incrementally from per-shard contribution fingerprints rather than rehashing every module/callable on each edit.

If a flat map remains necessary for an existing public API, materialize it lazily/cold, not in ordinary publication.

---

## 13.5 Advisory scale evidence

On a 5,000-module workspace with one body edit:

```text
advisory_modules_considered        <= exact affected region
advisory_callables_considered      <= exact affected call graph region
advisory_callables_recomputed      <= exact affected region
```

For an isolated callable:

```text
advisory modules recomputed = 1
advisory callables recomputed = 1
```

No 5,000-module sweep.

Keep all existing advisory correctness tests.

Suggested commits:

```text
perf(semantic): schedule advisory products through exact db dependencies
perf(semantic): retain advisory module and callable shards
```

---

# 14. F9 — Add a Strict “No Ordinary Workspace Scan” Gate

**Primary files:**
- `phalcom-modules/src/session.rs`
- `phalcom-semantic/src/session.rs`
- A7 tests

**Risk:** LOW after architecture patches

## Goal

Make it impossible to claim Plan A complete while hidden O(workspace) orchestration remains.

---

## 14.1 Extend module stats

Add deterministic counters such as:

```text
source_states_considered
interface_entries_touched
import_product_entries_touched
reverse_index_keys_touched
component_members_considered
full_product_materializations
```

`full_product_materializations` must remain zero on ordinary delta edits.

---

## 14.2 Extend semantic stats

Add:

```text
semantic_structure_shards_considered
contribution_delta_modules_considered

generic_header_reverse_candidates_considered
alias_reverse_candidates_considered
alias_scc_nodes_considered

hierarchy_work_items_considered
declaration_shell_work_items_considered
declaration_surface_work_items_considered
callable_signature_work_items_considered
field_signature_work_items_considered
callable_body_work_items_considered

callable_analysis_modules_considered

snapshot_module_contributions_touched
source_index_modules_considered

advisory_modules_considered
advisory_callables_considered

ordinary_workspace_scans
```

Increment `ordinary_workspace_scans` only at any remaining intentionally broad production-delta loop. Final acceptance requires:

```text
ordinary_workspace_scans == 0
```

Cold/full-input paths do not use this acceptance counter.

---

## 14.3 Add 10k semantic body-edit acceptance

Construct or extend deterministic scale fixture:

```text
10,000 modules
~100 exact semantic consumers
one body-only provider edit
```

Assert:

- module import/link work remains zero;
- semantic work is exact;
- no broad workspace scan counter;
- source/aggregate/advisory work does not grow with 10,000 unrelated modules.

Also run same logical edit at 1k, 5k, 10k and assert the exact-work counters remain equal or differ only by fixture-local exact consumers.

This is stronger evidence than wall-clock timing.

Suggested commit:

```text
test(plan-a): prove zero ordinary workspace scans
```

---

# 15. F10 — Final PA-1 Through PA-10 Acceptance Matrix

Do not rewrite already-valid fixtures unless architecture changes break them.

Run every acceptance test from one exact revision.

## PA-1 — body-only

Require:

```text
imports_resolved = 0
linked_components_recomputed = 0
cross-module semantic consumer recomputations = 0
ordinary_workspace_scans = 0
```

## PA-2 — one of 20 imports

Require exactly one re-resolution, 19 retained.

## PA-3 — unrelated creation with negative import

Require negative product reuse and zero re-resolution.

## PA-4 — missing target appears

Require only the relevant missing resolution to rerun.

## PA-5 — used vs unused export

Use the now-correct current fixture.

Require:

- positive `Used` dependency;
- no `Unused` dependency;
- consumer computation revision unchanged;
- consumer validation revision current;
- analysis retained;
- consumer body not recomputed.

## PA-6 — high fanout

Use ~5,000 reverse-connected / ~100 exact consumer fixture.

Require exact deterministic bounds and `ordinary_workspace_scans == 0`.

## PA-7 — disconnected components

Require unaffected components/products retained by identity where applicable.

## PA-8 — true stable intermediate

Use the current layered A→B→C fixture.

Require:

```text
B recomputed
B product fingerprint stable
C computation revision unchanged
C validation revision current
C not recomputed
```

## PA-9 — hierarchy/declaration isolation

Use required-scale fixture already present.

Require:

- one changed hierarchy edge;
- exact downstream hierarchy consumers recompute;
- unrelated hierarchy/surface/signature/body products retain computation revisions;
- cold hierarchy parity;
- no broad semantic scan.

## PA-10 — fresh-cold full presentation parity

Keep fresh cold session per mutation.

Run the complete 15-step sequence.

Require equality of the complete `SemanticParityProjection`.

If F3–F8 add new observable snapshot state, add it to the projection rather than silently omitting it.

## F10 exit gate

All PA-1…PA-10 green on one exact revision.

Suggested commit only if test fixture/instrumentation changes are required:

```text
test(plan-a): close final acceptance matrix
```

---

# 16. F11 — Full LSP Regression Gate

Plan A entered with full `phalcom-lsp` green.

Focused selective-import success is not sufficient.

Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-lsp
```

No Plan-A-window LSP regression may remain.

Specifically retain and verify:

```text
imported_binding_resolution::
imported_binding_definition_crosses_module_boundary_at_declaration_and_use
```

Both the selective import declaration and its usage must navigate to the exported declaration module.

If a failure occurs:

1. diagnose compiler-owned module/source/semantic product provenance first;
2. do not add LSP-local module identity, dependency, or source semantic authority;
3. do not introduce request-time AST rescans as a workaround;
4. fix the compiler snapshot/product boundary.

Run lifecycle/provider rename/delete tests explicitly after any LSP-related fix.

## Exit gate

Full LSP suite green at the same exact commit used for the PA matrix.

---

# 17. F12 — Exact `phalcom-core` Baseline Gate

Use the corrected script:

```bash
RUSTFLAGS='' ./scripts/verify_plan_a_core_baseline.sh
```

Baseline:

```text
d60e4589352ac5f4167ba295e7e2a5f6c870ef4b
```

The script must produce:

```text
current target/test incidents minus baseline target/test incidents:
<none>

status regressions:
<none>

RESULT: PASS
```

If the baseline integration lane is slow/hanging, let the script enumerate exact per-test incidents according to its timeout logic.

Do not classify by count alone.

Do not:

- fix unrelated core failures opportunistically;
- ignore tests;
- weaken assertions;
- change expected diagnostics to make the comparison green.

If current-only incidents exist, determine whether they are caused by Plan A. Any Plan-A regression must be repaired before completion.

Record the exact script output or a compact exact incident table in the final ledger.

---

# 18. F13 — Final Verification, Ledger Closure, Commit and Push

## 18.1 Required final commands

From the exact candidate revision:

```bash
RUSTFLAGS='' cargo test -p phalcom-modules

RUSTFLAGS='' cargo test -p phalcom-semantic \
  --test semantic incremental -- --test-threads=1

RUST_MIN_STACK=8388608 RUSTFLAGS='' \
  cargo test -p phalcom-semantic

RUSTFLAGS='' cargo test -p phalcom-lsp

RUSTFLAGS='' ./scripts/verify_plan_a_core_baseline.sh
```

Also run focused PA-1…PA-10 commands explicitly so their evidence is easy to record.

---

## 18.2 Formatting and diff hygiene

Run repository-appropriate formatting validation.

If a global formatting command fails solely because of known unrelated pre-existing files, format/check every touched Plan-A Rust file directly and record the unrelated global blocker rather than modifying unrelated code.

Run:

```bash
git diff --check
git status --short
git diff --stat
git diff -- <all intended Plan-A paths>
```

No temporary diagnostic logging, debug prints, disabled assertions, ignored tests, or benchmark-only semantic paths may remain.

---

## 18.3 Scope protection

The broad delivery commit at current head contains unrelated user files. Future Plan-A closure commits must not opportunistically modify unrelated areas.

Expected Plan-A closure files may include:

```text
phalcom-modules/src/session.rs
phalcom-modules/src/graph.rs
phalcom-modules/src/linker.rs
phalcom-modules/tests/workspace_session.rs
phalcom-modules/tests/checkpoint_a7.rs

phalcom-semantic/src/session.rs
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/advisory/*
phalcom-semantic/src/db/*
phalcom-semantic/tests/semantic/incremental/*

phalcom-lsp/*                 only if a compiler/LSP boundary regression requires it

scripts/verify_plan_a_core_baseline.sh
docs/work/modules/plan-a-implementation-state.md
```

Do not modify unrelated spec notes, Obsidian state, core runtime work, or other user-owned files unless a proven Plan-A regression requires it.

---

## 18.4 Rewrite the implementation ledger from current truth

`docs/work/modules/plan-a-implementation-state.md` is stale and contradictory.

Do not incrementally append more contradictory prose.

Rewrite the final A5–A7 state to match the exact final repository.

Required final matrix:

```text
A0 COMPLETE
A1 COMPLETE
A2 COMPLETE
A3 COMPLETE
A4 COMPLETE
A5 COMPLETE
A6 COMPLETE
A7 COMPLETE
```

A5 completion evidence must explicitly name:

- no all-source callable-body hash;
- no whole-linked-program callable-body hash;
- absent-name recovery;
- exact name/export dependencies;
- corrected PA-5;
- PA-6;
- corrected PA-8;
- PA-10 cold parity.

A6 completion evidence must explicitly name:

- per-module structural shards;
- exact contribution delta;
- declaration/header retained contributions;
- reverse header index;
- hierarchy direct-edge retention;
- semantic graph retained contributions;
- alias source shards;
- alias forward/reverse dependencies;
- bounded alias SCC;
- exact formal worklists;
- exact callable body worklists;
- retained callable analysis shards;
- retained diagnostic contributions;
- retained snapshot composition;
- incremental advisory products;
- `ordinary_workspace_scans == 0` evidence.

A7 completion evidence must explicitly name:

- PA-1…PA-10 PASS;
- modules PASS;
- semantic incremental PASS;
- semantic full PASS;
- LSP full PASS;
- core exact baseline PASS.

Remove stale references such as:

```text
latest semantic implementation commit: 5c5a9f82
missing PA-6/PA-9/PA-10
current LSP regression open
```

if no longer true.

---

## 18.5 Final commit discipline

Prefer coherent commits in this order:

```text
1. fix(modules): make ownership reclassification fully transactional
2. perf(modules): retain module products with exact transaction deltas
3. perf(modules): restrict linking to affected component regions

4. perf(semantic): derive source contributions from exact module deltas
5. perf(semantic): index generic header and alias reverse dependencies
6. perf(semantic): retain declaration hierarchy and graph contributions
7. perf(semantic): query exact formal and callable body worklists
8. perf(semantic): compose snapshots from retained exact contributions
9. perf(semantic): incrementalize advisory publication

10. test(plan-a): prove zero ordinary workspace scans
11. test(plan-a): close final acceptance and release matrix
12. docs(plan-a): record final completion evidence
```

If implementation naturally combines adjacent patches, keep commits reviewable; do not squash all architectural work into one opaque delivery commit.

Push the final commit(s).

Then verify remote `main` points at the tested completion commit.

---

# 19. Required Test Additions by Checkpoint

| Checkpoint | New/updated regression | Required property |
|---|---|---|
| F1 | late failure during ownership reclassification | no committed state mutation |
| F1 | failed staged provider invalidation | no committed provider semantic change |
| F2 | 10k one-source body edit | O(1) touched module products |
| F2 | 10k batch-K body edits | O(K) transaction/product work |
| F2 | component split/merge/retarget | affected-region partition only |
| F3 | 5k one-shard delta | one shard considered |
| F4 | generic reverse dependency fanout | exact reverse header closure |
| F4 | disconnected alias SCC population | bounded SCC region |
| F5 | graph contribution isolation | no all-module graph retention pass |
| F5 | exact signature/field/surface syntax lookup | no member scan for exact product |
| F6 | 5k body-only provider edit | one exact body work item |
| F6 | 100 exact signature consumers | ~101 body work items, not 5k |
| F7 | retained old snapshot after later edits | immutable snapshot remains unchanged |
| F7 | source/formal publication counts | only rebuilt module/callable queries |
| F8 | isolated advisory body edit in 5k workspace | one affected advisory region |
| F9 | ordinary scan counter | exactly zero |
| F10 | PA-1…PA-10 | all acceptance invariants |
| F11 | full LSP | zero failures |
| F12 | core baseline script | no current-only incident |

---

# 20. Performance Acceptance Rules

Do not use wall-clock time as the primary gate.

The final evidence must use deterministic work counts.

For a one-module body edit, workspace size `N` must not appear in the dominant ordinary-edit work counts.

Bad:

```text
N=1,000  → 1,000 semantic shards considered
N=5,000  → 5,000 semantic shards considered
N=10,000 → 10,000 semantic shards considered
```

Required:

```text
N=1,000  → 1 changed shard + exact dependency work
N=5,000  → 1 changed shard + exact dependency work
N=10,000 → 1 changed shard + exact dependency work
```

For ~100 exact consumers:

```text
work ≈ provider + 100 consumers + bounded exact support products
```

not 5,000 reverse module importers.

---

# 21. Explicit Non-Goals

Do not let final closure expand into Plan B.

Out of scope:

- parser incrementality;
- TypeStore COW redesign beyond what is strictly needed for Plan-A aggregate sharing;
- rename/refactor semantics;
- semantic-vs-textual reference-index redesign;
- LSP request scheduler redesign;
- editor overlay transaction redesign unrelated to compiler module ownership;
- VM representation;
- new language syntax/features;
- broad source/reference index optimization owned by Plan B;
- unrelated `phalcom-core` cleanup;
- general lint/warning cleanup.

A source-index change is allowed only when needed to stop Plan-A semantic orchestration from republishing every retained source/formal product; do not redesign its semantic model.

---

# 22. Stop Conditions

The implementation agent may return `PARTIAL` only if it can identify a concrete blocker with:

1. exact source location;
2. reproduced failing command/test;
3. why the blocker cannot safely be fixed within Plan A;
4. what invariant conflicts;
5. smallest proposed follow-on decision.

The following are **not** acceptable reasons to stop:

- focused tests are green;
- the remaining work is “mostly validation” while A0/A6 broad paths remain;
- a full suite has not yet been rerun;
- the patch is large;
- another agent could finish the ledger;
- PA-10 passes while ordinary workspace scans remain;
- current results “look proportional” without deterministic counters.

---

# 23. Final Definition of Done

Plan A is complete when the repository can truthfully demonstrate:

```text
one source changes
    ↓
only touched module transaction state is staged
    ↓
only changed module/interface/import/component products are derived
    ↓
only changed source-semantic contributions are compared
    ↓
only exact typed query roots are seeded
    ↓
only exact reverse semantic consumers are considered
    ↓
stable intermediate products stop propagation
    ↓
unaffected semantic/advisory/snapshot contributions remain structurally shared
    ↓
no ordinary total workspace semantic scan occurs
    ↓
incremental snapshot == fresh cold snapshot semantically/presentationally
    ↓
full LSP remains green
    ↓
no current-only phalcom-core baseline incident exists
```

And the final recorded matrix is:

```text
A0  COMPLETE
A1  COMPLETE
A2  COMPLETE
A3  COMPLETE
A4  COMPLETE
A5  COMPLETE
A6  COMPLETE
A7  COMPLETE

PA-1  PASS
PA-2  PASS
PA-3  PASS
PA-4  PASS
PA-5  PASS
PA-6  PASS
PA-7  PASS
PA-8  PASS
PA-9  PASS
PA-10 PASS

phalcom-modules             PASS
phalcom-semantic incremental PASS
phalcom-semantic full       PASS
phalcom-lsp full            PASS
phalcom-core baseline diff  PASS
```

Only then update the ledger to:

```text
PLAN A — COMPLETE
```
