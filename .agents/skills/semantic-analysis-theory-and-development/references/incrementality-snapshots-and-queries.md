# Incrementality, Snapshots, Queries, Cancellation, and Cache Validity

## 1. Incrementality is semantic dependency management

The goal is not “avoid recomputing files.” The goal is:

> Recompute exactly enough of the semantic dependency frontier that the published result equals a clean analysis of the final source, while reusing everything outside that frontier.

This requires explicit dependency ownership and coherent generations.

## 2. Current Phalcom anchor

**CURRENT:** `SemanticEngine` is a mutable single-writer worker; state contains `Arc`-shared files, classes, summaries, field/parameter facts, callable dependency/dependent maps, and module graph. Updates construct a candidate engine state, check cancellation, record reuse, and only replace live state if the candidate remains current. `SemanticDb` publishes an `Arc<SemanticSnapshot>` behind an `RwLock`. Body-only deltas can drive a callable-level frontier; broader source changes expand through module dependents. Parameter contributions can be replaced/retracted by source.

This is a strong current model: **mutable builder/worker + immutable read snapshot**. Future checker/LSP integration should generalize it rather than exposing mutable half-updated tables to queries.

## 3. Generations and coherence

Define a monotonically increasing semantic generation `g`. A published snapshot represents one coherent state:

```text
Snapshot(g) = all source surfaces, identities, facts, summaries, graphs
              computed from one compatible source/configuration universe
```

Queries must not combine `Summary(f, g)` with `ScopeGraph(file, g+1)` unless the system proves those components are reusable and exposes them through the same published generation.

A useful stamp:

```rust
struct SnapshotStamp {
    generation: SemanticGeneration,
    file_revision: FileRevision,
}
```

Consumer caches can reject responses when stamps no longer match.

## 4. Dependency graph

Every derived product should be understood as a query node:

```text
Q(k) = compute(inputs Q1(k1), Q2(k2), ..., source/config facts)
```

Record dependencies from `Q(k)` to inputs it actually reads. On change, invalidate reverse dependents.

For a callable summary:

```text
Summary(A) depends on:
  body(A)
  scope/bindings(A)
  class/member surface relevant to sends
  parameter contributions(A)
  Summary(B), Summary(C) for resolved callees
  native/core contracts read
```

If only body(A) changes and its public surface is identical, unrelated module surfaces need not rebuild.

## 5. Cache specification template

Never approve “add a cache” without all fields:

```text
Key:                what semantic identity/query arguments?
Value:              what immutable derived result?
Validity condition: exactly when is it correct?
Dependencies:       which source/semantic/config nodes were read?
Invalidation:       which events/revisions make it stale?
Concurrency:        who writes, who reads, what synchronization?
Memory bound:       eviction/generation reclamation/intern policy?
Failure/recovery:   can cancelled/partial computation be cached?
```

A cache with no validity rule is technical debt disguised as performance work.

## 6. Change classification

A source edit can be classified by semantic impact, for example:

```text
TextOnlyIrrelevant       formatter/comment changes, if parse semantics unchanged
BodyOnly                 callable body changed, declaration surface stable
DeclarationSurface       selector/field/class/import changed
ModuleResolution         import/provider/package graph changed
GlobalConfig             language/profile/native-contract configuration changed
```

Classification must be conservative. A false “broader change” wastes work; a false “body only” produces stale incorrect semantics. Tests should mutate every surface field to ensure the classifier escalates.

## 7. Retraction is mandatory

Monotone joining is insufficient for incremental deletion/change. If an old call site contributed `String` to a parameter and the call is deleted, the old evidence must disappear.

Use contribution ownership:

```text
joined(slot) = ⊔ contributions(slot, source_i)
```

Replacing/removing `source_i` recomputes only touched slots. The same pattern applies to diagnostics, call graph edges, field evidence, references, and future constraints if they are aggregations from independent sources.

## 8. Cancellation

Editor updates can supersede expensive analysis. Cancellation policy:

1. compute against a private/candidate state;
2. check cancellation at bounded work units (files, SCC nodes, callables, large loops);
3. if stale, abandon candidate without publication;
4. never publish a partially updated generation;
5. avoid poisoning reusable immutable subproducts unless their validity is independently established.

Do not sprinkle cancellation checks inside every tiny operation; choose latency-relevant safe points. Avoid long uninterruptible SCC/body traversals.

## 9. Immutable snapshots and reader concurrency

Read-heavy LSP workloads benefit from lock-light immutable snapshots:

```text
writer: build candidate -> atomic/small-lock publish Arc<Snapshot>
reader: clone Arc<Snapshot> -> query without observing mutation
```

Readers should not hold global locks while performing completion/reference traversals. If a query memo cache exists inside the snapshot, it needs its own concurrency/memory policy and must not mutate semantic truth in a way that changes results.

## 10. Structural reuse versus semantic reuse

`Arc::ptr_eq` can measure product reuse but is not semantic equality. Reuse is safe only if the product's dependencies are unchanged. Copy-on-write storage (`Arc::make_mut`) is effective when most state survives each edit, but measure clone amplification: a single mutation to a large map can copy the entire map.

Possible future improvements include persistent maps, segmented arenas, per-module slabs, or query databases. Adopt them only when profiling shows current COW granularity is costly.

## 11. Query architecture

Protocol-neutral query examples:

```text
occurrence_at(file, offset)
visible_bindings_at(file, offset)
class_surface(class_id)
member_surface(callable_id)
dispatch_candidates(receiver_fact, selector, context)
binding_fact(binding_id, program_point)
callable_summary(callable_id)
references(target)
```

A query should state whether it is exact, best-effort, recovery-aware, or advisory. LSP handlers adapt results; they should not perform hidden whole-workspace analysis on every request.

## 12. Incremental correctness equation

For any edit sequence ending in source universe `S`:

```text
Observe(IncrementalAnalyze(S0 -> ... -> S))
    ==
Observe(FullAnalyze(S))
```

where `Observe` includes all public semantic facts/queries for which equality is defined. This is the central property test for invalidation.

## 13. Performance metrics

Track semantic work, not only wall time:

```text
files/modules reparsed
surfaces/scopes rebuilt
callables analyzed
SCC/worklist iterations
parameter/evidence slots recomputed
Arc/COW state clones
cache hits/misses
published generations
cancelled candidates
query latency p50/p95/p99
snapshot memory / retained generations
```

A “fast” run that publishes stale data is a failure.

## 14. Tests

- body-only edit touches only callable/dependents intended;
- selector/import/class-surface edit expands frontier;
- deleting call site retracts parameter contribution;
- removing module repairs providers/import dependents;
- cancellation before/during/after solving publishes no stale candidate;
- concurrent readers see either old or new complete snapshot, never mixture;
- random edit sequences: incremental observable facts equal clean rebuild;
- repeated edits do not cause unbounded retained snapshots/interned data;
- deterministic results regardless of worklist/map iteration order.

## 15. Review questions

1. What is the semantic dependency frontier for this edit?
2. What product owns each dependency edge?
3. Can evidence be retracted, or only joined forever?
4. What generation validates the query result?
5. Can readers observe partial mutation?
6. Where are cancellation safe points?
7. Is a cache key missing language/config/module assumptions?
8. What bounds memory across many editor revisions?
9. Does the incremental/full equivalence property cover this new fact?
