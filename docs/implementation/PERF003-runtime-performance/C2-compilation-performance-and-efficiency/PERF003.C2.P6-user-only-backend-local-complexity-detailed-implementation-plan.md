# PERF003.C2.P6 — User-Only Backend / Local Complexity Closure
## Detailed Patch-Grade Implementation Plan

**Program:** `PERF003`  
**Checkpoint:** `PERF003.C2`  
**Normalized plan:** `PERF003.C2.P6`  
**Repository:** `aureat/phalcom-lang` (`phalcom`)  
**Target branch:** `perf/universe-characterization`  
**Prepared:** 2026-10-13  
**Predecessor:** `PERF003.C2.P5` — Canonical Publication + Persistent Image Boundary  
**Successor:** `PERF003.C2.P7` — `.telosc` Schema + Persisted Semantic Compiler Image  
**Primary objective:** make compiler projection and semantic lowering proportional to local runtime work, not to the size of the canonical Universe or the merged semantic world.

---

# 0. Repository synchronization note

The user reports:

```text
P4 complete and pushed
P5 partially implemented
P5 partial work committed and pushed to the remote branch
```

During preparation of this plan, the connected GitHub ref endpoint continued to resolve:

```text
refs/heads/perf/universe-characterization
→ c0155584972ec8c9f0960e411a9501a9c003de92
```

whose parent:

```text
764c2937a343364069d24c3664b96dce9d140b56
```

contains the visible P4 closure-driven metadata exporter/reachability cutover.

Because the user explicitly states that newer P5 work is already on the remote branch, **the implementer MUST treat the SHA above as an observation lag, not as the normative P6 starting revision**.

The first P6 action is therefore a bounded synchronization gate:

```text
git fetch origin
git rev-parse origin/perf/universe-characterization
git log --oneline --decorate -n 20 origin/perf/universe-characterization
```

Record the actual remote SHA as:

```text
P6_START_SHA
```

Then compare:

```text
c0155584972ec8c9f0960e411a9501a9c003de92..P6_START_SHA
```

Only inspect P5 changes touching the P6-owned symbols listed in §8.

Do not repeat P1–P4 investigation.

---

# 1. Executive implementation decision

P6 is a **backend locality and exact-indexing plan**.

P5 establishes:

```text
immutable canonical semantic parent
+
local semantic overlay/delta
```

P6 makes the compiler use that structure correctly.

The central invariant is:

> **Compiling one local runtime module must not iterate a whole canonical-aware semantic aggregate and then filter by module.**

The intended cost model is:

```text
O(local module products + exact referenced dependency lookups)
```

rather than:

```text
O(total canonical/local world × runtime-module count)
```

or:

```text
O(total trait requirements × trait invocation count)
```

or:

```text
O(callables × source occurrences)
```

The main source changes belong in:

```text
phalcom-semantic
    authoritative semantic indexes and exact lookup APIs

phalcom-core
    compiler/backend consumption of those indexed APIs

telos-observability
    structural work-cardinality counters and regression gates
```

P6 MUST NOT create a second independently synchronized backend semantic database.

---

# 2. Normalized program position

```text
P1  Canonical module / one-shot overlay boundary
P2  Complete canonical formal image + immutable queries
P3  Conformance/callable incrementalization
P4  Root-driven metadata + cross-pool authority
P5  Canonical publication + persistent image boundary
P6  User-only backend/local complexity closure
P7  .telosc schema + persisted semantic compiler image
P8  Portable executable Universe image
P9  Artifact-first loader/cache/toolchain certification
```

P6 is deliberately after P5 because module indexes must compose with the final parent/local aggregate model.

P6 is deliberately before P7 because loading `universe.telosc` quickly is insufficient if the backend then scans the loaded Universe for every user compile.

---

# 3. P6 outcome

At P6 completion:

```text
User runtime module M
    ↓
module-indexed semantic products for M
    ├── callable analyses
    ├── callable definitions
    ├── declaration facts
    ├── data semantics
    ├── enum semantics
    ├── intrinsic runtime facades
    ├── source attachments
    └── local lowering facts
        ↓
exact canonical/global lookup only when referenced
        ↓
module lowering
```

A local compile must not perform:

```text
scan all canonical declarations
scan all canonical data/enum entries
scan all canonical callable products
scan all local-workspace callables once per runtime module
scan every trait surface to resolve one trait requirement slot
scan all conditional member sets to resolve one ImplId
scan all occurrences once per callable to recover binding references
```

---

# 4. P5 predecessor contract

P6 assumes P5 supplies the following capabilities.

## 4.1 Canonical parent/local overlay

For canonical-seeded aggregate families:

```text
canonical facts = immutable parent
local facts     = local overlay
```

Local mutations must never deep-copy canonical entries.

## 4.2 Explicit local/merged iteration

Backend-relevant tables need cost-transparent APIs:

```text
exact lookup
local-only iteration
merged iteration
module-scoped iteration
canonical-only iteration where required for diagnostics/testing
```

## 4.3 Canonical copy gate

P6 must preserve the P5 acceptance condition:

```text
CanonicalAggregateEntriesCopied == 0
```

for ordinary warm image-backed session creation.

## 4.4 No local snapshot history aliasing

Preserve:

```text
canonical image != last_snapshot
canonical image != last_known_good
```

P6 may add indexes/views; it must not collapse canonical authority into local revision history.

---

# 5. Current source-grounded P6 targets

The following residual patterns are present in the audited source and must be rechecked at `P6_START_SHA`.

## 5.1 `compile.rs` runtime-root discovery

Owner:

```text
phalcom-core/src/modules/compile.rs
ProgramCompiler::project_analyzed_inner
```

Audited pattern:

```rust
for (callable, analysis) in analyzed
    .semantic
    .callable_analyses
    .iter()
    .filter(|(callable, _)| callable.owner.module() == id)
{
    ...
}
```

This makes module selection depend on the size of the entire analysis table.

## 5.2 `semantic_lowering.rs` repeated analysis scans

Owner:

```text
phalcom-core/src/modules/semantic_lowering.rs
build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls
```

Observed independent scans include:

```text
constructor-super initializer projection
binding projection
binding read-safety projection
main expression resolution projection
type-form value projection
pack construction discovery
anonymous product projection
anonymous product runtime-root discovery
```

## 5.3 Enum table filter

Audited pattern:

```rust
for (owner, enum_info) in &snapshot.enum_semantics.enums {
    if owner.module != *module {
        continue;
    }
    ...
}
```

## 5.4 Data table filter

Audited pattern:

```rust
for (owner, data_info) in &snapshot.data_semantics.data_decls {
    if owner.module != *module {
        continue;
    }
    ...
}
```

## 5.5 Intrinsic runtime declaration filter

Audited pattern:

```rust
snapshot
    .declarations
    .intrinsic_runtime_declarations()
    .filter(|(declaration, _)| &declaration.module == module)
```

## 5.6 Callable-definition filter

Audited patterns include:

```rust
for definition in snapshot.callable_definitions.values() {
    ...
    if impl_id.module != *module {
        continue;
    }
}
```

and:

```rust
snapshot
    .callable_definitions
    .values()
    .filter(|definition| definition.callable.module() == module)
```

## 5.7 Binding `callables × occurrences`

Audited lowering shape:

```text
for each callable in module
    build binding map
    scan every occurrence in module
    retain occurrences matching bindings owned by callable
```

This is approximately:

```text
O(callables × occurrences)
```

## 5.8 Global trait slot reconstruction

Audited helper:

```text
phalcom-core/src/modules/semantic_lowering.rs
trait_default_requirement_slots
```

builds one map by iterating:

```rust
for (_, surface) in snapshot.trait_surfaces.iter() {
    for (slot, (requirement, _)) in surface.iter().enumerate() {
        ...
    }
}
```

and `project_trait_invocation_spec` calls it again.

## 5.9 Exact `ImplId` recovered by global scan

Audited helper:

```text
phalcom-core/src/modules/compile.rs
inherent_impl_domain_for_callable
```

uses:

```rust
snapshot
    .dispatch
    .conditional_members()
    .values()
    .flat_map(|members| members.members.iter())
    .find(|member| &member.impl_id == source_impl)
```

## 5.10 Existing derived-rule index not used

`DerivedConformanceRuleIndex` already exposes:

```rust
get_rule(&ImplId)
```

but lowering still contains an `.iter().find(...)` path.

## 5.11 Existing source interval indexes

Current source already has:

```text
source_index::interval::RangeIndex
OccurrenceIndex::occurrence_at
ModuleSourceIndex::expression_site_at
SemanticSourceIndex target reverse lookup
```

Therefore the historical plan to create basic range indexes is obsolete.

---

# 6. Fixed P6 architectural invariants

## INV-P6-01 — Backend locality

For a fixed local runtime module, unrelated canonical population growth must not increase backend entity visitation.

## INV-P6-02 — Module selection is an indexed operation

Forbidden hot-path shape:

```rust
table.iter().filter(|entity| entity.module() == module)
```

for a canonical-aware or workspace-wide table.

Required shape:

```rust
table.iter_module(module)
```

or an equivalent module shard.

## INV-P6-03 — Exact identity uses exact lookup

If the caller has:

```text
CallableId
DeclarationId
ImplId
TraitRequirementId
SourceSiteId
```

it must not scan unrelated entries to recover the same entity.

## INV-P6-04 — Semantic owner owns semantic secondary indexes

Indexes needed to keep semantic products coherent MUST be maintained by the semantic aggregate that owns insertion/removal.

Do not create a core cache that must independently mirror semantic updates.

## INV-P6-05 — Parent-aware indexes remain parent-aware

For P5 parented tables:

```text
parent module index = immutable
local module index  = local
```

`iter_module()` may compose the two without materializing a merged table.

## INV-P6-06 — No parent shadowing

Canonical and local stable IDs are expected to be disjoint.

If a local insert collides with a parent identity:

```text
STOP / invariant failure
```

Do not silently shadow.

## INV-P6-07 — Determinism is preserved

Any ordering observable by:

```text
lowering
metadata
diagnostics
artifact persistence
reflection
```

must remain deterministic.

## INV-P6-08 — Trait slots are trait-local

Requirement slot numbering depends only on:

```text
the owning TraitSurface
+
generic-evidence prefix semantics
```

It must not depend on unrelated traits.

## INV-P6-09 — Existing interval semantics remain authoritative

Do not replace containment-aware `RangeIndex` behavior with exact-range maps for queries that mean "innermost containing site."

## INV-P6-10 — No full merged-map materialization hidden behind an index API

This is forbidden:

```rust
fn iter_module(&self, module: &ModuleId) -> impl Iterator<...> {
    self.materialize_merged_map()
        .into_iter()
        .filter(move |x| x.module() == module)
}
```

## INV-P6-11 — P6 does not define durable artifact identity

Do not change:

```text
stable declaration identity
stable callable identity
stable field identity
metadata dependency identity
artifact content identity
```

## INV-P6-12 — P6 does not implement executable AOT Universe state

Portable executable Universe compilation belongs to P8.

---

# 7. Non-goals

P6 MUST NOT absorb:

```text
.telosc wire encoding
artifact section layout
artifact schema versioning
fresh-process semantic image loading
artifact relocation
portable Chunk/Callable serialization
Universe initializer bytecode persistence
VM-local executable relocation
package publishing
remote cache protocol
```

P6 also MUST NOT reopen P4 metadata retention semantics or P5 canonical ownership merely for backend convenience.

---

# 8. P6 write-set and conflict map

Because P5 is still underway, the implementation must explicitly avoid conflicting with unfinished P5 ownership refactors.

## 8.1 Primary semantic files

```text
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs

phalcom-semantic/src/declarations.rs
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/enum_semantics.rs

phalcom-semantic/src/traits.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/derived_conformance.rs

phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/occurrence.rs
```

Possible new semantic table/index files only if justified after P5:

```text
phalcom-semantic/src/callable_analysis_table.rs
phalcom-semantic/src/callable_definition_table.rs
```

or a narrowly scoped internal secondary-index helper.

## 8.2 Primary core files

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

Potential test registration:

```text
phalcom-core/tests/core/mod.rs
```

## 8.3 Observability

```text
telos-observability/src/metric.rs
```

Prefer existing phases in:

```text
telos-observability/src/phase.rs
```

Do not add phases when counters inside existing phases are sufficient.

## 8.4 Lifecycle docs

At P6 activation/closure:

```text
docs/implementation/PERF003-runtime-performance/
  C2-compilation-performance-and-efficiency/
    CHECKPOINT.md
    PERF003.C2.P6-user-only-backend-local-complexity-implementation-plan.md
    PERF003.C2.P6-walkthrough.md
    PERF003.C2.P6-handoff.md
```

---

# 9. Task graph

```text
P6.T0  Live P5 delta/rebase gate
   ↓
P6.T1  Backend locality observability and baseline census
   ↓
P6.T2  Semantic module-index contract / shared secondary-index rules
   ├──────────────┬────────────────┐
   ↓              ↓                ↓
P6.T3          P6.T4            P6.T5
callable       declaration/     source binding
analysis/      data/enum        reverse lookup
definitions    indexes
   └──────┬───────┘                │
          ↓                        │
       P6.T6                       │
  core module-view cutover ◀───────┘
          ↓
   ┌──────┴────────┐
   ↓               ↓
P6.T7            P6.T8
trait slots      ImplId exact lookup
   └──────┬────────┘
          ↓
       P6.T9
 residual global-scan audit
          ↓
      P6.T10
 measured local pass/allocation cleanup
          ↓
      P6.T11
 structural scaling gates
          ↓
      P6.T12
 semantic/performance certification
          ↓
      P6.T13
 lifecycle closure + P7 handoff
```

---

# 10. P6.T0 — Live P5 delta/rebase gate

## Purpose

Bind P6 to the actual partial P5 implementation rather than to the stale connector snapshot.

## Required actions

Record:

```text
P6_START_SHA
P5_CURRENT_STATUS
P5_ACTIVE_TASK
P5_CHANGED_P6_OWNERS
```

Run:

```sh
git fetch origin
git status --short
git rev-parse HEAD
git rev-parse origin/perf/universe-characterization
git log --oneline --decorate -n 20 origin/perf/universe-characterization
```

If the P6 implementation worktree is not already on the remote head:

```sh
git rebase origin/perf/universe-characterization
```

or follow the repository's active branch policy.

Do not overwrite another implementer's dirty P5 work.

## Bounded delta audit

Inspect only P5 changes to:

```text
SemanticSnapshot::callable_analyses
SemanticSnapshot::callable_definitions

DeclarationTypeTable
CallableSignatureTable
FieldSignatureTable
MapTypeHierarchy
SurfaceDispatchResolver
TraitSurfaceTable
DataSemanticTable
EnumSemanticTable

SemanticWorkspaceSession::from_image
SemanticWorkspaceSession update-local aggregate construction
```

For each table, record:

```text
parent-aware? yes/no
local overlay? yes/no
iter_local? yes/no
iter_module? yes/no
clone cost O(local) or O(canonical)?
merged iteration materializes? yes/no
```

## Entry ruling

P6 may begin when:

```text
P5 ownership semantics for a touched aggregate are stable enough to extend
```

P6 does not need P5 final certification before work begins on independent exact-index tasks, but P6 final acceptance depends on P5's canonical ownership contract.

## STOP conditions

Stop if P5 is actively rewriting the same table representation in the same worktree/branch.

Do not land competing abstractions.

## Exit evidence

Add the actual P6 start SHA and table-shape census to:

```text
PERF003.C2.P6-walkthrough.md
```

---

# 11. P6.T1 — Backend locality observability and baseline census

## Purpose

Make residual scans measurable before replacing them.

## Primary files

```text
telos-observability/src/metric.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

## Required counters

Add counters with exact semantic intent.

Recommended names:

```rust
CompilerBackendCallableAnalysesVisited,
CompilerBackendCallableDefinitionsVisited,
CompilerBackendDeclarationsVisited,
CompilerBackendDataDeclarationsVisited,
CompilerBackendEnumDeclarationsVisited,

CompilerBackendCanonicalEntitiesVisited,
CompilerBackendEntitiesRejectedByModule,

CompilerBindingOccurrenceFullScans,
CompilerBindingOccurrencesVisited,
CompilerBindingOccurrencesResolvedByTargetIndex,

CompilerTraitRequirementLayoutsBuilt,
CompilerTraitRequirementSlotsLookedUp,
CompilerTraitSurfacesVisitedForSlotLookup,

CompilerInherentImplDomainLookups,
CompilerConditionalMembersScannedForImplDomain,

CompilerSourceExactSiteLinearScans,
```

Exact spelling may be adapted to `PerfCounter` naming conventions.

## Counter rules

Counters must represent **work cardinality**, not results only.

Example:

```text
CallableAnalysesVisited
```

increments when an analysis is inspected by the backend.

It must not mean:

```text
number of analyses eventually retained.
```

`CompilerBackendEntitiesRejectedByModule` should increment when code still visits an entity and then rejects it because its module differs.

The preferred post-P6 value in hot paths is:

```text
0
```

## Baseline workloads

Capture at minimum:

```text
A. tiny one-module user fixture
B. multi-module local project
C. generic/trait-heavy fixture
D. anonymous product fixture
E. conditional inherent impl fixture
```

Record:

```text
runtime module count
local callable analysis count
local callable definition count
local data count
local enum count
source occurrence count
visible canonical declaration count
visible canonical data/enum count
trait count
trait requirement count
conditional impl member count
```

## Existing phases

Use existing phases where possible:

```text
CompilerProjectionRuntimeRootDiscovery
CompilerProjectionMetadataExport
CompilerProjectionRuntimeTypeRecipes
CompilerProjectionGenericEvidence
CompilerProjectionConditionalCalls
CompilerProjectionTraitEnvironments
CompilerProjectionModuleLowering
```

No new phase is required merely to count module table visits.

## Acceptance

A machine-readable baseline demonstrates the pre-P6 cardinality defects.

Do not optimize first and reconstruct the baseline later.

---

# 12. P6.T2 — Define the semantic module-index contract

## Purpose

Establish one consistent pattern for module-indexed semantic tables before converting multiple families.

## Rule

Module indexes are **secondary indexes of authoritative semantic tables**.

They are not independent semantic products.

## Preferred internal shape

Where a table owns IDs:

```rust
struct Table {
    by_id: ...,
    by_module: HashMap<ModuleId, BTreeSet<EntityId>>,
    ...
}
```

For P5 parent-aware tables:

```rust
struct Table {
    parent: Option<Arc<Table>>,
    local_by_id: ...,
    local_by_module: HashMap<ModuleId, BTreeSet<EntityId>>,
    ...
}
```

## Required semantics

Every indexed table must support:

```rust
get(id)
contains(id)

iter()
iter_local()

iter_module(module)
iter_module_local(module)

local_len()
merged_len()      // only if efficiently available
```

Not every method must be public if no consumer needs it.

## Parent-aware module iteration

Representative:

```rust
pub fn iter_module<'a>(
    &'a self,
    module: &'a ModuleId,
) -> impl Iterator<Item = ...> + 'a {
    self.parent
        .as_deref()
        .into_iter()
        .flat_map(move |parent| parent.iter_module(module))
        .chain(self.iter_module_local(module))
}
```

This is acceptable only if parent recursion is bounded by the canonical-parent design and no merged map is allocated.

## Duplicate policy

On local insert:

```rust
if self.parent.as_ref().is_some_and(|p| p.contains(&id)) {
    invariant violation
}
```

Do not silently shadow canonical facts.

## Ordering

Use:

```text
BTreeSet<EntityId>
```

when deterministic ID order matters.

A `HashSet` is acceptable only where order is provably unobservable.

## Shared helper policy

Do NOT prematurely create a large generic index framework.

After two tables implement the same maintenance pattern, optionally extract a private helper such as:

```rust
SecondaryModuleIndex<Id>
```

with:

```rust
insert(module, id)
remove(module, id)
remove_module(module)
ids(module)
```

The helper must not own semantic values.

## Tests

Add focused unit tests for:

```text
insert updates module index
replacement does not duplicate
remove updates index
remove_module removes only local entries
parent/local iteration composes without copy
local-parent collision fails
clone copies local state only
deterministic module iteration
```

---

# 13. P6.T3 — Module-index callable analyses and callable definitions

## Purpose

Remove repeated workspace-wide callable filtering from compiler projection and lowering.

## 13.1 Callable analyses

Audited current snapshot shape:

```rust
pub callable_analyses:
    Arc<HashMap<CallableId, Arc<CallableAnalysis>>>
```

### Preferred target

If P5 has not already introduced an indexed table:

```rust
pub struct CallableAnalysisTable {
    by_id: HashMap<CallableId, Arc<CallableAnalysis>>,
    by_module: HashMap<ModuleId, BTreeSet<CallableId>>,
}
```

Possible location:

```text
phalcom-semantic/src/callable_analysis_table.rs
```

or a semantically appropriate existing module.

### Required API

```rust
pub fn get(&self, callable: &CallableId)
    -> Option<&Arc<CallableAnalysis>>;

pub fn insert(
    &mut self,
    callable: CallableId,
    analysis: Arc<CallableAnalysis>,
);

pub fn remove(&mut self, callable: &CallableId);

pub fn retain(...);

pub fn iter(&self) -> impl Iterator<...>;

pub fn keys(&self) -> impl Iterator<...>;

pub fn values(&self) -> impl Iterator<...>;

pub fn iter_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<Item = (&CallableId, &Arc<CallableAnalysis>)>;
```

If the table remains local-only by architectural design, do not add a canonical parent merely for symmetry.

If P5 has parented it, compose with the parent design.

### Migration rule

All mutation must go through index-aware methods.

Do NOT implement:

```rust
DerefMut<Target = HashMap<...>>
```

because it would permit mutation that bypasses the module index.

A read-only `Deref` is permissible only as a temporary compatibility aid and should be avoided if explicit methods are manageable.

### Session integration

Audit every current mutation in:

```text
phalcom-semantic/src/session.rs
```

including:

```text
insert
retain
remove
cycle settlement replacement
final revalidation replacement
downstream reanalysis replacement
```

Each must preserve `by_module`.

### Alternative only if P5 already has a better authority

If P5 introduces a stable local module shard containing exact analyzed callable IDs, P6 may use that shard instead of creating `CallableAnalysisTable`.

The alternative is acceptable only if:

```text
it includes every lowering-relevant analyzed callable
generated/default/conformance bodies are covered
lookup does not scan a global map
module identity is exact
```

Document proof.

---

## 13.2 Callable definitions

Audited shape:

```rust
Arc<BTreeMap<CallableId, EffectiveCallableDefinition>>
```

Preferred target:

```rust
pub struct CallableDefinitionTable {
    by_id: BTreeMap<CallableId, EffectiveCallableDefinition>,
    by_module: BTreeMap<ModuleId, BTreeSet<CallableId>>,
}
```

Likely semantic owner:

```text
phalcom-semantic/src/impls.rs
```

Required API:

```rust
get
insert
remove
remove_module
iter
values
iter_module
```

Index by:

```rust
definition.callable.module()
```

not only by `origin` module.

For inherent-impl-specific queries the `ImplId` exact index in T8 is separate.

---

## 13.3 Snapshot API

Expose module-bounded read methods from `SemanticSnapshot` if this prevents core from reaching into table internals:

```rust
pub fn callable_analyses_in_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<...>;

pub fn callable_definitions_in_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<...>;
```

Do not expose raw secondary-index maps to core.

---

## 13.4 Core cutover candidates

Replace global filters in:

```text
ProgramCompiler::project_analyzed_inner
build_module_lowering_semantics_*
project_type_form_values
canonical_pack_construction_ranges
project_anonymous_products
anonymous_product_type_roots
```

with the exact indexed API.

---

## 13.5 Verification

Unit/integration tests must include:

```text
multiple callables same module
callables in multiple modules
callable replacement
module removal
constructor body identity
trait default
conformance witness
generated delegation
conditional impl callable
```

Acceptance:

```text
CompilerBackendEntitiesRejectedByModule == 0
```

for converted callable loops.

---

# 14. P6.T4 — Module-index declarations, data semantics, and enum semantics

## Purpose

Remove whole canonical-aware table filtering from local module lowering.

This task must be implemented directly on top of P5-final parent-aware tables if they exist.

---

## 14.1 `DeclarationTypeTable`

Primary file:

```text
phalcom-semantic/src/declarations.rs
```

Audited current table owns:

```text
entries
intrinsic_static_domains
intrinsic_type_owners
```

### Required indexes

At minimum:

```rust
module_declarations:
    HashMap<ModuleId, BTreeSet<DeclarationId>>
```

For P5 parented table:

```text
local module_declarations only
parent owns canonical module_declarations
```

### APIs

```rust
pub fn iter_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<Item = (&DeclarationId, &DeclarationTypeInfo)>;

pub fn intrinsic_runtime_declarations_in_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<Item = (&DeclarationId, IntrinsicRuntimeOwner)>;
```

Avoid:

```rust
self.entries.iter().filter(...)
```

inside those APIs.

### Mutation maintenance

Update:

```text
store_authoritative
remove
remove_module / local remove equivalent
parent-aware overlay factory
intrinsic projection registration where declaration membership changes
```

`register_intrinsic_static_domain()` currently removes ordinary declaration entries; ensure module membership index mirrors that semantic change.

---

## 14.2 `DataSemanticTable`

Primary file:

```text
phalcom-semantic/src/data_semantics.rs
```

Add:

```rust
module_data:
    HashMap<ModuleId, BTreeSet<DeclarationId>>
```

or the P5-equivalent local index.

API:

```rust
pub fn iter_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<Item = (&DeclarationId, &Arc<DataInfo>)>;
```

Update:

```text
insert
replace
remove
remove_module
parent/local composition
```

---

## 14.3 `EnumSemanticTable`

Primary file:

```text
phalcom-semantic/src/enum_semantics.rs
```

Add:

```rust
module_enums:
    HashMap<ModuleId, BTreeSet<DeclarationId>>
```

API:

```rust
pub fn iter_module(
    &self,
    module: &ModuleId,
) -> impl Iterator<Item = (&DeclarationId, &Arc<EnumInfo>)>;
```

Maintain variant lookup behavior unchanged.

---

## 14.4 Parent/local constraints

For all three tables:

```text
local project module lookup:
    parent index miss O(1)
    local index hit

Universe module lookup:
    parent index hit
    local index miss
```

No canonical/local merged allocation.

---

## 14.5 Core lowering cutover

Replace:

```text
global enum scan + module comparison
global data scan + module comparison
global intrinsic-runtime declaration scan + module comparison
```

with module-indexed APIs.

Acceptance:

```text
CompilerBackendDataDeclarationsVisited
==
local module data declarations actually iterated

CompilerBackendEnumDeclarationsVisited
==
local module enum declarations actually iterated

CompilerBackendEntitiesRejectedByModule == 0
```

for these lanes.

---

# 15. P6.T5 — Replace binding `callables × occurrences` with reverse-target lookup

## Purpose

Eliminate one of the clearest multiplicative backend algorithms.

## Current semantic authority

The source index already provides:

```text
SemanticTargetId
SemanticSourceIndex::occurrences_for_target
SemanticSnapshot::occurrences_for_target
```

and exact source-site identities.

## Current problem

The backend builds per-callable binding maps and rescans:

```rust
module_index.occurrences.all()
```

for each callable.

## Required algorithm

For each formal binding:

```text
1. obtain canonical binding declaration SourceSiteId
2. form SemanticTargetId::Binding(declaration_site)
3. query reverse reference index
4. project only those reference sites
```

Representative pseudocode:

```rust
for (_, analysis) in snapshot.callable_analyses_in_module(module) {
    let Some(attachment) = snapshot.source_index().formal_attachment(&analysis.callable)
    else {
        continue;
    };

    for (binding, declaration_site) in &attachment.formal_bindings {
        if let Some(site) = snapshot.source_index().source_site(declaration_site) {
            bindings.insert(site.range, *binding);
        }

        let target = SemanticTargetId::Binding(declaration_site.clone());

        for occurrence_site in snapshot
            .occurrences_for_target(&target)
            .into_iter()
            .flatten()
        {
            let Some(site) = snapshot.source_index().source_site(occurrence_site)
            else {
                continue;
            };

            bindings.insert(site.range, *binding);
        }
    }
}
```

Exact API shape may differ.

## Semantics to preserve

Preserve:

```text
definition site
read references
write references
call references if binding-targeted
source range
owner identity
duplicate/coalescing semantics
shadowing
nested scopes
```

Do not infer binding identity from text names.

## Counter gates

Before cutover:

```text
CompilerBindingOccurrenceFullScans > 0
```

After:

```text
CompilerBindingOccurrenceFullScans == 0
```

Normal work should scale with:

```text
number of referenced binding target sites
```

## Tests

Add cases with:

```text
many unrelated occurrences
nested lexical scopes
same-name shadowing
multiple callables
read/write references
formal parameter binding
local binding
captured binding
```

Assert exact source ranges and `BindingId`s remain unchanged.

---

# 16. P6.T6 — Introduce a borrowed `ModuleLoweringInputView` and cut core over

## Purpose

Make it structurally difficult for core lowering to accidentally regress to whole-world iteration.

## Preferred shape

A core-side borrowed facade may compose semantic-owned indexes:

```rust
pub struct ModuleLoweringInputView<'a> {
    module: &'a ModuleId,
    snapshot: &'a SemanticSnapshot,
}
```

with methods:

```rust
fn callable_analyses(&self) -> impl Iterator<...>
fn callable_definitions(&self) -> impl Iterator<...>
fn data_declarations(&self) -> impl Iterator<...>
fn enum_declarations(&self) -> impl Iterator<...>
fn intrinsic_runtime_declarations(&self) -> impl Iterator<...>
fn source_index(&self) -> Option<&ModuleSourceIndex>
```

or a richer struct storing borrowed iterator authorities.

## Hard rule

Constructing the view MUST NOT:

```text
allocate vectors of all matching entities
materialize merged parent/local tables
clone semantic entries
scan canonical tables
```

It is a façade over semantic-owned indexes.

## Integration

Use one view inside:

```text
build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls
```

Pass the view to helper functions where practical.

Avoid functions reaching back into `snapshot.<global table>.iter()`.

## Runtime-root discovery

`compile.rs` should use the same semantic module-index authority, even if it does not reuse the exact core view type.

The invariant is shared data authority, not necessarily shared wrapper type.

## Compile loop

Where semantically equivalent, change:

```rust
for (id, linked_module) in &analyzed.linked.modules {
    if !analyzed.runtime_modules.contains(id) {
        continue;
    }
    ...
}
```

to direct runtime-module iteration with exact linked lookup:

```rust
for id in &analyzed.runtime_modules {
    let linked_module = analyzed
        .linked
        .modules
        .get(id)
        .ok_or(...)?;
    ...
}
```

Only do this where runtime-module order remains deterministic and identical or where order is unobservable.

## Acceptance

Core module lowering no longer contains converted patterns of:

```rust
.iter().filter(|...| module == ...)
```

over workspace/canonical semantic tables.

---

# 17. P6.T7 — Precompute per-trait requirement layouts

## Purpose

Replace global trait-slot reconstruction with trait-local semantic data.

## Primary file

```text
phalcom-semantic/src/traits.rs
```

## Current authority

`TraitSurface` owns:

```rust
pub members:
    BTreeMap<TraitRequirementId, TraitSurfaceMember>
```

The BTree order currently defines slot order indirectly through:

```rust
surface.iter().enumerate()
```

## Required representation

Preferred:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitRequirementLayout {
    slots: BTreeMap<TraitRequirementId, u32>,
    ordered: Arc<[TraitRequirementId]>,
}
```

Then:

```rust
pub struct TraitSurface {
    ...
    requirement_layout: TraitRequirementLayout,
}
```

An equally valid design is storing:

```text
slot: u32
```

inside a stable immutable member projection, provided ordering semantics remain identical.

## Why `u32`, not immediately `u16`

Current lowering performs:

```rust
u16::try_from(slot)
```

and can return `ProjectionError::SlotOverflow`.

Precomputing as `u32` preserves the ability to perform the same checked conversion at lowering time.

Do not silently truncate.

## Construction

`build_trait_surface` currently mutates `surface.members` and returns the completed surface.

Add a finalization step after all members have been inserted:

```rust
surface.finalize_requirement_layout();
```

or build the layout before return.

Layout order MUST exactly equal:

```rust
surface.members.keys()
```

in BTree order.

## APIs

```rust
pub fn requirement_slot(
    &self,
    requirement: &TraitRequirementId,
) -> Option<u32>;

pub fn requirement_layout(
    &self,
) -> &TraitRequirementLayout;
```

## Core cutover

Remove the global helper:

```text
trait_default_requirement_slots(snapshot)
```

or reduce it to a test/reference-only function during transition.

`project_trait_invocation_spec` should do:

```text
exact trait surface lookup
exact requirement slot lookup
checked u16 conversion
generic-evidence prefix adjustment
```

If a caller needs the entire slot map for one trait:

```text
use only that trait's layout
```

## Required parity invariants

Prove:

```text
old slot == new slot for every existing canonical trait
unrelated trait insertion does not affect slot
user-defined trait uses same mechanism
trait default body and caller agree
generic header evidence prefix remains unchanged
slot overflow remains fail-closed
```

## Regression test

Construct:

```text
Trait A with requirements a1, a2
Trait B with many unrelated requirements
```

Record A slots.

Add/reorder B-only content.

A slots MUST remain identical.

---

# 18. P6.T8 — Exact `ImplId` dispatch-domain lookup and derived-rule direct lookup

## Purpose

Replace exact-identity global scans with exact semantic indexes.

---

## 18.1 Inherent impl domain

Primary file:

```text
phalcom-semantic/src/dispatch.rs
SurfaceDispatchResolver
```

Current fields include:

```text
surfaces
conditional_members
conditional_targets_by_owner
type_declarations
module_surfaces
declaration_types
```

Add:

```rust
inherent_impl_domains:
    HashMap<ImplId, Arc<InherentImplDomain>>
```

Optional richer shape:

```rust
struct IndexedInherentImpl {
    target: InherentImplTarget,
    domain: Arc<InherentImplDomain>,
}
```

Use richer shape if target identity eliminates other scans.

## Registration

Inside:

```rust
register_conditional_members(
    target,
    members,
)
```

for each member:

```text
impl_id
domain
```

insert or validate existing domain.

If an existing `ImplId` has a non-equal domain:

```text
semantic invariant failure
```

`InherentImplDomain` is `Eq + PartialEq`; use that coherence property.

Do not overwrite silently.

## Removal

When removing:

```text
target
surface
module
```

remove `ImplId` entries that were contributed by removed member sets.

Because multiple members from one impl may share one domain, do not remove the domain until no retained member set references that `ImplId`.

Recommended bookkeeping:

```rust
impl_targets:
    HashMap<ImplId, InherentImplTarget>
```

or derive the set of unique impl IDs from the removed target set and remove each exact entry once.

If one impl is guaranteed to target exactly one `InherentImplTarget`, assert that invariant.

## API

```rust
pub fn inherent_impl_domain(
    &self,
    impl_id: &ImplId,
) -> Option<&Arc<InherentImplDomain>>;
```

Optional:

```rust
pub fn inherent_impl_target(
    &self,
    impl_id: &ImplId,
) -> Option<&InherentImplTarget>;
```

## Core cutover

Replace:

```text
compile.rs::inherent_impl_domain_for_callable
semantic_lowering equivalent domain lookup
```

global conditional-member scans.

`callable_definitions.get(callable)` may still be used to recover the source `ImplId`, then dispatch performs exact lookup.

---

## 18.2 Derived conformance rules

Current semantic index already has:

```rust
DerivedConformanceRuleIndex::get_rule(&ImplId)
```

Replace core lowering:

```rust
derived_conformance_rules
    .iter()
    .find(|rule| &rule.impl_id == impl_id)
```

with:

```rust
derived_conformance_rules
    .get_rule(impl_id)
```

No new semantic index is required.

## Acceptance

Post-cutover:

```text
CompilerConditionalMembersScannedForImplDomain == 0
```

Normal exact lookup count:

```text
CompilerInherentImplDomainLookups
```

may be nonzero.

---

# 19. P6.T9 — Residual global-scan audit

## Purpose

After the major indexes exist, remove remaining whole-world scans rather than assuming the obvious call sites were exhaustive.

## Scope

Audit:

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
```

and lowering helpers directly called from them.

Search patterns:

```text
.iter().filter(
.values().filter(
.keys().filter(
flat_map(
conditional_members().values()
trait_surfaces.iter()
callable_analyses.iter()
callable_definitions.values()
data_semantics.data_decls
enum_semantics.enums
declarations.iter()
```

## Classification

Every surviving scan must be classified:

```text
A. local module scan — acceptable if module-indexed
B. exact referenced dependency lookup — acceptable
C. true whole-program operation — must be semantically justified
D. accidental whole-world scan — fix in P6
E. P7/P8-owned operation — document and defer
```

## True whole-program operations

Do not force every operation to be local if semantics genuinely require a whole-program closure.

For each true whole-program scan, record:

```text
why it is global
when it runs
what population it visits
whether it runs on ordinary user compilation
whether canonical portion can be precomputed in P5/P7
```

If an ordinary user compile repeatedly performs a global canonical scan whose result depends only on canonical data:

```text
STOP / move precomputation to canonical authority
```

---

# 20. P6.T10 — Measured local-pass fusion and allocation cleanup

## Purpose

Only after asymptotic global scans are removed, optimize remaining demonstrated local hot paths.

## 20.1 Local pass fusion

Module lowering may still iterate local analyses multiple times.

Measure:

```text
local analysis passes
analyses visited per pass
duration per pass
```

If material, introduce a single collection pass for compatible facts:

```text
constructor super initializers
binding read safety
type-form values
pack construction facts
anonymous-product expression types
```

Do not fuse semantically unrelated error-producing operations if that changes ordering/failure behavior.

## 20.2 Allocation cleanup

Audit formatted runtime-root keys such as:

```rust
format!("trait-environment-...")
format!("call-environment-...")
```

Classify each as:

```text
transient lookup key
diagnostic/debug label
metadata persisted key
artifact-facing stable string
```

Only transient keys are automatic candidates for typed-key replacement.

Do not replace persisted/debug identities with opaque hashes merely for speed.

## 20.3 Temporary map churn

Profile temporary:

```text
BTreeMap
BTreeSet
Vec
String
```

allocation inside:

```text
runtime root discovery
generic evidence projection
conditional call projection
module lowering
```

Prefer small temporary allocation cleanup only when:

```text
counter/profile evidence exists
semantic ordering is unchanged
```

## 20.4 P5 boundary

Do NOT use this task to reopen:

```text
snapshot-wide canonical map materialization
canonical parent storage
local overlay storage
```

Those remain P5 architecture.

---

# 21. P6.T11 — Structural scaling tests and cardinality gates

## Purpose

Prevent hardware speedups from masking algorithmic regressions.

## Core test file

Create:

```text
phalcom-core/tests/core/compiler/backend_locality.rs
```

Register in:

```text
phalcom-core/tests/core/mod.rs
```

Recommended module name:

```rust
#[path = "compiler/backend_locality.rs"]
mod compiler_backend_locality;
```

## Semantic index tests

Use focused unit tests in owning semantic modules and, where cross-revision behavior matters, add:

```text
phalcom-semantic/tests/semantic/incremental/backend_indexes.rs
```

Register in:

```text
phalcom-semantic/tests/semantic/incremental/mod.rs
```

---

## 21.1 Canonical-size independence test

Hold local source constant.

Vary unrelated canonical/test-parent population.

Assert identical local backend visit counts.

If constructing two canonical parents is impractical, use a semantic-table unit fixture that exercises the same indexed contract.

## 21.2 Unrelated local-module independence

Workspace:

```text
A = runtime module
B...N = unrelated local modules
```

Compile A.

Assert A lowering does not visit B...N products.

## 21.3 Trait population independence

Keep one trait invocation fixed.

Add unrelated traits and requirements.

Assert:

```text
same requirement slot
same lowering output
same trait-surface visit count
```

## 21.4 Conditional impl population independence

Keep one selected inherent impl fixed.

Add unrelated conditional member sets.

Assert:

```text
exact same selected domain
scan count remains zero
lookup count remains constant
```

## 21.5 Binding occurrence scaling

Keep one binding reference fixed.

Add many unrelated occurrences.

Assert binding resolution work grows with references to the target binding, not total occurrences × callables.

## 21.6 Parent/local module index test

For each P5-parented table:

```text
canonical parent contains Universe module entries
local overlay contains project module entries
iter_module(Universe) returns parent entries
iter_module(local) returns local entries
no merged allocation/copy counter increments
```

---

# 22. P6.T12 — Semantic parity and performance certification

## Purpose

Prove indexed traversal changes complexity, not semantics.

## 22.1 Focused semantic parity

Cover:

```text
constructors
super initializers
formal/local bindings
binding read safety
captured bindings
data construction
enum construction
Tuple/Record/Pack intrinsic facades
inherent impls
conditional inherent impls
generated delegations
trait defaults
trait requirement calls
exact conformance witnesses
derived conformance
generic evidence
for iteration
associated resolution
callable references
anonymous tuples/records/packs
type-form values
static constants
runtime type recipes
metadata/reflection roots
```

## 22.2 Existing relevant suites

Run focused filters in the existing test binaries.

Examples:

```sh
cargo test -p phalcom-semantic --test semantic backend_index
cargo test -p phalcom-semantic --test semantic traits
cargo test -p phalcom-semantic --test semantic source_index
cargo test -p phalcom-semantic --test semantic derived_conformance

cargo test -p phalcom-core --test core backend_locality
cargo test -p phalcom-core --test core semantic_boundary
cargo test -p phalcom-core --test core declaration_dispatch
cargo test -p phalcom-core --test core product_optimization
```

Use actual test function/module filters after implementation; do not assume a filter that selects zero tests is a pass.

Record:

```text
number of tests selected
pass/fail/ignored count
```

## 22.3 Build checks

At minimum:

```sh
cargo check -p telos-observability
cargo check -p phalcom-semantic
cargo check -p phalcom-core --features perf-observability
```

Run standard workspace checks required by repository policy before closure.

## 22.4 Formatting

```sh
cargo fmt --all -- --check
```

## 22.5 Performance evidence

Use the existing PERF003 performance harness from the checkpoint/P1 instrumentation.

Measure:

```text
cold canonical build
warm semantic session attach
local semantic update
compiler projection
runtime root discovery
metadata export
generic evidence
trait environments
module lowering
materialization planning
overall fixture compile
```

P6 acceptance is based first on work cardinality, then wall-clock improvement.

---

# 23. Required structural acceptance gates

## G6.0 — P5 integration gate

Every modified P5-parented table:

```text
retains immutable parent semantics
copies zero canonical entries into local overlay
module index composes parent/local without merged materialization
```

## G6.1 — Callable backend locality

For a local module:

```text
CompilerBackendCallableAnalysesVisited
≈ local callable analyses actually consumed
```

No growth with unrelated canonical or local modules.

## G6.2 — Definition locality

```text
CompilerBackendCallableDefinitionsVisited
```

tracks module definitions, not workspace definitions.

## G6.3 — Data locality

```text
CompilerBackendDataDeclarationsVisited
==
module data declarations consumed
```

## G6.4 — Enum locality

```text
CompilerBackendEnumDeclarationsVisited
==
module enum declarations consumed
```

## G6.5 — Zero module-rejection after indexed selection

Preferred hot-path gate:

```text
CompilerBackendEntitiesRejectedByModule == 0
```

## G6.6 — Binding reverse-reference gate

```text
CompilerBindingOccurrenceFullScans == 0
```

## G6.7 — Trait slot locality

Normal requirement lookup:

```text
CompilerTraitSurfacesVisitedForSlotLookup <= 1
```

No global trait slot-map construction.

## G6.8 — Impl exact lookup

```text
CompilerConditionalMembersScannedForImplDomain == 0
```

## G6.9 — Canonical backend visitation

For ordinary local lowering:

```text
broad canonical entities visited == 0
```

Exact referenced canonical lookups are allowed and must be counted separately if needed.

## G6.10 — P5 copy gate preserved

No increase in:

```text
CanonicalAggregateEntriesCopied
SnapshotFullMapMaterializations
```

relative to P5 closing baseline.

## G6.11 — Deterministic output parity

Compiled/lowered observable outputs and diagnostics remain equivalent.

---

# 24. Detailed table mutation requirements

This section is normative for every new secondary index.

## 24.1 Insert

When an entity is inserted:

```text
1. validate parent collision if parented
2. remove/update prior local value if replacing
3. insert by-id value
4. insert ID into module index
5. verify module in index matches entity identity
```

## 24.2 Replace

Replacement with the same semantic ID but a different module should normally be impossible because module is part of canonical identity.

If observed:

```text
STOP / identity invariant violation
```

Do not migrate secondary index across modules silently.

## 24.3 Remove

Remove from:

```text
by-id
module ID set
auxiliary indexes
```

Delete an empty module bucket.

## 24.4 Remove module

Use the module ID set to find local IDs.

Do not:

```rust
by_id.retain(|id, _| id.module() != module)
```

for large parent-aware tables once the module index exists.

## 24.5 Clone

Clone complexity:

```text
O(local entries + local index state)
```

for P5 overlay tables.

No canonical entry copy.

## 24.6 Equality/debug

If table `Eq/PartialEq` is used in tests:

```text
semantic equality must remain meaningful
secondary indexes must be internally consistent
```

It is acceptable for derived secondary indexes to participate in equality only if deterministic and guaranteed coherent.

---

# 25. Callable analysis table migration strategy

Changing `SemanticSnapshot::callable_analyses` can touch many semantic session helpers.

Use a controlled migration.

## Slice A — introduce table type

Implement:

```text
by-id storage
module index
read APIs
index-aware mutation APIs
unit tests
```

No core cutover yet.

## Slice B — migrate session mutations

Use compiler errors to find every mutation that previously touched raw `HashMap`.

Convert all:

```text
insert
remove
retain
replacement
cycle settlement
revalidation
```

Do not expose a mutable underlying map.

## Slice C — snapshot field type

Change:

```rust
Arc<HashMap<...>>
```

to:

```rust
Arc<CallableAnalysisTable>
```

only after session helpers compile against the abstraction.

## Slice D — core consumers

Cut runtime-root discovery and semantic lowering to `iter_module`.

## Alternative

If P5 already introduced an authoritative module-partitioned callable-analysis product, use it instead and document why it covers all lowering callables.

Do not maintain two indexes.

---

# 26. Callable definition table migration strategy

The callable definition change should mirror analyses but is smaller.

## Required ownership

`EffectiveCallableDefinition` is semantic authority.

Keep exact ID lookup:

```text
O(log n) or O(1)
```

and add module iteration.

## Origin versus callable module

Index by:

```rust
definition.callable.module()
```

because module lowering consumes callable ownership.

For exact inherent `ImplId` behavior, use T8's dispatch index.

Do not overload one index for both dimensions.

---

# 27. Trait layout migration details

## 27.1 Construction timing

`TraitSurface` is built mutable and then published behind `Arc`.

Compute `requirement_layout` only after all `members.insert(...)` operations have completed.

## 27.2 Stable ordering

Current authoritative order is:

```rust
BTreeMap<TraitRequirementId, ...>
```

Therefore:

```rust
for (slot, requirement) in surface.members.keys().enumerate()
```

is the reference algorithm.

## 27.3 Slot storage

Preferred:

```text
u32
```

with checked conversion in lowering.

Do not make surface construction fail solely because runtime lowering uses `u16`, unless the language/compiler contract is intentionally changed through a separate ruling.

## 27.4 Fingerprints

Audit whether `TraitSurface` fingerprint/equality/publication uses fields structurally.

A derived slot layout that is purely a function of members MUST NOT accidentally alter semantic fingerprints unless intended.

Prefer either:

```text
exclude derived layout from semantic fingerprint
```

or guarantee fingerprint remains equivalent because it hashes the same member ordering.

Do not create spurious incremental invalidation.

---

# 28. Dispatch `ImplId` index migration details

## 28.1 Registration

One inherent impl can contribute several members.

All members with one `ImplId` must reference equal:

```text
InherentImplDomain
```

and the same target.

Assert this.

## 28.2 Removal bookkeeping

Because `remove_surface` may remove several conditional targets, collect unique impl IDs from each removed set before deleting the sets.

Then remove those impl IDs from exact indexes.

If an impl can legally appear in multiple retained targets, model ref-count/target-set semantics explicitly rather than deleting prematurely.

Investigate before coding; do not assume uniqueness without checking current impl construction.

## 28.3 Parent-aware dispatch

If P5 parents `SurfaceDispatchResolver`:

```text
canonical impl domains live in parent
local impl domains live locally
```

Exact lookup:

```text
local first only if stable-ID spaces permit local/canonical collision
otherwise assert disjoint and use local/parent deterministic lookup
```

No canonical map copy.

---

# 29. Source-index exact-site follow-up

P6 does not recreate range indexes.

## Existing indexed paths

Keep:

```text
OccurrenceIndex::occurrence_at
ModuleSourceIndex::expression_site_at
RangeIndex
target reverse reference index
```

## Remaining linear paths

Audited:

```text
OccurrenceIndex::occurrence_for_site
SemanticSourceIndex::source_site fallback over attachments/expression sites
```

### Rule

Instrument first.

If:

```text
CompilerSourceExactSiteLinearScans
```

is negligible after binding reverse-index cutover, defer.

If hot, exploit `SourceSiteId.owner`.

For callable-owned sites:

```text
module
→ attachment for callable
→ owner-local exact site map/index
```

Do not add a large global duplicate structure if owner-local identity already gives the route.

---

# 30. Failure and rollback invariants

P6 secondary indexes must not compromise semantic transaction behavior.

## 30.1 Failed semantic update

If session update rolls back:

```text
authoritative by-id tables
secondary module indexes
dispatch ImplId indexes
```

must all return to the same prior revision.

## 30.2 No partially updated index

Do not mutate an index outside the same update-local table instance that owns the values.

## 30.3 Canonical parent

Canonical parent indexes are immutable and never part of local undo mutation.

## 30.4 Tests

Add failure-path tests where practical:

```text
local module update introduces semantic failure
session preserves previous valid snapshot
module index still exposes previous products
new invalid products do not leak into indexes
```

---

# 31. Performance model and expected effects

P6 should document before/after complexity.

## Callable analysis selection

Before:

```text
O(runtime modules × all analyses)
```

After:

```text
O(sum of analyses in runtime modules)
```

## Data/enum lowering

Before:

```text
O(runtime modules × all data/enum declarations visible)
```

After:

```text
O(data/enum declarations owned by runtime modules)
```

## Binding projection

Before:

```text
O(callables × occurrences)
```

After:

```text
O(bindings + references to those binding targets)
```

## Trait slots

Before:

```text
O(all trait requirements × trait invocation count)
```

After:

```text
O(1) or O(log requirements-in-owning-trait) per lookup
```

plus one layout construction when trait surface is published.

## Inherent impl domain

Before:

```text
O(all conditional members) per exact impl lookup
```

After:

```text
O(1) expected / O(log n) deterministic-map lookup
```

---

# 32. Commit slicing

P6 should land as reviewable commits.

## P6.A — activation + counters

Files:

```text
CHECKPOINT.md
P6 plan/walkthrough
telos-observability/src/metric.rs
minimal instrumentation call sites
```

No semantic behavior change.

## P6.B — callable analyses/definitions indexes

Files:

```text
semantic table owners
snapshot/session
focused tests
```

No core cutover until semantic indexes are proven.

## P6.C — declaration/data/enum module indexes

Files:

```text
declarations.rs
data_semantics.rs
enum_semantics.rs
P5 parent integration
focused unit tests
```

## P6.D — core module-local cutover

Files:

```text
compile.rs
semantic_lowering.rs
backend_locality tests
```

Includes binding reverse-target cutover if review size permits; otherwise split.

## P6.E — trait layout

Files:

```text
traits.rs
semantic_lowering.rs
trait tests
```

Keep separate because slot semantics are high-value and review-sensitive.

## P6.F — dispatch exact impl index

Files:

```text
dispatch.rs
compile.rs
semantic_lowering.rs
derived-rule callsite
tests
```

## P6.G — residual audit + measured cleanup

Only evidence-backed changes.

## P6.H — certification/lifecycle

```text
cardinality gates
full focused verification
P6 walkthrough
P6 handoff
CHECKPOINT transition
```

---

# 33. Forbidden implementation shortcuts

Do NOT:

```text
create a core-side HashMap cache mirroring semantic tables
rebuild a module index by scanning the canonical world per compile
materialize parent + local merged maps merely to filter them
use source names instead of semantic IDs for binding lookup
make trait slots depend on global trait order
serialize/cache P6 indexes as a substitute for P7 artifact design
switch RuntimePublic to RuntimeMinimal to make tests fast
delete canonical metadata/reference authority
hide scans behind an API called iter_module
use elapsed time as the only acceptance gate
accept a cargo test command that selected zero tests
```

---

# 34. STOP / CONSULT triggers

The implementer MUST stop and produce an incident/consultation packet if:

## 34.1 P5 table ownership is still changing under the same write set

Coordinate before editing.

## 34.2 `iter_module` requires full canonical/local materialization

Fix the aggregate/index composition first.

## 34.3 Backend semantically requires enumerating all canonical entities

Document why; determine whether result should be precomputed into the canonical image.

## 34.4 Trait slot identity changes when unrelated traits change

This contradicts the current semantic contract.

## 34.5 One `ImplId` has conflicting domains/targets

Treat as semantic coherence failure.

## 34.6 Exact source-site semantics cannot be preserved using existing owner identity

Do not invent fuzzy range matching.

## 34.7 An index cannot participate correctly in session rollback

Do not land an eventually-consistent index.

## 34.8 Stable artifact/metadata identity would need to change

That requires separate architecture review.

---

# 35. P6 walkthrough requirements

`PERF003.C2.P6-walkthrough.md` must record:

```text
P6_START_SHA
P5 state at entry
files changed by P5 since the P6 analysis
table-shape census
pre-P6 cardinality baseline

task-by-task implementation
exact commits
exact tests and selected test counts
all failures and classification
structural counters before/after
wall-clock measurements before/after
deferred source-site/micro-optimization items
P7 handoff
```

For every converted table include:

```text
old iteration shape
new index shape
mutation maintenance
parent composition
clone/materialization behavior
```

---

# 36. P6 final acceptance checklist

P6 is complete only when all applicable checks are true.

## Architecture

- [ ] backend local module iteration no longer filters whole canonical-aware aggregates
- [ ] semantic table owners maintain secondary indexes
- [ ] P5 parent/local ownership remains intact
- [ ] no hidden merged-map materialization in module index APIs
- [ ] stable IDs are unchanged

## Callable products

- [ ] callable analyses have module-local iteration
- [ ] callable definitions have module-local iteration
- [ ] runtime-root discovery uses module-local analyses
- [ ] module lowering uses module-local analyses/definitions

## Structural products

- [ ] declarations have module-local access needed by lowering
- [ ] data semantics have module-local iteration
- [ ] enum semantics have module-local iteration
- [ ] intrinsic runtime facades use module-local declaration access

## Source binding

- [ ] no callable × all-occurrences binding scan
- [ ] reverse semantic target references are authoritative
- [ ] shadowing/reference parity passes

## Traits

- [ ] requirement slots are precomputed per trait surface
- [ ] unrelated traits do not affect slots
- [ ] no global trait-slot map rebuild per invocation

## Dispatch

- [ ] exact `ImplId` resolves inherent impl domain directly
- [ ] derived conformance uses existing `get_rule`
- [ ] no conditional-member global scan for exact impl domain

## Observability

- [ ] backend visitation counters exist
- [ ] module-rejection counter is zero for converted hot paths
- [ ] binding full-scan counter is zero
- [ ] trait global-scan counter is zero
- [ ] conditional impl scan counter is zero
- [ ] P5 canonical-copy counters do not regress

## Verification

- [ ] semantic focused tests pass
- [ ] core focused tests pass
- [ ] rollback/incremental tests pass
- [ ] formatting passes
- [ ] required crate checks pass
- [ ] performance harness measurements recorded

---

# 37. P7 handoff contract

P7 receives a compiler with two important properties.

## 37.1 Canonical authority is persistable

From P5:

```text
explicit immutable canonical semantic authority
persistence inventory
canonical metadata authority
eligible frozen queries
```

## 37.2 Loaded canonical authority is cheap to consume

From P6:

```text
module-indexed backend semantic tables
trait-local requirement layouts
exact ImplId domain lookup
reverse source target lookup
no broad canonical scans during user lowering
```

P7 can therefore implement:

```text
universe.telosc
```

without merely changing:

```text
slow source reconstruction
```

into:

```text
fast load + slow repeated whole-Universe backend scans.
```

---

# 38. Final completion statement

The correct P6 completion statement is:

> **Compiler projection and semantic lowering now consume module- and identity-indexed semantic authority. Local runtime compilation visits only the selected module's semantic products plus exact referenced dependencies. Callable analyses, callable definitions, declaration/data/enum products are module-addressable; binding projection uses semantic target reverse references; trait requirement slots are trait-local precomputed layouts; inherent-impl domains and derived-conformance rules use exact identity lookup. These indexes compose with P5's immutable canonical parent/local overlay without merged-map materialization or canonical copying. Structural counters prove that ordinary user backend work is independent of unrelated canonical population size.**

Do NOT claim:

```text
Universe is loaded from universe.telosc
```

or:

```text
compiler only compiles user code in a fresh process
```

until P7–P9 complete.

---

# Appendix A — Existing source architecture that P6 should reuse

## A.1 Callable signature indexes

`CallableSignatureTable` already demonstrates:

```text
by_id
module_callables
owner_callables
iter_module
iter_owner
```

This is the model for P6 secondary indexes.

## A.2 Field signature indexes

Field signatures similarly maintain module/owner indexes.

## A.3 Derived conformance exact index

`DerivedConformanceRuleIndex` already owns:

```text
rules_by_impl
get_rule
```

Use it.

## A.4 Source range and reverse indexes

Existing source-index architecture includes:

```text
RangeIndex
OccurrenceIndex
expression interval index
SemanticTarget reverse references
module-sharded source index
```

Do not duplicate it.

---

# Appendix B — Expected P6 code-review questions

Every patch should answer:

```text
What whole-world scan is removed?
What exact/indexed authority replaces it?
Who owns index mutation?
How is remove/rollback handled?
How does this compose with P5 parent/local state?
What is clone complexity?
What counter proves the old scan disappeared?
What parity test proves semantics are unchanged?
```

If a patch cannot answer these questions, it is not ready.

---

# Appendix C — Suggested first implementation sequence

Once `P6_START_SHA` is pinned:

```text
1. inspect P5 delta for affected aggregate shapes
2. add P6 counters
3. capture baseline
4. add callable-analysis module index
5. add callable-definition module index
6. add declaration/data/enum module indexes
7. run semantic index tests
8. cut compile.rs runtime-root discovery
9. cut semantic_lowering module scans
10. replace binding nested occurrence scan
11. add per-TraitSurface requirement layout
12. add ImplId domain index
13. use DerivedConformanceRuleIndex::get_rule
14. run residual global-scan audit
15. optimize remaining local passes only if measured
16. run structural scaling gates
17. run focused semantic/core parity
18. record performance
19. close lifecycle
20. hand P7 the indexed in-memory compiler authority
```

---

# Appendix D — Minimum test command matrix

Use actual repository test filters and verify nonzero selected tests.

```sh
cargo check -p telos-observability
cargo check -p phalcom-semantic
cargo check -p phalcom-core --features perf-observability

cargo test -p phalcom-semantic --test semantic backend_index
cargo test -p phalcom-semantic --test semantic source_index
cargo test -p phalcom-semantic --test semantic traits
cargo test -p phalcom-semantic --test semantic derived_conformance

cargo test -p phalcom-core --test core backend_locality
cargo test -p phalcom-core --test core semantic_boundary
cargo test -p phalcom-core --test core declaration_dispatch
cargo test -p phalcom-core --test core product_optimization

cargo fmt --all -- --check
```

If repository policy requires broader suites, run them before P6 closure and classify any inherited baseline failures using same-revision comparison evidence.

---

# Appendix E — Performance evidence table template

Populate in the P6 walkthrough.

| Metric | P5 baseline | After P6 | Gate |
|---|---:|---:|---|
| backend callable analyses visited | TBD | TBD | local-only |
| backend callable definitions visited | TBD | TBD | local-only |
| backend data declarations visited | TBD | TBD | local-only |
| backend enum declarations visited | TBD | TBD | local-only |
| entities rejected by module | TBD | `0` | exact |
| binding full occurrence scans | TBD | `0` | exact |
| trait surfaces visited per slot lookup | TBD | `<=1` | exact |
| conditional members scanned for ImplId | TBD | `0` | exact |
| canonical aggregate entries copied | P5 gate | no regression | `0` preferred |
| snapshot full-map materializations | P5 gate | no regression | no increase |
| runtime-root discovery wall time | TBD | TBD | improve |
| module lowering wall time | TBD | TBD | improve |
| total warm fixture compile | TBD | TBD | improve |

---

# Appendix F — P6/P7 boundary sentence for CHECKPOINT.md

Recommended checkpoint wording after P6 closure:

> `PERF003.C2.P6` closes residual backend whole-world complexity. Semantic/compiler backend reads are module- or exact-identity-indexed, and ordinary local lowering does not linearly traverse unrelated canonical authority. The canonical image is still source-built and in-memory; `PERF003.C2.P7` owns the sectioned, relocatable `.telosc` semantic artifact and fresh-process artifact load path.
