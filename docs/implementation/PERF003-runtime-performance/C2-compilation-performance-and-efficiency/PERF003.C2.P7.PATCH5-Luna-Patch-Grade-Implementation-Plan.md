---
id: PERF003.C2.P7.PATCH5
title: P7 Allocation and Evidence Hardening — Luna Patch-Grade Implementation Plan
program: PERF003
checkpoint: PERF003.C2
parent_phase: PERF003.C2.P7
status: PLANNED
completion: NOT_STARTED
verification: UNVERIFIED
branch: perf/universe-characterization
prepared_against_revision: a134a0650461b2dd6760a8d42a2e944be505849f
underlying_p7_revision: e45030869cfcd5fb00c1e7d221cdaebce8984b27
execution_agent: Luna
execution_mode: patch-grade
depends_on:
  - PERF003.C2.P7.PATCH4
must_haves:
  truths:
    - One AllocationBudget transaction accounts every allocator request owned by a successful compiler-artifact load, through final LoadedUniverseArtifact publication.
    - Source-built mutable representations may remain native; artifact-loaded immutable authority uses explicitly budgeted frozen representations behind common read contracts.
    - Independent counting-allocator evidence agrees with the production ledger for the complete source-free Universe load.
    - A valid realistic package compiles through both source-backed and artifact-backed compiler paths.
    - The audit harness reports only claims supported by raw evidence.
  prohibitions:
    - Do not estimate std BTreeMap/BTreeSet node memory.
    - Do not use production global allocator interception or nightly allocator APIs.
    - Do not weaken limits, semantic completeness, ABI validation, corruption rejection, or source-free compilation.
    - Do not implement P8 executable-Universe/runtime work.
    - Do not mark compile-error timing as successful package compilation.
    - Do not claim leak, workspace-speedup, physical source denial, or cross-process determinism without the corresponding discriminator.
---

# PERF003.C2.P7.PATCH5 — P7 Allocation and Evidence Hardening

## 0. Mission

Implement the **remaining P7 hardening work** on `perf/universe-characterization` without crossing into P8.

The patch has three outcomes:

1. **Close exact cumulative allocation accounting for the complete loaded compiler artifact.**
2. **Repair the P7 benchmark/audit harness so every claim is mechanically supported by its raw evidence.**
3. **Normalize `packages/slotmap` sufficiently to make it a valid realistic compilation discriminator, then prove source/artifact compiler parity on successful compilation or record the first genuine compiler blocker.**

This plan supersedes any narrower interpretation that the remaining allocation defect exists only in `CanonicalUniverseFormalImage::from_loaded_artifact`. Repository inspection shows the live gap is broader:

- `phalcom-modules/src/compiler_artifact.rs` still constructs native map/set authority and deeply cloned identities outside the ledger.
- `phalcom-semantic/src/types/artifact.rs::TypeStore::import_type_graph` has no allocation-budget parameter and reconstructs persistent arenas/interning indexes through allocator-owning structures.
- `phalcom-semantic/src/static_value/artifact.rs::import_static_value_graph` has no allocation-budget parameter and uses persistent arenas/interners plus native maps/vectors.
- `phalcom-semantic/src/artifact/import.rs` explicitly says semantic materialization still contains unaccounted allocation boundaries, while `require_semantic_import_allocation_accounting()` currently returns `Ok(())`.
- `phalcom-semantic/src/artifact/query.rs` passes a ledger around outer scratch but reconstructs many products, maps, `Arc`s, identities, and dependency owners outside it.
- final `ModuleQueryProducts`, `SemanticSnapshot`, and `CanonicalUniverseFormalImage` publication still allocates outside the ledger.

Therefore PATCH5 is a **full loaded-compiler allocation closure**, not merely a three-`Arc` fix.

---

# 1. Authority and architectural decisions

## 1.1 Read these first, in this order

Before editing code, read:

1. `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-allocation-materialization-architecture-ruling-2026-10-15.md`
2. `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-allocation-map-accounting-incident-2026-10-15.md`
3. `docs/perf003-p7-audit/FINDINGS.md`
4. `docs/perf003-p7-audit/MEMORY.md`
5. `phalcom-type-meta/src/allocation.rs`
6. `phalcom-semantic/src/checker/analysis_map.rs`
7. `phalcom-semantic/src/checker/analysis_set.rs`
8. `phalcom-semantic/src/checker/analysis.rs` around `CallableAnalysisTable::from_budgeted_entries`
9. `phalcom-modules/src/compiler_artifact.rs`
10. `phalcom-semantic/src/types/artifact.rs` around `TypeStore::import_type_graph`
11. `phalcom-semantic/src/static_value/artifact.rs` around `import_static_value_graph`
12. `phalcom-semantic/src/artifact/import.rs` around `validate_and_import_semantic_world_with_allocation_budget`
13. `phalcom-semantic/src/artifact/query.rs`
14. `phalcom-semantic/src/canonical_image.rs::from_loaded_artifact`
15. `phalcom-semantic/src/snapshot.rs::ModuleQueryProducts`
16. `phalcom-core/src/modules/canonical_artifact.rs`
17. `phalcom-semantic/tests/semantic/artifact_allocation.rs`
18. `phalcom-core/examples/perf_audit_p7.rs`

Do not start implementation until the allocation ruling is understood. It is already decided architecture, not an open design question.

## 1.2 Fixed allocation contract

The implementation must preserve:

```text
one load
  -> one whole-load AllocationBudget
  -> section/stage child ledgers
  -> debit requested Layout BEFORE allocator request
  -> no refund for successful scratch
  -> child commits only after stage success
  -> failed stage publishes nothing
  -> failed child does not commit to parent
```

The unit of accounting is **allocator-requested bytes**, not:

- encoded bytes;
- logical entry bytes;
- RSS;
- allocator usable capacity;
- guessed B-tree overhead;
- `size_of::<K,V>() * len`;
- container item count.

## 1.3 Fixed representation ruling

For immutable artifact-owned ordered authority:

```text
source-built state:
    native mutable BTreeMap/BTreeSet/HashMap/im structures as currently appropriate

artifact-loaded state:
    explicitly charged frozen ordered buffers
    (BudgetedSortedMap / BudgetedSortedSet or equivalent)
    behind the same semantic read interface
```

Use the existing repository pattern in:

- `checker/analysis_map.rs`;
- `checker/analysis_set.rs`;
- `CallableAnalysisTable` frozen/source storage.

Do not introduce a custom B-tree allocator. Do not mirror std B-tree node layout. Do not globally replace source-side collections.

## 1.4 Fixed scope boundary

PATCH5 ends at a rigorously loaded **compiler image**.

Do not implement:

- `EXECUTABLE` `.telosc` section;
- VM bytecode/prototype serialization;
- `VM::try_new_with_universe_artifact`;
- `ProductKind` runtime repair;
- artifact-backed runtime bootstrap;
- automatic cache discovery;
- CLI artifact selection;
- incremental artifact rebuilding.

Those are successor work.

---

# 2. Repository baseline and branch discipline

## PATCH5.G0 — Baseline gate

Before modification:

```sh
git branch --show-current
git rev-parse HEAD
git status --short
git rev-parse origin/perf/universe-characterization
```

Expected:

```text
branch = perf/universe-characterization
HEAD = a134a0650461b2dd6760a8d42a2e944be505849f
working tree = clean
HEAD == origin/perf/universe-characterization
```

If remote has advanced, stop and rebase this plan against the new head before implementation.

Do not switch branches.

## Baseline compiler feedback

Run sequentially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib --example perf_audit_p7
```

Record any baseline warning/error separately. Do not fix unrelated warnings.

---

# 3. Execution strategy

Implement in six waves.

| Wave | Tasks | Purpose |
|---|---|---|
| 0 | T0–T1 | Independent allocator discriminator + exact-owner helpers |
| 1 | T2–T3 | Module-world loaded authority |
| 2 | T4–T6 | Type/static/semantic imported authority |
| 3 | T7–T8 | Query image + final formal-image publication |
| 4 | T9–T10 | Full-load proof + realistic package parity |
| 5 | T11–T12 | Harness correction + refreshed evidence/closure |

Do not jump to later waves because an earlier compiler check passes. Each wave has its own acceptance discriminator.

---

# 4. Wave 0 — Build the proof harness before broad migration

## PATCH5.T0 — Add a dedicated full-load allocation discriminator

### Objective

Create an **independent allocator-observed test** for the complete in-memory `.telosc` load. The production `AllocationBudget` cannot prove its own completeness.

### Files

Likely:

```text
phalcom-core/Cargo.toml
phalcom-core/tests/artifact_allocation.rs        # new isolated test target
```

Name may differ, but keep the counting global allocator isolated from parallel suites.

### Reference pattern

Copy the measurement discipline—not the specific fixtures—from:

```text
phalcom-semantic/tests/semantic/artifact_allocation.rs
```

That test already:

- installs a test-only `GlobalAlloc` wrapper;
- counts allocation/reallocation request sizes;
- scopes `MEASURING`;
- compares independent allocator requests against `AllocationBudget`;
- tests exact fit and `N-1`.

### Required test structure

1. Build the canonical source program and `BuiltUniverseTelosc` **before** allocator measurement.
2. Keep artifact bytes alive outside measurement.
3. Discover the minimum successful whole-load budget `N` using the public loader and `ValidationLimits`, with allocator measurement disabled.
4. Reset independent counter.
5. Enable allocator measurement.
6. Call `load_universe_telosc_from_bytes(&built.bytes, limits_with_N)`.
7. Disable allocator measurement immediately after the load returns.
8. Assert the load succeeds.
9. Assert independent allocator-requested bytes equal the production ledger's complete required amount.
10. Drop the returned artifact with measurement disabled.
11. Repeat with `N - 1`; assert typed allocation exhaustion/failure.
12. Keep test-harness output/formatting outside measured interval.

### Required instrumentation seam

The public loader currently does not expose `AllocationBudget::requested()`.

Add a **test-only/internal** stage result or hook, not a new public product API.

Preferred shape:

```rust
#[cfg(test or feature appropriate to isolated test)]
struct ArtifactLoadAllocationSummary {
    requested: usize,
    ...
}
```

or an internal loader helper returning:

```rust
(Result<Arc<LoadedUniverseArtifact>, ...>, requested)
```

The production public API must remain unchanged.

If integration-test visibility makes `#[cfg(test)]` insufficient, expose a narrowly named `#[doc(hidden)]` test-support function behind an existing test-support feature or create an internal unit-test target. Do not expose allocator diagnostics as normal compiler API.

### Initial expected result

At current head, the new independent equality assertion is expected to **fail**. That failure is the baseline discriminator, not a reason to weaken the assertion.

Record:

```text
ledger requested bytes
independent requested bytes
delta
```

Do not commit a permanently failing test to the final patch. During implementation it may remain local or temporarily ignored with an explicit PATCH5 marker; final closure requires it active and passing.

### Acceptance

T0 is complete when:

- the isolated test builds;
- it detects current under-accounting independently;
- it can later become the final full-load equality gate without changing measurement methodology.

### Stop/consult

Stop if measuring the operation itself necessarily allocates through the test harness in a way that cannot be isolated. Do not subtract a guessed constant.

---

## PATCH5.T1 — Normalize exact-owner helpers and prove them independently

### Objective

Use existing allocation primitives consistently and add only missing exact conversion helpers needed by later tasks.

### Read first

```text
phalcom-type-meta/src/allocation.rs
phalcom-semantic/src/artifact/import.rs::budgeted_arc_slice
phalcom-semantic/src/checker/analysis_map.rs
phalcom-semantic/src/checker/analysis_set.rs
```

### Existing primitives to reuse

Do not duplicate:

```rust
AllocationBudget::vector
AllocationBudget::grow_vector_exact
AllocationBudget::string
AllocationBudget::boxed_str
AllocationBudget::boxed
AllocationBudget::arc
AllocationBudget::sorted_map_builder
AllocationBudget::sorted_set_builder

BudgetedSortedMap
BudgetedSortedSet
```

### Required audit

Search loaded paths for:

```text
Arc::new
Arc::from
into_boxed_slice
Vec::with_capacity
Vec::new followed by growth
String::from / String::new / to_string / to_owned
Box::<str>::from / .into() from &str/String
collect::<Vec<_>>
collect::<BTreeMap<...>>
collect::<BTreeSet<...>>
HashMap::with_capacity
im::HashMap insertion
im::Vector push
Clone on heap-owning identities/products
```

Do not mechanically replace source-side occurrences. Only artifact-load paths matter.

### `budgeted_arc_slice`

The current helper charges conversion-related layouts but uses:

```rust
Arc::from(values.into_boxed_slice())
```

Before trusting it globally, add an isolated allocator test that proves:

```text
observed allocation requests == ledger debit
```

for:

- empty slice;
- exact-capacity vector;
- over-capacity vector requiring shrink/replacement;
- nontrivial `T`.

If the helper double-charges or misses the intermediate boxed-slice request, correct it once and reuse it.

### Owned identity helper requirement

Use existing budget-aware ownership seams where available.

Example:

```rust
ModuleComponent::from_owned_identifier(
    allocations.boxed_str(spelling)?
)
```

Do **not** call `ModuleComponent::from_identifier(spelling)` on the charged import path, because it allocates its own `Box<str>` invisibly.

### New helpers allowed

Only add helpers whose requested layouts can be proved independently, e.g.:

```text
budgeted_boxed_slice
budgeted_arc_slice
budgeted_owned_identifier
budgeted_exact_vec_to_arc_vec if actually necessary
```

Each new helper needs an exact-fit and independent allocator test.

### Acceptance

No later task should need to invent ad hoc accounting for `Arc`, strings, or frozen sorted buffers.

---

# 5. Wave 1 — Module-world materialization

The current `load_module_world_with_allocation_budget` name is stronger than its implementation. The child ledger covers selected vectors/`Arc`s, but retained module authority still uses unobserved native maps/sets and deep clones.

## PATCH5.T2 — Budget all module identities and nested interface owners

### Files

Primary:

```text
phalcom-modules/src/compiler_artifact.rs
phalcom-modules/src/identity.rs
phalcom-modules/src/interface.rs
```

Only touch additional module files when directly required by the loaded representation.

### Exact current problems

In `load_module_world_inner`:

- `ModuleComponent::from_identifier(part)` allocates uncharged owned strings.
- `ModulePath`/`ModuleId` clones may recursively clone boxed path/component storage.
- declaration/export/import/metadata reconstruction clones owned `String`, `Box<str>`, path components, aliases, selectors, symbols.
- nested interface collections are assembled through native `BTreeMap`/`BTreeSet`.
- linked exports/bindings/read identities create additional owned names.

### Instruction

Create budget-aware import helpers for every **stable owned module identity** produced from wire data.

Examples, names flexible:

```rust
import_module_component_with_allocation_budget(...)
import_module_path_with_allocation_budget(...)
clone_module_id_with_allocation_budget(...)       // only where reuse cannot avoid clone
import_symbol_name_with_allocation_budget(...)
import_import_path_with_allocation_budget(...)
import_module_metadata_with_allocation_budget(...)
```

Prefer **ownership reuse** over cloning.

If the same canonical `ModuleId` already exists in a charged `ids` arena and a consumer can retain an index/reference/`Arc`, do not deep-clone it repeatedly merely to satisfy an old value-owned map shape. The artifact representation may use stable indexes or shared owners behind a read adapter as authorized by the architecture ruling.

### Do not

- debit `size_of::<ModuleId>()` and then call `clone()`;
- treat `Clone` as allocation-free;
- allocate an owned error string after the budget is exhausted when a typed/static error can be returned.

### Acceptance

An independent allocator probe around module-world materialization must no longer report identity/string allocations outside the child ledger.

---

## PATCH5.T3 — Replace loaded module BTreeMap/BTreeSet authority with frozen read storage

### Files

Expected:

```text
phalcom-modules/src/canonical_products.rs
phalcom-modules/src/compiler_artifact.rs
phalcom-modules/src/query.rs
phalcom-modules/src/topology.rs
phalcom-modules/src/graph.rs
phalcom-modules/src/linker.rs            # only if LinkedProgram owner shape requires it
phalcom-semantic/src/snapshot.rs         # read adapter integration only if necessary
```

Do not globally rewrite module source processing.

### Repository pattern to follow

Model the physical duality after:

```text
AnalysisMap:
    Source(BTreeMap)
    Frozen(Arc<BudgetedSortedMap>)

AnalysisSet:
    Source(BTreeSet)
    Frozen(Arc<BudgetedSortedSet>)
```

### Required loaded owners

Close all artifact-created persistent ordered owners reachable from:

```text
CanonicalExternalModuleWorld
LinkedProgram
ModuleGraphs
ReferenceGraph
RuntimeDependencyGraph
SemanticGraph
ModuleTopology
UnlinkedModuleInterface
LinkedModuleInterface
module binding layouts/indexes
```

The architecture ruling explicitly requires nested owners; changing only `CanonicalExternalModuleWorld.interfaces` is insufficient.

### Preferred implementation technique

Introduce **narrow module-authority read wrappers**, not one repository-global map abstraction.

Example concept:

```rust
enum ModuleAuthorityMap<K, V> {
    Source(BTreeMap<K, V>),
    Frozen(Arc<BudgetedSortedMap<K, V>>),
}
```

and set equivalent for sets.

Required read operations should be only what current module/query consumers use:

```text
get
contains_key
iter
keys
values
len
is_empty
range if genuinely used
```

Source mutation methods, if needed, must reject/avoid Frozen storage.

### `ModuleQueryFacade`

Current `ModuleQueryFacade` stores direct references to concrete `BTreeMap`/`BTreeSet`. Adapt it to the common read contract so both source and frozen module authority are queryable without reconstituting native maps.

Do not copy frozen data back into a `BTreeMap` just to call the facade.

### Scratch

Validation-only structures should not become frozen persistent maps unnecessarily.

Use:

```text
budgeted Vec
sorted borrowed index vectors
bitsets/dense indices
CSR/worklists
```

for:

- duplicate checks;
- initialization-order positions;
- topology temporary indexes;
- graph DAG checks.

The runtime DAG dense scratch already follows this approach; preserve it.

### Construction

For each frozen map/set:

1. `allocations.vector(expected)` or sorted builder.
2. import/construct each owned key/value under the same child ledger.
3. `sort_unstable_by`.
4. adjacent duplicate check.
5. freeze with `BudgetedSortedMap::from_sorted_entries` or equivalent.
6. charge final `Arc` owner.
7. never convert to `BTreeMap`.

### Required tests

Add module-level tests proving:

- Source and Frozen read results are identical.
- Frozen representation preserves canonical order.
- Duplicate loaded keys are rejected.
- Frozen mutation is impossible.
- Exact `N` succeeds; `N-1` fails.
- counting allocator request bytes == module child-ledger debit.
- runtime DAG/canonical topology fingerprints remain identical.

### Gate

Run sequentially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

Then module-focused tests only.

### Stop/consult

Stop if a production module consumer genuinely requires mutable access to artifact-loaded canonical authority. Do not silently make Frozen mutable or clone into a B-tree. Report the consumer and required mutation.

---

# 6. Wave 2 — Semantic store and owner closure

This is the largest wave. Keep each task causal and reviewable.

## PATCH5.T4 — Add budget-aware TypeStore artifact import

### Files

Primary:

```text
phalcom-semantic/src/types/artifact.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
```

Additional type files only for directly reachable imported owners.

### Current problem

`validate_and_import_semantic_world_inner` currently calls:

```rust
TypeStore::import_type_graph(&world.type_graph, limits)?
```

with no ledger.

`TypeStore` uses persistent `im::Vector`/`im::HashMap` arenas and interning indexes. Their internal allocator requests cannot be made exact by precharging logical element counts.

### Required API

Add:

```rust
TypeStore::import_type_graph_with_allocation_budget(
    graph,
    limits,
    allocations
)
```

Keep the old `import_type_graph` as a compatibility wrapper using a local budget if non-P7 callers require it.

The P7 loader must call only the budget-aware path.

### Physical representation requirement

Do not attempt to “account” `im::HashMap`/`im::Vector` internals by estimation.

The imported canonical prefix needs a physical representation whose allocation requests are explicit.

Use one of these, in priority order:

1. **Frozen imported prefix + mutable overlay**, preserving current public `TypeStore` behavior.
2. Source/Frozen backing for the relevant arena/index families.
3. Another explicitly fallible contiguous frozen representation approved by the existing allocation ruling.

The expected direction is:

```text
canonical artifact prefix:
    charged contiguous arenas
    charged sorted lookup indexes

user compilation fork:
    reuses immutable prefix
    appends/mutates through existing overlay/persistent mechanics
```

Do not rewrite source-built `TypeStore` creation.

### Critical semantic invariant

`fork_from_baseline()` and `extends_canonical_prefix()` must continue to work.

P7's value depends on user compilation extending the canonical loaded store without reconstructing it.

Tests must prove:

```text
loaded prefix IDs preserved
fork gets fresh TypeStoreId where current semantics require
all preexisting TypeId/KindId/etc numeric coordinates unchanged
post-fork user type insertion does not mutate the canonical prefix
lookups across source/frozen backing are semantically identical
```

### Nested ownership

Every imported heap-bearing type component must be charged:

- `Box<str>`;
- `Box<[T]>`;
- selector labels;
- declaration/module identities;
- type parameter names/owners;
- type lambda arrays;
- record rows/labels;
- family members;
- static-domain owners;
- constructor names/parameter-kind arrays;
- relocation vectors/indexes.

### Exact tests

Create a dedicated type-graph import counting test:

```text
wire graph constructed before measurement
measure import only
ledger requested == independent allocator requested
exact N succeeds
N-1 fails
semantic relocation IDs equal old importer
```

### Stop/consult

If preserving efficient post-load `TypeStore` extension requires changing the canonical prefix/overlay architecture beyond this source/frozen physical split, stop and report. Do not replace the TypeStore semantic model.

---

## PATCH5.T5 — Add budget-aware StaticValueStore artifact import

### Files

Primary:

```text
phalcom-semantic/src/static_value/artifact.rs
phalcom-semantic/src/static_value/store.rs
```

### Current problem

`import_static_value_graph` currently allocates through:

- `StaticValueStore::new`;
- `im::Vector` arenas;
- `im::HashMap` interners;
- `BTreeMap` relocations;
- temporary `Vec`s;
- boxed strings/slices;
- `BigInt` parsing/ownership;
- record/pack/data/variant owners.

No transaction ledger is passed.

### Required API

Add:

```rust
import_static_value_graph_with_allocation_budget(
    ...,
    allocations
)
```

P7 loader must use it.

Retain existing wrapper if source/test callers need it.

### Representation

Apply the same principle as TypeStore:

```text
artifact canonical prefix:
    frozen charged arenas + frozen charged indexes

subsequent user session:
    read canonical prefix
    extend through compatible overlay/fork mechanics
```

Do not estimate `im` node allocation.

### Special attention

Prove exact accounting for:

- parsed large integers (`BigInt`);
- owned static strings/symbols;
- tuples/sequences/records;
- data/variant component arrays;
- whole-pack backing;
- sliced packs;
- record field keys;
- duplicate-label scratch.

If a third-party type such as `BigInt` cannot expose a fallible/exact allocation seam, isolate it explicitly. Do not silently call it and pretend the ledger covers it. If exact requested-byte accounting for that legal artifact value is impossible under current APIs, stop with an incident naming the exact type and call site.

### Tests

- source/static graph round-trip parity;
- independent allocator equality;
- N/N-1;
- duplicate/malformed inputs fail atomically;
- store prefix extension remains correct.

---

## PATCH5.T6 — Migrate remaining semantic owner families to frozen budgeted construction

### Files

Primary:

```text
phalcom-semantic/src/artifact/import.rs
```

Likely owner modules as required:

```text
phalcom-semantic/src/parent_map.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/module_bindings.rs
phalcom-semantic/src/derived_conformance.rs
```

Do not touch all of these automatically; touch only an owner that allocates during artifact import.

### Current top-level warning

The live code says:

```rust
/// It currently fails closed while validation and materialization still have
/// allocation requests outside that ledger.
```

but:

```rust
fn require_semantic_import_allocation_accounting() -> Result<(), ArtifactError> {
    Ok(())
}
```

Do not preserve this false guard as closure evidence.

### Required migrated families

Audit and close all retained allocations for:

```text
structural primary contracts
callable signatures
callable definitions
enum requirements
trait headers
trait surfaces
declaration dispatch surfaces
field signatures
conditional inherent domains/members
conformance index
conformance witness plans
derived conformance rules
constructor initialization products
associated families
type-member declarations
associated static domains
static case catalogs/signatures
static definitions
static constants
module static bindings
owner indexes
final SemanticSnapshot
```

Callable analysis already contains a correct Source/Frozen pattern; use it as the model.

### Construction rule

Any artifact-path table that currently does:

```rust
let mut map = BTreeMap::new();
...
map.insert(...)
```

or:

```rust
Arc::new(...)
```

must be classified as one of:

1. source-only and therefore irrelevant;
2. temporary scratch — convert to budgeted dense/sorted scratch;
3. retained artifact authority — convert to frozen budgeted owner;
4. pre-existing charged owner — reuse without allocating;
5. unresolved legal external allocator — stop and report.

Do not leave an occurrence unclassified.

### Error paths

Allocation-exhaustion paths must not allocate error messages.

Prefer typed/static errors for resource exhaustion.

Validation errors may allocate only when the relevant allocation is itself charged and the budget has capacity. Avoid `format!` after exhaustion.

### Guard disposition

At the end of T6:

- either delete `require_semantic_import_allocation_accounting()` because the function is no longer necessary;
- or turn it into a meaningful structural assertion.

Do **not** leave `Ok(())` paired with a comment claiming unresolved accounting.

### Tests

Extend the isolated semantic allocation test to cover complete `validate_and_import_semantic_world_with_allocation_budget` on a representative nontrivial semantic world.

Required:

```text
independent requested == child ledger requested
N succeeds
N-1 fails
malformed input does not commit child
source/imported semantic family parity remains unchanged
```

### Gate

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
```

Do not proceed to query/finalization while semantic import still has known independent allocator delta.

---

# 7. Wave 3 — Query reconstruction and final publication

## PATCH5.T7 — Make query-image reconstruction fully budget-aware

### Files

```text
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/db/query.rs or owning query-image module as required
phalcom-semantic/src/module_product.rs
```

### Current gaps

`reconstruct_query_image_with_allocation_budget` has a child ledger, but `product_and_fingerprints` does not receive it.

Known loaded allocations include:

- linked-interface clone wrapped in `Arc::new`;
- resolved-import `BTreeMap`;
- `import.path().to_string()`;
- `ResolvedImportsProduct` `Arc`;
- declaration/surface/signature/field/data/enum products wrapped with `Arc::new`;
- temporary variant vectors;
- module static binding catalogs;
- hierarchy-edge products;
- outer `Arc<SemanticProduct>`;
- native `entries` and `fingerprints` BTreeMaps;
- `QueryKey` deep clones;
- dependency `Arc::from(edges.into_boxed_slice())`;
- loaded query-image internal indexes.

### Required change

Thread:

```rust
&mut AllocationBudget
```

into `product_and_fingerprints` and every loaded-owner constructor it calls.

### Query entries/fingerprints

Use frozen charged ordered storage behind the existing `CanonicalSemanticQueryImage` reader contract.

Do not construct native BTreeMaps and then call a loaded-image constructor.

Follow Source/Frozen pattern where source query images need mutable/native maps.

### Resolved import product

Do not rebuild another opaque BTreeMap from module authority if a read-only product can refer to already loaded canonical reference/import authority.

If product semantics require an owned map, build a frozen budgeted map.

Do not deep-clone module paths/IDs without budget.

### Dependency arrays

Use the verified `budgeted_arc_slice`/equivalent helper.

### Tests

For complete query reconstruction:

```text
allocator observed == ledger requested
N succeeds
N-1 fails
all portable roots reconstructed
missing owner remains fail-closed
dependency fingerprints match source
query reader result parity
```

### Gate

Run query-focused semantic tests plus `artifact-allocation`.

---

## PATCH5.T8 — Close ModuleQueryProducts and formal-image publication

### Files

```text
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/canonical_image.rs
phalcom-core/src/modules/canonical_artifact.rs
phalcom-modules/src/query.rs             # only if common read adapter not already done in T3
```

### Current finalization gap

`CanonicalUniverseFormalImage::from_loaded_artifact` currently:

- shallow-clones `SemanticSnapshot`;
- reconstructs `ModuleQueryProducts::from_external_modules`;
- `Arc::new`s module products;
- `Arc::new`s snapshot;
- `Arc::new`s final image.

All must participate in the same parent transaction.

### Avoid duplicated module authority

`ModuleQueryProducts::from_external_modules` currently clones:

- full unlinked interfaces;
- linked interfaces;
- resolved import target map;
- reverse import map/set.

Where T3 already created immutable canonical module authority, prefer a **view/index over that authority** rather than duplicating the whole structure.

If the query API requires dedicated indexes, build only those indexes as frozen charged data.

### Required API seam

Make finalization accept the ledger:

```rust
CanonicalUniverseFormalImage::from_loaded_artifact_with_allocation_budget(
    ...,
    allocations
)
```

Name flexible; public source-built constructors remain unchanged.

Charge:

```text
new ModuleQueryProducts owner, if allocated
new SemanticSnapshot Arc owner
new CanonicalUniverseFormalImage Arc owner
```

Do not charge `Arc::clone` when it performs only a refcount increment.

### Timings

Keep `TeloscLoadTimings` semantics coherent.

If finalization work moves between `relocation` and `publication`, update timing boundaries deliberately; do not silently make measured phase names inaccurate.

### Acceptance

The formal image must still satisfy:

```rust
sources().is_empty()
snapshot().is_source_free_artifact_complete()
reader() succeeds
no advisory products
all dependency fingerprints validate
```

and compilation of existing synthetic fixtures still succeeds.

---

# 8. Wave 4 — Full closure proof and realistic package

## PATCH5.T9 — Make full-load allocator equality the hard gate

### Objective

Close the architecture ruling with independent evidence.

### Final in-memory load discriminator

For the real canonical artifact:

```text
N = minimum successful max_total_allocation_bytes
```

Require:

```text
budget N       -> full load succeeds
budget N - 1   -> typed budget failure
ledger debit   == independent allocator requested bytes
```

This must include:

```text
container parse
module materialization
semantic decode/import
type store import
static store import
query reconstruction
metadata decode
module-query finalization
snapshot publication
formal image publication
LoadedUniverseArtifact publication
```

### Stage-limit discriminators

Also prove:

1. section-local budget can fail while whole-load budget has headroom;
2. whole-load budget can fail while each individual section/stage limit is sufficient;
3. a failed child does not commit to the parent;
4. no partially loaded artifact is published;
5. retry with a sufficient budget succeeds.

### Disk acquisition

Keep the existing bounded-file-acquisition test.

Optionally add independent counting around `load_universe_telosc(path, ...)` if the in-memory test does not cover the bounded file buffer. Do not conflate input artifact bytes supplied by the caller with loader-created memory in the in-memory contract.

### Exact number

The previous evidence's `322,547,484` bytes is **not a required golden value**. It is expected to change when previously uncharged allocations become visible.

Do not assert the old number.

The invariant is equality and exact boundary, not preservation of N.

### Closure rule

Do not mark allocation hardening complete if:

```text
observed > ledger
```

(undercharge)

or:

```text
ledger > observed
```

(double/logical charging)

Both are failures of the requested-byte contract.

---

## PATCH5.T10 — Normalize `slotmap` and establish realistic compiler parity

### Files

```text
packages/slotmap/src/main.ph
packages/slotmap/tests/runtime/src/basic_test.ph      # rename
packages/slotmap/tests/runtime/src/dense_test.ph      # rename
packages/slotmap/tests/runtime/src/main.ph            # only if logical imports require update
phalcom-core/examples/perf_audit_p7.rs
```

### Fixture corrections

1. Change:

```telos
for (entry in propositions) {
```

to:

```telos
for entry in propositions {
```

2. Rename:

```text
basic_test.ph -> basic-test.ph
dense_test.ph -> dense-test.ph
```

3. Do **not** automatically rename logical imports.

Physical kebab case maps to logical snake_case in the module system. Verify whether:

```telos
import slotmap_tests.basic_test
import slotmap_tests.dense_test
```

remains correct after the physical rename. Change imports only if the resolver proves otherwise.

### Compile sequence

After only these known fixture corrections:

1. Compile `packages/slotmap` source-backed.
2. Compile it artifact-backed.
3. Compile `packages/slotmap/tests/runtime` source-backed.
4. Compile it artifact-backed.

### Required classification

If new failure appears:

```text
FIXTURE_DEFECT
BASELINE_COMPILER_DEFECT
ARTIFACT_SPECIFIC_DEFECT
UNSUPPORTED_LANGUAGE_FEATURE
```

Do not repeatedly rewrite the package to avoid a compiler bug.

### P7 acceptance

Preferred:

```text
packages/slotmap:
    source SUCCESS
    artifact SUCCESS
```

If a newly exposed baseline compiler defect prevents this, PATCH5 can still close allocation/evidence hardening only if the defect is documented as the first blocker and source/artifact behavior agrees. However **do not label realistic package compilation PASS** until both compile successfully.

### Runtime

Do not require execution. `VM::new()` / `ProductKind` is P8 scope.

---

# 9. Wave 5 — Benchmark/evidence correctness

## PATCH5.T11 — Repair `perf_audit_p7` evidence mechanics

### File

```text
phalcom-core/examples/perf_audit_p7.rs
```

### T11.1 — Valid CSV

Current runtime diagnostic output contains commas but is concatenated directly into CSV.

Implement one deterministic CSV field encoder used by all raw outputs.

Requirements:

```text
quote field if it contains comma, quote, CR, or LF
double embedded quotes
preserve diagnostic contents exactly
one logical sample = one CSV row
```

Prefer a small local helper over adding a dependency solely for the example, unless the workspace already has an approved CSV dependency.

### T11.2 — Digest terminology

The artifact digest implementation is BLAKE3-256 with domain framing.

Do not change the algorithm.

Correct report text from `SHA-256` to:

```text
BLAKE3-256 domain-separated digest
```

### T11.3 — Separate determinism properties

Keep:

```text
same-image serialization determinism
```

but add fresh-process full-build determinism:

```text
Process A: source build -> A.telosc
Process B: source build -> B.telosc
compare bytes exactly
compare artifact/source/section digests
```

Add a child subcommand if needed, e.g.:

```text
generate-single
determinism-check
```

Do not share the canonical `OnceLock`/compiler image between the two builds.

### T11.4 — Correct source-free wording

Current check establishes:

```text
loaded image contains zero source units
source-free completeness is true
dependent user package compiles
```

Call this:

```text
source-free compiler authority
```

Do not call it physical source denial unless the process actually has source access blocked/counted.

Physical source-denial runtime proof is P8.

### T11.5 — Peak RSS wording

`getrusage(...).ru_maxrss` is peak RSS.

It does not prove current residency or leak absence.

Remove/rewrite claims such as:

```text
0 memory leakage because ru_maxrss remained flat
```

Use:

```text
no higher peak RSS observed during later samples
```

unless a real live-memory metric is added.

### T11.6 — Statistics

Generate summary values programmatically from raw rows.

For each scenario report:

```text
sample count
success count
error count
min
mean
median
p95
max
standard deviation
maximum peak RSS
```

Declare the p95 convention once.

Do not manually transcribe table values.

### T11.7 — Failed workload classification

If a workload ends in `COMPILE_ERROR`, report:

```text
time to identical diagnostic
```

not:

```text
compilation speedup / realistic package PASS
```

Once slotmap succeeds after T10, it can become a successful benchmark.

### T11.8 — Runtime phase baseline

Preserve separate:

```text
compiler init
user compile
VM bootstrap
VM execution
total
peak RSS
```

The current artifact runtime path should visibly show the second source-backed Universe bootstrap. Do not hide it behind a single failure duration.

### T11.9 — Workspace testing

Unless a true source-vs-artifact suite A/B is implemented, report:

```text
artifact test correctness: PASS
workspace testing speedup: UNVERIFIED
```

Do not infer workspace speedup from the 113 artifact tests.

---

## PATCH5.T12 — Regenerate evidence and close the patch

### Evidence directory

Update:

```text
docs/perf003-p7-audit/REPORT.md
docs/perf003-p7-audit/FINDINGS.md
docs/perf003-p7-audit/FAILURES.md
docs/perf003-p7-audit/MEMORY.md
docs/perf003-p7-audit/COMMANDS.md
docs/perf003-p7-audit/ENVIRONMENT.md
docs/perf003-p7-audit/ARTIFACT.json
docs/perf003-p7-audit/RAW-COLD.csv
docs/perf003-p7-audit/RAW-WARM.csv
docs/perf003-p7-audit/RAW-RUNTIME.csv
docs/perf003-p7-audit/RAW-TESTING.csv
```

Add a PATCH5 implementation/walkthrough record under the existing P7 implementation directory.

### Build release harness

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --release -p phalcom-core --example perf_audit_p7
```

### Generate artifact

```sh
./target/release/examples/perf_audit_p7 generate \
  --out target/telos/universe/universe.telosc \
  --json target/perf003-p7-audit/ARTIFACT.json
```

Use the final harness commands recorded in `COMMANDS.md`; adjust names for any new determinism subcommand.

### Run focused suites

Sequentially:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test artifact-allocation -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --lib artifact::

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test artifact-allocation -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core --test core modules_canonical_artifact::
```

Run module-query parity tests corresponding to any changed module read adapter. Use the repository's registered test target rather than inventing a command if `autotests=false`.

### Re-run characterization

At minimum:

```text
fresh-process cold compilation
warm compilation
runtime baseline
slotmap evaluation
allocation exact-fit/equality
cross-process determinism
targeted artifact test lanes
```

### Report rules

The final report must distinguish:

```text
MEASURED PASS
MEASURED FAIL
BLOCKED
UNVERIFIED
```

Do not use “PASS” as a generic synonym for “the command ran”.

---

# 10. File-by-file expected change map

The exact diff may be smaller, but Luna should expect the following ownership areas.

## Foundation

```text
phalcom-type-meta/src/allocation.rs
```

Only if a missing exact owner/conversion helper is needed. Do not redesign it unnecessarily.

## Module world

```text
phalcom-modules/src/compiler_artifact.rs
phalcom-modules/src/canonical_products.rs
phalcom-modules/src/query.rs
phalcom-modules/src/topology.rs
phalcom-modules/src/graph.rs
phalcom-modules/src/identity.rs
phalcom-modules/src/interface.rs
```

Potentially linker owner structs if their maps are retained by the artifact.

## Semantic stores

```text
phalcom-semantic/src/types/artifact.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/static_value/artifact.rs
phalcom-semantic/src/static_value/store.rs
```

## Semantic owner import

```text
phalcom-semantic/src/artifact/import.rs
```

Plus exact owner modules only where a Source/Frozen storage seam must be added.

## Query/finalization

```text
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/canonical_image.rs
phalcom-core/src/modules/canonical_artifact.rs
```

## Tests/harness

```text
phalcom-core/Cargo.toml
phalcom-core/tests/artifact_allocation.rs
phalcom-semantic/tests/semantic/artifact_allocation.rs
phalcom-core/tests/core/modules/canonical_artifact.rs
phalcom-core/examples/perf_audit_p7.rs
```

## Fixture

```text
packages/slotmap/src/main.ph
packages/slotmap/tests/runtime/src/basic-test.ph
packages/slotmap/tests/runtime/src/dense-test.ph
packages/slotmap/tests/runtime/src/main.ph    # only if required
```

## Evidence

```text
docs/perf003-p7-audit/*
docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/*
```

---

# 11. Luna coding rules

These are mandatory.

## 11.1 Never infer allocation-freedom from syntax

The following may allocate:

```rust
clone()
to_string()
to_owned()
into()
collect()
Arc::new()
Arc::from()
into_boxed_slice()
format!()
Vec::with_capacity()
HashMap::with_capacity()
BTreeMap::insert()
BTreeSet::insert()
im::Vector::push_back()
im::HashMap::insert()
```

Before using one in the artifact-load path, prove one of:

```text
A. it allocates nothing for this concrete type/operation;
B. the allocator request is explicitly charged before it occurs;
C. it reuses an already charged owner without allocating;
D. it is outside the measured transaction by contract.
```

If none applies, it is a defect.

## 11.2 Deep clone rule

Types such as:

```text
ModuleComponent
ModulePath
ModuleId
selectors with owned labels
DeclarationId / CallableId owners containing module paths/names
interfaces containing Strings/Boxes/Vectors
```

may allocate during `Clone`.

Do not debit only the outer struct.

## 11.3 Source/Frozen rule

When a source-built table needs mutability but an imported table does not:

```text
do not force one physical representation
```

Use:

```text
Source(native mutable representation)
Frozen(charged immutable representation)
```

behind a common read API.

## 11.4 No map reconstruction for adapters

Never do:

```text
Frozen -> collect::<BTreeMap<_>>() -> existing reader
```

That defeats the architecture.

Change the reader boundary to consume both backings.

## 11.5 Preserve semantic ownership

A frozen sorted-vector representation changes **physical storage only**.

Do not change:

- key identity;
- canonical ordering;
- query authority;
- revision semantics;
- fingerprints;
- source versus generated ownership;
- import resolution;
- conformance selection;
- type IDs;
- static IDs.

## 11.6 Fail-closed rule

On:

```text
budget exhaustion
layout overflow
allocator failure
duplicate imported identity
malformed artifact
incompatible owner
```

return a typed error before publication.

Do not substitute defaults, skip rows, or continue.

## 11.7 No fake completion guard

Do not use a function named like:

```text
require_*_allocation_accounting()
```

that returns unconditional success while known unaccounted allocations remain.

---

# 12. Verification protocol by wave

## After Wave 0

```sh
cargo check -p phalcom-type-meta --lib
cargo check -p phalcom-core --test artifact-allocation
```

The full-load equality test may still fail at this stage; record the delta.

## After Wave 1

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

Run module allocation/query parity discriminators.

## After Wave 2

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
```

The semantic-world independent allocator delta must be zero before continuing.

## After Wave 3

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib --example perf_audit_p7
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture
```

Full-load equality must pass.

## After Wave 4

Run P7 artifact tests and source/artifact slotmap compilation.

## After Wave 5

Run all focused artifact lanes and regenerate evidence.

Do not run broad workspace tests merely because a focused gate fails. Fix the causal failure first.

---

# 13. Acceptance matrix

PATCH5 may be marked COMPLETE only when this matrix is accurate.

| Requirement | Required result |
|---|---|
| Whole-load production ledger | PASS |
| Independent allocator equality | PASS |
| Complete-load `N` | PASS |
| Complete-load `N-1` | typed FAIL as expected |
| Section-local exhaustion | PASS discriminator |
| Whole-load cumulative exhaustion | PASS discriminator |
| Failure atomicity | PASS |
| Module-world loaded allocation closure | PASS |
| TypeStore loaded allocation closure | PASS |
| StaticValueStore loaded allocation closure | PASS |
| Semantic owner allocation closure | PASS |
| Query-image allocation closure | PASS |
| Final snapshot/formal-image publication accounting | PASS |
| Source-free compiler completeness | PASS |
| Existing P7 artifact corruption/ABI guards | PASS |
| Same-image serialization determinism | PASS |
| Fresh-process full-build determinism | PASS or explicitly UNVERIFIED; cannot be falsely PASS |
| Synthetic source/artifact compilation | PASS |
| `slotmap` valid source compilation | PASS preferred; otherwise first genuine blocker recorded |
| `slotmap` valid artifact compilation | PASS preferred; otherwise parity/blocker recorded |
| Runtime execution | BLOCKED by P8 boundary; unchanged |
| Runtime source-free bootstrap | OUT OF SCOPE |
| CSV structural validity | PASS |
| Digest terminology | BLAKE3-256 |
| Peak RSS description | accurate |
| Leak claim | UNVERIFIED unless separately measured |
| Workspace speedup | UNVERIFIED unless actual A/B executed |

---

# 14. Performance non-regression

Correctness is primary, but record performance movement.

Before/after compare:

```text
artifact bytes
artifact load median
artifact-backed cold total median
artifact-backed user compile median
peak RSS
warm compile median
allocation request bytes
allocation count if independently available
```

Investigate, do not automatically reject, if:

```text
artifact load median increases > 15%
artifact-backed peak RSS increases > 15%
warm compile median increases > 15%
artifact size changes > 5% without transported semantic authority change
```

An increase in **ledger N** alone is expected when previously missing charges are made visible. Compare actual allocator requests/RSS as well.

---

# 15. Stop/consult conditions

Luna must stop and produce an incident instead of improvising when any of these occurs.

1. A legal artifact value is constructed by a third-party/internal allocator whose exact requested layout cannot be known before allocation and no frozen representation seam exists.
2. A loaded canonical structure must remain mutable for correctness, making the Source/Frozen contract insufficient.
3. TypeStore or StaticValueStore prefix extension would require changing semantic ID meaning or invalidating current `fork_from_baseline` behavior.
4. Eliminating an unbudgeted allocation requires a wire-format change.
5. Correct accounting appears to require production global allocator interception, nightly allocator APIs, or std private-layout mirroring.
6. A source/artifact semantic parity discrepancy appears after representation migration.
7. A valid repaired `slotmap` input exposes a compiler bug unrelated to artifact transport.
8. Fixing the issue would require P8 runtime/bootstrap work.
9. The remote branch advances during implementation and touched ownership code diverges materially.
10. The independent allocator reports a residual delta that cannot be localized after two focused instrumentation passes.

Incident report must contain:

```text
first failing owner/call site
expected allocation contract
observed allocator request
ledger debit
minimal reproduction
why current helper/representation cannot satisfy it
smallest architecture choice required
```

---

# 16. Rejected shortcuts

Never use:

- `size_of::<(K,V)>() * len` for B-tree accounting;
- hard-coded std B-tree node capacities;
- guessed multipliers;
- charging encoded DTO bytes as a proxy for decoded allocations;
- production counting allocator;
- nightly custom allocator APIs;
- `catch_unwind` around allocation failure;
- per-section reset masquerading as whole-load accounting;
- hidden native map reconstruction behind a frozen adapter;
- weakening default limits until tests pass;
- excluding scratch because it is later freed;
- marking a ledger exact because `N-1` fails without independent allocator comparison;
- rewriting `slotmap` around a compiler defect;
- implementing P8 to make a P7 test green.

---

# 17. Commit structure

Prefer reviewable commits at causal seams.

Suggested sequence:

```text
fix(artifact): add full-load allocation discriminator
fix(allocation): harden exact frozen owner conversions
fix(modules): freeze budgeted imported module authority
fix(semantic): budget imported type and static stores
fix(semantic): freeze remaining imported semantic authority
fix(query): budget loaded query products and indexes
fix(artifact): account final canonical image publication
test(perf): normalize slotmap compiler fixture
fix(perf): make P7 audit evidence mechanically reproducible
docs(perf): refresh P7 hardening evidence
```

Do not squash unrelated architectural seams into one giant commit while implementing. The final project owner can squash later if desired.

Do not commit or push until the requested execution workflow authorizes it. If an implementation handoff explicitly requires commit/push, push only after the listed focused gates pass and the worktree is clean.

---

# 18. Final implementation report required from Luna

The implementer handoff must report:

## Repository

```text
starting SHA
ending SHA
branch
clean/dirty
pushed/not pushed
```

## Code changes

For each task:

```text
files changed
representation/API change
remaining known unaccounted allocations
```

The final task must say:

```text
remaining known unaccounted allocations: NONE
```

before allocation closure can be claimed.

## Allocation proof

Report:

```text
complete-load ledger bytes N
independent allocator requested bytes
difference
N-1 result
section-local exhaustion result
whole-load exhaustion result
failure-atomicity result
```

## Compiler proof

Report:

```text
synthetic package source result
synthetic package artifact result
slotmap source result
slotmap artifact result
runtime-test package source compile result
runtime-test package artifact compile result
```

## Tests

Exact commands and pass/fail counts.

## Performance

Updated cold/warm/load/RSS numbers from regenerated raw files.

## Residual boundaries

Explicitly state:

```text
P8 runtime execution remains blocked/out of PATCH5 scope
automatic artifact freshness remains out of PATCH5 scope
workspace A/B speedup remains UNVERIFIED unless actually measured
```

---

# 19. Definition of done

`PERF003.C2.P7.PATCH5` is done only when:

1. The repository is on the intended P7 branch and current revision lineage.
2. The loaded module world performs no unaccounted retained allocator requests.
3. TypeStore artifact import performs no unaccounted allocator requests.
4. StaticValueStore artifact import performs no unaccounted allocator requests.
5. Semantic owner reconstruction performs no unaccounted allocator requests.
6. Query reconstruction performs no unaccounted allocator requests.
7. Module-query/final image publication performs no unaccounted allocator requests.
8. The complete load's independent allocator byte count equals the production ledger exactly.
9. Full load succeeds at `N` and fails at `N-1`.
10. Section limits and whole-load limits remain independent.
11. Failure does not partially publish or commit child ledger state.
12. Existing semantic/query/source-free artifact invariants still pass.
13. The audit harness produces valid machine-readable raw files.
14. BLAKE3 terminology is correct.
15. Determinism claims distinguish same-image from independent fresh-process builds.
16. Peak RSS is not presented as leak evidence.
17. Compile-error timings are not called successful compilation.
18. `slotmap` known syntax/naming fixture defects are fixed.
19. Source/artifact realistic compilation succeeds or the first genuine baseline compiler blocker is explicitly recorded.
20. No P8 runtime/executable work has entered the patch.

At that point P8 receives a defensible foundation:

```text
universe.telosc
  -> source-free compiler authority
  -> exact fail-closed whole-load resource accounting
  -> independently verified allocation ledger
  -> realistic package compiler parity
  -> reproducible empirical baseline
```

That is the handoff boundary. Do not extend PATCH5 beyond it.
