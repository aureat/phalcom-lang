# Pyrefly dependency graph and incremental invalidation

## Purpose

This document explains how Pyrefly decides which semantic work is stale after a module changes. It focuses on the concrete dependency keys, change flags, reverse edges, epoch loop, cycle fallback, and interaction with answer/module caches.

The central finding is that Pyrefly does not use one file-level dirty bit. It uses a coarse module reverse graph for reachability and fine-grained dependency facts inside each module to decide whether a particular export change matters.

## Evidence boundary

Pinned revision: 43467e64e36550f232a18e89f24fda79b1020b6b.

Primary files:

- [state.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/state.rs) — ModuleDeps, ModuleChanges, invalidated_by, reverse dependencies, epochs, and cycle handling.
- [module.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/module.rs) — per-module epochs and clean/compute coordination.
- [steps.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/steps.rs) — old/new staged products used for change comparison.
- [import_tracker.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/import_tracker.rs) — import/dependency tracking support.
- [performance improvements](https://pyrefly.org/blog/2026/02/06/performance-improvements/) — motivation and measured impact of finer-grained type dependencies.

Local Phalcom mapping:

- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/module_graph.rs
- phalcom-semantic/src/snapshot.rs
- phalcom-semantic/src/identity.rs
- phalcom-semantic/src/types/evidence.rs

## Dependency architecture

Pyrefly maintains two kinds of graph information:

~~~text
module graph:
    which modules can depend on which other modules

fine-grained module dependencies:
    what an importing module actually uses from each dependency
~~~

The reverse module graph answers reachability: when module A changes, which modules might need inspection? ModuleDeps answers relevance: which of those reverse dependents actually consume the changed export or metadata?

This two-level design avoids scanning every module's source on every edit while avoiding invalidating every reverse dependent for every change.

## ModuleDeps

A module dependency product contains:

~~~text
names:
    map exported name -> NameDep flags

wildcard:
    depends on wildcard export set

classes:
    set of class definition indexes

type_aliases:
    set of type alias indexes

django_relations:
    specialized module-level metadata dependency
~~~

NameDep has two flags:

~~~text
metadata:
    depends on deprecation, docstring, export-origin, or related metadata

type:
    depends on the exported type/answer
~~~

Presence of a name in the names map implies dependency on the name's existence. A type or metadata dependency therefore also implies existence.

This is an asymmetric dependency model. A consumer that only asks whether a name exists should not be invalidated by a type-only change. A consumer that uses the type is invalidated by type change. A consumer that uses metadata is invalidated by metadata change.

Phalcom should make its key space explicit:

~~~rust
enum DependencyKey {
    ModuleExists(ModuleId),
    ExportExists { module: ModuleId, name: Symbol },
    ExportType { module: ModuleId, name: Symbol },
    ExportMetadata { module: ModuleId, name: Symbol },
    WildcardExports(ModuleId),
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    NativeContract(NativeContractId),
    CoreSurface(CoreSurfaceId),
}
~~~

A dependency key should be recorded when a query asks for the fact, not guessed from the import statement after the fact.

## ModuleChanges

A ModuleChanges product represents what changed in a module's exports. It uses the same fields as ModuleDeps but different flag semantics:

~~~text
default NameDep in changes:
    name existence changed; added or removed

type flag:
    name's type/answer changed while it still exists

metadata flag:
    name's metadata changed while it still exists
~~~

The invalidation rule is:

~~~text
if wildcard dependency or wildcard change:
    invalidate

for each name used by the dependent:
    if changed name is an existence change:
        invalidate any dependency on that name
    if changed type and dependent depends on type:
        invalidate
    if changed metadata and dependent depends on metadata:
        invalidate

if shared class/type-alias/specialized metadata key changed:
    invalidate
~~~

This is more precise than comparing a whole module fingerprint.

## Why existence and type are asymmetric

Suppose module A imports a name only to check whether it exists for a re-export path. If module B changes the type of that name but keeps the export, A's existence fact is still valid.

Suppose A uses the value in a callable argument. A depends on type and existence; a type change must invalidate A.

Suppose A requests only a docstring/deprecation marker. A depends on metadata and existence; a body-only type change should not invalidate its metadata query.

The dependency engine must encode this asymmetry or it will either over-recompute or retain stale answers.

## Recording dependencies

A semantic lookup records dependency facts while it executes:

~~~text
lookup export exists
    -> ModuleDep::NameExists(name)

lookup exported answer/type
    -> ModuleDep::Key(exported_key)
    -> names[name].type = true

lookup doc/deprecation/origin metadata
    -> names[name].metadata = true

lookup wildcard export set
    -> wildcard = true

lookup class fields/MRO/variance
    -> classes[class_index] += 1

lookup type alias
    -> type_aliases[alias_index] += 1
~~~

The demand tree can label each lookup separately for debugging, while ModuleDeps merges them into the compact invalidation representation.

Phalcom should retain the demand edge and its compact dependency key:

~~~rust
struct RecordedDependency {
    key: DependencyKey,
    origin: SourceOrigin,
    query: QueryKind,
}
~~~

The compact key drives invalidation. The origin/query is for diagnostics and metrics.

## Reverse dependencies

Each module stores:

~~~text
deps:
    maps dependency module -> ModuleDeps

rdeps:
    set of modules that depend on this module
~~~

The invariant is symmetric: if A's deps contains B, B's rdeps contains A. Updating both edges must be atomic with respect to the dependency graph update.

At invalidation time:

~~~text
changed module
  -> read direct rdeps
  -> for each rdep:
         load its ModuleDeps for changed module
         call invalidated_by(ModuleChanges)
         mark dirty only if true
  -> put newly dirty modules in work queue
~~~

The direct rdep step is normal incremental propagation. A separate breadth-first transitive invalidation exists as a cycle fallback.

## Source changes versus derived export changes

An edit first invalidates the module's own products. The module is recomputed through the demanded stage. Only after its new exports/answers are compared does Pyrefly generate ModuleChanges.

This prevents unnecessary invalidation:

~~~text
body edit with unchanged export type
  -> module body/answers may recompute
  -> ModuleChanges is empty or metadata-only
  -> dependents remain clean

signature/export edit
  -> ModuleChanges contains changed name/type
  -> matching dependents dirty
~~~

The same approach should be used for Phalcom callable summaries. A body edit that preserves a callable's exported signature should not rebuild all consumers. A declaration or contract change must.

## Epochs and clean state

Each module has checked/computed epochs. An epoch indicates whether the module has been cleaned and computed for the current recheck run.

Reader fast path:

~~~text
if checked == current_epoch:
    module is clean for this run
else:
    try to start cleaning
~~~

Cleaning atomically takes dirty flags. Dirty flags set after the take remain for the next clean cycle rather than being lost.

The epoch is not a semantic revision by itself. A semantic answer also needs source revision, dependency fingerprint, and relevant configuration/core revision. Epochs coordinate one run; revisions determine cache validity across runs.

## Demand computation and contention

The demand path:

~~~text
optimistically inspect checked epoch/current step
  -> if enough product exists, return
  -> acquire compute ownership only when needed
  -> calculate missing stage
  -> release stage marker
  -> release compute flag
~~~

Wait time is measured. This exposes whether a slow recheck is caused by semantic computation or threads waiting for one module.

Phalcom's worker-owned engine already avoids many request-time races. It should retain the same metric split:

- invalidation-frontier calculation;
- queue wait;
- module computation;
- solver work;
- publication;
- LSP response rendering.

## Recheck epochs and stabilization

Pyrefly's run loop repeatedly computes changed modules until no export changes remain.

~~~text
run_internal
  -> prewarm/calculate stable stdlib/core products
  -> run one step over new/dirty modules
  -> collect changed modules and ModuleChanges
  -> if empty: finish
  -> detect overlapping repeated changes
  -> normal path: continue next epoch
  -> cycle path: invalidate transitive cycle and run again
~~~

The implementation caps the total number of epochs as a defense against unexpected dependency patterns.

Cycle detection uses overlap of changed export keys, not merely repeated module occurrence. If A changes export x due to B and later changes independent export y due to C, that is not automatically a mutable cycle. If the same export repeatedly changes through mutual propagation, coarse invalidation is safer.

## Cycle fallback

When a mutable dependency cycle is detected:

~~~text
changed cycle modules
  -> breadth-first traverse reverse dependencies
  -> mark all discovered modules dirty
  -> rerun the step
~~~

This is intentionally coarser than normal invalidation. It is a termination and correctness fallback for a dependency graph that cannot stabilize through narrow propagation.

Phalcom should expose the fallback:

~~~rust
enum InvalidationMode {
    FineGrained,
    CycleFallback,
    CoreSurfaceFallback,
}
~~~

A fallback should be measured and reported. Otherwise a project can silently regress to whole-workspace recomputation.

## Fine-grained type dependencies

Pyrefly's performance report describes an evolution from module-level invalidation toward finer-grained type dependency tracking. The motivation was over-invalidation in large projects: a change in one type could force thousands of modules to recheck even when only a small dependent set used that type.

The transfer lesson is staged adoption:

1. module reverse graph;
2. export-name dependency keys;
3. type/metadata/class/alias dependency keys;
4. declaration/member/callable dependency keys;
5. expression-level keys only where measurements justify the storage and maintenance cost.

Do not jump directly to expression-level persistence. Each finer key increases dependency recording, memory, invalidation complexity, and risk of stale edges.

## Phalcom current bridge

CURRENT or EXPERIMENTAL Phalcom already has:

- SourceChangeKind for body/import/declaration/file/core changes;
- deterministic BTreeSet queues;
- callable dependency and dependent maps;
- affected closure computation;
- candidate-state rebuilding;
- generation publication;
- cancellation checks;
- product reuse accounting.

The missing transfer is to make dependency keys semantic and shared:

~~~text
current LSP body/import/declaration classification
  -> semantic DependencyKey changes
  -> module/declaration/callable reverse graph
  -> query answer invalidation
  -> immutable generation publication
~~~

Do not build a second cache graph inside the formal checker.

## Phalcom dependency products

Recommended structures:

~~~rust
struct ModuleDependencies {
    keys: SmallMap<DependencyKey, DependencyUse>,
    wildcard: bool,
}

struct DependencyUse {
    existence: bool,
    type_fact: bool,
    metadata: bool,
    origin: Option<SourceOrigin>,
}

struct ModuleChanges {
    changed: SmallMap<DependencyKey, ChangeKind>,
}
~~~

For body facts:

~~~rust
struct CallableDependencies {
    uses: SmallSet<CallableId>,
    fields: SmallSet<FieldId>,
    declarations: SmallSet<DeclarationId>,
    modules: SmallSet<ModuleId>,
}
~~~

Keep dependency ownership in the product that made the query. A module export product should not be forced to know every expression dependency inside a downstream callable.

## Invalidation algorithm

Recommended Phalcom algorithm:

~~~text
on source revision:
  classify local change
  invalidate local stages according to change kind
  build local candidate
  compare old/new exports, surfaces, callable summaries
  emit precise ChangeSet
  walk direct reverse dependencies
  invalidate only matching keys
  enqueue newly dirty owners
  compute dependency closure
  solve affected SCCs
  publish one new generation
~~~

Each answer carries the dependency keys it observed. Invalidating a key marks the owning query stale. The next demand recomputes it; eager recomputation is a scheduling policy, not an invalidation requirement.

## Cache validity

A reusable answer needs:

~~~text
query key
source revision
semantic generation or input revision
dependency key set
dependency fingerprint
type-system/core revision
solver policy/budget
publication state
~~~

Do not use only file modification time. Do not use only module ID. Do not use only pointer identity.

## Performance mechanism

Precise invalidation improves performance by reducing the amount of semantic work, not by making one solver operation faster.

Measure:

- candidate modules;
- dirtied modules;
- modules actually recomputed;
- queries invalidated;
- queries reused;
- export changes by kind;
- fine-grained versus fallback invalidations;
- reverse-edge traversal count;
- epoch count;
- cycle fallback count;
- queue wait;
- clean/full equivalence.

If a change reduces invalidated modules but changes semantic results, it is a correctness bug, not an optimization.

## Verification gates

- existence-only consumer survives type-only export change;
- type consumer invalidates on type/contract change;
- metadata consumer invalidates on metadata change;
- wildcard consumer invalidates on wildcard-set change;
- class member/MRO change invalidates class-dependent facts;
- body-only edit preserves unchanged surface consumers;
- reverse edges remain symmetric after add/remove/update;
- cycle fallback reaches stable result;
- source revision cannot reuse an old answer;
- clean full rebuild equals incremental recheck;
- diagnostics do not come from stale generations;
- invalidation frontier is deterministic.

## Implementation sequence

1. Reuse current SourceChangeKind and generation model.
2. Define shared DependencyKey and ChangeKind.
3. Record export/type/metadata/class/alias dependencies during semantic lookup.
4. Add reverse module/declaration/callable edges.
5. Compare old/new surface and summary products.
6. Add fine-grained invalidation for exports first.
7. Add declaration/member/callable invalidation.
8. Add cycle fallback and epoch metrics.
9. Differential-test every edit category.
10. Add expression-level dependencies only after a measured invalidation bottleneck.

## Conclusion

Pyrefly achieves incremental performance by knowing exactly what a consumer asked for. Module reverse edges find candidates; fine-grained keys decide relevance; epochs coordinate a run; old/new products generate change sets; cycle fallback protects termination. Phalcom should adopt this layered invalidation engine before attempting more granular expression caches.
