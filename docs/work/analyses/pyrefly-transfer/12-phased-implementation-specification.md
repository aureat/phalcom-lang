# Phalcom phased implementation specification

## 1. Scope and purpose

This is the execution specification for the Pyrefly-to-Phalcom transfer. It converts the preceding dossiers into nine ordered phases, numbered 0 through 8. Each phase has concrete repository seams, data structures, APIs, invariants, ownership rules, failure behavior, tests, benchmark work, migration risks, completion gates, and explicit non-goals.

This document specifies architecture and work order. It does not claim that any phase is implemented.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6.

**CURRENT / PHALCOM** observations use the current checkout. The existing worker, semantic engine, snapshots, module resolver, graph products, diagnostics, and performance counters are described in dossiers 02, 05, 06, 07, 08, 09, and 11.

Primary Phalcom seams:

- phalcom-lsp/src/analysis_service.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/snapshot.rs
- phalcom-lsp/src/semantic/query.rs
- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-lsp/src/perf.rs
- phalcom-semantic/src
- phalcom-modules/src
- phalcom-lsp/tests
- phalcom-semantic/tests
- phalcom-modules/tests

Source mirror: [Pyrefly commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

Implement in this order:

~~~text
0 measure and inventory
1 canonical identities and indexes
2 bindings and flow
3 TypeStore and equality
4 constraints and subset relations
5 query cells and SCC answers
6 module products and invalidation
7 worker and LSP publication
8 memory optimization and bounded parallelism
~~~

The order protects semantics. Identity and ownership must settle before caching; type equality must settle before fixed-point comparison; module products must settle before broad invalidation; stamped publication must settle before LSP parallelism.

Each phase may ship a smaller vertical slice. A later phase may use an adapter around an earlier product, but it must not bypass the earlier ownership and identity contract.

## 4. Pyrefly execution path being transferred

The transfer target combines:

1. staged module products;
2. canonical semantic identities;
3. indexed bindings and flow versions;
4. demand-driven query keys;
5. canonical type construction and semantic equality;
6. bounded constraint solving;
7. SCC-local placeholders and batch publication;
8. interface-aware module invalidation;
9. worker-owned mutable construction;
10. immutable generation snapshots;
11. separate diagnostics, traces, and metrics.

The target remains Phalcom-specific at expression semantics, selector dispatch, classes/metaclasses, native descriptors, reflection, dynamic values, and runtime behavior.

## 5. Concrete data structures

Foundational values:

~~~text
FileId, ModuleId, SourceRevision, ProjectRevision
BindingId, ScopeId, CallableId, SelectorId, ClassId
TypeId, QueryId, DiagnosticId, SnapshotStamp
~~~

Product families:

~~~text
SourceSurface
DeclarationSurface
ExportSurface
ModuleEnvironment
BindingIndex
FlowFacts
TypeStore
ConstraintSet
QueryAnswer
SccResult
CallableSummary
DispatchSummary
DiagnosticFileProduct
TraceFileProduct
SemanticSnapshot
~~~

Every product includes the minimum source, project, and semantic stamp needed to reject stale use.

## 6. State machines and transitions

The global lifecycle is:

~~~text
source mutation
    -> source revision
    -> affected identities
    -> candidate products
    -> query/SCC evaluation
    -> diagnostics and traces
    -> publication validation
    -> immutable generation
~~~

Failure transitions are explicit:

~~~text
candidate -> cancelled
candidate -> stale
candidate -> failed, preserving prior generation
candidate -> published
~~~

No phase may silently turn cancellation, stale work, unknown knowledge, semantic error, and internal failure into one status.

## 7. Cache keys and validity

All cache keys must state:

- semantic owner;
- input source revision;
- project/module/interface revision;
- query kind;
- dependency fingerprint;
- configuration or policy revision where relevant.

Cache values must state:

- computed stamp;
- dependency facts;
- result status;
- retained inputs;
- diagnostics/traces owner if present.

Equality of cache keys is not equality of semantic results. Both require tests.

## 8. Ownership and concurrency

Initial ownership:

~~~text
LSP threads -> immutable snapshots
worker      -> mutable candidate engine
solver      -> local mutable variables and SCC state
resolver    -> project/interface products through controlled owner
publisher   -> one deterministic commit point
~~~

Phases 0–7 must work with one semantic worker. Phase 8 may add isolated parallel jobs. No phase authorizes shared mutable solver state or unsafe publication.

## 9. Memory and allocation

Long-lived:

- canonical identity indexes;
- project/module keys;
- immutable published products while referenced;
- compact native and reflection descriptors.

Revision-scoped:

- source text;
- syntax and body products;
- flow facts;
- diagnostics and traces;
- query answers with source dependencies.

Eviction is allowed only after a product dependency table and recomputation test exist.

## 10. Complexity and performance

Every phase adds counters before claiming an optimization:

- work submitted and completed;
- cache hits and misses;
- affected identities;
- solver rounds and SCC iterations;
- bytes allocated and retained;
- stale/cancelled work;
- snapshot sharing;
- diagnostic publication latency.

Benchmark clean, warm, body edit, declaration edit, import edit, deletion, and workspace scan cases.

## 11. Failure, cancellation, recursion, and cycles

Every phase must define:

- safe fallback value;
- whether fallback is cacheable;
- diagnostic policy;
- cancellation point;
- stale-result check;
- recursion/cycle behavior;
- recovery path.

Semantic SCCs may converge. Runtime dependency cycles remain runtime errors where current module policy requires acyclicity.

## 12. Phalcom mapping

The phase map is:

| Phase | Primary result |
| --- | --- |
| 0 | measured baseline and accepted boundaries |
| 1 | stable semantic identities and indexes |
| 2 | binding/flow facts and incremental body frontier |
| 3 | canonical TypeStore and equality |
| 4 | constraints and bounded relations |
| 5 | demand queries, SCC answers, and publication |
| 6 | module interfaces, environments, and invalidation |
| 7 | worker snapshots, diagnostics, traces, and LSP |
| 8 | retention, memory, isolated parallelism, and tuning |

## 13. Mechanisms not copied

The implementation must not copy:

- Python-specific type or import semantics;
- Pyrefly raw pointer publication before unsafe review;
- all-global semantic caches;
- path strings as logical identity;
- diagnostics as cache validity;
- closed-world selector dispatch;
- full arena-interning claims without evidence;
- parallel mutable solver state;
- Pyrefly benchmark thresholds as Phalcom acceptance thresholds.

## 14. Proposed Phalcom data structures

Phase-specific structures are listed below. Names are provisional; invariants are binding.

~~~text
RevisionStamp
IdentityIndex
BindingKey
FlowVersion
TypeStore
RelationKey
QueryKey
QueryCell
SccWork
ModuleAvailability
InterfaceDependency
PublicationBatch
DiagnosticSnapshot
TraceSnapshot
~~~

## 15. Proposed APIs and module seams

The implementation should add small modules rather than enlarge one engine file:

- phalcom-semantic/src/identity.rs
- phalcom-semantic/src/bindings.rs
- phalcom-semantic/src/flow.rs
- phalcom-semantic/src/types/store.rs
- phalcom-semantic/src/types/equality.rs
- phalcom-semantic/src/relations.rs
- phalcom-semantic/src/query.rs
- phalcom-semantic/src/scc.rs
- phalcom-semantic/src/publication.rs
- phalcom-semantic/src/diagnostics.rs
- phalcom-semantic/src/traces.rs
- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-lsp/src/semantic/snapshot.rs

Existing modules remain compatibility adapters until each replacement product passes its phase gate.

## 16. Implementation order

Do not skip phases because a later subsystem can be prototyped faster. A prototype may use adapters, but the production path must enter through the phase's API and tests.

## 17. Tests

Every phase below has required unit, component, integration, property/stress, and manual tests where applicable.

## 18. Benchmarks and metrics

Every phase below names minimum measurements. Phase 0 owns the baseline; later phases report deltas against it and explain unrelated variance.

## 19. Risks and open questions

Cross-phase risks:

- current semantic facts may not yet have stable identity;
- current TypeStore may not support all target relations;
- module interfaces and LSP snapshots may use different generation notions;
- diagnostics conversion exists before complete semantic publication;
- worker cancellation can expose races when new parallel products are added;
- concurrent worktree changes may alter paths or test assumptions.

Resolve each risk at the earliest phase that owns its invariant.

## Phase specifications

### Phase 0 — Measurement and current inventory

**Objective.** Record current behavior and establish acceptance vocabulary before architectural changes.

**Files to create or modify.**

- docs/work/analyses/pyrefly-transfer/README.md
- docs/work/analyses/pyrefly-transfer/10-testing-benchmarking-and-evidence-architecture.md
- phalcom-lsp/tests/performance.rs
- optional benchmark fixture directory under phalcom-lsp/tests/fixtures

**Data structures and APIs.**

- EvidenceCase;
- BenchmarkProfile;
- current CounterSnapshot export for test consumption;
- clean-versus-incremental harness.

**Invariants.**

1. current source/spec behavior is recorded before changing semantic ownership;
2. every metric has a named workload;
3. baseline, unrelated, deferred, and unverified results remain distinct;
4. no benchmark result is treated as semantic proof.

**Ownership.** Test harness owns fixtures and scheduler controls. Production worker ownership remains unchanged.

**Failure behavior.** A missing benchmark dependency is a deferred evidence gap, not a failed semantic claim. A harness error must name fixture and stage.

**Tests.**

- current module, checker, LSP, diagnostics, and workspace tests;
- clean-versus-incremental equivalence for existing fixtures;
- repeated-run deterministic serialization;
- worker stale/cancel event assertions where current APIs expose them.

**Benchmarks.**

- cold startup;
- warm no-op query;
- body edit;
- declaration edit;
- import edit;
- workspace scan with interruption.

**Migration risks.** Existing counters may be cumulative or not externally stable; existing integration tests may include baseline behavior.

**Completion gates.**

- baseline command list recorded;
- current status and diff boundary cleanly identified;
- at least one fixture per edit class;
- metrics output reproducible enough to compare;
- no source semantic behavior changed.

**Explicit non-goals.**

- no new solver;
- no parallel worker;
- no cache redesign;
- no message wording migration;
- no acceptance threshold invented from one machine.

### Phase 1 — Canonical identities and indexed storage

**Objective.** Make source, module, declaration, binding, callable, selector, scope, and query identities explicit and stable within a revision.

**Files to create or modify.**

- phalcom-semantic/src/identity.rs
- phalcom-semantic/src/index.rs
- phalcom-semantic/src/lib.rs
- phalcom-lsp/src/semantic/query.rs
- phalcom-lsp/src/semantic/snapshot.rs
- phalcom-modules/src/id.rs or the current ModuleId owner

**Data structures and APIs.**

- FileId, ModuleId, BindingId, ScopeId, CallableId, SelectorId, QueryId;
- IdentityStore;
- RevisionStamp and SnapshotStamp;
- stable key constructors;
- indexed lookup and reverse lookup.

**Invariants.**

1. canonical source paths map to one FileId per project/source generation;
2. logical modules are not identified by path strings alone;
3. selector identity is independent of receiver type;
4. declaration and use identities are distinct;
5. IDs used in a snapshot cannot resolve to a different semantic entity inside that snapshot;
6. identity allocation order does not change semantic equality.

**Ownership.** The project/analysis owner creates identities. Request threads read indexes through immutable snapshots.

**Failure behavior.** Invalid paths produce structured resolution errors. An unavailable identity is explicit Unknown/Unresolved, never a fabricated valid ID.

**Tests.**

- same source path under different projects;
- aliases and re-exports preserve target identity;
- duplicate declarations get distinct declaration identities and a diagnostic;
- selectors resolve without receiver closure;
- snapshot IDs remain stable across read requests;
- clean and incremental builds produce equivalent canonical identities.

**Benchmarks.**

- identity lookup;
- index build;
- reverse lookup;
- memory per identity and map entry;
- snapshot clone/share ratio.

**Migration risks.** Existing ad hoc keys may be embedded in semantic maps, LSP fixtures, or module graph serialization. Compatibility conversions may hide duplicate identities.

**Completion gates.**

- all new semantic products use explicit IDs;
- no path string is used as a semantic identity without a documented adapter;
- identity laws pass under repeated and reordered insertion;
- snapshot queries return stamped identities;
- baseline behavior remains unchanged.

**Explicit non-goals.**

- no type interning;
- no fixed-point solver;
- no parallel identity allocation;
- no language semantic changes.

### Phase 2 — Indexed bindings and flow versions

**Objective.** Replace broad name/body scans with binding-keyed scope and flow products while preserving Phalcom control-flow semantics.

**Files to create or modify.**

- phalcom-semantic/src/bindings.rs
- phalcom-semantic/src/scopes.rs
- phalcom-semantic/src/flow.rs
- phalcom-semantic/src/checker/context.rs
- phalcom-semantic/src/checker/typed_expr.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/invalidation.rs

**Data structures and APIs.**

- BindingIndex;
- BindingKind and binding provenance;
- ScopeId parent/index tables;
- FlowVersion and FlowFact;
- value_at(binding, program_point);
- resolve(scope, name, point);
- body-only callable frontier.

**Invariants.**

1. define, use, anonymous, import, and export bindings remain distinguishable;
2. branch facts are keyed by binding and program point;
3. joins create explicit phi facts;
4. a body edit does not change interface identity unless its declarations/exports change;
5. unresolved names retain source range and scope provenance;
6. flow facts never masquerade as global declaration types.

**Ownership.** The semantic worker builds candidate binding and flow tables. Solver contexts may read them and create local facts; request threads read published snapshots.

**Failure behavior.** Missing binding returns unresolved knowledge and a diagnostic fact where policy requires. Unsupported control flow falls back to bounded unknown, not a guessed exact type.

**Tests.**

- shadowing and nested scopes;
- define/use ordering;
- branch narrowing and join;
- loops and repeated fixed points;
- imports, aliases, exports, and re-exports;
- callable body-only edit;
- declaration edit invalidating dependents;
- unknown receiver and dynamic selector;
- clean versus incremental flow facts.

**Benchmarks.**

- binding lookup depth;
- flow joins by changed binding count;
- body-only edit frontier size;
- source files with many scopes;
- repeated LSP position queries.

**Migration risks.** Existing semantic engine maps may have implicit binding keys or store facts at declaration level. Moving them can change duplicate or unknown behavior.

**Completion gates.**

- all checker results identify their binding/flow owner;
- body-only invalidation is narrower than declaration invalidation on fixtures;
- branch joins are deterministic;
- LSP queries reject mismatched document revision;
- semantic consistency tests pass.

**Explicit non-goals.**

- no full type inference;
- no cross-module demand solver;
- no dispatch-family redesign;
- no parallel flow analysis.

### Phase 3 — Canonical TypeStore and semantic equality

**Objective.** Establish canonical type construction, semantic equality, simplification, and representation boundaries without claiming that every value is arena-interned.

**Files to create or modify.**

- phalcom-semantic/src/types/store.rs
- phalcom-semantic/src/types/equality.rs
- phalcom-semantic/src/types/canonicalize.rs
- phalcom-semantic/src/types/mod.rs
- existing type representation and TypeKnowledge modules
- phalcom-semantic/src/checker/typed_expr.rs

**Data structures and APIs.**

- TypeStore;
- TypeId/TypeRef;
- semantic_eq(left, right, context);
- canonical_union(items);
- canonical_intersection(items);
- normalize(type);
- TypeEqContext with memoization and limits;
- representation/provenance wrapper distinct from semantic type.

**Invariants.**

1. ordinary Rust equality, representation equality, and semantic type equality are distinct;
2. union/intersection construction is deterministic;
3. recursive equality terminates under explicit depth/gas limits;
4. Unknown, Dynamic, Error, and Placeholder remain distinguishable;
5. type-store identity is not used as proof of source or query freshness;
6. evidence/provenance is not silently discarded by canonicalization.

**Ownership.** TypeStore mutation occurs through the semantic owner or an isolated solver store. Published type products are immutable.

**Failure behavior.** Equality limits return a documented conservative result or unknown relation. Malformed type structure becomes Error/Unknown according to type policy; it does not panic in editor paths.

**Tests.**

- equal unions under permutation;
- duplicate and nested union simplification;
- recursive nominal/structural equality;
- callable variance and parameter identity;
- dynamic/open-world types;
- error and unknown non-equivalence;
- canonicalization preserves evidence needed by diagnostics;
- TypeStore reset and generation behavior;
- property laws for equality symmetry and reflexivity where valid.

**Benchmarks.**

- TypeStore hit/miss;
- equality depth and memo hit rate;
- union normalization size;
- allocations per canonical type;
- recursive equality fallback frequency.

**Migration risks.** Existing TypeHeap or type wrappers may be partially canonicalized. Conflating TypeId with semantic equality can create false cache hits.

**Completion gates.**

- equality laws and recursion limits pass;
- all solver comparison sites use semantic equality;
- TypeStore memory and hit metrics exist;
- no claim of completed arena interning remains unsupported;
- current checker diagnostics remain equivalent on baseline fixtures.

**Explicit non-goals.**

- no complete relation solver;
- no new language type rules without specification authority;
- no raw-pointer type publication;
- no global type cache across incompatible project revisions.

### Phase 4 — Constraints and bounded subset relations

**Objective.** Add a relation interface and bounded constraint worklist that can support callable, flow, dispatch, and module facts without owning name lookup.

**Files to create or modify.**

- phalcom-semantic/src/relations.rs
- phalcom-semantic/src/constraints.rs
- phalcom-semantic/src/solver.rs
- phalcom-semantic/src/checker/context.rs
- type relation tests under phalcom-semantic/tests

**Data structures and APIs.**

- RelationKind;
- RelationKey;
- Constraint;
- ConstraintOrigin;
- SolverState;
- BoundSet;
- RelationCache;
- solve_constraints(input, budget);
- relate(left, relation, right, context).

**Invariants.**

1. relation solving does not perform module or name lookup;
2. every constraint retains origin and dependency identity;
3. bounds are monotonic within a solve attempt;
4. speculative branches use snapshots/rollback or isolated state;
5. worklist and recursion have explicit gas/iteration budgets;
6. Dynamic/open-world facts do not become exact closed-world proof;
7. diagnostics are collected separately from relation truth.

**Ownership.** A solver invocation owns mutable variables and worklists. Shared TypeStore reads are immutable; writes use the owner or isolated transaction.

**Failure behavior.** Budget exhaustion returns bounded Unknown/Incomplete with metrics and an optional diagnostic. Contradiction returns a structured relation failure. Cancellation discards uncommitted bounds.

**Tests.**

- equality versus subset;
- callable parameter/return variance;
- union/intersection bounds;
- constraint propagation and rollback;
- budget exhaustion;
- cancellation;
- repeated solve convergence;
- relation cache invalidation;
- native/dynamic fallback;
- source-origin diagnostic mapping.

**Benchmarks.**

- constraints processed;
- solver rounds;
- relation cache hits/misses;
- rollback count and bytes;
- maximum worklist size;
- budget fallback rate.

**Migration risks.** Existing checker logic may encode relation policy in expression traversal. A new solver can produce different unknown/error boundaries.

**Completion gates.**

- relation API is used by at least one real checker path;
- bounded solver never loops on generated cycle cases;
- clean and incremental outputs agree;
- relation cache keys include source/dependency stamp;
- all budget fallbacks are observable.

**Explicit non-goals.**

- no full language-wide type inference;
- no cross-module query publication;
- no unsafe concurrency;
- no replacement of runtime dispatch.

### Phase 5 — Demand queries, SCC answers, and publication

**Objective.** Build demand-driven query identities, recursive placeholders, SCC fixed points, answer tables, and safe batch publication.

**Files to create or modify.**

- phalcom-semantic/src/query.rs
- phalcom-semantic/src/query_cell.rs
- phalcom-semantic/src/scc.rs
- phalcom-semantic/src/publication.rs
- phalcom-semantic/src/solver.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/snapshot.rs

**Data structures and APIs.**

- QueryKey;
- QueryCell with safe mutex/condvar first;
- QueryStatus;
- SccWork and SccNodeState;
- Placeholder;
- AnswerTable;
- PublicationBatch;
- compute_query(key, context);
- reserve_scc(scc);
- commit_scc(batch);
- rollback_scc(batch).

**Invariants.**

1. same-thread recursion is detected without waiting on itself;
2. SCC answers are private until convergence and batch commit;
3. placeholders are not final user types;
4. diagnostics and traces commit with final answers;
5. first-writer or canonical answer policy is deterministic;
6. stale input stamps reject publication;
7. cancellation rolls back reservations and does not leave pending state;
8. query cache validity includes dependency fingerprint.

**Ownership.** Query cells may coordinate immutable result publication. Solver variables and SCC state remain invocation-owned. Only the publication owner mutates shared answer tables.

**Failure behavior.** Non-convergent SCCs return bounded fallback and metrics. Internal failure rolls back the batch and preserves the prior generation. Cancellation and stale input are discarded without semantic side effects.

**Tests.**

- acyclic query chain;
- self-recursive query;
- mutual recursion;
- SCC membership expansion;
- placeholder finalization;
- iteration/demotion limits;
- first-answer determinism;
- side-effect commit and rollback;
- concurrent readers of old/new generation;
- stale publication rejection;
- cancellation during SCC computation.

**Benchmarks.**

- query hit/miss;
- SCC sizes and iterations;
- placeholder count;
- duplicate work;
- answer bytes;
- commit/rollback time;
- query latency under rapid edits.

**Migration risks.** Existing engine computation may publish partial maps or use local inferred values not represented by QueryKey. Introducing answer tables can change when diagnostics appear.

**Completion gates.**

- recursive fixtures terminate;
- no partial SCC answer is observable;
- clean and incremental query outputs agree;
- stale/cancelled batches are counted and discarded;
- solver and publication tests pass under forced interleavings.

**Explicit non-goals.**

- no cross-thread duplicate calculation using mutable solver state;
- no raw AnswerSlot pointer tagging;
- no module-wide invalidation redesign;
- no diagnostic protocol changes beyond product plumbing.

### Phase 6 — Module products, environments, and invalidation

**Objective.** Connect canonical module interfaces, import environments, demand-specific dependencies, reverse invalidation, and semantic/runtime graph policies.

**Files to create or modify.**

- phalcom-modules/src/interface.rs
- phalcom-modules/src/resolver.rs
- phalcom-modules/src/graph.rs
- phalcom-lsp/src/semantic/module_graph.rs
- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-semantic/src/module.rs or an equivalent semantic adapter

**Data structures and APIs.**

- ModuleAvailability;
- InterfaceDependency;
- ModuleEnvironment;
- ImportBindingState;
- InterfaceFingerprint;
- reverse_dependents(module, demand);
- update_interface(module, source_revision);
- compute_affected(changes);
- semantic_sccs(affected);
- runtime_initialization_order().

**Invariants.**

1. ModuleId is canonical and project-aware;
2. source, interface, semantic, and runtime revisions are distinct;
3. unresolved import edges remain recorded;
4. aliases, selective imports, wildcard, re-exports, and special exports retain provenance;
5. missing, parse-error, interface-error, partial, and stale environments differ;
6. semantic SCCs do not bypass runtime cycle validation;
7. negative cache entries are generation-scoped;
8. invalidation follows demanded facts, not only module existence.

**Ownership.** Resolver/project model owns source and interface products. Semantic worker owns candidate environment and graph updates. Publication commits graph/index changes together with semantic products.

**Failure behavior.** Missing dependencies produce unresolved facts and structured diagnostics. Interface failure may retain a previous interface only under explicit stale policy. Runtime cycle remains an error where current graph contract requires it.

**Tests.**

- all import forms and aliases;
- selective/wildcard/re-export behavior;
- exposure and project roots;
- builtin/std/native module surfaces;
- missing and partial dependencies;
- provider generation negative cache;
- interface and runtime cycles;
- clean versus incremental affected closure;
- one interface edit with many consumers;
- stale interface rejection.

**Benchmarks.**

- resolver cache hit/miss;
- interface extraction;
- graph update;
- reverse invalidation breadth;
- SCC and runtime-order computation;
- unresolved candidate memory.

**Migration risks.** Current module graph and modules crate may expose overlapping representations. Wiring semantic invalidation too early can broaden changes unexpectedly.

**Completion gates.**

- import and graph tests remain green;
- interface change invalidates only recorded dependents for supported demands;
- missing dependency never panics analysis;
- runtime cycle policy remains intact;
- LSP snapshots carry interface/project revision.

**Explicit non-goals.**

- no Python import fallback;
- no new reflection semantics;
- no runtime execution during static interface resolution;
- no parallel module extraction yet.

### Phase 7 — Worker, snapshots, diagnostics, traces, and LSP

**Objective.** Make the worker-to-generation-to-LSP path truthful, stamped, cancellable, and observable.

**Files to create or modify.**

- phalcom-lsp/src/analysis_service.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/snapshot.rs
- phalcom-lsp/src/semantic/query.rs
- phalcom-lsp/src/diagnostics.rs
- phalcom-lsp/src/backend.rs
- phalcom-lsp/src/perf.rs
- phalcom-semantic/src/diagnostic.rs
- phalcom-semantic/src/checker/result.rs
- new semantic diagnostic/trace product modules as needed

**Data structures and APIs.**

- SemanticGeneration;
- SnapshotStamp;
- DiagnosticId and DiagnosticFileProduct;
- TraceKey and TraceFileProduct;
- PublicationEffects;
- cancellation token tied to WorkerShared epoch;
- publish(candidate) with stamp validation;
- per-file diagnostic replacement;
- status/stale/error event envelopes.

**Invariants.**

1. request threads read immutable snapshots only;
2. one worker owns mutable SemanticEngine state;
3. publication is all-or-nothing for a candidate generation;
4. diagnostics and traces carry source and generation stamps;
5. semantic diagnostics are separate from syntax diagnostics;
6. stale batches cannot publish facts or diagnostics;
7. a valid empty diagnostic product clears old diagnostics;
8. LSP request document revision must match the semantic file product;
9. status events may stream, semantic replacements cannot be mistaken for partial appends.

**Ownership.** WorkerShared owns pending work, epoch, cancellation, and event sequencing. SemanticEngine owns candidate computation. Backend owns protocol delivery but not semantic mutation. Diagnostic and trace stores publish immutable products.

**Failure behavior.** Shutdown and stale epoch discard work. Internal failure preserves the previous generation and emits an error event. Missing source maps produce protocol-safe diagnostics or unavailable results. A backend publication failure does not mutate semantic state.

**Tests.**

- worker shutdown and restart;
- interactive work priority over scan;
- source coalescing;
- cancellation during expensive analysis;
- stale batch discard;
- old/new snapshot atomicity;
- semantic diagnostic publication and clear;
- source-aware related locations;
- trace demand on/off;
- LSP document revision mismatch;
- request after server rebuild and configured server path;
- manual output/status panel check.

**Benchmarks.**

- queue wait;
- debounce and coalescing;
- analysis and publication latency;
- stale/cancelled work;
- snapshot sharing;
- diagnostic rendering;
- LSP p95 request latency;
- retained generation count.

**Migration risks.** Current backend syntax publication and semantic diagnostic conversion are not the same path. Integrating them may change editor output and fixture expectations.

**Completion gates.**

- semantic facts, diagnostics, traces, and status carry consistent stamps;
- no stale diagnostic publication observed;
- all LSP semantic requests pin one snapshot;
- rebuild/server-path/restart manual flow documented and verified;
- integration target and focused tests pass with baseline scope recorded.

**Explicit non-goals.**

- no parallel semantic mutation;
- no automatic diagnostic suppression policy;
- no full explanation UI;
- no unsafe publication optimization.

### Phase 8 — Retention, memory optimization, and bounded parallelism

**Objective.** Reduce measured cost and add isolated parallel work without weakening semantic, publication, or language boundaries.

**Files to create or modify.**

- phalcom-semantic/src/retention.rs
- phalcom-semantic/src/publication.rs
- phalcom-lsp/src/analysis_service.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/perf.rs
- benchmark fixtures and optional dedicated benchmark crate
- unsafe modules only if a separately reviewed optimization is accepted

**Data structures and APIs.**

- RetentionClass;
- ProductLease or generation reader accounting;
- AnalysisJob;
- JobResult with input stamp;
- bounded worker pool or scoped job executor;
- deterministic join and publication validator;
- allocation/retention metrics.

**Invariants.**

1. parallel jobs consume immutable inputs;
2. each job has one owner and isolated mutable output;
3. every result carries input generation and dependency fingerprint;
4. job completion order cannot change semantic output;
5. stale results are discarded before publication;
6. eviction cannot remove a retained dependency;
7. cancellation joins or safely abandons every job;
8. no unsafe code is added without a memory-ordering proof and stress tests.

**Ownership.** The worker schedules and joins jobs. Jobs own isolated candidate products. Only the worker/publisher mutates the published database. Request threads remain read-only.

**Failure behavior.** A failed job leaves its product absent and preserves the previous generation unless policy defines a structured partial result. A cancelled job releases local state. A pool shutdown joins workers before service teardown.

**Tests.**

- serial versus bounded parallel semantic equivalence;
- forced out-of-order completion;
- job cancellation and shutdown;
- one module failure among independent jobs;
- retention and recomputation;
- old snapshot readers during publication;
- memory stress over repeated generations;
- lock/condvar and query-cycle stress;
- sanitizer or Miri review if unsafe code exists.

**Benchmarks.**

- serial/parallel speedup;
- queue and lock contention;
- duplicate work;
- bytes allocated/retained/evicted;
- maximum live generations;
- module throughput;
- p95/p99 LSP latency under scan;
- stale and cancellation latency.

**Migration risks.** Parallelism may amplify nondeterministic map iteration, memory pressure, cross-module invalidation, and native/reflection races. Retention may trade memory for recomputation and worsen editor latency.

**Completion gates.**

- parallel output equals serial output on deterministic fixtures;
- no stale or partial publication;
- memory high-water mark and recomputation cost are measured;
- speedup is positive on a representative workload;
- cancellation and shutdown stress pass;
- unsafe review is complete or no unsafe optimization is retained.

**Explicit non-goals.**

- no unbounded thread creation;
- no shared mutable solver variables;
- no replacement of worker ownership with request-thread analysis;
- no Pyrefly threshold or retention policy copied without measurement.

## 20. Final transfer checklist

- [ ] Phase 0 baseline and evidence manifest complete.
- [ ] Phase 1 identities and indexes adopted by semantic products.
- [ ] Phase 2 bindings and flow versions drive body invalidation.
- [ ] Phase 3 TypeStore and semantic equality govern comparisons.
- [ ] Phase 4 bounded relation engine is observable and terminating.
- [ ] Phase 5 query/SCC answers publish atomically.
- [ ] Phase 6 module environments and demand invalidation are connected.
- [ ] Phase 7 worker/LSP diagnostics and traces are generation-safe.
- [ ] Phase 8 memory and parallelism are benchmark-justified.
- [ ] Current, partial, deferred, and unverified scope is reported separately.
