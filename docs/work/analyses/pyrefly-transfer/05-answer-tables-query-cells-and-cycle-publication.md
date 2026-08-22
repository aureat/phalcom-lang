# Pyrefly answer tables, query cells, and cycle-safe publication

## Purpose

This document isolates the implementation mechanics that make semantic answers reusable and publishable under recursion and concurrency. It covers:

- low-level cached calculations;
- answer tables;
- first-writer-wins slots;
- same-thread and cross-thread recursion;
- SCC-local answer generations;
- trace/error side effects;
- immutable publication;
- memory and failure behavior.

The distinction from the constraint-solving document is deliberate. Constraint solving explains how answers are derived. This document explains how answer state is stored, synchronized, reused, and made visible.

## Evidence boundary

Pinned revision: 43467e64e36550f232a18e89f24fda79b1020b6b.

Primary files:

- [calculation.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_graph/src/calculation.rs) — Calculation state machine, thread-local cycle detection, write-once result.
- [answers.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/alt/answers.rs) — AnswerSlot, AnswerTable, Answers, Solutions, indexes, trace handling.
- [answers_solver.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/alt/answers_solver.rs) — SCC-local storage and commit protocol.
- [module.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/module.rs) — module read/write publication ordering.
- [steps.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/steps.rs) — ArcSwap stage products and release publication.

Local Phalcom mapping:

- phalcom-semantic/src/snapshot.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-semantic/src/types/store.rs
- phalcom-semantic/src/types/evidence.rs

## Two answer mechanisms

Pyrefly contains two related but distinct publication patterns.

### Calculation cell

Calculation<T> is a reusable cached calculation with three states:

~~~text
NotCalculated
Calculating
Calculated
~~~

It owns:

~~~text
atomic status
write lock for SCC batch commit
uninitialized result storage
condition variable
niche field for Option size
~~~

The result is written exactly once before status changes to Calculated. Readers perform an acquire load of status and then read the initialized result.

### AnswerSlot

AnswerSlot<T> is a first-write-wins answer slot backed by an AtomicPtr to an Arc allocation. Pointer states are:

~~~text
null                         unpublished
non-null pointer + pending bit  reserved/pending
non-null pointer, no bit        published
~~~

A slot owns one raw Arc strong reference. Readers clone an Arc from a published pointer. This avoids a mutex on the common read path but requires strict pointer ownership and memory-ordering invariants.

These mechanisms solve publication. They do not decide invalidation or semantic equality.

## Calculation state machine

The low-level Calculation path is:

~~~text
get()
  -> acquire-load status
  -> if Calculated, clone/read result
  -> otherwise return None

calculate(callback)
  -> propose_calculation()
  -> if Calculated, return cached value
  -> if Calculatable, enter thread-local guard
  -> if same-thread guard already exists, return None
  -> evaluate callback
  -> record_value(result)
  -> publish Calculated with release store
~~~

The thread-local set is necessary because a Calculating state deliberately allows a different thread to calculate the same cell. Without local cycle detection, same-thread recursion would re-enter forever.

The source explicitly permits duplicate calculations on different threads. This is intentional: if thread A calculates A and thread B calculates B, and both wait for the other in a mutual cycle, they can deadlock. Allowing both to calculate preserves progress; the first final writer wins.

## Why a second thread may calculate the same value

A naive single-flight cache would make all other threads wait whenever one thread enters Calculating. That is unsafe for mutually recursive calculations:

~~~text
thread 1: calculating A -> waits for B
thread 2: calculating B -> waits for A
~~~

If both cells require the other to publish before proceeding, neither thread can finish.

Pyrefly's policy is:

- same thread: detect recursion and return a cycle-breaking result;
- different thread: permit duplicate calculation;
- final publication: first writer wins;
- later writers: discard their result and use the published value.

This trades duplicate work for deadlock avoidance. Phalcom should measure whether the trade is worthwhile before adding cross-thread waits.

## Atomic status protocol

The status ordering is:

~~~text
NotCalculated -> Calculating -> Calculated
~~~

It is monotonic. The result is initialized before the release-store to Calculated. A reader's acquire-load of Calculated synchronizes with the result write.

The implementation uses AtomicU8 because the state has three values. This keeps the cell compact and makes it possible to pack Option niche behavior with an explicit NonZeroU8 field.

Phalcom safe first implementation:

~~~rust
enum CellState<T> {
    Empty,
    Computing { owner: QueryOwner },
    Complete(Arc<T>),
    Blocked(BlockReason),
    Cancelled,
}
~~~

Use a mutex or worker-owned state first. Move to atomics only after tests establish the publication invariants and profiling shows contention.

## First-writer-wins publication

Calculation::record_value behaves:

~~~text
if already Calculated:
    return published value, did_write = false

lock write gate
wait while an SCC batch owns the gate

if still Calculating:
    write result once
    release-store Calculated
    return value, did_write = true

if another writer finished:
    return published value, did_write = false
~~~

The result returned to the losing calculation is the canonical published result, not the loser's locally calculated result. This ensures that downstream work uses one value.

This is important for diagnostics too. A losing calculation must not publish independent errors/traces that do not correspond to the canonical answer.

## SCC batch commit and answer cells

SCC solving temporarily owns a group of answer cells. The batch path:

~~~text
discover SCC
  -> mark member state
  -> calculate current iteration answers locally
  -> keep side effects in SCC node state
  -> acquire cell write locks in commit order
  -> write final answer for each member
  -> release Calculated status
  -> publish errors/traces associated with those answers
~~~

The write lock exists because ordinary writers and SCC batch publication can otherwise race. Reads do not need the write lock; only result publication does.

The commit order must be deterministic or at least semantically independent of order. If diagnostics order is observable, sort diagnostics after publication by source position and diagnostic identity.

## Answer table and index structure

Answers owns more than a map of BindingId to TypeId. It includes:

~~~text
Solver
AnswerTable
Solutions
optional Index
optional Traces
~~~

The index tracks references such as:

- external references;
- attribute references;
- constructor references;
- parent method references.

These indexes serve downstream queries and reports. They are enabled only when a Require level needs them. This avoids paying the memory cost for every CLI or LSP request.

Phalcom should distinguish:

~~~rust
struct AnswerTable {
    facts: IndexMap<CalcId, Arc<CommittedFact>>,
}

struct AnswerIndex {
    references: Arc<[ReferenceEdge]>,
    member_targets: Arc<[MemberTarget]>,
    constructor_targets: Arc<[CallableId]>,
}

struct AnswerTrace {
    demand_edges: Arc<[DemandEdge]>,
}
~~~

Do not retain indexes and traces by default if the consumer does not request them.

## Empty, computing, blocked, and cancelled are different

An absent answer slot can mean several things. The implementation must preserve the state:

~~~text
uncomputed:
    no work has started

computing:
    a query is active; recursive re-entry may require placeholder

complete:
    final answer is published for the revision

blocked:
    dependency or dynamic boundary prevents completion

cancelled:
    candidate was abandoned and must not be reused
~~~

A cancelled result is not an Unknown semantic fact. A blocked result is not Dynamic. A computing cell is not an error.

## Trace side effects

Pyrefly keeps traces separate from answer values. A trace may describe:

- why an answer was demanded;
- which bindings contributed;
- which dependency edge was used;
- which recursive break occurred;
- which diagnostic was produced.

Trace collection can be disabled for ordinary fast paths. When enabled, trace side effects are attached to the calculation/SCC and merged only if the answer commits.

Phalcom should use the same policy:

~~~rust
struct QueryTrace {
    dependencies: SmallVec<[DependencyKey; 4]>,
    steps: SmallVec<[TraceStep; 8]>,
    truncated: bool,
}
~~~

Bound traces by depth/count. Do not let explainability become an unbounded memory leak.

## Memory ownership and destruction

Calculation stores an uninitialized result and drops it only if status is Calculated. The status is therefore the initialization marker. AnswerSlot owns one raw Arc strong reference and must release it exactly once on destruction.

These optimizations save allocations and locks, but they are unsafe if ownership contracts are violated. Phalcom must not copy the raw-pointer approach until it has:

- safe reference implementation;
- Miri/sanitizer or equivalent concurrency testing;
- tests for drop exactly once;
- tests for pending/published transitions;
- tests for cancellation and abandoned candidates;
- explicit Send/Sync invariants.

## Safe Phalcom design first

Start with a worker-owned query table:

~~~rust
struct QueryCell<T> {
    state: QueryState<T>,
    owner: Option<QueryOwner>,
    dependencies: SmallVec<[DependencyKey; 4]>,
    generation: SemanticGeneration,
}

enum QueryState<T> {
    Uncomputed,
    Computing { cycle: CycleId },
    Complete(Arc<T>),
    Blocked(BlockReason),
    Cancelled,
}
~~~

Readers access immutable snapshot products. The worker owns mutation and publication. This fits the current SemanticEngine architecture and avoids making every LSP request contend on a global lock.

If profiling shows high read contention, replace only the hot state transition with an atomic cell. Keep the semantic API and tests unchanged.

## Generation and cache validity

Every answer must carry or be reachable through:

~~~text
source revision
semantic generation
dependency fingerprint
type-system/core revision
solver policy/budget class
~~~

A pointer that is still alive is not necessarily valid. Arc lifetime answers memory safety, not semantic validity.

Phalcom should define:

~~~rust
struct AnswerValidity {
    source_revision: SourceRevision,
    semantic_generation: SemanticGeneration,
    dependency_fingerprint: DependencyFingerprint,
    core_revision: CoreRevision,
    solver_policy: SolverPolicyId,
}
~~~

Reuse requires equality of the relevant validity fields. Pointer reuse can be measured separately.

## Interaction with cancellation

A query may be cancelled after computing a local value but before publication. The local value must be dropped or retained only in a private candidate. It must not enter the shared answer table.

Publication sequence:

~~~text
calculate locally
  -> check cancellation
  -> validate dependency revisions
  -> acquire publication ownership
  -> check cancellation/revision again
  -> publish complete fact
~~~

The second check matters because cancellation or a newer edit can arrive while the calculation is finishing.

## Interaction with invalidation

Publication and invalidation are different operations:

~~~text
invalidation:
    decides whether an old answer remains eligible

query cell:
    records current state and prevents duplicate unsafe publication

solver:
    derives a value

snapshot:
    makes a complete generation visible
~~~

Never invalidate by deleting an answer without recording why it became stale. The reason is needed to test dependency correctness and diagnose unexpected recomputation.

## Performance model

The answer layer saves work through:

- O(1) or compact indexed lookup;
- acquire-only reads of completed facts;
- no reader mutex for immutable products;
- demand-driven calculation;
- query-local recursive memoization;
- first-writer-wins deduplication;
- SCC-local batch publication;
- optional trace/index retention;
- product eviction;
- duplicate cross-thread computation instead of deadlock waits.

The main costs are:

- Arc cloning;
- atomic state traffic;
- duplicate work under contention;
- SCC write-lock coordination;
- trace/index retention;
- memory held by old snapshots;
- unsafe implementation complexity if raw pointers are used.

Measure the trade-off instead of assuming lock-free is faster.

## Phalcom implementation sequence

1. Introduce QueryKey, QueryState, and QueryAnswer in a worker-owned table.
2. Add generation/dependency validity metadata.
3. Add same-thread query stack and cycle result.
4. Add immutable snapshot publication.
5. Add SCC-local batch commit for recursive facts.
6. Add optional traces/indexes behind a require policy.
7. Measure lock/Arc/duplicate-work cost.
8. Optimize the hot state transition only if required.
9. Add concurrency tests before any unsafe pointer/tag implementation.
10. Keep LSP request handlers as read-only query clients.

## Verification gates

- same-thread cycle terminates;
- mutual cross-thread cycle does not deadlock;
- first writer wins and losers observe canonical value;
- result is initialized before readers can see Calculated;
- drop occurs exactly once;
- cancelled candidates never publish;
- stale generation cannot overwrite newer generation;
- traces from abandoned calculations do not leak;
- eviction leaves requested consumers functional;
- serial and concurrent reads return identical semantic facts.

## Metrics

Record:

- query-cell hit/miss rate;
- computing re-entry count;
- duplicate cross-thread calculations;
- first-writer wins/losses;
- average publication wait;
- SCC write-lock wait;
- Arc clones;
- retained snapshot bytes;
- evicted AST/answer bytes;
- trace/index retention;
- cancelled calculations;
- stale publication attempts.

## Conclusion

Pyrefly's answer performance is implemented through precise state transitions and ownership rules. The transferable design is not merely “cache answers”; it is a cache with cycle state, generation validity, safe publication, side-effect isolation, retention policy, and measurable contention. That is the level Phalcom needs before any lock-free or unsafe optimization.
