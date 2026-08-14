# Incremental and Demand-Driven Analysis

## Dependency-driven recomputation

Cache entries need explicit dependencies:

```text
query key -> result + semantic revision deps
```

When a source contribution changes, invalidate/recompute the transitive dependents whose inputs changed.

## Whole-file versus sub-file

Whole-file semantic recomputation is acceptable when parser/AST updates are whole-file and analyses are fast. Stale fine-grained caches are worse than simpler correct rebuilds.

## Query systems

A Salsa/rust-analyzer-style query architecture treats semantic computations as pure-ish memoized functions of stable inputs. Benefits:

- automatic dependency tracking;
- demand-driven evaluation;
- incremental invalidation;
- easier parallel reads.

Costs:

- key stability design;
- cycles;
- memory/caching policy;
- migration complexity.

Phalcom can evolve toward queries without rewriting every fact at once.

## Immutable snapshots

Current Phalcom LSP publishes coherent immutable snapshots from mutable worker state. Preserve generation consistency: consumers must not observe half-updated call graph/field facts.

## Demand-driven analysis

Some expensive facts should be computed only for queried modules/callables. But editor diagnostics may need eager coverage. Classify analyses by latency budget.

## Invalidation granularity

Potential keys:

```text
ModuleId + revision
Class surface revision
CallableId + body revision
Type/protocol descriptor revision
Native signature version
```

Avoid source byte offsets as durable cross-reparse identities.

## Cache correctness

Cache hits require semantic inputs, not only text hashes if environment/import/core versions can change.

## Performance metrics

Track:

- modules/callables recomputed;
- query latency p50/p95;
- memory retained by snapshots/caches;
- fixed-point iterations;
- invalidation fanout.
