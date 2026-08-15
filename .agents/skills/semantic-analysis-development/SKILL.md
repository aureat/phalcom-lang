---
name: semantic-analysis-development
description: >-
  Use when changing Phalcom semantic analysis, name/scope resolution, semantic
  identities, flow/CFG facts, runtime-shape inference, dispatch reasoning,
  interprocedural summaries, effects, incremental invalidation, semantic LSP
  queries, checker/prover integration, or semantic-performance behavior in Rust.
compatibility: Designed for coding agents working on the Phalcom repository (aureat/phalcom-lang).
---

# Semantic Analysis Development for Phalcom

This is the implementation companion to the `phalcom-semantic-model` skill.
Use that skill for Phalcom-wide semantic doctrine and normative language interpretation; use this one for the architecture, algorithms, representations, invariants, and verification of the semantic-analysis engine. The neighboring `type-theory`, `phalcom-typed-language`, `type-checker-development`, `static-prover-development`, `parser-development`, and `lsp-development` skills own their deeper specialist machinery. This skill teaches the bridges to them rather than duplicating each discipline.

The objective is not "make the LSP smarter" in isolation. The objective is:

> Extend one reusable semantic engine so editor tooling, future typing, lints,
> checker/prover work, and eventually compiler optimizations can consume the
> same identities and facts without semantic drift.

## Start here: do not code from memory

Before changing semantic analysis:

1. Read repository instructions (`CLAUDE.md`, `AGENTS.md`).
2. If `graphify-out/graph.json` exists, query graphify for the question and affected symbols.
3. Read the relevant normative spec/ADR/PDR. Proposed typing docs are not current runtime behavior.
4. Inspect `phalcom-lsp/src/semantic/` current implementation.
5. Inspect the corresponding `phalcom-ast` nodes and `phalcom-core` compiler/VM/runtime behavior.
6. Run focused baseline tests before edits.

Typical orientation commands in a local checkout:

```sh
graphify query "how is <semantic concept> represented" --budget 2000
graphify affected "<symbol>"
graphify path "<source symbol>" "<consumer symbol>"

scripts/test.sh lsp
cargo test -p phalcom-lsp
```

Use actual repository commands if they have changed.

## Code standards

### Public semantics must be documented

Phalcom repository conventions require professional Rust documentation. New public/module
items need `//!`/`///` documentation and must survive `cargo doc`/missing-doc gates where
applicable.

### No semantic panics on user source

Malformed/incomplete user source is normal in an editor. Do not `panic!`, `unwrap`, or
`expect` merely because a parsed construct is incomplete or unresolved.

`expect` is appropriate only for internal invariants already established by construction,
and the invariant should be obvious/documented.

### Typed IDs over strings

Prefer:

```rust
BindingId
ScopeId
ModuleId
ClassId
CallableId
FieldId
```

over long-lived maps keyed by unqualified `String`.

Strings remain necessary for source names/selectors/display, but identities should carry
semantic qualification.

### Deterministic facts

Semantic outputs and tests should be deterministic. Prefer stable ordering (`BTreeMap`,
`BTreeSet`, sorted vectors) where externally observed ordering matters. If hash maps are
used on hot paths, normalize/sort at the publication boundary when needed.

### Compact hot-path facts

Avoid cloning AST subtrees, entire module snapshots, long provenance chains, or arbitrary
strings into every fact. Store IDs/ranges/compact enums and resolve through the snapshot.

### Mutation stays in the worker

Preserve the current architecture: mutable analysis state is worker-owned; LSP/other
queries read coherent immutable snapshots.

## Non-negotiable analysis contract

Before treating a fact as shared semantic truth, identify its concrete proposition, abstract domain, information order, join, soundness polarity, invalidation assumptions, and provenance. `Unknown`, unreachable, ambiguous, not-yet-analyzed, dynamic-by-language-choice, inconsistent, and budget-exhausted are different states unless a specific domain proves otherwise.

Read [references/semantic-foundations-and-analysis-contract.md](references/semantic-foundations-and-analysis-contract.md) before introducing a new fact domain or allowing a new correctness consumer to rely on an existing advisory fact. For heap/property refinements, captured mutation, user-code callbacks, reflection, or suspension points, also read [references/heap-alias-mutation-and-refinement.md](references/heap-alias-mutation-and-refinement.md).

## The development loop

Follow this sequence for every semantic feature.

### Step 1 — State the semantic question

Bad:

> Add better inference for fields.

Good:

> At source offset `p`, given receiver fact `R`, determine the joined runtime shape of
> instance field `f` from declaration initializers and reachable writes in the current
> semantic generation, with provenance and class-side separation.

The semantic question determines identity, domain, flow sensitivity and invalidation.

### Step 2 — Classify the required analysis tier

Use the lowest tier that can answer correctly:

```text
syntax
binding/scope
semantic occurrence
surface/declaration
local expression fact
flow-sensitive fact
dispatch
field/parameter fact
callable summary
module/project fixed point
type/checker fact (future)
proof/effect fact (future)
```

Do not trigger interprocedural analysis for a query that only needs lexical bindings.

### Step 3 — Find the existing owner

Before adding a type/map/walker, inspect:

- `ids.rs` for identity;
- `scope.rs` for binding/scope;
- `surface.rs` for declaration surface;
- `occurrence.rs` for source target/reference;
- `facts.rs` for abstract value/evidence domain;
- `analyzer.rs` for expression semantics;
- `dispatch.rs` for lookup;
- `flow.rs` for flow/call/field/block effects;
- `callable.rs` / `infer.rs` for summaries/fixed points;
- `module_graph.rs` / `invalidation.rs` / `engine.rs` for incrementality;
- `snapshot.rs` / `query.rs` for consumer-facing access.

Extend the owner rather than creating a feature-local duplicate.

### Step 4 — Define identity and lifetime

Write down:

- key type;
- whether it survives file reparse;
- whether it is module-qualified;
- whether source range is location only or part of identity;
- whether fact is valid for one semantic generation.

Never cache file-local `BindingId` across reparses.

### Step 5 — Define the abstract domain

Before implementation, specify:

```text
values
unknown/top
bottom/unreachable if needed
precision order
join
widening/cap
fixed-point equality
provenance
confidence/proof strength
```

If you cannot define the join, you do not yet have a flow analysis.

Read [references/abstract-domains-and-dataflow.md](references/abstract-domains-and-dataflow.md).

### Step 6 — Define transfer and control-flow behavior

For every relevant construct answer:

- What input state is read?
- What state/fact is produced?
- Does evaluation terminate this path?
- Are there alternate paths?
- What happens at merge?
- What happens on loop back-edge?
- What does block construction do versus block execution?
- What dynamic/reflection case forces conservatism?

Read [references/control-flow-and-cfg.md](references/control-flow-and-cfg.md).

### Step 7 — Define interprocedural contract if needed

If the fact crosses calls, add/extend a summary rather than recursively embedding callee
analysis.

Specify:

- summary input facts;
- output facts;
- direct dependencies;
- effects;
- recursion seed;
- convergence/widening;
- invalidation edges.

Read [references/interprocedural-analysis.md](references/interprocedural-analysis.md).

### Step 8 — Implement source collection once

If a new syntax form introduces declarations/bindings/occurrences, update the shared
source walkers so every consumer sees it.

Commonly inspect/update:

```text
surface builder
scope builder
occurrence builder
expression analyzer
flow walker
module dependency extractor
```

Do not patch completion/hover first and leave semantic indexes unaware.

### Step 9 — Update invalidation before caching

For any persistent/summary fact answer:

- what source contribution creates it;
- what other facts it reads;
- which reverse dependency edges are needed;
- what happens when a file is removed;
- how batch updates publish coherently.

Read [references/incrementality-and-snapshots.md](references/incrementality-and-snapshots.md).

### Step 10 — Expose a semantic query

Prefer a narrow semantic query returning semantic data, not LSP types.

Good conceptual API:

```rust
fn value_at(&self, module: &ModuleId, offset: usize) -> Option<InferredValue>;
fn visible_bindings_at(&self, module: &ModuleId, offset: usize) -> Vec<BindingInfo>;
fn resolve_member(&self, receiver: &ValueShape, selector: &str) -> DispatchResult;
```

Bad:

```rust
fn completion_items_for_dot(...) -> Vec<lsp_types::CompletionItem>
```

inside the semantic engine.

The consumer converts semantic answers into protocol/UI shapes.

### Step 11 — Add provenance and uncertainty rendering hooks

A new fact should be explainable in tests/debugging even if normal UI does not show all
evidence.

Never represent "analysis did not implement this" as an exact user fact.

### Step 12 — Test semantics before UI

First test the semantic engine/query directly. Then test completion/hover/diagnostics.

A UI fixture passing is not enough: it can hide duplicate inference or accidental rendering
behavior.

### Step 13 — Test incrementality

Update a dependency and verify:

- the intended facts change;
- unrelated modules do not change/rebuild unexpectedly;
- removed declarations disappear;
- re-addition recovers;
- generation/stamps are coherent.

### Step 14 — Measure hot paths

Use existing performance counters/benchmarks where available. Record baseline and post-change
for features that affect every edit/query.

Avoid optimizing by weakening semantics without an explicit precision policy.

### Step 15 — Verify repository gates

At minimum run focused tests and formatting/lints. Before declaring a substantial semantic
change complete, use the repository's workspace/full gates as appropriate:

```sh
cargo fmt --check
cargo test -p phalcom-lsp
scripts/test.sh lsp
cargo clippy --workspace
scripts/test.sh workspace
# scripts/test.sh full when required by change scope
```

After code changes, update graphify if repository instructions require it.

## Current analysis architecture

Conceptually, current LSP semantic analysis is:

```text
Program
  |-- build ModuleSurface
  |-- build ScopeGraph
  |-- build OccurrenceIndex
  |-- update ModuleGraph
  v
affected module frontier
  v
interprocedural solve
  |-- CallableSummary fixed point
  |-- ParameterFacts
  v
flow passes
  |-- LocalFacts
  |-- FieldFacts
  |-- call/effect evidence
  v
SemanticState
  v
immutable SemanticSnapshot
  v
semantic queries
  v
LSP features
```

This is already a real reusable semantic engine. Extend it deliberately.
Read [references/current-architecture.md](references/current-architecture.md).

## Scope and name-resolution rules

When adding a declaration-bearing syntax form:

1. identify exact scope boundary;
2. decide declaration visibility order;
3. assign `SemanticBindingKind` or a new justified kind;
4. assign `BindingId` from the file snapshot;
5. record exact declaration token range;
6. visit initializer expressions in correct lexical order;
7. add occurrences for declaration and reads/writes;
8. update completion/refactoring tests.

Do not infer runtime value as part of lexical resolution.

Read [references/scopes-bindings-occurrences.md](references/scopes-bindings-occurrences.md).

## Expression analysis rules

Expression inference should be a pure-ish function of an explicit context:

```text
expression
+ current class/dispatch side
+ scope/binding state
+ known class/module surface
+ callable/field facts
+ dispatch resolver
-> InferredValue
```

Avoid hidden global mutable state.

When adding an expression form:

- preserve evaluation order;
- infer exact syntax facts when possible;
- route member sends through shared dispatch;
- carry class/module-qualified identities;
- widen gracefully on dynamic packs/reflection;
- preserve provenance;
- add malformed/incomplete AST tests.

Read [references/expression-inference-and-dispatch.md](references/expression-inference-and-dispatch.md).

## Flow-analysis rules

The current `flow.rs` already shares one structured traversal across local, summary, field,
and call-site analysis. Preserve that sharing.

A future explicit CFG is appropriate when multiple new analyses need reusable program points,
branch edges, dominance, loops, or proof predicates. Do not create separate mini-CFGs in the
checker, linter, and prover.

Read [references/control-flow-and-cfg.md](references/control-flow-and-cfg.md).

## Adding a new fact category

Prefer this architecture:

```rust
struct NewFacts {
    by_id: BTreeMap<SemanticId, NewFact>,
}

struct NewFact {
    value: AbstractDomain,
    provenance: Small/BoundedEvidence,
}
```

Then define:

```text
record/update
join
query
merge between file and project state
invalidation
snapshot publication
```

Do not expose internal mutable maps directly to consumers.

## Adding a new callable effect

Potential future effects include `may_throw`, `may_yield`, `may_block`, `mutates`,
`escapes_parameter`, `does_io`, and `does_not_return`.

For each effect:

1. decide may/must polarity;
2. identify syntax/native operations that originate it;
3. determine propagation through resolved calls;
4. determine dynamic-call fallback;
5. define higher-order/block propagation;
6. add it to summary equality/fixed point;
7. add invalidation tests;
8. document consumer policy.

Do not infer "pure" merely because no effect was observed in an incomplete analysis.

Read [references/effects-closures-concurrency.md](references/effects-closures-concurrency.md).

## Adding future typing

Do **not** mutate `ValueShape` into the checker type system.

Recommended layering:

```text
semantic identities/surfaces/flow
        |              |
        |              +-> advisory ValueShape facts (existing)
        |
        +-> resolved type annotations -> TypeStore/TypeId (future)
                              |
                     constraint/checking engine
                              |
                     typed semantic facts
                              |
          +-------------------+-------------------+
          |                   |                   |
         LSP                checker            prover
```

The checker should reuse:

- `ModuleId`/class/callable/field identities;
- scope/binding graph;
- dispatch resolver/surfaces;
- shared CFG/flow program points;
- callable dependency graph;
- provenance/invalidation infrastructure.

Read [references/type-checker-integration.md](references/type-checker-integration.md).

## Adding static proving

Start with a fact domain and CFG, not an SMT call.

Implementation sequence:

1. define proof question/contract;
2. lower condition to a trusted predicate representation;
3. compute cheap dataflow/refinement facts;
4. build pre/post/loop obligations;
5. classify `Proved | Refuted | Unknown`;
6. only then add an SMT backend for supported residual formulas if justified;
7. keep solver timeout/unknown explicit;
8. retain proof provenance for diagnostics.

Read [references/static-analysis-and-proving.md](references/static-analysis-and-proving.md).

## Core/standard library and native semantics

Semantic intelligence will increasingly depend on library contracts for:

- collections;
- `Option`/`Result`;
- strings/bytes/path;
- I/O/process/OS;
- fibers/concurrency;
- reflection;
- FFI/native packages.

Prefer source-visible declarations and trusted semantic metadata over scattered hard-coded
LSP special cases.

A native semantic signature must never promise behavior stronger than the runtime primitive.

Read [references/library-native-and-ffi-semantics.md](references/library-native-and-ffi-semantics.md).

## Performance budget model

Editor-semantic work has at least four costs:

```text
update latency
snapshot publication cost
query latency
memory retained per workspace
```

Measure all relevant costs. A fast query backed by a 500 ms every-keystroke rebuild is not
fast tooling.

Preferred strategies:

- IDs + arenas/maps rather than cloned graphs;
- file contribution replacement;
- affected-frontier/SCC recomputation;
- bounded unions/provenance;
- immutable snapshots;
- lazy expensive queries when safe;
- reuse shared walkers;
- performance counters around passes.

Read [references/performance-and-rust.md](references/performance-and-rust.md).

## Required test matrix

For any nontrivial semantic feature, select from this matrix:

| Category | What it catches |
|---|---|
| exact syntax fact | basic transfer/inference |
| lexical shadowing | text/name confusion |
| same class name in different modules | identity bugs |
| instance/class side | side conflation |
| inheritance/super | dispatch origin bugs |
| branch merge | traversal-order bugs |
| loop | missing fixed point |
| return/throw/break/continue | reachability bugs |
| closure capture | lifetime/effect bugs |
| higher-order callback | callable-effect propagation |
| recursive calls | convergence bugs |
| dynamic pack/send | false certainty |
| field initializer + constructor + general write | evidence/source categorization |
| unresolved import | recovery/module graph bugs |
| malformed/incomplete source | editor robustness |
| edit/remove/re-add | stale-state/invalidation |
| deterministic reparse | unstable identity/output |
| large union/deep structure | widening/resource limits |

Read [references/testing-and-fuzzing.md](references/testing-and-fuzzing.md).

## Common feature recipes

Read [references/feature-recipes.md](references/feature-recipes.md) for step-by-step recipes
covering:

- a new binding syntax;
- a new literal/container shape;
- a new member/send form;
- better field inference;
- new parameter/call-site inference;
- branch refinement;
- a new callable effect;
- a new module edge;
- checker type facts;
- contract/static proof facts;
- native/stdlib semantic signatures;
- fiber/yield/block effects.

## Forbidden shortcuts

Do not merge code that does any of these without a compelling documented migration reason:

- re-walks the entire AST separately in hover/completion for an answer semantics already owns;
- keys cross-file class facts by bare name;
- stores `BindingId` across file revisions;
- treats `ValueShape` as normative language type;
- changes selector identity based on type annotations;
- returns exact facts from heuristic use-site evidence;
- joins branches by analysis order;
- recursively analyzes callees without recursion handling;
- caches facts without invalidation dependencies;
- publishes half-updated semantic state;
- hides solver/widening failure as a concrete type/shape;
- assumes dynamic sends have no unknown effects;
- treats failure to prove as proof of violation;
- puts `lsp_types::*` structures into core semantic facts;
- hardcodes standard-library behavior in one LSP feature when it can be represented in shared semantic/core metadata.

## Skill pressure tests

Before approving a major semantic design, mentally run these cases. The correct response is more important than a particular representation:

- A parameter has only observed `Integer` callers: retain advisory evidence; do not manufacture a normative type.
- A loop adds a new alternative on its second iteration: require fixed-point reasoning, not one-pass traversal.
- A closure captures and later writes a refined local: the refinement must be invalidated at the semantically possible write.
- An immutable local references a mutable object field: local immutability alone does not stabilize the field refinement.
- A fiber yields after proving a shared field fact: suspension may invalidate heap-dependent facts.
- A dynamic send has no known target: preserve facts unrelated to its conservative effects, but do not assume purity/no-throw/no-yield.
- An import or declaration is half-written: preserve independently valid identities/facts while marking recovery; do not reinterpret invalid complete syntax as valid semantics.
- A cache has no declared dependency/invalidation rule: reject the cache design.
- An SMT solver times out: classify proof as unknown/resource-exhausted, never proved or refuted.
- A proposal wants type annotations to choose ordinary method identity: reject unless a future normative Phalcom dispatch design explicitly changes that rule.

## Completion checklist

Before saying semantic work is complete:

- [ ] Normative behavior identified.
- [ ] Current graph/source inspected.
- [ ] Semantic question and tier written down.
- [ ] Identity/lifetime correct.
- [ ] Domain + unknown + join + widening documented.
- [ ] Flow/reachability correct.
- [ ] Dispatch agrees with runtime.
- [ ] Interprocedural recursion/convergence handled.
- [ ] Provenance retained.
- [ ] Invalidation edges updated.
- [ ] Snapshot query exposed rather than consumer-specific logic.
- [ ] Semantic tests pass before UI tests.
- [ ] Incremental edit/removal tests pass.
- [ ] Determinism/recovery tested.
- [ ] Performance/rebuild scope checked.
- [ ] Rust docs/clippy/format gates pass.
- [ ] Graphify updated when required.
- [ ] Future typing/proving boundary remains clean.

For a detailed review rubric, read [references/review-and-debugging.md](references/review-and-debugging.md).

## Navigation

| Reference | Use it for |
|---|---|
| [semantic-foundations-and-analysis-contract.md](references/semantic-foundations-and-analysis-contract.md) | Concrete-vs-abstract meaning, soundness, information order, may/must polarity, recovery boundaries |
| [current-architecture.md](references/current-architecture.md) | Current `phalcom-lsp::semantic` dataflow and code ownership |
| [scopes-bindings-occurrences.md](references/scopes-bindings-occurrences.md) | Lexical analysis, binding IDs, source targets, references/rename |
| [expression-inference-and-dispatch.md](references/expression-inference-and-dispatch.md) | Expressions, receiver inference, selectors, members, `self`/`super`, packs |
| [abstract-domains-and-dataflow.md](references/abstract-domains-and-dataflow.md) | Lattices, joins, transfer functions, widening, may/must analyses |
| [control-flow-and-cfg.md](references/control-flow-and-cfg.md) | Structured flow today, future CFG, branches, loops, reachability, dominance |
| [semantic-ir-and-lowering.md](references/semantic-ir-and-lowering.md) | Decision boundary and invariants for a future semantic CFG/IR and source lowering |
| [interprocedural-analysis.md](references/interprocedural-analysis.md) | Callable summaries, SCC/fixed point, parameters, recursion, dynamic calls |
| [effects-closures-concurrency.md](references/effects-closures-concurrency.md) | Blocks, captures, non-local returns, future throw/yield/block/escape effects |
| [heap-alias-mutation-and-refinement.md](references/heap-alias-mutation-and-refinement.md) | Heap places, aliasing, strong/weak updates, refinement kills, callbacks, reflection, fiber stability |
| [incrementality-and-snapshots.md](references/incrementality-and-snapshots.md) | Module graph, affected frontier, invalidation, immutable publication |
| [modules-and-project-analysis.md](references/modules-and-project-analysis.md) | Module/package/project identities, imports, exports, cycles and project invalidation |
| [type-checker-integration.md](references/type-checker-integration.md) | Future TypeStore/constraints/bidirectional checking without corrupting `ValueShape` |
| [static-analysis-and-proving.md](references/static-analysis-and-proving.md) | Definite facts, refinements, abstract interpretation, contracts, SMT boundary |
| [library-native-and-ffi-semantics.md](references/library-native-and-ffi-semantics.md) | Core/std/native/FFI semantic contracts |
| [diagnostics-provenance-and-uncertainty.md](references/diagnostics-provenance-and-uncertainty.md) | Evidence chains, uncertainty, stable diagnostics and explainable analysis |
| [testing-and-fuzzing.md](references/testing-and-fuzzing.md) | Unit/integration/metamorphic/property/fuzz/incremental testing |
| [performance-and-rust.md](references/performance-and-rust.md) | Rust structures, allocations, recursion limits, profiling, query/update latency |
| [feature-recipes.md](references/feature-recipes.md) | Concrete implementation playbooks |
| [review-and-debugging.md](references/review-and-debugging.md) | Failure diagnosis, review gates, stale-fact/debug workflow |
