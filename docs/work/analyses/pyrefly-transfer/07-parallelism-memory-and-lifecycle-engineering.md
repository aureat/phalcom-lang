# Pyrefly parallelism, memory, and lifecycle engineering

## 1. Scope and purpose

This dossier isolates execution ownership, parallel work, publication safety, allocation lifetime, and eviction. It answers one transfer question: which concurrency and memory mechanisms make Pyrefly fast without making Phalcom's first semantic engine unsafe or nondeterministic?

The recommendation is conservative. Phalcom should first make one worker authoritative over mutable semantic state and publish immutable generations. Parallelism should enter at explicit module or query boundaries after counters prove that serialized analysis is the bottleneck. Pyrefly's low-level raw-pointer publication is a later optimization, not a starting contract.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use local checkout /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6, inspected 2026-08-22.

| Mechanism | Pinned source |
| --- | --- |
| Calculation status, UnsafeCell, MaybeUninit, condvar, first-writer publication | crates/pyrefly_graph/src/calculation.rs |
| Query answer slots and pending reservations | pyrefly/lib/alt/answers.rs |
| SCC-local ownership and batch publication | pyrefly/lib/alt/answers_solver.rs |
| Module compute guard, epochs, and eviction | pyrefly/lib/state/module.rs |
| Staged products and ArcSwap publication | pyrefly/lib/state/steps.rs |
| Retention policy and epoch loop | pyrefly/lib/state/state.rs |
| Threaded worker, cancellation, and immutable snapshots | phalcom-lsp/src/analysis_service.rs, phalcom-lsp/src/semantic/engine.rs, phalcom-lsp/src/semantic/snapshot.rs |
| Runtime counters and spans | phalcom-lsp/src/perf.rs |

The GitHub source mirror is pinned at [Pyrefly commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

Pyrefly has two different parallelism layers:

1. module scheduling, where independent modules can compute concurrently;
2. calculation cells, where multiple threads may race to calculate one query and the first successful publication wins.

These layers are safe because mutable calculation state is guarded, readers acquire before reading, publication is one-way, SCC side effects remain private until commit, and module steps have explicit epochs and clean-state synchronization.

Phalcom currently has a dedicated phalcom-lsp-analyzer worker. WorkerShared owns pending work, an epoch, cancellation state, and publication counters; SemanticEngine is mutable and worker-owned; SemanticSnapshot is immutable and read-only. This is already a viable lifecycle boundary.

**PROPOSED / PHALCOM:** keep worker ownership as the initial invariant. Add parallel module jobs only when each job consumes an immutable input snapshot, writes an isolated result, and commits through one deterministic owner. Do not introduce shared mutable type variables or shared UnsafeCell answer storage in the first implementation.

## 4. Pyrefly execution path

The relevant Pyrefly path is:

1. A module has staged products: load, AST, exports, answers, and solutions.
2. ModuleState uses an epoch and dirty state to decide whether a product may be reused.
3. A compute guard prevents redundant module work when another thread is computing the same step.
4. A demand-driven query enters an answer table.
5. A calculation cell checks its atomic status.
6. Same-thread recursion is detected through thread-local calculation state; cross-thread calculation may proceed rather than wait on a cycle.
7. A query computes into local state and records an answer.
8. An SCC stores answers, diagnostics, and traces in SCC-local state.
9. The SCC reserves result slots, publishes all members, or rolls reservations back.
10. Module errors and traces become visible only through committed products.
11. Later steps may evict AST or answer products when downstream products retain enough information.

The important property is not “many threads.” It is that every boundary says who may mutate, who may read, which generation is current, and which side effects are allowed to escape.

## 5. Concrete data structures

### Calculation cells

Calculation<T> contains:

- atomic status: NotCalculated, Calculating, or Calculated;
- a mutex used for exclusive write access;
- UnsafeCell<MaybeUninit<T>> for in-place result storage;
- a condition variable for waiters;
- a niche-optimized representation for the cell state.

The unsafe Sync implementation relies on a narrow invariant: the result is initialized before the release transition to Calculated, and readers perform an acquire load before reading it.

AtomicStatus::start_calculating allows another thread to calculate even when status is already Calculating. That avoids a cross-thread mutual-cycle deadlock at the cost of duplicate computation. The write path remains first-writer-wins.

### Answer slots

AnswerSlot<T> stores an atomic pointer representing an owned Arc strong reference. A null pointer means unpublished; a pending bit means an SCC reservation exists; otherwise the pointer is the published answer. Reservations make group publication possible without exposing a partial SCC.

### Module state

ModuleState retains:

- current required stage;
- current and computed epochs;
- dirty flags;
- staged Arc products;
- a computing flag protected by a mutex;
- a condition variable for module waiters.

After a downstream step is complete, ModuleState can evict AST or answer products. Eviction is valid only because the retained product no longer requires the evicted representation.

### Phalcom current structures

WorkerShared owns:

- epoch: AtomicU64;
- pending-work mutex and condition variable;
- shutdown flag;
- open-document set;
- per-URI source epochs;
- scan state and performance counters.

SemanticEngine owns mutable SemanticState and counters on one worker. SemanticSnapshot owns Arc-backed maps for files, classes, callable summaries, field facts, parameter facts, module graph, and documents. publish_engine creates the next immutable snapshot and publishes it through the database.

## 6. State machines and transitions

### Pyrefly calculation cell

~~~text
NotCalculated
    -> Calculating       one thread starts, another may duplicate
Calculating
    -> Calculated        first valid writer stores value, release-publishes
Calculating
    -> Calculating        duplicate or recursive work; no partial read
~~~

There is no meaningful “failed but cached as valid” transition in the result cell. Errors and traces are separate side effects and must be associated with the committed answer.

### Pyrefly module

~~~text
clean at epoch E
    -> dirty at E+1
    -> computing
    -> clean at E+1 with retained products
    -> evicted intermediate products
~~~

### Phalcom batch

~~~text
pending
    -> claimed(batch_epoch)
    -> computing with cancellation checks
    -> cancelled/stale and discarded
    -> candidate semantic state
    -> published generation
~~~

Only the worker may move mutable engine state. Read-side clients see either the prior immutable generation or the newly published one.

## 7. Cache keys and validity

Parallel execution does not define cache validity. Validity requires explicit keys:

| Product | Minimum key |
| --- | --- |
| source surface | canonical file identity, source revision |
| module interface | canonical module ID, project/universe revision, source revision |
| callable summary | callable identity, body revision, dependency fingerprint |
| type query | query identity, semantic generation or dependency fingerprint |
| diagnostic facts | diagnostic identity, source revision, semantic generation |
| LSP response | document revision, snapshot generation, request parameters |

An Arc only manages lifetime. It does not prove that a product belongs to the current source revision. A lock only protects a transition. It does not prove semantic freshness.

## 8. Ownership and concurrency

### Pyrefly

Pyrefly separates:

- thread-local solver and SCC state;
- shared answer cells;
- module-level compute coordination;
- immutable or atomically swapped staged products;
- error and trace accumulation that is committed after calculation.

Cross-thread duplicate calculation is acceptable only where calculations are side-effect free until publication. Shared mutable solver variables would violate this assumption.

### Phalcom first implementation

Use this ownership rule:

~~~text
LSP/request threads -> immutable SemanticSnapshot only
worker thread       -> mutable SemanticEngine and candidate state
filesystem/scanner  -> source bytes and pending work only
publication         -> one worker-controlled database swap
~~~

Request code must never call Arc::make_mut on an engine-owned object, wait on a semantic mutex while holding an LSP request lock, or mutate diagnostic state in place after publication.

### Parallel extension

When needed, introduce AnalysisJob:

- immutable input generation;
- sorted module/job IDs;
- isolated output products;
- cancellation token tied to batch epoch;
- result envelope carrying input generation and dependency fingerprint.

The worker joins jobs, sorts results by stable identity, validates all preconditions, and publishes one generation. Parallel jobs must not directly publish individual files.

## 9. Memory and allocation

### What Pyrefly optimizes

Pyrefly uses Arc to share staged products, ArcSwap for atomic product replacement, in-place answer cells for hot query results, and explicit eviction to limit retained AST and answer memory. It also avoids allocating trace sinks during cold calculations when traces will be discarded, while still installing a sink in iterative paths to prevent trace leakage.

The state.rs retention comments show that queue policy affects multi-gigabyte workloads. Retention is therefore a measured policy, not a cleanup detail.

### Safe Phalcom baseline

Start with:

- immutable Arc snapshots;
- BTreeMap or indexed vectors where deterministic iteration matters;
- candidate-state copy-on-write at file and product granularity;
- no unsafe interior result storage;
- explicit Drop/eviction counters only after product dependencies are documented.

Prefer arena or interning work only behind a TypeStore seam. Do not turn every semantic fact into an arena allocation before measuring lookup, cloning, and memory retention.

### Eviction contract

Each product type needs a retention predicate:

~~~text
retain(product) iff some retained product or configured query can still read it
~~~

Eviction must be generation-scoped and observable. A discarded product must be recomputable without changing semantic identity or diagnostic ordering.

## 10. Complexity and performance

Primary costs:

- module scheduling: O(number of runnable modules plus dependency edges);
- duplicate calculation: extra solve cost proportional to contested queries;
- SCC commit: O(number of SCC members plus side effects);
- snapshot publication: O(number of changed product maps) with structural sharing;
- eviction: O(number of evicted products), plus later recomputation;
- lock contention: workload-dependent and invisible in pure semantic counters.

Measure separately:

- queue wait time;
- worker compute time;
- duplicate query work;
- mutex/condvar wait time;
- bytes allocated and retained by product class;
- snapshot clone/share ratio;
- stale work discarded;
- cancellation checks and cancellation latency.

Do not optimize by adding parallel workers until these counters distinguish CPU saturation from lock, allocation, filesystem, or invalidation cost.

## 11. Failure, cancellation, recursion, and cycles

Failure classes:

- cancellation: computation intentionally stops; no product is published;
- stale batch: computation completed for an obsolete source epoch; discard result;
- semantic failure: product may contain unknown/error facts and can be published if policy allows;
- internal failure: preserve prior generation, emit observable worker error, and reset processing state;
- recursion: use SCC-local placeholders or a bounded unknown result;
- cross-thread contention: duplicate only if calculation is pure until publication.

Phalcom's worker already checks shutdown or epoch mismatch through a cancellation closure and emits StaleBatchDiscarded when a completed batch no longer matches current work. Extend this rule to every parallel job result.

Never use a lock wait as cycle handling. If ownership graph says A waits for B and B can wait for A, introduce SCC/cycle state or return a bounded placeholder.

## 12. Phalcom mapping

| Pyrefly mechanism | Phalcom mapping |
| --- | --- |
| staged module products | SemanticState products plus SemanticSnapshot |
| module epoch | worker epoch and source revisions |
| calculation cell | future QueryCell owned by semantic query engine |
| SCC-local answer state | solver-local SccWork |
| atomic final publication | worker-only snapshot publication |
| retained/evicted steps | explicit source/surface/body product retention |
| duplicate cross-thread calculation | deferred until query purity is proven |
| trace/error commit | separate diagnostic and trace products |
| demand scheduling | pending work and dirty callable frontier |

## 13. Mechanisms not copied

Do not copy:

- UnsafeCell<MaybeUninit<T>> before a measured need and a reviewed unsafe proof;
- raw Arc pointer tagging;
- cross-thread duplicate solving with mutable shared type variables;
- condvar waits as a substitute for cycle detection;
- Python module import semantics;
- Pyrefly retention thresholds or queue policy without Phalcom measurements;
- implicit “first writer wins” where outputs depend on nondeterministic iteration;
- global locks around all semantic queries;
- a second mutable semantic engine per request thread.

## 14. Proposed Phalcom data structures

~~~text
AnalysisEpoch(u64)
SourceRevision(u64)
SnapshotStamp { generation, epoch }
CancellationToken { epoch, shutdown }
AnalysisJob { input_stamp, module_id, kind, dependencies }
JobResult { input_stamp, product, diagnostics, traces, metrics }
PublicationBatch { stamp, products, diagnostics, traces, metrics }
RetentionClass { Keep, EvictAfter(kind), Recompute }
~~~

Required invariants:

1. every result carries the input stamp used to compute it;
2. every mutable job has one owner;
3. publication rejects stale stamps;
4. diagnostics and traces cannot outlive their source revision;
5. deterministic ordering is established before publication;
6. eviction cannot remove a product required by a retained product.

## 15. Proposed APIs and module seams

Candidate seams:

- phalcom-lsp/src/analysis_service.rs: scheduling, cancellation, join, publication;
- phalcom-lsp/src/semantic/engine.rs: mutable worker-owned computation;
- phalcom-lsp/src/semantic/snapshot.rs: immutable read product;
- phalcom-lsp/src/semantic/query.rs: stamped query keys;
- phalcom-semantic/src/query_cell.rs: future calculation cells, initially mutex-based;
- phalcom-semantic/src/publication.rs: deterministic batch validation and commit;
- phalcom-semantic/src/retention.rs: product lifetime and eviction policy;
- phalcom-lsp/src/perf.rs: queue, contention, allocation, and stale-work metrics.

Suggested API shape:

~~~text
engine.compute(job, cancel) -> Result<JobResult, ComputeError>
publication.validate(batch, current) -> PublicationDecision
database.publish(batch) -> PublishedGeneration
snapshot.query(key) -> QueryResult
~~~

compute must not publish. publish must not perform deep semantic computation.

## 16. Implementation order

1. Document current worker ownership and stamp every existing publication event.
2. Add cancellation checks to every expensive frontier loop.
3. Add queue wait, compute, stale, and retained-product counters.
4. Split diagnostics/traces from semantic product ownership.
5. Introduce isolated module jobs with serial deterministic join.
6. Add bounded parallel execution behind a configuration gate.
7. Only after profiling, consider ArcSwap or specialized cells.
8. Review unsafe code separately if low-level publication becomes necessary.

## 17. Tests

Required tests:

- shutdown during source scan leaves worker joinable;
- newer update cancels or discards older batch;
- two requests observe a complete old or complete new snapshot;
- failed batch preserves prior published generation;
- deterministic publication is independent of job completion order;
- module job cannot mutate input snapshot;
- eviction followed by recomputation preserves semantic result;
- recursion terminates without a lock cycle;
- duplicate work never publishes conflicting answers;
- retained diagnostic facts match retained source revision.

Use phalcom-lsp/tests/analysis_status.rs, performance.rs, workspace_semantics.rs, and semantic_consistency.rs as integration anchors. Add direct unit tests beside new ownership and publication modules.

## 18. Benchmarks and metrics

Minimum counters:

- jobs submitted/completed/cancelled/stale;
- queue wait and compute duration;
- duplicate calculations;
- lock wait and condvar wait;
- snapshot bytes shared/cloned;
- product bytes retained/evicted/recomputed;
- maximum live generations;
- diagnostic and trace products retained;
- publication rejection count.

Benchmark workloads:

1. one large module;
2. many independent modules;
3. deep import chain;
4. diamond dependency graph;
5. recursive callable SCC;
6. rapid edits to one open document;
7. workspace scan interrupted by interactive edits;
8. memory-limited repeated recheck.

Record p50, p95, and maximum latency; averages hide stale-batch and contention spikes.

## 19. Risks and open questions

- Can Phalcom type queries be made pure enough for duplicate cross-thread calculation?
- Which products are safe to evict while LSP requests hold a snapshot?
- Does BTreeMap iteration dominate snapshot cost compared with indexed storage?
- How should native descriptors participate in job dependencies?
- Should parallel jobs share a TypeStore, or receive immutable store snapshots?
- What cancellation latency is acceptable for an editor request?
- How many generations may remain alive under slow clients?
- Which unsafe optimization, if any, is justified by measurements?

These remain **OPEN / UNVERIFIED** until benchmark evidence exists.

## 20. Final transfer checklist

- [x] Pyrefly worker/module/cell execution path identified.
- [x] Atomic publication and SCC side-effect ownership separated.
- [x] Current Phalcom worker and immutable snapshot boundary recorded.
- [x] Safe first implementation keeps mutable semantic state worker-owned.
- [x] Cache validity is stamped independently of Arc lifetime.
- [x] Cancellation, stale results, recursion, and eviction have explicit policies.
- [x] Parallelism is gated by measurements and deterministic join.
- [x] Unsafe pointer publication is deferred.
- [ ] Phalcom contention and allocation baselines measured.
- [ ] Parallel job API implemented and stress-tested.
