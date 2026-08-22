# Pyrefly transfer: executive report for Phalcom

## Executive decision

Phalcom should adopt Pyrefly's semantic-engine shape as the performance foundation for formal typing:

1. Stable semantic IDs and dense indexed tables.
2. A staged module pipeline: load, parse, export/surface, bind/flow, solve.
3. Immutable published products with worker-owned mutable construction.
4. Demand-driven query cells with explicit cycle and cancellation states.
5. Fine-grained dependency keys inside module-oriented invalidation.
6. A canonical type store with semantic equality, normalization, and complexity caps.
7. A bounded constraint/worklist solver with explicit SCC handling and a narrow semantic-order boundary.
8. LSP and CLI as clients of one semantic snapshot.

Do not copy Pyrefly's Python type semantics wholesale. Do not put type metadata into Phalcom selector identity, method lookup, inline-cache identity, or runtime dispatch. Do not treat every unresolved fact as `Dynamic`, or every solver limit as a valid type result. Transfer the control flow, data ownership, caching, and proof-state distinctions.

## What makes Pyrefly fast

Pyrefly is fast because it avoids repeatedly paying for expensive semantic work. Its key optimization is architectural reuse across layers:

```text
stable IDs
  -> O(1) indexed lookup and compact dependency keys
  -> staged module products
  -> precise invalidation
  -> parallel module work
  -> bounded type/constraint solving
  -> immutable answer publication
  -> cheap repeat queries
```

No single item explains performance. Rust removes interpreter overhead and makes compact ownership possible, but the larger gain comes from deciding what must be recomputed, what can be shared, and when a result is safe to publish.

### Efficiency decomposition

| Layer | Pyrefly mechanism | Performance effect | Phalcom transfer |
|---|---|---|---|
| Identity | `Idx<K>`, `NonZeroU32`, typed indexes | Small keys, dense lookup, fewer string/hash operations | Add typed IDs for modules, declarations, bindings, queries, constraints, and expressions. |
| Storage | `IndexMap<K, V>` over vectors, `Arc`, `ArcSwap` | O(1) indexed access and cheap immutable sharing | Extend current `TypeStore`, `SemanticSnapshot`, and worker candidate model rather than adding global locks. |
| Module work | Load → AST → exports → answers → solutions | Reuses completed stages and parallelizes independent work | Make module products explicit in `phalcom-semantic`; keep VM execution out of checking. |
| Dependency tracking | Export/name/type/metadata/class/type-alias keys | Avoids rebuilding unaffected reverse dependencies | Replace broad file invalidation with module-plus-declaration/callable frontiers. |
| Query calculation | `NotCalculated` / `Calculating` / `Calculated`, cycle-aware cells | Avoids duplicate publication and deadlock; supports recursion | Add safe answer cells with generation and cancellation state. |
| Recursive solving | `Type::Var` placeholders, SCC batches, previous answers | Terminates mutually recursive analysis and warm-starts rechecks | Add explicit recursive variables and fixed-point statuses; never silently widen recursion to `Dynamic`. |
| Type operations | Semantic equality context, pair memoization, union simplification | Avoids repeated deep comparisons and type-tree blowup | Separate structural equality, semantic equivalence, subtype, assignability, consistency, and conformance. |
| Complexity control | Union/literal/enum caps, recursive type-argument truncation, solver budgets | Prevents pathological programs from consuming unbounded time/memory | Add observable widening or budget outcomes with provenance. |
| UX scheduling | Streaming diagnostics, open-file priority, separate recheck thread | First useful result arrives before full propagation | Keep LSP worker ownership; publish coherent snapshots and prioritize open files. |

## Source-grounded observations

### 1. Pyrefly is module-oriented, not a fully fine-grained Salsa graph

**OBSERVED / PYREFLY.** The official architecture describes three major semantic stages: exports, bindings/scope/flow, and solving bindings with cross-module dependencies. It explicitly chooses module-level incrementality and parallelism because large strongly connected components are normal in Python projects. Pyrefly's architecture document says it does not model every identifier as an independently recomputed Salsa/Rust Analyzer-style query.

This is important for Phalcom. Maximum granularity is not maximum speed. A useful split is:

- module-level products for parsing, exports, declarations, and dependency scheduling;
- declaration/callable-level products for body facts and summaries;
- expression-level calculations only while a query is active, unless measurements prove persistence worthwhile.

That hybrid matches Phalcom's current `SemanticEngine`, which already builds candidate state, computes affected closures, and publishes immutable maps after cancellation checks.

### 2. Staged products are explicit cache boundaries

**OBSERVED / PYREFLY.** `Steps` stores load, AST, exports, answers, and solutions independently. Each stage is published through immutable shared storage. The reader protocol uses acquire/release ordering around the current step and checked epoch. This is more than a cache: it defines which intermediate semantic products are valid and which later products depend on them.

**TRANSFER / PHALCOM.** Make the same boundaries visible in Phalcom. A source edit should be classified before analysis:

```text
body-only edit
  -> preserve exports and declaration surface
  -> invalidate callable/body facts and reverse dependents that consume them

signature/import/export edit
  -> invalidate surface, bindings, affected module dependents, and callable summaries

core/native/runtime contract edit
  -> invalidate the relevant project-wide contract frontier
```

Current Phalcom already has `SourceChangeKind`, callable dependency frontiers, deterministic queues, generation snapshots, and product reuse accounting. Formal typing should consume this infrastructure instead of inventing a parallel invalidation system in LSP.

### 3. Calculation cells solve publication and recursion separately from dependency invalidation

**OBSERVED / PYREFLY.** `Calculation<T>` has an atomic status, a write-once result, a condition variable for commit coordination, and thread-local tracking of calculations currently being evaluated. Same-thread recursive entry returns no completed result; mutually recursive calculations can be evaluated by different threads without waiting into a deadlock. `AnswerSlot<T>` uses pending and published pointer states with an acquire fence and first-writer-wins publication.

These are low-level synchronization mechanisms. They do not decide whether a source file is stale. Pyrefly has separate module dependency state for that. Phalcom should retain this separation:

- invalidation decides whether an answer is eligible for reuse;
- a query cell decides whether an eligible answer is available or currently being calculated;
- the solver decides what a recursive or incomplete answer means;
- the publisher decides when a generation becomes visible.

Do not import unsafe once-publication code into Phalcom as a first step. Start with safe `Arc`/lock-free-read or worker-owned cells, measure, then isolate any unsafe optimization behind tested invariants.

### 4. TypeHeap is a seam, not proof that all types are arena interned

**OBSERVED / PYREFLY.** At the pinned commit, `TypeHeap` provides constructors and a heap identity check, but the current implementation still returns boxed type values. The file describes a future arena direction. The type representation is large and solver-oriented; it is not a simple universal `TypeId` arena already completed.

**TRANSFER / PHALCOM.** Phalcom already has a stronger current starting point in `TypeStore`: `TypeId`, hash-consing, a `Vec<TypeData>`, a reverse `HashMap`, and normalized flat unions. The next move is not “replace it with Pyrefly's arena.” The next move is:

- define snapshot/session ownership for interned types;
- keep `TypeId` compact and generation-scoped where necessary;
- make normalization deterministic and bounded;
- add semantic equality for recursive/alpha-equivalent types;
- separate persistent type descriptors from temporary inference variables;
- measure allocation and lookup before selecting arenas, slabs, or unsafe pointers.

### 5. Canonicalization is layered

Pyrefly does not have one universal canonicalization pass. It has several smaller canonical forms:

- stable index identity for bindings, classes, functions, and module keys;
- semantic equality contexts for alpha-equivalence and recursive structures;
- union/intersection flattening, sorting, deduplication, and collapse;
- truncation of recursively nested type arguments;
- dependency keys that distinguish name existence, metadata, type, class, alias, and wildcard facts;
- pointer identity for cheap reuse checks, not semantic equality.

Phalcom should copy this layering. A single “normalize everything” routine would blur semantic boundaries and make dynamic language features harder to represent.

### 6. Constraint solving is bounded semantic work, not a generic hash map substitution

**OBSERVED / PYREFLY.** Pyrefly separates lookup of Python semantic facts from the subset/constraint solver through `TypeOrder`. Its solver stores variables and bounds, supports snapshots and unification, and uses answer lookup rather than embedding module resolution in every type relation. The alternate answer solver detects cycles, records SCC state, uses placeholders, retains previous answers for warm starts, and applies demotion guards.

**CURRENT / PHALCOM.** Phalcom's experimental `LocalConstraintSolver` already has equality, subtype, member constraints, substitutions, and recursive substitution over composites. It is a useful seed, but it is not yet a complete incremental solver: member constraints are delegated, there is no demonstrated occurs check, constraints have limited provenance, and solving is sequential rather than a dependency-aware worklist/SCC process.

The transfer target is a staged solver with explicit outcomes:

```text
Solved
Underconstrained
Ambiguous
Inconsistent
BlockedByDynamicBoundary
RecursiveFixpoint
BudgetExceeded
Cancelled
```

`BudgetExceeded` must not masquerade as `Dynamic`, and `BlockedByDynamicBoundary` must not be reported as a proven type. This preserves Phalcom's existing distinction between `Known`, `Unknown`, and `Dynamic` evidence.

## Performance evidence and limits

The Pyrefly project reports very strong benchmark numbers, including roughly 1.85M lines/second in its README and large wins over other Python checkers in its speed-and-memory comparison. The comparison reports no cache between benchmark runs and cites examples such as pandas and scipy. These are vendor-maintained measurements, not Phalcom acceptance criteria. They depend on checker version, repository shape, Python features, hardware, configuration, and maturity.

The official diagnostic-performance report describes an improvement from approximately 3.6 seconds to under 200ms in an M4 workload after combining finer-grained type dependency tracking with streamed diagnostics and separated recheck/LSP execution. It also reports typical in-editor rechecks under 10ms for common edits. Treat these as evidence that invalidation and scheduling dominate interactive latency, not as universal guarantees.

Pyrefly also documents where it pays cost:

- inference is more expensive than checking explicit annotations;
- wide unions and overload sets create solver and diagnostic work;
- unannotated code forces more inference;
- large recursive graphs require SCC handling and demotion guards;
- framework-heavy or dynamic code reduces useful precision;
- complexity caps trade precision for bounded time and memory;
- parallel duplicate calculations can be acceptable when avoiding cross-thread deadlock.

Phalcom should publish its own baseline before claiming speed improvements. Required measurements are listed in the implementation breakdown.

## Transfer to Phalcom's type philosophy

### Keep semantic layers distinct

Phalcom needs at least these distinct notions:

| Notion | Meaning | Must not be conflated with |
|---|---|---|
| Runtime class identity | What object dispatch and allocation observe | Static annotation |
| Static type descriptor | Contract used by checking and inference | Selector identity |
| Flow shape | Current branch-sensitive knowledge | Declared type |
| Proof/evidence | Why a fact is trusted and where it came from | Mere syntactic appearance |
| Dispatch fact | Result of selector/side/member lookup | Type equality |
| `Dynamic` | Explicit or semantically unavoidable escape boundary | Proven top type |
| `Unknown` | Analysis has insufficient authority or budget | `Dynamic` and `Any` |
| `Never` | Empty/bottom type or unreachable result | Missing analysis |

This agrees with the current typing direction: type metadata must not alter selector encoding, method-table identity, inline-cache identity, or dynamic dispatch. It also gives the solver room to report uncertainty without destroying soundness claims.

### Make relation names precise

The implementation should not use one `is_compatible` predicate for every question. Define and test separate relations:

- structural/semantic equivalence;
- nominal or structural subtyping;
- assignment compatibility;
- consistency with `Dynamic`;
- protocol/conformance satisfaction;
- callable variance compatibility;
- runtime-class compatibility;
- dispatch availability.

Variance must be explicit. Current experimental Phalcom applied types recurse covariantly; that is acceptable as a seed for simple containers but not a final generic model. Callable parameters are contravariant and results covariant; method/field mutability and protocol rules need their own policy.

### Preserve provenance and authority

Every inferred or checked type fact should carry enough context to distinguish:

- declared contract;
- trusted native/runtime contract;
- exact syntax fact;
- proof from flow or constraints;
- advisory LSP shape;
- unresolved, opaque, recursive, or budget-limited state.

Diagnostics can then explain “inferred from branch refinement,” “blocked by dynamic send,” or “widened after solver budget” instead of presenting all failures as nominal type mismatches.

## Recommended target architecture

```text
                    ┌────────────────────────┐
                    │ Source + module loader │
                    └───────────┬────────────┘
                                │
                    ┌───────────▼────────────┐
                    │ Parse / source snapshot│
                    └───────────┬────────────┘
                                │
            ┌───────────────────▼───────────────────┐
            │ Exports + declaration surfaces       │
            │ module keys, class/member contracts  │
            └───────────────┬───────────────────────┘
                            │
            ┌───────────────▼───────────────────────┐
            │ Bindings + scopes + flow facts         │
            │ define/use/anonymous/export identities │
            └───────────────┬───────────────────────┘
                            │
        ┌───────────────────▼────────────────────┐
        │ Query/answer layer                     │
        │ callable summaries, member facts,     │
        │ recursive placeholders, dependencies  │
        └───────────────────┬────────────────────┘
                            │
        ┌───────────────────▼────────────────────┐
        │ Constraint + semantic-order solver    │
        │ subtype, equality, call, member, SCC  │
        └───────────────────┬────────────────────┘
                            │
                 ┌──────────▼──────────┐
                 │ Immutable generation │
                 │ snapshot + evidence │
                 └───────┬──────────────┘
                         │
             ┌───────────▼───────────┐
             │ CLI / LSP / diagnostics│
             └────────────────────────┘
```

The type store and relation engine sit below the query layer as reusable semantic services. The query layer owns dependency recording and cycle state. The snapshot owns publication. The LSP does not reimplement inference.

## Priority order

### Highest return

1. Formalize current Phalcom type identity, evidence, relation names, and selector independence.
2. Make `TypeStore` canonical, deterministic, snapshot-aware, and measurable.
3. Move semantic checking onto staged source/module products rather than VM execution.
4. Add query cells and explicit recursive/budget outcomes.
5. Replace broad invalidation with export/member/contract dependency keys.
6. Upgrade the experimental constraint solver to a provenance-carrying worklist/SCC solver.

### Later, after measurements

- pointer-tagged answer slots;
- unsafe once-publish cells;
- arena or slab allocation for type nodes;
- broad parallel solver execution;
- persistent expression-level caches;
- aggressive union widening;
- cross-process or disk caches.

These can improve throughput, but only after semantic invariants and cache hit rates are visible.

## Main risks

| Risk | Failure mode | Countermeasure |
|---|---|---|
| Python semantic mismatch | Phalcom adopts rules for Python objects, imports, or protocols that do not fit message sends, families, reflection, or open classes | Transfer mechanisms; specify Phalcom relations independently. |
| Selector contamination | Type annotations become part of method identity or dispatch | Keep `Selector`, `CallableId`, method tables, and inline caches type-independent. |
| Unsound widening | Solver limits turn into false certainty | Preserve `Unknown`, `Dynamic`, `BudgetExceeded`, and evidence authority. |
| Cache poisoning | Stale answers survive source/core/native changes | Generation-tag every product and track semantic revision dependencies. |
| Over-fine invalidation | Every local body edit rebuilds project graph | Use module products plus precise declaration/callable frontiers. |
| Over-coarse invalidation | Type change silently leaves stale dependent facts | Track exported name/type/metadata/class/alias dependencies. |
| Recursive nontermination | Cycles keep expanding types or constraints | Use recursive variables, SCC iteration, depth/width budgets, and explicit outcomes. |
| Unsafe optimization too early | Publication race or memory bug | Implement safe reference model and concurrency tests before unsafe fast path. |
| LSP drift | Editor shows advisory shape while CLI reports formal type | One semantic snapshot and one query API. |
| Dirty-work confusion | New design docs imply current Rust implementation is complete | Label current, experimental, proposed, deferred; preserve unrelated changes. |

## Bottom line

Phalcom should become a semantic knowledge engine with a formal type checker, not a slower VM pass with annotations bolted on. Pyrefly's transferable genius is disciplined reuse: represent facts cheaply, compute only demand-relevant work, publish coherent results, retain enough provenance to explain uncertainty, and cap pathological complexity. The implementation breakdown turns that principle into Phalcom-specific phases and acceptance gates.
