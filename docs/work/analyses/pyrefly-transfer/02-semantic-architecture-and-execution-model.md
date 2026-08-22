# Pyrefly semantic architecture and execution model

## Purpose

This document reconstructs how Pyrefly turns source files into reusable semantic products. The efficiency is implemented through staged state, ownership, atomic publication, demand scheduling, invalidation, retention, and measurements.

Every expensive product has:

- an owner;
- a stage;
- a publication slot;
- an invalidation condition;
- a retention policy;
- a reader protocol;
- a measurement point.

## Evidence boundary

Pinned Pyrefly revision: 43467e64e36550f232a18e89f24fda79b1020b6b, inspected 2026-08-22.

Primary files:

- [ARCHITECTURE.md](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/ARCHITECTURE.md) — module-centric design and three semantic stages.
- [steps.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/steps.rs) — Load, Ast, Exports, Answers, Solutions state and computation.
- [module.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/module.rs) — frozen/mutable module state and atomic ordering.
- [state.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/state/state.rs) — dependency graph, epochs, dirty propagation, and recheck loop.
- [calculation.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_graph/src/calculation.rs) — cached calculation cell.

Phalcom mapping points:

- phalcom-semantic/src/snapshot.rs
- phalcom-lsp/src/semantic/engine.rs
- phalcom-lsp/src/semantic/invalidation.rs
- phalcom-lsp/src/semantic/infer.rs
- phalcom-semantic/src/surface.rs
- phalcom-semantic/src/identity.rs

## Architecture choice: module unit plus demand-driven internals

Pyrefly schedules a module and requested stage, but calculates individual binding answers on demand inside that module.

~~~text
scheduler granularity:
    module -> requested stage

semantic calculation granularity:
    binding -> answer query

recursive convergence granularity:
    SCC of binding calculations

publication granularity:
    immutable module step / final answer batch
~~~

This avoids both a global fine-grained query graph and a fully eager module pass. The module is large enough to amortize scheduler overhead; binding queries avoid irrelevant work; SCCs terminate recursion.

## Module stage machine

~~~text
Load
  -> Ast
  -> Exports
  -> Answers
  -> Solutions
~~~

Products:

~~~text
Load:
    file contents, module path, source metadata, load errors

Ast:
    parsed module and optional lexer tokens

Exports:
    exported names, wildcard/re-export behavior, metadata

Answers:
    bindings plus answer solver and per-binding facts

Solutions:
    finalized diagnostics, reports, and consumer products
~~~

A stage is both a product and a reuse boundary:

- missing Solutions can reuse Answers;
- body edits can preserve exports;
- export edits can invalidate dependent modules;
- open-file token consumers can require AST retention;
- completed closed-file solutions can evict AST and answers.

## Steps and StepsMut

The frozen module state contains plain optional Arcs:

~~~rust
struct Steps {
    last_step: Option<Step>,
    load: Option<Arc<Load>>,
    ast: Option<Arc<ParsedModule>>,
    exports: Option<Arc<Exports>>,
    answers: Option<Arc<(Bindings, Arc<Answers>)>>,
    solutions: Option<Arc<Solutions>>,
}
~~~

Mutable construction state uses ArcSwapOption slots and an AtomicStep marker:

~~~rust
struct StepsMut {
    current_step: AtomicStep,
    load: ArcSwapOption<Load>,
    ast: ArcSwapOption<ParsedModule>,
    exports: ArcSwapOption<Exports>,
    answers: ArcSwapOption<(Bindings, Arc<Answers>)>,
    solutions: ArcSwapOption<Solutions>,
    old_exports: ArcSwapOption<Exports>,
    old_answers: ArcSwapOption<(Bindings, Arc<Answers>)>,
    old_solutions: ArcSwapOption<Solutions>,
}
~~~

Old products are temporary diff inputs, not a general history. They are retained across rebuild until the changed-export comparison runs.

## Publication ordering

Writer:

~~~text
1. store stage product in ArcSwap
2. release-store current_step
~~~

Cleaner:

~~~text
1. reset products and current_step with relaxed stores
2. release-store checked epoch
~~~

Reader:

~~~text
1. acquire-load checked epoch
2. acquire-load current_step
3. load the product ArcSwap slot
~~~

The current-step marker is the synchronization point. A reader that sees stage N must see the product written before the release-store. Arc references keep the data alive while the reader uses it.

## Module ownership

ModuleState is the frozen committed form. ModuleStateMut is the mutable transaction form. Mutable state includes:

~~~text
StepsMut
checked epoch
computed/dirty epoch
require level
computing flag
condition variable
waiter count
~~~

One module has one active writer for stage computation. Readers do not wait merely to read completed products. Independent modules can compute in parallel.

The computing flag protects mutation of one module. It is not a global lock around all semantic reads.

## Compute path

~~~text
demand(module, requested_step)
  -> clean module for current epoch
  -> check current_step
  -> try exclusive compute access
  -> compute the next missing step
  -> read prior products from ArcSwap
  -> run step-specific function
  -> store output in ArcSwap
  -> release-store current_step
  -> release compute flag
  -> compare old/new products if required
  -> dirty matching reverse dependents
~~~

Step functions are separate and intentionally easy to profile. Pyrefly computes Load, Ast, Exports, Answers, and Solutions with distinct functions rather than one opaque pass.

The Answers step constructs Solver, Bindings, and Answers. The Solutions step calls Answers.solve and turns answers into final products.

## Retention and eviction

Require levels decide which products must remain resident:

~~~text
open file + semantic tokens:
    retain AST and tokens

closed file + completed solutions:
    retain compact solutions and needed exports
    evict AST and answers when legal

debug/report mode:
    retain traces, indexes, and answers
~~~

This is a core memory optimization. The system does not retain every intermediate representation for every module forever.

Phalcom should add retention policy to semantic products. LSP features must not accidentally pin the full AST, binding table, and solver for the entire workspace.

## Demand fast path

Demand checks current_step and checked epoch optimistically before taking the computing mutex. If another thread finishes the requested stage while this thread waits, the waiter observes the new marker and stops instead of recomputing.

Timing data records clean time, compute time, wait time, and wait counts. This allows performance work to distinguish semantic cost from contention.

## Recheck epochs

A recheck run repeatedly computes changed modules and propagates changed exports:

~~~text
compute new/changed modules
collect changed export keys
for each direct reverse dependency:
    compare its ModuleDeps with changed ModuleChanges
    dirty only when invalidated
repeat while export changes propagate
stop when no changes remain
~~~

Pyrefly tracks name existence, type, metadata, wildcard, class, type-alias, and specialized metadata dependencies. A type-only change does not invalidate a consumer that depends only on documentation metadata. An existence change invalidates any dependency on that name.

If the same export changes again through a dependency cycle, Pyrefly falls back to coarse transitive invalidation for that cycle. A module appearing twice for independent exports is not automatically treated as a cycle.

## Why module-level incrementality

The module choice reduces:

- persistent query-object count;
- cross-thread dependency edges;
- invalidation bookkeeping;
- synchronization complexity;
- memory retained by fine-grained graph nodes;
- profiling noise.

It also creates natural parallel work units. The trade-off is that an unchanged identifier inside a rebuilt module may be recalculated. Pyrefly accepts that cost because Rust execution and compact representation make full module solving cheap enough.

Phalcom target:

~~~text
project scheduling:
    module and dependency component

module products:
    exports, surfaces, bindings, callable summaries

query-local:
    expression facts, constraints, relation cache

published:
    immutable semantic generation
~~~

## Phalcom target structures

Introduce an explicit stage model:

~~~rust
enum SemanticStage {
    Source,
    Parsed,
    Exports,
    Surfaces,
    Bindings,
    Facts,
    Solutions,
}

struct ModuleSemanticState {
    current: AtomicStage,
    source: ArcSwapOption<SourceProduct>,
    parsed: ArcSwapOption<ParsedProduct>,
    exports: ArcSwapOption<ExportProduct>,
    surfaces: ArcSwapOption<SurfaceProduct>,
    bindings: ArcSwapOption<BindingProduct>,
    facts: ArcSwapOption<FactProduct>,
    solutions: ArcSwapOption<SolutionProduct>,
}
~~~

Do not implement every stage immediately. Start with Source, Parsed, Surfaces, and Facts. Add Bindings and Solutions as the checker moves from local inference to shared products.

SemanticSnapshot remains the public reader product:

~~~rust
struct SemanticSnapshot {
    generation: SemanticGeneration,
    source_revision: SourceRevision,
    type_store: Arc<TypeStore>,
    modules: Arc<ModuleProducts>,
    surfaces: Arc<SurfaceTable>,
    facts: Arc<FactTable>,
    diagnostics: Arc<DiagnosticTable>,
}
~~~

The worker builds candidates privately, checks cancellation, freezes Arcs, then publishes a complete generation.

## Current Phalcom bridge

CURRENT or EXPERIMENTAL infrastructure already includes:

- worker-owned SemanticEngine;
- immutable SemanticSnapshot generations;
- SourceChangeKind classification;
- affected callable/module frontiers;
- ModuleGraph and callable dependency products;
- Arc pointer-reuse metrics;
- a shared semantic checker;
- a TypeStore and experimental constraint solver.

The missing architectural step is to make semantic products and their stage/revision contracts shared between CLI, checker, and LSP. LSP must not remain the only owner of formal semantic facts.

Recommended migration:

1. Move neutral source/module products from LSP into phalcom-semantic.
2. Keep LSP request scheduling and worker ownership in phalcom-lsp.
3. Make the checker read immutable products.
4. Add query-local mutable solver state over a snapshot.
5. Publish complete facts into a new generation.
6. Let CLI and LSP consume the same generation.

## Failure states

The stage engine must distinguish:

- missing source;
- parse recovery;
- unresolved module;
- stale dependency;
- cancelled candidate;
- recursive calculation;
- solver budget exhaustion;
- semantic contradiction;
- publication race;
- internal invariant violation.

Do not return one empty product for all failures. Readers need to know whether a product is absent, incomplete, stale, valid-but-unknown, or valid.

## Required verification

- stage marker never appears before its product;
- cancelled candidate never publishes;
- repeated stage request does not recompute;
- body-only edit preserves unchanged surface products;
- surface edit reaches every dependent consumer;
- independent modules compute in parallel with deterministic output;
- module cycle reaches stable output or explicit fallback;
- clean full rebuild equals incremental recheck;
- legal AST/answer eviction does not break requested consumers;
- original URI remains available for diagnostics while canonical keys drive caches.

## Performance counters

Record per module and per generation:

~~~text
stage time
product reuse
bytes allocated
bytes retained
dependent modules dirtied
queries demanded
wait time
cancellation count
type-store nodes
constraints and SCC work
diagnostics produced
snapshot size
source-to-publication latency
~~~

## Conclusion

Pyrefly's semantic architecture is fast because it is executable state management: staged products, atomic publication, one writer per module, lock-free reads, demand scheduling, revision-aware invalidation, memory eviction, and epoch stabilization. Phalcom needs those concrete rules, not only the phrase “incremental semantic analysis.”
