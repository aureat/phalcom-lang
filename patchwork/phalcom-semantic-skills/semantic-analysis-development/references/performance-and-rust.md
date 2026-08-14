# Performance and Rust Engineering for Semantic Analysis

## Cost model

Measure:

1. update/rebuild latency per edit;
2. query latency;
3. memory retained by snapshot(s);
4. publication/clone cost;
5. worst-case pathological source behavior.

## IDs and data locality

Prefer compact typed IDs indexing vectors/maps over nested `Arc` graphs when data is owned by one
semantic store.

Advantages:

- smaller facts;
- cheap equality/hash;
- easier invalidation;
- fewer borrow/lifetime problems;
- predictable traversal.

Do not introduce arena IDs without a clear lifetime/generation story.

## Strings

Selectors/names are common. Avoid repeated allocation where profiling shows it matters:

- intern canonical selectors/symbols if shared infrastructure supports it;
- use IDs for class/module identity;
- store name once in declaration surface and reference ID elsewhere;
- avoid `format!` in hot analysis loops unless for diagnostics/debug.

Correctness first; profile before complex interning changes.

## Collections

`BTreeMap`/`BTreeSet` give deterministic order and are fine for many semantic maps.
For dense numeric IDs, `Vec`/index maps can be faster and smaller.

Choose based on:

- key density;
- mutation pattern;
- deterministic output need;
- snapshot cloning cost.

## Snapshot cloning

Current engine clones data into `Arc`-backed immutable snapshots. As data grows, profile clone
cost.

Possible evolution:

- structural sharing at module granularity;
- `Arc<FileSemanticSnapshot>` reused for unchanged modules;
- immutable arenas with generation ownership;
- copy-on-write maps.

Do not prematurely adopt complex persistent data structures without a measured bottleneck.

## Locking

Keep analysis mutation off query path when possible.

- engine worker lock should protect update transaction;
- snapshot read lock should only clone an `Arc`, then release;
- expensive query should operate on cloned snapshot without holding global lock.

Avoid lock ordering dependencies between semantic DB and LSP document store.

## Recursion depth

ASTs, nested types, imports and call graphs can be adversarially deep.

Protect:

- recursive expression walkers;
- recursive type normalization/substitution;
- inheritance lookup cycles;
- module graph cycles;
- recursive provenance formatting.

Use iterative worklists or depth caps where practical.

## Union/shape growth

Current bounded union cap is a good editor-safety pattern. Similar caps may be needed for:

- structural nesting depth;
- record field count retained in advisory shape;
- path predicates;
- proof obligations;
- diagnostic candidates.

Widen conservatively when cap hit.

## Interprocedural solving

Optimize in this order:

1. correct summary fixed point;
2. reverse dependencies;
3. affected frontier;
4. SCC solving;
5. semantic-change hashing/versioning;
6. parallelism only if needed.

Parallelizing a globally invalidating algorithm often increases complexity without solving main
cost.

## Parallelism

Current engine is deliberately single-threaded mutable worker. This simplifies consistency.
If parallel analysis is introduced:

- partitions must have explicit ownership;
- summary joins must be deterministic;
- publication remains atomic;
- avoid per-fact locks;
- do not expose partial states.

Profile first.

## Allocation discipline

In hot visitors:

- reuse small vectors where safe;
- avoid cloning large `InferredValue` repeatedly if references/cheap clones possible;
- bound provenance;
- collect only events needed by current pass;
- prefer iterator pipelines only when they do not obscure costly clones/allocations.

Rust elegance is secondary to clear semantic ownership and measured cost.

## Error handling

User source errors are ordinary results, not exceptional Rust failures.

Use typed error/recovery states. Reserve `panic!` for violated internal invariants.

Do not use unsafe code for semantic-analysis performance without strong benchmark proof and a
small audited abstraction.

## Instrumentation

Maintain counters around expensive passes:

```text
surface builds
scope builds
flow passes
callable solves
modules/callables recomputed
query counts
widening events
```

Counters make performance regressions diagnosable and can support targeted tests.

## Benchmark scenarios

- one large file edit;
- one leaf module edit;
- one high-fanout API module edit;
- same-file keystroke/incomplete syntax;
- recursive call graph;
- union-heavy dynamic code;
- many same-named classes across modules;
- large core/std import fanout.
