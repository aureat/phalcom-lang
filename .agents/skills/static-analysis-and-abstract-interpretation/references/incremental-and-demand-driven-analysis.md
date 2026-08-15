# Incremental and Demand-Driven Analysis

Incremental static analysis is not “cache the previous answer.” It is a semantic maintenance problem: identify what a source change can affect, retract stale contributions, recompute the dependent frontier to stability, and publish one coherent generation. For a live Phalcom editor, stale-but-plausible facts are correctness bugs.

## 1. Semantic incrementality

Let a semantic query be:

```text
Q(k, I) -> R
```

where `k` is a stable semantic key and `I` is the set of semantic inputs/dependencies. A cached result is valid only while every dependency that can change its meaning remains equivalent under the query's dependency relation.

A useful cache record is conceptually:

```text
CacheEntry {
    key,
    result,
    semantic_dependencies,
    source_generation_or_input_revisions,
    trust/provenance,
}
```

The critical question is not “does the text hash match?” but “are all semantic inputs relevant to this result unchanged?”

## 2. Dependency graph model

Think in layers:

```text
source revision
    ↓
parse/recovery product
    ↓
source semantic surface
    ↓
semantic identities/scopes/occurrences
    ↓
module/import graph
    ↓
class/member/callable surfaces
    ↓
call/field/parameter/type/effect dependencies
    ↓
fixed-point facts/summaries
    ↓
immutable published generation
    ↓
LSP/checker/lint/prover queries
```

A change should invalidate the transitive semantic dependents whose inputs changed—not everything, and not nothing.

## 3. Identity stability is not cache validity

A stable `CallableId` can survive a body edit. That is desirable for references/dependencies. But the cached summary under that ID may be invalid.

Distinguish:

```text
semantic identity: same callable declaration across revision
structural equality: same declaration/body/fact content
revision identity: produced in source/semantic generation N
cache validity: all semantic inputs unchanged
source position: current byte/range location
```

Do not use source offset as durable semantic identity. A comment inserted before a method changes offsets without changing the method's identity or semantics.

CURRENT Phalcom invalidation code explicitly fingerprints declarations without source ranges and compares callable source slices for body-only changes. Tests verify that range shifts do not mark an unchanged callable dirty. Preserve this semantic/source distinction.

## 4. Change classification

Different source edits have different invalidation frontiers.

A useful classification:

```text
BodyOnly
ImportSurface
DeclarationSurface
FileAddedRemoved
CoreSurface
```

This is CURRENT in Phalcom's LSP invalidation layer.

### Body-only edit

Potentially dirty:

- changed callable summary/local facts;
- callers if summary changes;
- parameter contributions emitted by changed caller;
- fields/effects written by changed callable;
- dependent analyses whose inputs actually change.

Unrelated callable declarations/surfaces can remain reused.

### Declaration-surface edit

Potentially dirty:

- class/member lookup surfaces;
- dispatch targets;
- references/occurrences involving changed declarations;
- callers/receiver member queries;
- dependent modules.

### Import-surface edit

Potentially dirty:

- name/class/module resolution;
- module provider graph;
- reachable dispatch surfaces;
- every semantic fact that depended on changed import resolution.

### Core change

Potentially broad. Core classes/selectors/native semantics can affect nearly every file. Use explicit core generation/dependency tracking rather than pretending this is an ordinary local edit.

## 5. Retraction is as important as propagation

Incremental systems must remove facts no longer supported.

Bad design:

```text
parameter[f] = parameter[f] ⊔ newly_seen_argument
```

forever. Deleting a call never removes its old alternative.

Better:

```text
slot -> source -> contribution
joined(slot) = ⊔ contributions(slot)
```

When one source changes:

```text
remove old source contributions
insert replacement source contributions
recompute only touched joins
propagate only semantically changed joined facts
```

CURRENT Phalcom `ParameterContributions` uses exactly this pattern with:

- `by_slot`;
- `slots_by_source` reverse index;
- cached `joined` values;
- `replace_source` returning deltas.

Generalize this principle to future effect/type/proof evidence when retraction matters.

## 6. Reverse dependency edges

Forward dependency:

```text
caller summary -> callee summary
```

is useful for explanation. Incremental invalidation needs reverse edges:

```text
callee -> callers/dependents
```

When callee summary changes semantically, enqueue dependents.

Maintain both directions when removal/update must be efficient:

```text
callable_dependencies: CallableId -> Set<CallableId>
callable_dependents:   CallableId -> Set<CallableId>
```

CURRENT Phalcom `SemanticState` stores both maps.

Whenever a callable's dependency set changes:

```text
old - new: remove reverse edges
new - old: add reverse edges
```

A stale reverse edge wastes work; a missing reverse edge causes stale facts.

## 7. Semantic equality gates propagation

A source body can change without changing its summary:

```text
f() { 1 }
```

versus semantically equivalent formatting/comment edit.

If summary meaning is unchanged, callers need not be recomputed.

Define:

```text
semantic_equal(summary1, summary2)
```

using fields that affect dependents:

```text
parameter facts
return fact
effects
dependencies / target set
other externally visible contracts
```

Exclude:

```text
publication generation
source range shifts if they do not change semantics
allocation identity
nonsemantic provenance sample ordering
```

CURRENT Phalcom tests explicitly verify an unchanged summary stops callable propagation and unrelated published `Arc` products remain pointer-reused.

## 8. Immutable published snapshots

Live analysis usually has two worlds:

```text
mutable worker state
      ↓ atomic publish
immutable query snapshot
```

Queries should never observe:

```text
new local facts + old call summaries
new module graph + old class surfaces
half-replaced parameter contributions
```

CURRENT Phalcom `SemanticEngine` builds mutable/candidate state and publishes `SemanticSnapshot` backed by `Arc` maps. Query paths operate on the immutable snapshot.

### Generation coherence

Assign one semantic generation to a coherent publication. A query can compare its source/document epoch with the snapshot stamp as needed.

Generation identity is provenance, not necessarily semantic equality. Two generations can contain equal facts.

## 9. Copy-on-write candidate state

CURRENT Phalcom cancellation-aware updates clone the semantic engine state (cheaply sharing `Arc` products), apply mutations to a candidate, abandon the candidate if cancelled/stale, and commit it atomically only on success.

This pattern provides strong publication semantics:

```text
live state never half-mutated
cancelled solve never visible
unaffected Arc products reused
```

Trade-off: `Arc::make_mut` can clone maps when modified. Measure candidate clone/map-copy cost as workspace scale grows.

Potential future optimization:

- persistent maps;
- per-shard immutable stores;
- query database with revisioned inputs;
- finer-grained arenas with generation roots.

Do not migrate without profiling.

## 10. Cancellation is a semantic boundary

An editor worker may receive revision `N+1` while solving `N`. Cooperative cancellation should check at bounded work points:

```text
before expensive batch
between worklist items
between modules/callables
before final publication
```

If cancelled:

```text
return no partial result
keep last coherent published generation
```

Do not publish “best effort so far” into the normal semantic snapshot unless the API explicitly represents partial/incomplete generations and consumers understand them.

CURRENT Phalcom callable solver checks a cancellation callback between worklist items and source passes; the engine commits candidate state only after a final stale check.

## 11. Whole-file versus fine-grained incremental parsing

Do not conflate parser granularity with semantic invalidation granularity.

A system can reparse a whole file but still:

- preserve declaration identities;
- fingerprint declaration surfaces;
- compare callable body slices;
- recompute only changed callables and semantic dependents.

This is often simpler and sufficiently fast.

Adopt incremental parsing only if measured parse cost or source-identity stability requires it. Fine-grained parser caches add their own invalidation correctness risks.

## 12. Demand-driven queries

Some semantic facts need not be computed eagerly for the whole workspace.

Demand-driven:

```text
hover(receiver) -> compute/lookup receiver fact
completion -> compute member surface
prove function -> compute expensive proof domain only for requested target
```

Eager:

```text
workspace diagnostics
index/reference surface
cheap common callable summaries
```

Classify analyses by:

```text
cost
query frequency
staleness tolerance (usually none for semantic truth)
need for workspace-wide diagnostics
sharing among consumers
```

A hybrid architecture is often best.

## 13. Query-system architecture

A Salsa/rust-analyzer-style model treats semantic computations as memoized functions of revisioned inputs:

```text
fn class_surface(ModuleId) -> Arc<ClassSurface>
fn callable_summary(CallableId) -> Arc<Summary>
fn references(TargetId) -> Arc<[Occurrence]>
```

The query engine records dependencies automatically.

Benefits:

- demand-driven computation;
- dependency tracking;
- incremental invalidation;
- memoization;
- parallel read snapshots if architecture supports it.

Costs:

- query key design;
- cycle handling;
- memory retention;
- accidental fine-grained fan-out;
- migration complexity;
- debugging hidden dependencies.

Phalcom can evolve toward a query system incrementally. The current explicit engine already has coherent snapshots and dependency edges; do not rewrite it merely to adopt a fashionable framework.

## 14. Cycles in queries

Semantic queries can cycle:

```text
summary(f) -> summary(g) -> summary(f)
```

A general memoization system cannot simply recursively evaluate until stack overflow. Cyclic semantic computations require:

- fixed-point query groups;
- cycle recovery values;
- SCC solvers;
- explicit worklists outside ordinary memoization.

Do not hide recursive abstract interpretation inside a generic “memoized function” abstraction without a cycle protocol.

## 15. Dependency granularity

Possible keys:

```text
ModuleId + source revision
ClassId + declaration-surface revision
CallableId + body revision
ParameterSlot + contribution sources
FieldId + evidence dependencies
Type/Protocol descriptor revision
Native signature/effect version
Core semantic version
```

Choose granularity from measured invalidation needs. Too coarse recomputes unnecessarily; too fine increases bookkeeping and memory.

### Frontier proportionality

A good target:

```text
cost(edit) ≈ cost(changed semantic frontier)
```

not:

```text
cost(edit) ≈ total workspace size
```

for body-local edits whose public summary does not change.

CURRENT Phalcom tests assert precise body frontier behavior: only a changed callable is visited when its summary does not affect unrelated callables; a changed summary propagates to the dependent caller but not an unrelated sibling.

## 16. Module-graph incrementality

Imports introduce a separate dependency layer. Maintain a `ModuleGraph` with provider resolution and dependent closure.

On import/provider change:

```text
repair graph
identify modules whose resolution changed
invalidate semantic surfaces/facts downstream
```

Do not assume file path text alone is the module identity. Use the project's canonical `ModuleId`/source resolver semantics.

For package registry/project evolution, module identity may later include package/version/project context. Keep analysis keys abstract enough to evolve.

## 17. Current Phalcom source-delta classification

At baseline `b5477b74…`, `classify_source_delta`:

- compares imports separately;
- compares a typed declaration fingerprint containing superclass/member selector/side/kind/visibility/constructor/native-return/parameter label+name and fields;
- treats range/body changes separately from declaration surface;
- for body-only changes, compares per-callable source slices to identify exact changed callables;
- separately detects top-level executable source changes.

This is CURRENT implementation. Future type annotations will become part of declaration fingerprints if they affect semantic surfaces/checker facts, even if they do not affect selector identity.

That distinction is important: “type annotations do not change dispatch selector identity” does **not** mean “type annotation changes are body-only.” They change typed declaration metadata and must invalidate type/checker consumers.

## 18. Invalidation for future type/proof facts

Future type system adds new dependencies:

```text
type alias/descriptor changes
generic parameter bounds
protocol/member requirements
annotation changes
inferred signature changes
contract changes
native type/effect signatures
```

A callable's runtime selector can remain unchanged while its checker-facing interface changes. Maintain separate revision/fingerprint dimensions if useful:

```text
DispatchSurfaceRevision
TypingSurfaceRevision
ContractSurfaceRevision
BodyRevision
```

Consumers subscribe to the dimensions they need. This prevents a documentation-only/source-range edit from rebuilding the checker while ensuring a type-bound change does.

## 19. Provenance and invalidation

Provenance can itself depend on source ranges and change even when semantic facts do not. Decide whether a consumer needs refreshed provenance.

Example:

```text
summary returns String semantically unchanged
return statement moved to a new line
```

Caller analysis may not need recomputation. Hover/diagnostic provenance for the callee may need updated source location.

This suggests splitting:

```text
semantic product
presentation/source provenance product
```

or allowing source-local provenance refresh without propagating semantic dependents.

Do not include all ranges in semantic equality just to keep diagnostics current; that can explode invalidation.

## 20. Memory bounds and cache eviction

Incremental systems can leak memory through:

- old generations retained by query handles;
- provenance DAGs;
- contexts/path partitions;
- memoized query results never evicted;
- reverse dependency edges for removed IDs;
- `Arc` cycles (avoid strong cycles in ownership graph);
- retained parsed source/text.

Track:

```text
current snapshot bytes
retained old snapshots
cache entries by query
provenance nodes
contexts/path partitions
removed-ID tombstones
```

Use bounded LRU/weak references/generation pruning where appropriate. Never evict a fact whose absence a consumer interprets as semantic top without a recomputation path.

## 21. Concurrency and snapshot reads

Immutable snapshots enable lock-free or low-lock concurrent reads. Mutable worker ownership can remain single-threaded initially.

If analysis becomes parallel:

- deterministic semantic result must not depend on race order;
- worklist/shared cache synchronization must avoid duplicate expensive work or tolerate it safely;
- publication remains atomic;
- cancellation epochs are thread-safe;
- query caches cannot observe mutable partially initialized entries.

Parallelism is not a substitute for incremental algorithmic efficiency. First minimize work.

## 22. Performance instrumentation

Useful metrics:

```text
parse/rebuild latency p50/p95/p99
semantic generation latency
modules recomputed
callables seeded/visited/changed
parameter sources replaced
slots touched/changed
reverse-dependency fanout
solver rounds/steps
flow passes
widenings/budget exhaustions
Arc product reuse
snapshot memory
query latency
cancellation rate and wasted work
```

CURRENT Phalcom `PerfCounters` already covers many semantic-work and product-reuse dimensions. Keep performance changes evidence-based.

## 23. Failure modes

### Cache keyed only by semantic ID

ID remains stable while body/interface changes. Add dependency/revision validity.

### Full workspace rebuild for every edit

Correct but can destroy editor latency. Use source-delta classification and dependency frontier.

### Hyper-fine invalidation before profiling

Complexity and stale-cache risk exceed benefit. Start at semantic boundaries that can be proved correct.

### Old contribution cannot be removed

Facts only widen across editor history. Use source-indexed contributions.

### Cancellation mutates live state in place

Queries observe partial solve. Use candidate transaction/rollback or versioned immutable products.

### Source ranges in semantic equality

Whitespace/comment edit propagates globally. Separate source provenance from semantic content.

### Semantically changed native/core contract not versioned

Cached callers stay stale even though source files are unchanged. Treat environment/native/core metadata as semantic inputs.

### Query cycles recurse through memoizer

Stack overflow or incomplete result. Use fixed-point cycle protocol.

## 24. Testing obligations

### Edit/rebuild equivalence

For every supported edit sequence:

```text
incremental(final source) == clean_full_rebuild(final source)
```

Compare semantic content, not generation IDs.

### Retraction

- add call contribution;
- add second alternative;
- remove second call;
- verify joined parameter narrows.

### Frontier

- comment/range shift leaves callable semantic summary unchanged;
- body edit visits changed callable;
- unchanged summary does not visit caller;
- changed summary visits caller;
- declaration edit invalidates dispatch dependents;
- import change invalidates resolution dependents.

### Cancellation

- cancel before solve;
- cancel mid-worklist;
- cancel during final stabilization;
- verify live snapshot unchanged;
- next noncancelled generation succeeds.

### Product reuse

Assert unaffected immutable products are pointer-reused where architecture promises it, while semantic equality remains independent of pointer identity.

### Environmental invalidation

Change core/native signature/version without changing user source and verify dependent facts invalidate.

## 25. Review questions

1. What is the cache/query key?
2. What exact semantic inputs make the result valid?
3. Which reverse dependencies are recorded?
4. Can old evidence be retracted?
5. How are body, declaration, import, core, and future typing-surface edits classified?
6. What semantic equality gates propagation?
7. Which provenance changes can be refreshed without semantic propagation?
8. Can cancellation expose partial state?
9. How are old snapshots/caches bounded in memory?
10. Does incremental final state equal a clean rebuild?
11. Is recomputation proportional to the changed semantic frontier?
12. What environment/native/package inputs invalidate facts even if source text did not change?

Incremental analysis is correct only when every answer is explicit.
